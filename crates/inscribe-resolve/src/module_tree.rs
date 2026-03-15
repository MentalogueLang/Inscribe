use inscribe_ast::nodes::{Item, Module, Visibility};
use inscribe_ast::span::Span;
use std::path::PathBuf;

use crate::resolver::FunctionKey;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleTree {
    pub entry: PathBuf,
    pub modules: Vec<ModuleNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleNode {
    pub name: String,
    pub path: PathBuf,
    pub imports: Vec<ImportNode>,
    pub items: Vec<ItemNode>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportNode {
    pub path: Vec<String>,
    pub resolved_path: Option<PathBuf>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemNode {
    Struct { name: String, span: Span },
    Function {
        key: FunctionKey,
        visibility: Visibility,
        span: Span,
    },
}

impl ModuleTree {
    pub fn from_module(module: &Module) -> Self {
        let imports = module
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Import(import) => Some(ImportNode {
                    path: import.path.segments.clone(),
                    resolved_path: None,
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
                    visibility: function.visibility,
                    span: function.span,
                }),
                Item::Import(_) => None,
            })
            .collect();

        Self {
            entry: PathBuf::from("<memory>"),
            modules: vec![ModuleNode {
                name: "root".to_string(),
                path: PathBuf::from("<memory>"),
                imports,
                items,
                span: module.span,
            }],
        }
    }

    pub fn from_nodes(entry: PathBuf, modules: Vec<ModuleNode>) -> Self {
        Self { entry, modules }
    }
}
