pub mod interaction;

use std::{
    collections::HashSet,
    error::Error,
    fmt::Display,
    pin::Pin,
    sync::Arc,
    thread::{self, JoinHandle},
};

use futures::future::join_all;
use interaction::{
    Communicator, MessageSender, ValueMessage, ValueRequest, ValueResponse, message_channel,
};
use mlua::{FromLua, IntoLua, Lua};
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
    String(Arc<str>), // Using Arc<str> instead of String as TableValue is never mutated without cloning, but cloning itself happens often
    Err(TableError),
}

impl Display for TableValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => write!(f, "{s}"),
            Self::Err(e) => write!(f, "#ERR: {e}"),
            Self::Empty => write!(f, ""),
        }
    }
}

pub type SourceTable = DataTable<Arc<str>>;
pub type CacheTable = HashTable<TableValue>;
pub type DependencyChannelTable = HashTable<Vec<oneshot::Sender<TableValue>>>;
pub type GraphTable = HashTable<HashSet<CellPos>>;

#[derive(Default, Debug)]
pub struct LuaTable {
    source: SourceTable,
    cache: CacheTable,
    required_by: GraphTable,  // required_by is inversed dependencies
    dependencies: GraphTable, // dependencies is inversed required_by
    invalid_caches: HashSet<CellPos>,
}

impl LuaTable {
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
        self.invalid_caches.insert(pos);
        if self.cache.contains_key(&pos) {
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

    /// Handles the ValueRequest by either responding with cached value immideatly or adding it to the dependency list
    fn handle_request(&mut self, request: ValueRequest, dependencies: &mut DependencyChannelTable) {
        // Keep track of dependencies
        // TODO: Check for cycles
        self.dependencies
            .entry(request.requester)
            .or_default()
            .insert(request.cell);
        self.required_by
            .entry(request.cell)
            .or_default()
            .insert(request.requester);

        if self.source.get(request.cell).is_none() {
            _ = request.sender.send(TableValue::Empty); // If there's no source for the cell the cell
        // is empty (it cannot be determined from cache because empty cells are not cached for
        // performance)
        } else if let Some(v) = self.cache.get(&request.cell) {
            _ = request.sender.send(v.clone()); // Send the value if it is cached
        } else {
            // Save the dependency otherwise
            dependencies
                .entry(request.requester)
                .or_default()
                .push(request.sender);
        }
    }
}

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

impl Table for LuaTable {
    type Item = TableValue;
    fn get(&self, pos: CellPos) -> Option<&Self::Item> {
        if !self.invalid_caches.is_empty() {
            panic!("Table values should never be accessed with invalid caches present!");
            // TODO: cache values on get request using interior mutability
        }
        self.cache.get(&pos)
    }
}

pub async fn evaluate(source: impl AsRef<str>, communicator: Communicator) {
    fn make_func(
        communicator: Communicator,
    ) -> impl Fn(Lua, (mlua::Value, CellPos)) -> Pin<Box<dyn Future<Output = mlua::Result<TableValue>> + Send + Sync>>
    {
        move |_, (_, pos): (mlua::Value, CellPos)| {
            Box::pin({
                let mut communicator = communicator.clone();
                async move { communicator.request(pos).await }
            })
        }
    }
    let lua = Lua::new();
    let f = lua
        .create_async_function(make_func(communicator.clone()))
        // .create_async_function(async |_, (tab, v): (mlua::Value, mlua::Value)| {
        //     eprintln!("__index: {v:?}");
        //     Ok(())
        // })
        .unwrap();

    let metatable = lua.create_table().expect("no error is documented");

    metatable.set("__index", f).unwrap();
    lua.globals().set_metatable(Some(metatable)).unwrap();

    let chunk = lua.load(source.as_ref());
    let res = chunk.eval_async::<TableValue>().await;
    communicator
        .respond(res.unwrap_or_else(|err| TableValue::Err(TableError::LuaError(Arc::new(err)))))
        .await;
}

impl FromLua for TableValue {
    fn from_lua(value: mlua::Value, _lua: &Lua) -> mlua::Result<Self> {
        Ok(match value.to_string() {
            Ok(s) => TableValue::String(s.into()),
            Err(e) => TableValue::Err(TableError::LuaError(Arc::new(e))),
        })
    }
}

impl IntoLua for TableValue {
    fn into_lua(self, lua: &Lua) -> mlua::Result<mlua::Value> {
        match self {
            Self::Empty => mlua::Nil.into_lua(lua),
            Self::String(s) => s.to_string().into_lua(lua),
            Self::Err(e) => match e {
                TableError::LuaError(le) => le.as_ref().to_owned().into_lua(lua),
                TableError::OtherError(e) => e.to_string().into_lua(lua),
            },
        }
    }
}

impl FromLua for CellPos {
    fn from_lua(value: mlua::Value, _lua: &Lua) -> mlua::Result<Self> {
        eprintln!("parsing from lua: {value:?}");
        let err = Err(mlua::Error::FromLuaConversionError {
            from: "",
            to: "CellPos".into(),
            message: Some("CellPos can be created from a string in format [A-Za-z][0-9]".into()),
        });

        let mlua::Value::String(pos) = value else {
            return err;
        };
        let Ok(pos) = pos.to_str() else { return err };
        let Ok(pos) = pos.parse() else { return err };

        Ok(pos)
    }
}
