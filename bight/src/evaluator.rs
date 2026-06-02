//! The module that contains the base evaluation logic.
//! The EvaluatorTable evaluates a TableValue for each non-empty cell of its source. If the source
//! starts with '\', that backslash is ignored. If the source starts with '=', the '=' is ignored
//! and the rest of the source is interpreted as a lua formula (the '=' is replaced with "return "
//! and the value of the chuck is evaluated. See [`lua`] module for more info). Else the value is the source string itself.
//!
//! Some evaluation API is exposed. The expected way of accuiring evaluation results is from
//! [`Table`] trait implementation on [`EvaluatorTable`] (see item-level docs for more info).

pub mod interaction;
pub mod lua;
mod source;

pub use source::SourceTable;
use std::{collections::HashSet, fmt::Display, sync::Arc};

use tokio::sync::{Mutex, RwLock, RwLockWriteGuard, oneshot};

use crate::{
    evaluator::interaction::CellInfo,
    file::BightFile,
    sync::{self, Rc, RcStr},
    table::{HashTable, Table, TableMut, cell::CellPos},
};

/// The type representing an error that occured during evaluation. Can be caused either by an error
/// that was raised in the code of the cell being evaluated, or by a dependency cycle.  
///
/// PartialEq is implemented for this type, but will always return false
#[derive(Debug, thiserror::Error, Clone)]
pub enum TableError {
    /// A lua error was raised during the evaluation
    #[error(transparent)]
    LuaError(Rc<mlua::Error>),
    /// Non-lua error raised during the evaluation
    #[error(transparent)]
    OtherError(std::sync::Arc<dyn sync::StdError>),
}

impl PartialEq for TableError {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

#[derive(Debug, Clone, PartialEq)]
/// The value that the sourcewas evaluated to. Note that error during evaluation is also considered
/// a value, and it will be propagated into any cell that depend on the cell that raised the error
/// (The [`mlua::IntoLua`] implementation for [`TableValue::Err`] returns a lua error)
pub enum TableValue {
    /// An empty value. This is the value of a cell with no source, or the cell with a formula that
    /// returned nil
    Empty,
    /// Value of a non-formula cell or a cell with a formula that returned a string
    Text(RcStr), // Using Rc<str> (or Arc with feature `multi-thread`) instead of String as TableValue is never mutated, but cloning happens often
    /// Value of a cell a formula in which returned a number (float or integer)
    Number(f64),
    /// An error occured during the evaluation
    Err(TableError),
}

#[derive(Debug, thiserror::Error)]
/// The error that occured during evaluation of a cell's value. This is only returned when a
/// dependency cycle was present right after the request. All other cells that depend on the value
/// of the cell that got this response are evaluated normally. Usually, this error should not be
/// handled and be let bubble up so all other cell also return this error
pub enum EvaluationError {
    #[error("Dependency cycle detected")]
    DependencyCycle,
}

impl From<Result<TableValue, EvaluationError>> for TableValue {
    fn from(value: Result<TableValue, EvaluationError>) -> Self {
        match value {
            Ok(val) => val,
            Err(e) => TableValue::other_error(e),
        }
    }
}

impl TableValue {
    /// Shorthand for creating a non-lua error table value
    pub fn other_error(error: impl sync::StdError + 'static) -> Self {
        Self::Err(TableError::OtherError(Arc::new(error)))
    }
    /// Shorthand for creating a lua error table value
    pub fn lua_error(error: mlua::Error) -> Self {
        Self::Err(TableError::LuaError(Rc::new(error)))
    }
    pub fn is_err(&self) -> bool {
        matches!(self, Self::Err(_))
    }
    /// Formats the value to the giving length, aligning to the right and filling with whitespaces
    /// if the formatted value's length is less than requested.
    pub fn format_to_length(&self, length: usize) -> String {
        format!("{:<length$}", self.to_string().lines().next().unwrap_or(""))
            .chars()
            .take(length)
            .collect()
    }
    pub fn from_text(s: impl ToString) -> Self {
        Self::Text(s.to_string().into())
    }
    pub fn from_number(n: impl Into<f64>) -> Self {
        Self::Number(n.into())
    }
}

impl Display for TableValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(s) => write!(f, "{s}"),
            Self::Number(value) => write!(f, "{value}"),
            Self::Err(e) => write!(f, "#ERR: {e}"),
            Self::Empty => write!(f, ""),
        }
    }
}
#[derive(Debug, thiserror::Error)]
#[error("The TableValue couldn't be converted")]
pub struct TableValueConversionError;
impl TryFrom<TableValue> for f64 {
    type Error = TableValueConversionError;
    fn try_from(value: TableValue) -> Result<Self, Self::Error> {
        use TableValue::{Empty, Number};
        match value {
            Empty => Ok(0.0),
            Number(v) => Ok(v),
            _ => Err(TableValueConversionError),
        }
    }
}

pub type CacheTable = HashTable<RwLock<Option<TableValue>>>;
pub type ValueTable = HashTable<TableValue>;
pub type DependencyChannelTable = HashTable<Vec<oneshot::Sender<TableValue>>>;
pub type GraphTable = HashTable<HashSet<CellPos>>;

#[derive(Debug, Default, PartialEq)]
pub struct EvaluatorTable {
    file: BightFile,
    result: ValueTable,
    required_by: GraphTable,  // required_by is inversed dependencies
    dependencies: GraphTable, // dependencies is inversed required_by
    invalid_caches: HashSet<CellPos>,
}

