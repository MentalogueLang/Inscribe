use std::collections::{HashMap, HashSet};

// TODO: Replace string-based module keys with stable module ids once the loader exists.

pub fn detect_cycles(graph: &HashMap<String, Vec<String>>) -> Vec<Vec<String>> {
    let mut visited = HashSet::new();
    let mut stack = Vec::new();
    let mut cycles = Vec::new();

    for node in graph.keys() {
        visit(node, graph, &mut visited, &mut stack, &mut cycles);
    }

    cycles
}

fn visit(
    node: &str,
    graph: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    stack: &mut Vec<String>,
    cycles: &mut Vec<Vec<String>>,
) {
    if let Some(index) = stack.iter().position(|entry| entry == node) {
        cycles.push(stack[index..].to_vec());
        return;
    }

    if !visited.insert(node.to_string()) {
        return;
    }

    stack.push(node.to_string());
    if let Some(edges) = graph.get(node) {
        for edge in edges {
            visit(edge, graph, visited, stack, cycles);
        }
    }
    let _ = stack.pop();
}
