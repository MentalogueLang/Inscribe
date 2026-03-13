pub mod expr;
pub mod item;
pub mod parser;
pub mod recovery;
pub mod stmt;

pub use parser::{parse_module, ParseError, Parser};

// TODO: Add convenience entry points that own lexing once the session crate wires phases together.

#[cfg(test)]
mod tests {
    use inscribe_ast::nodes::{ExprKind, Item, PatternKind, Stmt};
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

        let Item::Function(main_fn) = &module.items[3] else {
            panic!("expected main function");
        };
        let body = main_fn.body.as_ref().expect("main should have a body");
        assert_eq!(body.statements.len(), 3);
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
}