impl EvaluatorTable {
    pub fn new(source: SourceTable) -> Self {
        let invalid_caches: HashSet<CellPos> = source.inner_iter().map(|(pos, _)| *pos).collect();
        Self {
            file: BightFile::new(source),
            invalid_caches,
            ..Default::default()
        }
    }
    pub fn source_file(&self) -> &BightFile {
        &self.file
    }
    pub fn source_table(&self) -> &SourceTable {
        &self.file.source
    }
    pub fn set_source<S>(&mut self, pos: impl Into<CellPos>, src: Option<S>)
    where
        RcStr: From<S>,
    {
        let pos = pos.into();
        match &src {
            Some(_) => self.invalidate_cell(pos),
            None => self.remove_cell(pos),
        }
        match src {
            None => {
                self.file.source.set(pos, None);
            }
            Some(s) => {
                self.file.source.set(pos, Some(s.into()));
            }
        };
    }

    pub fn get_source(&self, pos: impl Into<CellPos>) -> Option<&RcStr> {
        let pos = pos.into();
        self.file.source.get(pos)
    }

    pub fn invalidate_cell(&mut self, pos: impl Into<CellPos>) {
        let pos = pos.into();
        if !self.invalid_caches.contains(&pos) {
            self.result.remove(&pos);
            self.invalid_caches.insert(pos);

            for dep in self
                .dependencies
                .get_mut(&pos)
                .map(std::mem::take)
                .into_iter()
                .flatten()
            {
                self.required_by.remove(&dep);
            }

            if let Some(set) = self.required_by.get(&pos) {
                for req in set.clone() {
                    self.invalidate_cell(req);
                }
            }
        }

        log::trace!("Invalidated cell {}", pos);
    }

    pub fn invalidate_all_cells(&mut self) {
        for (&pos, _) in self.result.iter() {
            self.invalid_caches.insert(pos);
        }
        self.result.clear();
        self.required_by.clear();
        self.dependencies.clear();
    }

    fn remove_cell(&mut self, pos: impl Into<CellPos>) {
        let pos = pos.into();
        self.invalidate_cell(pos);
        self.invalid_caches.remove(&pos);
    }

    pub fn evaluate(&mut self) {
        log::info!("Starting cell evaluation");
        let dep_tables = Mutex::new((
            std::mem::take(&mut self.dependencies),
            std::mem::take(&mut self.required_by),
        ));

        let intermediate_table: CacheTable = self
            .invalid_caches
            .iter()
            .map(|pos| (*pos, RwLock::new(None)))
            .collect();

        let invalid_cells = self
            .invalid_caches
            .iter()
            .map(|&pos| {
                CellInfo::new(
                    self.file
                        .source
                        .get(pos)
                        .expect("Only cells with source may be marked as invalid cache"),
                    pos,
                    &dep_tables,
                    &intermediate_table,
                    &self.result,
                )
            })
            .collect::<Vec<_>>();

        log::trace!("Invalid cells: {:#?}", invalid_cells);
        async fn make_evaluator_future<'a, F, FT>(
            mut guard: RwLockWriteGuard<'a, Option<TableValue>>,
            info: &'a CellInfo<'_>,
            eval_fn: F,
        ) where
            FT: Future<Output = TableValue> + 'a,
            F: Fn(&'a CellInfo<'a>) -> FT + 'a,
        {
            *guard = Some(eval_fn(info).await)
        }

        let futures: Vec<_> = invalid_cells
            .iter()
            .map(|info| {
                make_evaluator_future(
                    intermediate_table
                        .get(&info.pos())
                        .expect("Only cells with cache = None may be marked as invalid cache")
                        .try_write()
                        .expect("Each cache can only be locked for writing once"),
                    info,
                    evaluate,
                )
            })
            .collect();

        sync::block_on_all(futures);

        let dep_tables = dep_tables.into_inner();
        self.dependencies = dep_tables.0;
        self.required_by = dep_tables.1;
        std::mem::take(&mut self.invalid_caches)
            .iter()
            .for_each(|&pos| {
                let val = intermediate_table
                    .get(&pos)
                    .expect("All invalid cells are in intermediate_table")
                    .try_write()
                    .expect("No guard is held after evaluation")
                    .take()
                    .expect("All invalid cells were evaluated");
                self.result.insert(pos, val);
            });
        log::info!("Finished cell evaluation");
    }
}

impl Table for EvaluatorTable {
    type Item = TableValue;
    fn get(&self, pos: CellPos) -> Option<&Self::Item> {
        if !self.invalid_caches.is_empty() {
            panic!("Table values should never be accessed with invalid caches present!");
            // TODO: cache values on get request using interior mutability
        }
        self.result.get(&pos)
    }
}

async fn evaluate<'a>(info: &'a CellInfo<'a>) -> TableValue {
    let source = info.source();
    if source.starts_with('=') {
        let lua_source = source.split_at(1).1;
        lua::evaluate(lua_source, info).await
    } else {
        let out = if source.starts_with('\\') {
            RcStr::from(source.split_at(1).1)
        } else {
            source.clone()
        };
        TableValue::Text(out)
    }
}

#[cfg(test)]
mod test {
    use crate::evaluator::*;

    #[test]
    fn format_number() {
        assert_eq!(TableValue::from_number(6).format_to_length(5), "6    ");
        assert_eq!(TableValue::from_number(678910).format_to_length(5), "67891");
    }
    #[test]
    fn format_string() {
        assert_eq!(TableValue::from_text("6").format_to_length(5), "6    ");
        assert_eq!(TableValue::from_text("678910").format_to_length(5), "67891");
    }
}
