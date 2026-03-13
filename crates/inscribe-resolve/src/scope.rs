use std::collections::HashMap;

// TODO: Track declaration spans per scope entry once diagnostics need shadowing notes.

#[derive(Debug, Clone, Default)]
pub struct ScopeStack<T> {
    scopes: Vec<HashMap<String, T>>,
}

impl<T: Clone> ScopeStack<T> {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn push(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop(&mut self) {
        if self.scopes.len() > 1 {
            let _ = self.scopes.pop();
        }
    }

    pub fn define(&mut self, name: String, value: T) -> bool {
        let scope = self
            .scopes
            .last_mut()
            .expect("scope stack should always have at least one scope");
        if scope.contains_key(&name) {
            false
        } else {
            scope.insert(name, value);
            true
        }
    }

    pub fn lookup(&self, name: &str) -> Option<T> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).cloned())
    }
}
