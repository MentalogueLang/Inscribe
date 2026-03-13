use inscribe_ast::nodes::{Item, Module};
use inscribe_ast::span::Span;

use crate::resolver::FunctionKey;

// TODO: Expand this into a true multi-file module graph once file loading exists.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleTree {
    pub root: ModuleNode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleNode {
    pub name: String,
    pub imports: Vec<ImportNode>,
    pub items: Vec<ItemNode>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportNode {
    pub path: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemNode {
    Struct { name: String, span: Span },
    Function { key: FunctionKey, span: Span },
}

impl ModuleTree {
    pub fn from_module(module: &Module) -> Self {
        let imports = module
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Import(import) => Some(ImportNode {
                    path: import.path.segments.clone(),
                    span: import.span,
                }),
                _ => None,
            })
            .collect();

        let items = module
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Struct(decl) => Some(ItemNode::Struct {
                    name: decl.name.clone(),
                    span: decl.span,
                }),
                Item::Function(function) => Some(ItemNode::Function {
                    key: FunctionKey {
                        receiver: function
                            .receiver
                            .as_ref()
                            .map(|path| path.segments.join(".")),
                        name: function.name.clone(),
                    },
                    span: function.span,
                }),
                Item::Import(_) => None,
            })
            .collect();

        Self {
            root: ModuleNode {
                name: "root".to_string(),
                imports,
                items,
                span: module.span,
            },
        }
    }
}
