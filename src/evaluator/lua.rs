use std::{pin::Pin, sync::Arc};

use mlua::{FromLua, IntoLua, Lua};

use crate::{
    evaluator::{TableError, TableValue, interaction::Communicator},
    table::{cell::CellPos, slice::SlicePos},
};

fn global_cell_access(
    communicator: Communicator,
) -> impl Fn(Lua, (mlua::Value, CellPos)) -> TableLuaBoxFuture {
    move |_, (_, pos): (mlua::Value, CellPos)| {
        Box::pin({
            let mut communicator = communicator.clone();
            async move { communicator.request(pos).await }
        })
    }
}

fn sum_int(communicator: Communicator) -> impl Fn(Lua, SlicePos) -> TableLuaBoxFuture {
    move |_, pos: SlicePos| {
        Box::pin({
            let mut communicator = communicator.clone();
            async move {
                let mut sum: i64 = 0;
                for row in pos.rows() {
                    for column in pos.columns() {
                        let cell = (column, row).into();
                        let Ok(val) = communicator.request(cell).await else {
                            continue;
                        };
                        dbg!(&row, &column, &val);
                        if val.is_err() {
                            return Ok(val);
                        }
                        let TableValue::String(s) = val else {
                            continue;
                        };
                        let Ok(ival) = s.trim().parse::<i64>() else {
                            continue;
                        };
                        sum += ival;
                    }
                }
                Ok(sum.into())
                // Ok(TableValue::String(sum.to_string().into()))
            }
        })
    }
}

fn self_x(communicator: Communicator) -> impl Fn(Lua, ()) -> TableLuaBoxFuture {
    move |_, _| {
        Box::pin({
            let x = communicator.pos().x;
            async move { Ok(x.into()) }
        })
    }
}

fn self_y(communicator: Communicator) -> impl Fn(Lua, ()) -> TableLuaBoxFuture {
    move |_, _| {
        Box::pin({
            let y = communicator.pos().y;
            async move { Ok(y.into()) }
        })
    }
}

type TableLuaBoxFuture = Pin<Box<dyn Future<Output = mlua::Result<TableValue>> + Send + Sync>>;
pub async fn evaluate(source: impl AsRef<str>, communicator: Communicator) {
    let lua = Lua::new();
    let global_cell_access = lua
        .create_async_function(global_cell_access(communicator.clone()))
        .unwrap();

    let sum_int = lua
        .create_async_function(sum_int(communicator.clone()))
        .unwrap();

    let posx = lua
        .create_async_function(self_x(communicator.clone()))
        .unwrap();
    let posy = lua
        .create_async_function(self_y(communicator.clone()))
        .unwrap();

    let metatable = lua.create_table().expect("no error is documented");

    metatable.set("__index", global_cell_access).unwrap();
    lua.globals().set_metatable(Some(metatable)).unwrap();
    lua.globals().set("SUM_INT", sum_int).unwrap();
    lua.globals().set("POSX", posx).unwrap();
    lua.globals().set("POSY", posy).unwrap();

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
            Self::Err(TableError::LuaError(le)) => le.as_ref().to_owned().into_lua(lua),
            _ => self.to_string().into_lua(lua),
        }
    }
}

impl FromLua for CellPos {
    fn from_lua(value: mlua::Value, _lua: &Lua) -> mlua::Result<Self> {
        let err = Err(mlua::Error::FromLuaConversionError {
            from: "",
            to: "CellPos".into(),
            message: Some("CellPos can be created from a string in format [A-Za-z]+[0-9]+".into()),
        });

        let mlua::Value::String(pos) = value else {
            return err;
        };
        let Ok(pos) = pos.to_str() else { return err };
        let Ok(pos) = pos.parse() else { return err };

        Ok(pos)
    }
}

impl FromLua for SlicePos {
    fn from_lua(value: mlua::Value, _lua: &Lua) -> mlua::Result<Self> {
        dbg!(&value);
        let err = Err(mlua::Error::FromLuaConversionError {
            from: "",
            to: "SlicePos".into(),
            message: Some(
                "CellPos can be created from a string in format {CellPos}_{CellPos}".into(),
            ),
        });

        let mlua::Value::String(pos) = value else {
            return err;
        };
        let Ok(pos) = pos.to_str() else { return err };
        let Ok(pos) = pos.parse() else { return err };

        dbg!(&pos);
        Ok(pos)
    }
}
