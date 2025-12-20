use std::{pin::Pin, sync::Arc};

use mlua::{FromLua, IntoLua, Lua};

use crate::{
    evaluator::{TableError, TableValue, interaction::Communicator},
    table::{cell::CellPos, slice::SlicePos},
};

type TableLuaBoxFuture = Pin<Box<dyn Future<Output = mlua::Result<TableValue>> + Send + Sync>>;
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

fn sum(communicator: Communicator) -> impl Fn(Lua, SlicePos) -> TableLuaBoxFuture {
    move |_lua, pos: SlicePos| {
        Box::pin({
            let mut communicator = communicator.clone();
            async move {
                let mut sum: f64 = 0.0;
                for row in pos.rows() {
                    for column in pos.columns() {
                        let cell = (column, row).into();
                        let Ok(val) = communicator.request(cell).await else {
                            continue;
                        };
                        if val.is_err() {
                            return Ok(val);
                        }
                        let TableValue::Number(val) = val else {
                            continue;
                        };
                        sum += val;
                    }
                }
                Ok(TableValue::from_number(sum))
            }
        })
    }
}

fn rel_cell(communicator: Communicator) -> impl Fn(Lua, (i64, i64)) -> TableLuaBoxFuture {
    move |_lua, (shx, shy)| {
        Box::pin({
            let mut communicator = communicator.clone();
            async move {
                let x = communicator.pos().x as i64 + shx;
                let y = communicator.pos().y as i64 + shy;
                if x < 0 || y < 0 {
                    Ok(TableValue::Empty)
                } else {
                    communicator.request((x as usize, y as usize).into()).await
                }
            }
        })
    }
}

fn self_x(communicator: Communicator) -> impl Fn(Lua, ()) -> TableLuaBoxFuture {
    move |_lua, _| {
        Box::pin({
            let x = communicator.pos().x;
            async move { Ok(TableValue::from_number(x as f64)) }
        })
    }
}

fn self_y(communicator: Communicator) -> impl Fn(Lua, ()) -> TableLuaBoxFuture {
    move |_lua, _| {
        Box::pin({
            let y = communicator.pos().y;
            async move { Ok(TableValue::from_number(y as f64)) }
        })
    }
}

pub async fn evaluate(source: impl AsRef<str>, communicator: Communicator) {
    let lua = Lua::new();
    let global_cell_access = lua
        .create_async_function(global_cell_access(communicator.clone()))
        .unwrap();

    let sum = lua
        .create_async_function(sum(communicator.clone()))
        .unwrap();

    let posx = lua
        .create_async_function(self_x(communicator.clone()))
        .unwrap();
    let posy = lua
        .create_async_function(self_y(communicator.clone()))
        .unwrap();

    let rel = lua
        .create_async_function(rel_cell(communicator.clone()))
        .unwrap();

    let metatable = lua.create_table().expect("no error is documented");

    metatable.set("__index", global_cell_access).unwrap();
    lua.globals().set_metatable(Some(metatable)).unwrap();
    lua.globals().set("SUM", sum).unwrap();
    lua.globals().set("POSX", posx).unwrap();
    lua.globals().set("POSY", posy).unwrap();
    lua.globals().set("REL", rel).unwrap();

    let chunk = lua.load(source.as_ref());
    let res = chunk.eval_async::<TableValue>().await;
    communicator
        .respond(res.unwrap_or_else(|err| TableValue::Err(TableError::LuaError(Arc::new(err)))))
        .await;
}

impl FromLua for TableValue {
    fn from_lua(value: mlua::Value, _lua: &Lua) -> mlua::Result<Self> {
        use mlua::Value::{Integer, Number};
        match value {
            Number(n) => Ok(TableValue::Number(n)),
            Integer(n) => Ok(TableValue::Number(n as f64)),
            _ => match value.to_string() {
                Ok(s) => Ok(TableValue::from_stringable(s)),
                Err(e) => Ok(TableValue::lua_error(e)),
            },
        }
    }
}

impl IntoLua for TableValue {
    fn into_lua(self, lua: &Lua) -> mlua::Result<mlua::Value> {
        match self {
            Self::Empty => mlua::Nil.into_lua(lua),
            Self::Text(s) => s.to_string().into_lua(lua),
            Self::Err(TableError::LuaError(le)) => le.as_ref().to_owned().into_lua(lua),
            Self::Number(value) => Ok(value.into_lua(lua).expect("Failed to conver f64 to lua")),
            Self::Err(e) => e.to_string().into_lua(lua),
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

        Ok(pos)
    }
}
