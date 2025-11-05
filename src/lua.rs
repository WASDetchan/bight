use std::{collections::HashSet, thread};

use futures::future::join_all;
use mlua::{FromLua, Lua};
use tokio::sync::{mpsc, oneshot};

use crate::table::{DataTable, Table, cell::CellPos, slice::table::TableSlice};

const REQUEST_BUFFER: usize = 10;

#[derive(Debug, Default)]
pub enum Cache<T> {
    #[default]
    Invalid,
    Valid(T),
}

impl<T> Cache<T> {
    pub fn is_valid(&self) -> bool {
        !matches!(self, Self::Invalid)
    }
}

pub type SourceTable = DataTable<String>;
pub type Value = String;
pub type CacheTable = DataTable<Cache<Value>>;
pub type DependencyTable = DataTable<Vec<oneshot::Sender<Value>>>;
pub type RequiredByTable = DataTable<HashSet<CellPos>>;

#[derive(Default, Debug)]
pub struct LuaTable {
    source: SourceTable,
    cahce: CacheTable,
    required_by: RequiredByTable,
}

#[derive(Debug)]
pub struct ValueRequest {
    pub cell: CellPos,
    pub channel: oneshot::Sender<Value>,
}

#[derive(Debug)]
pub struct ValueResponse {
    pub cell: CellPos,
    pub value: Value,
}

#[derive(Debug)]
pub enum ValueMessage {
    Req(ValueRequest),
    Res(ValueResponse),
}

impl LuaTable {
    pub fn new(source: SourceTable) -> Self {
        Self {
            cahce: if let Some(slice) = source.full_slice() {
                Self::new_cache(slice)
            } else {
                CacheTable::new()
            },
            source,
            required_by: RequiredByTable::default(),
        }
    }
    pub fn set_source(&mut self, pos: impl Into<CellPos>, src: Option<String>) {
        let pos = pos.into();
        self.source.set(pos, src);
        self.invalidate_cache(pos);
    }
    pub fn get_source(&mut self, pos: impl Into<CellPos>) -> Option<&String> {
        let pos = pos.into();
        self.source.get(pos)
    }
    fn new_cache(source: TableSlice<'_, SourceTable>) -> CacheTable {
        let mut res = CacheTable::new();
        for row in source.row_indexes() {
            for col in source.col_indexes() {
                if source.get((col, row)).is_some() {
                    res.set((col, row).into(), Some(Cache::Invalid));
                }
            }
        }
        res
    }
    fn invalidate_cache(&mut self, pos: impl Into<CellPos>) {
        let pos = pos.into();
        if self.cahce.get(pos).is_none_or(|c| c.is_valid()) {
            self.cahce.set(pos, Some(Cache::Invalid));
            if let Some(req_slice) = self.required_by.full_slice() {
                let rows = req_slice.row_indexes();
                let cols = req_slice.col_indexes();
                for row in rows {
                    let cols = cols.clone();
                    for col in cols {
                        if let Some(set) = self.required_by.get_mut((col, row).into()) {
                            set.remove(&pos);
                        }
                    }
                }
            }
            if let Some(set) = self.required_by.get(pos) {
                for req in set.clone() {
                    self.invalidate_cache(req);
                }
            }
        }
        self.cahce.set(pos, Some(Cache::Invalid));
    }
    pub fn cache(&mut self) {
        let mut invalid_cells = Vec::new();
        if let Some(cahce_slice) = self.cahce.full_slice() {
            dbg!(&cahce_slice);
            let rows = cahce_slice.row_indexes();
            let cols = cahce_slice.col_indexes();
            for row in rows {
                let cols = cols.clone();
                for col in cols {
                    let pos = (col, row).into();
                    if self.cahce.get(pos).is_some_and(|c| !c.is_valid()) {
                        invalid_cells.push(pos);
                    }
                }
            }
        }
        let (req_send, mut req_recv) = mpsc::channel::<ValueMessage>(REQUEST_BUFFER);

        async fn request_cell(sender: mpsc::Sender<ValueMessage>, pos: CellPos) -> Option<Value> {
            let (send, recv) = oneshot::channel();
            let req = ValueMessage::Req(ValueRequest {
                channel: send,
                cell: pos,
            });
            _ = sender.send(req);

            recv.await.ok()
        }

        let eval_cell =
            async |sender: mpsc::Sender<ValueMessage>, pos: CellPos, src: Option<String>| {
                let request_fn = {
                    let sender = sender.clone();
                    move |pos| request_cell(sender.clone(), pos)
                };
                if let Some(src) = src {
                    sender
                        .send(ValueMessage::Res(ValueResponse {
                            cell: pos,
                            value: evaluate(&src, request_fn).await,
                        }))
                        .await
                        .unwrap();
                } else {
                    sender
                        .send(ValueMessage::Res(ValueResponse {
                            cell: pos,
                            value: "".into(),
                        }))
                        .await
                        .unwrap();
                }
            };

        dbg!(&self.source);
        dbg!(&self.cahce);
        dbg!(&invalid_cells);
        let futures: Vec<_> = invalid_cells
            .into_iter()
            .map(|cell| eval_cell(req_send.clone(), cell, self.source.get(cell).cloned()))
            .collect();

        eprintln!("prepared {} futures", futures.len());
        let handle = thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(join_all(futures))
        });
        eprintln!("spawned rt thread");
        let mut dependencies = DependencyTable::new();

        drop(req_send);

        eprintln!("joined");
        while let Some(msg) = req_recv.blocking_recv() {
            eprintln!("msg: {msg:?}");
            match msg {
                ValueMessage::Req(ValueRequest { cell, channel }) => {
                    if let Some(Cache::Valid(v)) = self.cahce.get(cell) {
                        channel.send(v.clone()).unwrap();
                    } else if let Some(d) = dependencies.get_mut(cell) {
                        d.push(channel);
                    } else {
                        dependencies.set(cell, Some(vec![channel]));
                    }
                }

                ValueMessage::Res(ValueResponse { cell, value }) => {
                    if let Some(d) = dependencies.get_mut(cell) {
                        let deps = std::mem::take(d);
                        for channel in deps {
                            channel.send(value.clone()).unwrap();
                        }
                    }
                    self.cahce.set(cell, Some(Cache::Valid(value)));
                }
            }
        }

        eprintln!("ready to join");

        _ = handle.join();
    }
}

