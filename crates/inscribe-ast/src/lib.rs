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
    walk_block, walk_block_mut, walk_block_result, walk_expr, walk_expr_mut, walk_expr_result,
    walk_function, walk_function_mut, walk_function_result, walk_item, walk_item_mut,
    walk_item_result, walk_module, walk_module_mut, walk_module_result, walk_pattern,
    walk_pattern_mut, walk_pattern_result, walk_stmt, walk_stmt_mut, walk_stmt_result,
    ResultVisitor, Visitor, VisitorMut,
};

pub type Ast = Module;

pub fn walk<V: Visitor + ?Sized>(visitor: &mut V, module: &Module) {
    visitor.visit_module(module);
}

pub fn walk_mut<V: VisitorMut + ?Sized>(visitor: &mut V, module: &mut Module) {
    visitor.visit_module(module);
}

pub fn walk_result<V: ResultVisitor + ?Sized>(
    visitor: &mut V,
    module: &Module,
) -> Result<(), V::Error> {
    visitor.visit_module(module)
}

#[cfg(test)]
mod tests {
    use super::{
        walk, walk_mut, walk_result, Expr, ExprKind, FunctionDecl, Item, Module, Param, Position,
        ResultVisitor, Span, Spanned, StructDecl, StructField, TypeRef, Visitor, VisitorMut,
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

    struct Renamer;

    impl VisitorMut for Renamer {
        fn visit_function(&mut self, function: &mut FunctionDecl) {
            function.name.push_str("_checked");
            super::walk_function_mut(self, function);
        }
    }

    struct ParamLimit {
        seen: usize,
        limit: usize,
    }

    impl ResultVisitor for ParamLimit {
        type Error = &'static str;

        fn visit_function(&mut self, function: &FunctionDecl) -> Result<(), Self::Error> {
            self.seen += function.params.len();
            if self.seen > self.limit {
                return Err("too many params");
            }
            super::walk_function_result(self, function)
        }
    }

    #[test]
    fn root_reexports_support_mutable_traversal() {
        let start = Position::new(0, 1, 1);
        let end = Position::new(14, 2, 8);
        let span = Span::new(start, end);
        let mut module = Module {
            items: vec![Item::Function(FunctionDecl {
                receiver: None,
                name: "measure".to_string(),
                params: vec![Param {
                    name: "value".to_string(),
                    ty: None,
                    span,
                }],
                return_type: None,
                body: Some(super::Block {
                    statements: vec![super::Stmt::Expr(Expr::new(
                        ExprKind::Literal(super::Literal::Integer("1".to_string())),
                        span,
                    ))],
                    span,
                }),
                span,
            })],
            span,
        };

        walk_mut(&mut Renamer, &mut module);

        let Item::Function(function) = &module.items[0] else {
            panic!("expected function");
        };
        assert_eq!(function.name, "measure_checked");
    }

    #[test]
    fn root_reexports_support_result_traversal() {
        let start = Position::new(0, 1, 1);
        let end = Position::new(12, 1, 13);
        let span = Span::new(start, end);
        let module = Module {
            items: vec![Item::Function(FunctionDecl {
                receiver: None,
                name: "sum".to_string(),
                params: vec![
                    Param {
                        name: "lhs".to_string(),
                        ty: None,
                        span,
                    },
                    Param {
                        name: "rhs".to_string(),
                        ty: None,
                        span,
                    },
                ],
                return_type: None,
                body: Some(super::Block {
                    statements: vec![super::Stmt::Expr(Expr::new(
                        ExprKind::Literal(super::Literal::Integer("0".to_string())),
                        span,
                    ))],
                    span,
                }),
                span,
            })],
            span,
        };

        assert_eq!(
            walk_result(&mut ParamLimit { seen: 0, limit: 1 }, &module),
            Err("too many params")
        );
    }
}
