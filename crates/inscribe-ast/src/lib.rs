pub mod nodes;
pub mod span;
pub mod visit;

pub use nodes::{
    BinaryOp, Block, ConstStmt, Expr, ExprKind, ForStmt, FunctionDecl, Import, Item, LetStmt,
    Literal, MatchArm, Module, Param, Path, Pattern, PatternKind, ReturnStmt, Stmt, StructDecl,
    StructField, StructLiteralField, TypeRef, UnaryOp, WhileStmt,
};
pub use span::{Position, Span, Spanned};
pub use visit::{
    walk_block, walk_expr, walk_function, walk_item, walk_module, walk_pattern, walk_stmt, Visitor,
};

pub type Ast = Module;

pub fn walk<V: Visitor + ?Sized>(visitor: &mut V, module: &Module) {
    visitor.visit_module(module);
}

#[cfg(test)]
mod tests {
    use super::{
        walk, Item, Module, Position, Span, Spanned, StructDecl, StructField, TypeRef, Visitor,
    };

    struct ItemCounter {
        items: usize,
    }

    impl Visitor for ItemCounter {
        fn visit_item(&mut self, item: &Item) {
            self.items += 1;
            super::walk_item(self, item);
        }
    }

    #[test]
    fn root_reexports_support_basic_traversal() {
        let start = Position::new(0, 1, 1);
        let end = Position::new(6, 1, 7);
        let span = Span::new(start, end);
        let module = Module {
            items: vec![Item::Struct(StructDecl {
                name: "Point".to_string(),
                fields: vec![StructField {
                    name: "x".to_string(),
                    ty: TypeRef {
                        path: super::Path::new(vec!["int".to_string()], span),
                        arguments: Vec::new(),
                        span,
                    },
                    span,
                }],
                span,
            })],
            span,
        };

        let mut counter = ItemCounter { items: 0 };
        walk(&mut counter, &module);

        assert_eq!(counter.items, 1);
        assert_eq!(module.span(), span);
    }
}