impl FromLua for CellPos {
    fn from_lua(value: mlua::Value, _lua: &Lua) -> mlua::Result<Self> {
        let err = Err(mlua::Error::FromLuaConversionError {
            from: "",
            to: "CellPos".into(),
            message: Some("CellPos can bre create from a table with fields x and y defined and convertable to non-negative integers".into()),
        });

        let mlua::Value::Table(pos) = value else {
            return err;
        };
        let Ok(x) = pos.get("x") else {
            return err;
        };
        let Ok(y) = pos.get("y") else {
            return err;
        };

        Ok((x, y).into())
    }
}

impl Table for LuaTable {
    type Item = Value;
    fn get(&self, pos: CellPos) -> Option<&Self::Item> {
        let cache = self.cahce.get(pos)?;
        match cache {
            Cache::Invalid => panic!("Cache must be fresh before queries to the table!"),
            Cache::Valid(s) => Some(s),
        }
    }
    fn get_mut(&mut self, _pos: CellPos) -> Option<&mut Self::Item> {
        unimplemented!("Table trait should be separated into Tabel and TableMut");
    }
    fn set(&mut self, _pos: CellPos, _item: Option<Self::Item>) {
        unimplemented!("Table trait should be separated into Tabel and TableMut");
    }
}

pub async fn evaluate<FUT: Future<Output = Option<Value>> + Send + 'static>(
    source: &str,
    request_fn: impl Fn(CellPos) -> FUT + Send + 'static + Clone,
) -> Value {
    let lua = Lua::new();
    let f = lua
        .create_async_function(move |_, pos| {
            let fut = request_fn(pos);
            async move { fut.await.ok_or_else(|| todo!("error handling")) }
        }) // TODO: handle the errors properly
        .unwrap();
    lua.globals().set("f", f).unwrap();
    let chunk = lua.load(source);
    let res = chunk.eval_async::<Value>().await;

    match res {
        Err(e) => format!("#ERR: {e}"),
        Ok(v) => v.to_string(),
    }
}
