use std::{marker::PhantomData, pin::Pin, sync::Arc};

use mlua::{FromLua, IntoLua, Lua};

use crate::{
    evaluator::{EvalationError, TableError, TableValue, interaction::CellInfo},
    table::{cell::CellPos, slice::SlicePos},
};

type TableLuaBoxFuture<'a> =
    Pin<Box<dyn Future<Output = mlua::Result<TableValue>> + Send + Sync + 'a>>;
fn global_cell_access<'a>(
    info: &'a CellInfo<'a>,
) -> impl Fn(Lua, (mlua::Value, CellPos)) -> TableLuaBoxFuture<'a> {
    move |_, (_, pos): (mlua::Value, CellPos)| {
        Box::pin(async move { Ok(info.get(pos).await.into()) })
    }
}

fn sum<'a>(info: &'a CellInfo<'a>) -> impl Fn(Lua, SlicePos) -> TableLuaBoxFuture<'a> {
    move |_lua, pos: SlicePos| {
        Box::pin({
            async move {
                let mut sum: f64 = 0.0;
                for row in pos.rows() {
                    for column in pos.columns() {
                        let cell = (column, row).into();
                        let res = info.get(cell).await;
                        let Ok(val) = res else {
                            return Ok(res.into());
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

fn rel_cell<'a>(info: &'a CellInfo<'a>) -> impl Fn(Lua, (i64, i64)) -> TableLuaBoxFuture<'a> {
    move |_lua, (shx, shy)| {
        Box::pin({
            async move {
                let x = info.pos().x as i64 + shx;
                let y = info.pos().y as i64 + shy;
                if x < 0 || y < 0 {
                    Ok(TableValue::Empty)
                } else {
                    Ok(info.get((x as usize, y as usize).into()).await.into())
                }
            }
        })
    }
}

fn self_x<'a>(info: &'a CellInfo<'a>) -> impl Fn(Lua, ()) -> TableLuaBoxFuture<'a> {
    move |_lua, _| {
        Box::pin({
            let x = info.pos().x;
            async move { Ok(TableValue::from_number(x as f64)) }
        })
    }
}

fn self_y<'a>(info: &'a CellInfo<'a>) -> impl Fn(Lua, ()) -> TableLuaBoxFuture<'a> {
    move |_lua, _| {
        Box::pin({
            let y = info.pos().y;
            async move { Ok(TableValue::from_number(y as f64)) }
        })
    }
}

pub struct CellEvaluator<'a> {
    lua: Lua,
    info: &'static CellInfo<'static>,
    _phantom_info: PhantomData<&'a CellInfo<'a>>,
}

impl<'a> CellEvaluator<'a> {
    fn new(info: &'a CellInfo<'a>, lua: Lua) -> Self {
        let info: &'static CellInfo<'static> = unsafe { &*(info as *const CellInfo) };

        Self {
            lua,
            info,
            _phantom_info: PhantomData,
        }
    }
    fn add_global_fn(
        &mut self,
        name: &str,
        f: impl Fn(&'static CellInfo<'static>) -> TableLuaBoxFuture<'static>,
    ) {
        let f = self.lua.create_async_function(f(self.info)).unwrap();
        self.lua.globals().set(name, f).unwrap();
    }

    async fn evaluate(&mut self, source: &str) -> mlua::Result<TableValue> {
        let global_cell_access = self
            .lua
            .create_async_function(global_cell_access(self.info))
            .unwrap();
        let metatable = self.lua.create_table().expect("no error is documented");
        metatable.set("__index", global_cell_access).unwrap();
        self.lua.globals().set_metatable(Some(metatable)).unwrap();

        let chunk = self.lua.load(source);
        let res = chunk.eval_async::<TableValue>().await;
        res
    }
}

pub async fn evaluate<'a>(source: &str, info: &'a CellInfo<'a>) -> TableValue {
    let mut ev = CellEvaluator::new(info, Lua::new());

    ev.add_global_fn("SUM", sum);
    ev.add_global_fn("POSX", self_x);
    ev.add_global_fn("POSY", self_y);
    ev.add_global_fn("REL", rel_cell);

    let res = ev.evaluate(source).await;

    res.unwrap_or_else(|err| TableValue::Err(TableError::LuaError(Arc::new(err))))
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
