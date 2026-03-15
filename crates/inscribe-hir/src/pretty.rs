use crate::nodes::{HirExpr, HirExprKind, HirItem, HirProgram, HirStmt};

// TODO: Replace this with a configurable pretty-printer once diagnostics want stable formatting.

pub fn render(program: &HirProgram) -> String {
    let mut out = String::new();
    for item in &program.items {
        match item {
            HirItem::Import(import) => {
                out.push_str("import ");
                out.push_str(&import.path.join("."));
                out.push('\n');
            }
            HirItem::Struct(decl) => {
                out.push_str("struct ");
                out.push_str(&decl.name);
                out.push_str(" {\n");
                for field in &decl.fields {
                    out.push_str("  ");
                    out.push_str(&field.name);
                    out.push_str(": ");
                    out.push_str(&field.ty.display_name());
                    out.push('\n');
                }
                out.push_str("}\n");
            }
            HirItem::Enum(decl) => {
                out.push_str("enum ");
                out.push_str(&decl.name);
                out.push_str(" {\n");
                for (name, discriminant) in &decl.variants {
                    out.push_str("  ");
                    out.push_str(name);
                    out.push_str(" = ");
                    out.push_str(&discriminant.to_string());
                    out.push('\n');
                }
                out.push_str("}\n");
            }
            HirItem::Function(function) => {
                if matches!(function.visibility, inscribe_ast::Visibility::Private) {
                    out.push_str("priv ");
                }
                out.push_str("fn ");
                if let Some(receiver) = &function.receiver {
                    out.push_str(receiver);
                    out.push('.');
                }
                out.push_str(&function.name);
                out.push_str(" -> ");
                out.push_str(&function.signature.return_type.display_name());
                out.push('\n');
                if function.is_declaration {
                    out.push_str("  <declaration>\n");
                } else if let Some(body) = &function.body {
                    for statement in &body.statements {
                        out.push_str("  ");
                        out.push_str(&render_statement(statement));
                        out.push('\n');
                    }
                }
            }
        }
    }
    out
}

fn render_statement(statement: &HirStmt) -> String {
    match statement {
        HirStmt::Let(binding) => format!(
            "let {}: {} = {}",
            binding.name,
            binding.ty.display_name(),
            render_expr(&binding.value)
        ),
        HirStmt::Const(binding) => format!(
            "const {}: {} = {}",
            binding.name,
            binding.ty.display_name(),
            render_expr(&binding.value)
        ),
        HirStmt::For(for_stmt) => format!(
            "for {} in {} -> {}",
            for_stmt.binding,
            render_expr(&for_stmt.iterable),
            for_stmt.binding_ty.display_name()
        ),
        HirStmt::While(while_stmt) => format!("while {}", render_expr(&while_stmt.condition)),
        HirStmt::Return(Some(expr), _) => format!("return {}", render_expr(expr)),
        HirStmt::Return(None, _) => "return".to_string(),
        HirStmt::Expr(expr) => render_expr(expr),
    }
}

fn render_expr(expr: &HirExpr) -> String {
    match &expr.kind {
        HirExprKind::Literal(value) => format!("{value}: {}", expr.ty.display_name()),
        HirExprKind::EnumVariant {
            enum_name,
            variant,
            discriminant,
        } => format!("{enum_name}.{variant}#{discriminant}: {}", expr.ty.display_name()),
        HirExprKind::Path(path) => format!("{}: {}", path.join("."), expr.ty.display_name()),
        HirExprKind::Array(items) => {
            let items = items.iter().map(render_expr).collect::<Vec<_>>().join(", ");
            format!("[{items}]: {}", expr.ty.display_name())
        }
        HirExprKind::RepeatArray { value, length } => {
            format!("[{}; {length}]: {}", render_expr(value), expr.ty.display_name())
        }
        HirExprKind::Unary { op, expr: inner } => {
            format!("({op} {}): {}", render_expr(inner), expr.ty.display_name())
        }
        HirExprKind::Binary { op, left, right } => format!(
            "({} {op} {}): {}",
            render_expr(left),
            render_expr(right),
            expr.ty.display_name()
        ),
        HirExprKind::Call { callee, args } => {
            let args = args.iter().map(render_expr).collect::<Vec<_>>().join(", ");
            format!(
                "{}({args}): {}",
                render_expr(callee),
                expr.ty.display_name()
            )
        }
        HirExprKind::Field { base, field } => {
            format!(
                "{}.{}: {}",
                render_expr(base),
                field,
                expr.ty.display_name()
            )
        }
        HirExprKind::Index { target, index } => format!(
            "{}[{}]: {}",
            render_expr(target),
            render_expr(index),
            expr.ty.display_name()
        ),
        HirExprKind::StructLiteral { path, .. } => {
            format!("{} {{...}}: {}", path.join("."), expr.ty.display_name())
        }
        HirExprKind::If { .. } => format!("if ...: {}", expr.ty.display_name()),
        HirExprKind::Match { .. } => format!("match ...: {}", expr.ty.display_name()),
        HirExprKind::Block(block) => format!("block -> {}", block.ty.display_name()),
        HirExprKind::Try(inner) => format!("{}?: {}", render_expr(inner), expr.ty.display_name()),
    }
}
