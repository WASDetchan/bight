pub mod interaction;
pub mod lua;

use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt::Display,
    sync::Arc,
    thread::{self, JoinHandle},
};

use futures::future::join_all;
use interaction::{
    Communicator, MessageSender, ValueMessage, ValueRequest, ValueResponse, message_channel,
};
use mlua::{IntoLua, Lua};
use tokio::sync::oneshot;

use crate::table::{DataTable, HashTable, Table, TableMut, cell::CellPos};

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
    Text(Arc<str>), // Using Arc<str> instead of String as TableValue is never mutated without cloning, but cloning happens often
    LuaValue(mlua::Value),
    Err(TableError),
}

impl TableValue {
    pub fn other_error(error: impl Error + Send + Sync + 'static) -> Self {
        Self::Err(TableError::OtherError(Arc::new(error)))
    }
    pub fn is_err(&self) -> bool {
        matches!(self, Self::Err(_))
    }
}

impl TableValue {
    pub fn from_stringable(s: impl ToString) -> Self {
        Self::Text(s.to_string().into())
    }
    pub fn from_into_lua(v: impl IntoLua, lua: &Lua) -> mlua::Result<Self> {
        Ok(Self::LuaValue(v.into_lua(lua)?))
    }
}

impl Display for TableValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(s) => write!(f, "{s}"),
            Self::LuaValue(value) => match value.to_string() {
                Ok(s) => s.fmt(f),
                Err(e) => e.fmt(f),
            },
            Self::Err(e) => write!(f, "#ERR: {e}"),
            Self::Empty => write!(f, ""),
        }
    }
}

pub type SourceTable = DataTable<Arc<str>>;
pub type CacheTable = HashTable<TableValue>;
pub type DependencyChannelTable = HashTable<Vec<oneshot::Sender<TableValue>>>;
pub type GraphTable = HashTable<HashSet<CellPos>>;

#[derive(Debug, Default)]
pub struct EvaluatorTable {
    source: SourceTable,
    cache: CacheTable,
    required_by: GraphTable,  // required_by is inversed dependencies
    dependencies: GraphTable, // dependencies is inversed required_by
    invalid_caches: HashSet<CellPos>,
}

