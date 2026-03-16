use std::collections::{HashMap, HashSet};

use crate::fingerprint::Fingerprint;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DepNode {
    pub query: &'static str,
    pub key: Fingerprint,
}

impl DepNode {
    pub fn new(query: &'static str, key: Fingerprint) -> Self {
        Self { query, key }
    }
}

#[derive(Debug, Default)]
pub struct DepGraph {
    dependencies: HashMap<DepNode, HashSet<DepNode>>,
    dependents: HashMap<DepNode, HashSet<DepNode>>,
}

impl DepGraph {
    pub fn add_dependency(&mut self, node: DepNode, depends_on: DepNode) {
        self.dependencies
            .entry(node)
            .or_default()
            .insert(depends_on);
        self.dependents.entry(depends_on).or_default().insert(node);
    }

    pub fn dependencies_of(&self, node: DepNode) -> impl Iterator<Item = &DepNode> {
        self.dependencies
            .get(&node)
            .into_iter()
            .flat_map(|set| set.iter())
    }

    pub fn dependents_of(&self, node: DepNode) -> impl Iterator<Item = &DepNode> {
        self.dependents
            .get(&node)
            .into_iter()
            .flat_map(|set| set.iter())
    }

    pub fn clear_dependencies(&mut self, node: DepNode) {
        if let Some(deps) = self.dependencies.remove(&node) {
            for dep in deps {
                if let Some(set) = self.dependents.get_mut(&dep) {
                    set.remove(&node);
                    if set.is_empty() {
                        self.dependents.remove(&dep);
                    }
                }
            }
        }
    }

    pub fn remove_node(&mut self, node: DepNode) {
        self.clear_dependencies(node);
        if let Some(dependents) = self.dependents.remove(&node) {
            for dependent in dependents {
                if let Some(set) = self.dependencies.get_mut(&dependent) {
                    set.remove(&node);
                    if set.is_empty() {
                        self.dependencies.remove(&dependent);
                    }
                }
            }
        }
    }

    pub fn transitive_dependents(&self, root: DepNode) -> HashSet<DepNode> {
        let mut visited = HashSet::new();
        let mut stack = Vec::new();

        stack.extend(self.dependents_of(root).copied());

        while let Some(node) = stack.pop() {
            if visited.insert(node) {
                stack.extend(self.dependents_of(node).copied());
            }
        }

        visited
    }
}

#[cfg(test)]
mod tests {
    use super::{DepGraph, DepNode};
    use crate::fingerprint::Fingerprint;

    #[test]
    fn records_dependencies() {
        let mut graph = DepGraph::default();
        let root = DepNode::new("root", Fingerprint::of(&1u64));
        let leaf = DepNode::new("leaf", Fingerprint::of(&2u64));
        graph.add_dependency(root, leaf);

        let deps = graph.dependencies_of(root).copied().collect::<Vec<_>>();
        assert_eq!(deps, vec![leaf]);
        let dependents = graph.dependents_of(leaf).copied().collect::<Vec<_>>();
        assert_eq!(dependents, vec![root]);
    }
}
