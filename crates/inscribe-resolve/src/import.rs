use std::collections::HashMap;

use inscribe_ast::nodes::{Item, Module};
use inscribe_ast::span::Span;
use serde::{Deserialize, Serialize};

use crate::resolver::ResolveError;

// TODO: Support aliases and selective imports once the language grows beyond whole-module imports.

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ImportEntry {
    pub alias: String,
    pub path: Vec<String>,
    pub span: Span,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct ImportTable {
    entries: HashMap<String, ImportEntry>,
}

impl ImportTable {
    pub fn from_module(module: &Module) -> Result<Self, Vec<ResolveError>> {
        let mut table = Self::default();
        let mut errors = Vec::new();

        for item in &module.items {
            let Item::Import(import) = item else {
                continue;
            };

            let alias = import
                .path
                .segments
                .last()
                .cloned()
                .unwrap_or_else(|| "root".to_string());

            let entry = ImportEntry {
                alias: alias.clone(),
                path: import.path.segments.clone(),
                span: import.span,
            };

            if table.entries.insert(alias.clone(), entry).is_some() {
                errors.push(ResolveError::new(
                    format!("duplicate import alias `{alias}`"),
                    import.span,
                ));
            }
        }

        if errors.is_empty() {
            Ok(table)
        } else {
            Err(errors)
        }
    }

    pub fn get(&self, alias: &str) -> Option<&ImportEntry> {
        self.entries.get(alias)
    }

    pub fn contains_alias(&self, alias: &str) -> bool {
        self.entries.contains_key(alias)
    }

    pub fn entries(&self) -> impl Iterator<Item = &ImportEntry> {
        self.entries.values()
    }
}
