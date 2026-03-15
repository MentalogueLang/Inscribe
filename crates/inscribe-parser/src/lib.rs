pub mod expr;
pub mod item;
pub mod parser;
pub mod recovery;
pub mod stmt;

pub use parser::{parse_module, ParseError, Parser};

// TODO: Add convenience entry points that own lexing once the session crate wires phases together.

#[cfg(test)]
mod tests {
    use inscribe_ast::nodes::{ExprKind, Item, PatternKind, Stmt, TypeRefKind, Visibility};
    use inscribe_lexer::lex;

    use crate::parse_module;

    #[test]
    fn parses_structs_methods_and_loops() {
        let source = r#"
import io.file

struct User {
    name: string
    age: int
}

fn User.greet(self) {
    print("Hello " + self.name)
}

fn main() -> int {
    let user = User { name: "Antonio", age: 3 }

    for i in 0..3 {
        user.greet()
    }

    0
}
"#;

        let module = parse_module(lex(source).expect("lexing should succeed"))
            .expect("parsing should succeed");

        assert_eq!(module.items.len(), 4);
        assert!(matches!(module.items[0], Item::Import(_)));
        assert!(matches!(module.items[1], Item::Struct(_)));

        let Item::Function(method) = &module.items[2] else {
            panic!("expected method item");
        };
        assert!(method.receiver.is_some());
        assert_eq!(method.name, "greet");
        let method_body = method.body.as_ref().expect("method should have a body");
        let Stmt::Expr(method_expr) = &method_body.statements[0] else {
            panic!("expected method body expression");
        };
        assert!(matches!(method_expr.kind, ExprKind::Call { .. }));
        let ExprKind::Call { args, .. } = &method_expr.kind else {
            unreachable!("checked above");
        };
        assert!(matches!(
            args.first().map(|expr| &expr.kind),
            Some(ExprKind::Binary { .. })
        ));
        let ExprKind::Binary { right, .. } = &args[0].kind else {
            unreachable!("checked above");
        };
        assert!(matches!(right.kind, ExprKind::Field { .. }));

        let Item::Function(main_fn) = &module.items[3] else {
            panic!("expected main function");
        };
        let body = main_fn.body.as_ref().expect("main should have a body");
        assert_eq!(body.statements.len(), 3);
        let Stmt::For(for_stmt) = &body.statements[1] else {
            panic!("expected for loop");
        };
        let Stmt::Expr(loop_expr) = &for_stmt.body.statements[0] else {
            panic!("expected loop body expression");
        };
        let ExprKind::Call { callee, .. } = &loop_expr.kind else {
            panic!("expected method call");
        };
        assert!(matches!(callee.kind, ExprKind::Field { .. }));
    }

    #[test]
    fn parses_match_and_try_syntax() {
        let source = r#"
fn main() {
    data = readFile("data.txt") ?

    match data {
        Ok(value) => print(value)
        Err(error) => print(error)
    }
}
"#;

        let module = parse_module(lex(source).expect("lexing should succeed"))
            .expect("parsing should succeed");

        let Item::Function(main_fn) = &module.items[0] else {
            panic!("expected main function");
        };
        let body = main_fn.body.as_ref().expect("main should have a body");

        match &body.statements[0] {
            Stmt::Expr(expr) => assert!(matches!(expr.kind, ExprKind::Binary { .. })),
            _ => panic!("expected assignment expression"),
        }

        match &body.statements[1] {
            Stmt::Expr(expr) => match &expr.kind {
                ExprKind::Match { arms, .. } => {
                    assert_eq!(arms.len(), 2);
                    assert!(matches!(
                        arms[0].pattern.kind,
                        PatternKind::Constructor { .. }
                    ));
                }
                _ => panic!("expected match expression"),
            },
            _ => panic!("expected match statement"),
        }
    }

    #[test]
    fn parses_priv_functions() {
        let source = r#"
priv fn helper() -> int {
    7
}
"#;

        let module = parse_module(lex(source).expect("lexing should succeed"))
            .expect("parsing should succeed");

        let Item::Function(function) = &module.items[0] else {
            panic!("expected function");
        };
        assert_eq!(function.visibility, Visibility::Private);
        assert_eq!(function.name, "helper");
    }

    #[test]
    fn parses_enums_arrays_and_indexing() {
        let source = r#"
enum Kind {
    Object
    Array
}

fn main() {
    let nums: [int; 3] = [1, 2, 3]
    let fill: [byte; 4] = [0; 4]
    nums[1]
}
"#;

        let module = parse_module(lex(source).expect("lexing should succeed"))
            .expect("parsing should succeed");

        assert!(matches!(module.items[0], Item::Enum(_)));
        let Item::Function(main_fn) = &module.items[1] else {
            panic!("expected function");
        };
        let body = main_fn.body.as_ref().expect("main should have a body");
        let Stmt::Let(nums) = &body.statements[0] else {
            panic!("expected array binding");
        };
        let Some(ty) = &nums.ty else {
            panic!("expected array annotation");
        };
        assert!(matches!(ty.kind, TypeRefKind::Array { length: 3, .. }));
        assert!(matches!(nums.value.kind, ExprKind::Array(_)));
        let Stmt::Let(fill) = &body.statements[1] else {
            panic!("expected repeat binding");
        };
        assert!(matches!(fill.value.kind, ExprKind::RepeatArray { length: 4, .. }));
        let Stmt::Expr(expr) = &body.statements[2] else {
            panic!("expected index expression");
        };
        assert!(matches!(expr.kind, ExprKind::Index { .. }));
    }
}
