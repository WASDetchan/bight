pub mod interaction;
pub mod lua;

use std::{collections::HashSet, error::Error, fmt::Display, sync::Arc};

use futures::future::join_all;
use tokio::sync::{Mutex, RwLock, RwLockWriteGuard, oneshot};

use crate::{
    evaluator::interaction::CellInfo,
    table::{HashTable, Table, cell::CellPos},
};

#[derive(Debug, thiserror::Error, Clone)]
pub enum TableError {
    #[error(transparent)]
    LuaError(Arc<mlua::Error>),
    #[error(transparent)]
    OtherError(Arc<dyn Error + Send + Sync>),
}
#[derive(Debug, Clone)]
pub enum TableValue {
    Empty,
    Text(Arc<str>), // Using Arc<str> instead of String as TableValue is never mutated, but cloning happens often
    Number(f64),
    Err(TableError),
}

#[derive(Debug, thiserror::Error)]
pub enum EvalationError {
    #[error("Dependency cycle detected")]
    DependencyCycle,
}

impl From<Result<TableValue, EvalationError>> for TableValue {
    fn from(value: Result<TableValue, EvalationError>) -> Self {
        match value {
            Ok(val) => val,
            Err(e) => TableValue::other_error(e),
        }
    }
}

impl TableValue {
    pub fn other_error(error: impl Error + Send + Sync + 'static) -> Self {
        Self::Err(TableError::OtherError(Arc::new(error)))
    }
    pub fn lua_error(error: mlua::Error) -> Self {
        Self::Err(TableError::LuaError(Arc::new(error)))
    }
    pub fn is_err(&self) -> bool {
        matches!(self, Self::Err(_))
    }
    pub fn format_to_length(&self, length: usize) -> String {
        format!("{:<length$}", self.to_string().lines().next().unwrap_or(""))
            .chars()
            .take(length)
            .collect()
    }
}

impl TableValue {
    pub fn from_stringable(s: impl ToString) -> Self {
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

pub type SourceTable = HashTable<Arc<str>>;
pub type CacheTable = HashTable<RwLock<Option<TableValue>>>;
pub type ValueTable = HashTable<TableValue>;
pub type DependencyChannelTable = HashTable<Vec<oneshot::Sender<TableValue>>>;
pub type GraphTable = HashTable<HashSet<CellPos>>;

#[derive(Debug, Default)]
pub struct EvaluatorTable {
    source: SourceTable,
    result: ValueTable,
    required_by: GraphTable,  // required_by is inversed dependencies
    dependencies: GraphTable, // dependencies is inversed required_by
    invalid_caches: HashSet<CellPos>,
}

impl EvaluatorTable {
    pub fn new(source: SourceTable) -> Self {
        let invalid_caches: HashSet<CellPos> = source.iter().map(|(pos, _)| *pos).collect();
        Self {
            source,
            invalid_caches,
            ..Default::default()
        }
    }
    pub fn source_table(&self) -> &SourceTable {
        &self.source
    }
    pub fn set_source<S>(&mut self, pos: impl Into<CellPos>, src: Option<S>)
    where
        Arc<str>: From<S>,
    {
        let pos = pos.into();
        match &src {
            Some(_) => self.invalidate_cell(pos),
            None => self.remove_cell(pos),
        }
        match src {
            None => {
                self.source.remove(&pos);
            }
            Some(s) => {
                self.source.insert(pos, s.into());
            }
        };
    }

    pub fn get_source(&self, pos: impl Into<CellPos>) -> Option<&Arc<str>> {
        let pos = pos.into();
        self.source.get(&pos)
    }
    fn invalidate_cell(&mut self, pos: impl Into<CellPos>) {
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
                    self.source
                        .get(&pos)
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
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(join_all(futures));

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
            Arc::<str>::from(source.split_at(1).1)
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
        assert_eq!(
            TableValue::from_stringable("6").format_to_length(5),
            "6    "
        );
        assert_eq!(
            TableValue::from_stringable("678910").format_to_length(5),
            "67891"
        );
    }
}
