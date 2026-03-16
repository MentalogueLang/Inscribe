use std::fmt;
use std::hash::Hash;

use serde::{de::DeserializeOwned, Serialize};

use crate::cache::{Cache, DiskCache};
use crate::dep_graph::{DepGraph, DepNode};
use crate::fingerprint::Fingerprint;

#[derive(Debug, Clone)]
pub struct QueryError {
    pub message: String,
}

impl QueryError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "query error: {}", self.message)
    }
}

impl std::error::Error for QueryError {}

#[derive(Debug, Default)]
pub struct QueryEngine {
    dep_graph: DepGraph,
    stack: Vec<DepNode>,
}

impl QueryEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn dep_graph(&self) -> &DepGraph {
        &self.dep_graph
    }

    pub fn dep_graph_mut(&mut self) -> &mut DepGraph {
        &mut self.dep_graph
    }

    pub fn execute<K, V, F>(
        &mut self,
        cache: &mut Cache<K, V>,
        query: &'static str,
        key: K,
        input_fingerprint: Fingerprint,
        compute: F,
    ) -> Result<V, QueryError>
    where
        K: Eq + Hash + Clone,
        V: Clone,
        F: FnOnce(&mut QueryEngine) -> Result<V, QueryError>,
    {
        let node = DepNode::new(query, Fingerprint::of(&key));
        if let Some(parent) = self.stack.last().copied() {
            self.dep_graph.add_dependency(parent, node);
        }

        if self.stack.contains(&node) {
            return Err(QueryError::new(format!(
                "cycle detected while evaluating `{}`",
                query
            )));
        }

        if let Some(value) = cache.get_if_fresh(&key, input_fingerprint) {
            return Ok(value.clone());
        }

        self.dep_graph.clear_dependencies(node);
        self.stack.push(node);
        let result = compute(self);
        self.stack.pop();

        match result {
            Ok(value) => {
                cache.insert(key, input_fingerprint, value.clone());
                Ok(value)
            }
            Err(error) => Err(error),
        }
    }

    pub fn execute_with_disk<K, V, F>(
        &mut self,
        cache: &mut Cache<K, V>,
        disk: Option<&DiskCache>,
        query: &'static str,
        key: K,
        input_fingerprint: Fingerprint,
        compute: F,
    ) -> Result<V, QueryError>
    where
        K: Eq + Hash + Clone,
        V: Clone + Serialize + DeserializeOwned,
        F: FnOnce(&mut QueryEngine) -> Result<V, QueryError>,
    {
        if let Some(disk) = disk {
            match disk.load_if_fresh::<K, V>(&key, input_fingerprint) {
                Ok(Some(value)) => {
                    cache.insert(key.clone(), input_fingerprint, value.clone());
                    return Ok(value);
                }
                Ok(None) => {}
                Err(error) => {
                    return Err(QueryError::new(format!(
                        "failed to read disk cache for `{}`: {error}",
                        query
                    )))
                }
            }
        }

        let result = self.execute(cache, query, key.clone(), input_fingerprint, compute)?;

        if let Some(disk) = disk {
            if let Err(error) = disk.store(&key, input_fingerprint, &result) {
                return Err(QueryError::new(format!(
                    "failed to write disk cache for `{}`: {error}",
                    query
                )));
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::{QueryEngine, QueryError};
    use crate::cache::Cache;
    use crate::fingerprint::Fingerprint;

    #[test]
    fn caches_query_results() -> Result<(), QueryError> {
        let mut engine = QueryEngine::new();
        let mut cache = Cache::default();

        let key = "input";
        let fingerprint = Fingerprint::of(&"v1");
        let mut runs = 0;

        let first = engine.execute(&mut cache, "test", key, fingerprint, |engine| {
            let _ = engine;
            runs += 1;
            Ok(10)
        })?;
        let second = engine.execute(&mut cache, "test", key, fingerprint, |engine| {
            let _ = engine;
            runs += 1;
            Ok(20)
        })?;

        assert_eq!(first, 10);
        assert_eq!(second, 10);
        assert_eq!(runs, 1);
        Ok(())
    }
}