impl EvaluatorTable {
    pub fn new(source: SourceTable) -> Self {
        Self {
            source,
            ..Default::default()
        }
    }
    pub fn set_source<S>(&mut self, pos: impl Into<CellPos>, src: Option<S>)
    where
        Arc<str>: From<S>,
    {
        let pos = pos.into();
        self.source.set(pos, src.map(From::from));
        self.invalidate_cache(pos);
    }
    pub fn get_source(&mut self, pos: impl Into<CellPos>) -> Option<&Arc<str>> {
        let pos = pos.into();
        self.source.get(pos)
    }
    fn invalidate_cache(&mut self, pos: impl Into<CellPos>) {
        let pos = pos.into();
        if !self.invalid_caches.contains(&pos) {
            self.invalid_caches.insert(pos);
            self.cache.remove(&pos);
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
                    self.invalidate_cache(req);
                }
            }
        }
    }
    pub fn cache(&mut self) {
        let invalid_cells = std::mem::take(&mut self.invalid_caches);
        let (req_send, mut req_recv) = message_channel();

        fn make_evaluator_future<F, FT>(
            sender: MessageSender,
            pos: CellPos,
            src: Arc<str>,
            eval_fn: F,
        ) -> impl Future<Output = ()>
        where
            FT: Future<Output = ()>,
            F: Fn(Arc<str>, Communicator) -> FT,
        {
            let comm = Communicator::new(pos, sender);
            eval_fn(src, comm)
        }

        let futures: Vec<_> = invalid_cells
            .into_iter()
            .flat_map(|cell| {
                Some(make_evaluator_future(
                    req_send.clone(),
                    cell,
                    self.source.get(cell)?.clone(),
                    evaluate,
                ))
            })
            .collect();
        drop(req_send); // Dropping the sender is required for the channel to close automatically
        // when all other senders are dropped (meaning all tasks are finished)

        let handle = spawn_evaluation_runtime(futures);

        let mut dependencies = DependencyChannelTable::new();

        while let Some(msg) = req_recv.blocking_recv() {
            match msg {
                ValueMessage::Req(req) => self.handle_request(req, &mut dependencies),
                ValueMessage::Res(res) => self.handle_response(res, &mut dependencies),
            }
        }

        _ = handle.join();
    }

    /// Handles the ValueResponse by storing its value in cache and sending it to the dependencies
    fn handle_response(
        &mut self,
        response: ValueResponse,
        dependencies: &mut DependencyChannelTable,
    ) {
        log::debug!("ValueResponse for {} arrived", response.cell);
        for dep in dependencies
            .get_mut(&response.cell)
            .map(std::mem::take)
            .into_iter()
            .flatten()
        {
            _ = dep.send(response.value.clone()); // TableValue can only store Arc so cloning is cheap
        }
        self.cache.insert(response.cell, response.value);
    }

    fn has_dependency_cycle(&self, start: CellPos, visited: &mut HashTable<Vertex>) -> bool {
        match visited.get(&start) {
            Some(Vertex::Parent) => true,
            Some(Vertex::Visited) => false,
            None => {
                visited.insert(start, Vertex::Parent);
                for &dep in self.dependencies.get(&start).into_iter().flatten() {
                    if self.has_dependency_cycle(dep, visited) {
                        return true;
                    }
                }
                visited.insert(start, Vertex::Visited);
                false
            }
        }
    }

    /// Handles the ValueRequest by either responding with cached value immideatly or adding it to the dependency list
    fn handle_request(&mut self, request: ValueRequest, dependencies: &mut DependencyChannelTable) {
        log::debug!("ValueRequest for {} by {}", request.cell, request.requester);
        // Keep track of dependencies
        self.dependencies
            .entry(request.requester)
            .or_default()
            .insert(request.cell);
        self.required_by
            .entry(request.cell)
            .or_default()
            .insert(request.requester);

        log::trace!(
            "dependencies: {:?};\n required_by: {:?};",
            self.dependencies,
            self.required_by
        );

        // If there's no source for the cell the cell
        // is empty (it cannot be determined from cache because empty cells are not cached for
        // performance)
        if self.source.get(request.cell).is_none() {
            _ = request.sender.send(TableValue::Empty);
        } else if self.has_dependency_cycle(request.requester, &mut HashMap::new()) {
            let value = TableValue::other_error(DependencyCycleError);
            self.cache.insert(request.cell, value.clone());
            for sender in dependencies
                .get_mut(&request.cell)
                .map(std::mem::take)
                .into_iter()
                .flatten()
            {
                _ = sender.send(value.clone());
            }
            _ = request.sender.send(value);
            log::warn!("Dependency cycle starting at {} detected!", request.cell)
        } else if let Some(v) = self.cache.get(&request.cell) {
            _ = request.sender.send(v.clone()); // Send the value if it is cached
        } else {
            dependencies
                .entry(request.requester)
                .or_default()
                .push(request.sender)
        }
    }
}

enum Vertex {
    Visited,
    Parent,
}

#[derive(Debug, thiserror::Error)]
#[error("Dependency cycle detected")]
struct DependencyCycleError;

///
/// Spawns a tokio runtime in a new thread and starts awaiting the futures
/// Returns JoinHandle to the spawned process
///
fn spawn_evaluation_runtime<F: Future<Output = ()> + Send + 'static>(
    futures: Vec<F>,
) -> JoinHandle<Vec<()>> {
    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(join_all(futures))
    })
}

impl Table for EvaluatorTable {
    type Item = TableValue;
    fn get(&self, pos: CellPos) -> Option<&Self::Item> {
        if !self.invalid_caches.is_empty() {
            panic!("Table values should never be accessed with invalid caches present!");
            // TODO: cache values on get request using interior mutability
        }
        self.cache.get(&pos)
    }
}

async fn evaluate(source: Arc<str>, communicator: Communicator) {
    if source.starts_with('=') {
        let lua_source = source.split_at(1).1;
        lua::evaluate(lua_source, communicator).await;
    } else {
        let out = if source.starts_with('\\') {
            Arc::<str>::from(source.split_at(1).1)
        } else {
            source
        };
        let table_value = TableValue::Text(out);
        communicator.respond(table_value).await;
    }
}
