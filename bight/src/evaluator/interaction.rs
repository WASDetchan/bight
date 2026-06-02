use std::collections::HashMap;

use crate::sync::RcStr;

use tokio::sync::Mutex;

use crate::{
    evaluator::{EvaluationError, TableValue, ValueTable},
    table::{HashTable, cell::CellPos},
};

use super::{CacheTable, GraphTable};
#[derive(Debug)]
pub struct CellInfo<'a> {
    source: &'a RcStr,
    pos: CellPos,
    dep_tables: &'a Mutex<(GraphTable, GraphTable)>,
    cache_table: &'a CacheTable,
    result_table: &'a ValueTable,
}

impl<'a> CellInfo<'a> {
    pub fn new(
        source: &'a RcStr,
        pos: CellPos,
        dep_tables: &'a Mutex<(GraphTable, GraphTable)>,
        cache_table: &'a CacheTable,
        result_table: &'a ValueTable,
    ) -> Self {
        Self {
            source,
            pos,
            dep_tables,
            cache_table,
            result_table,
        }
    }
    pub fn pos(&self) -> CellPos {
        self.pos
    }
    pub fn source(&self) -> &RcStr {
        self.source
    }
    pub async fn get(&self, req: CellPos) -> Result<TableValue, EvaluationError> {
        log::debug!("ValueRequest for {} by {}", req, self.pos);

        let mut dep_tables = self.dep_tables.lock().await;
        dep_tables.0.entry(self.pos).or_default().insert(req);
        dep_tables.1.entry(req).or_default().insert(self.pos);

        log::trace!(
            "dependencies: {:?};\n required_by: {:?};",
            dep_tables.0,
            dep_tables.1,
        );
        if has_dependency_cycle(&dep_tables.0, self.pos, &mut HashMap::new()) {
            log::warn!("Dependency cycle starting at {} detected!", self.pos);
            return Err(EvaluationError::DependencyCycle);
        }

        drop(dep_tables);

        if let Some(value) = self.result_table.get(&req) {
            log::trace!(
                "Value for {} requested by {} is immediately available: {value}",
                req,
                self.pos
            );
            return Ok(value.clone());
        }

        let Some(cache) = self.cache_table.get(&req) else {
            log::trace!("Value for {} requested by {} is empty", req, self.pos);
            return Ok(TableValue::Empty);
        };

        let value = cache
            .read()
            .await
            .clone()
            .expect("WriteGuard on the cache can only be dropped after the cache is evaluated");
        log::trace!(
            "Awaited Value for {} requested by {}: {value}",
            req,
            self.pos
        );
        Ok(value)
    }
}

enum Vertex {
    Visited,
    Parent,
}
fn has_dependency_cycle(
    dependencies: &GraphTable,
    start: CellPos,
    visited: &mut HashTable<Vertex>,
) -> bool {
    match visited.get(&start) {
        Some(Vertex::Parent) => true,
        Some(Vertex::Visited) => false,
        None => {
            visited.insert(start, Vertex::Parent);
            for &dep in dependencies.get(&start).into_iter().flatten() {
                if has_dependency_cycle(dependencies, dep, visited) {
                    return true;
                }
            }
            visited.insert(start, Vertex::Visited);
            false
        }
    }
}
