use crate::nodes::{HirExpr, HirExprKind, HirFunction, HirItem, HirProgram, HirStmt};

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
                out.push_str(program.symbol_name(decl.symbol));
                out.push_str(" {\n");
                for field in &decl.fields {
                    out.push_str("  ");
                    out.push_str(program.symbol_name(field.symbol));
                    out.push_str(": ");
                    out.push_str(&field.ty.display_name());
                    out.push('\n');
                }
                out.push_str("}\n");
            }
            HirItem::Enum(decl) => {
                out.push_str("enum ");
                out.push_str(program.symbol_name(decl.symbol));
                out.push_str(" {\n");
                for variant in &decl.variants {
                    out.push_str("  ");
                    out.push_str(program.symbol_name(variant.symbol));
                    out.push_str(" = ");
                    out.push_str(&variant.discriminant.to_string());
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
                    out.push_str(program.symbol_name(*receiver));
                    out.push('.');
                }
                out.push_str(&function_base_name(program, function));
                out.push_str(" -> ");
                out.push_str(&function.signature.return_type.display_name());
                out.push('\n');
                if function.is_declaration {
                    out.push_str("  <declaration>\n");
                } else if let Some(body) = &function.body {
                    for statement in &body.statements {
                        out.push_str("  ");
                        out.push_str(&render_statement(program, statement));
                        out.push('\n');
                    }
                }
            }
        }
    }
    out
}

fn function_base_name(program: &HirProgram, function: &HirFunction) -> String {
    let name = program.symbol_name(function.symbol);
    if let Some(receiver) = function.receiver {
        let receiver_name = program.symbol_name(receiver);
        let qualified_prefix = format!("{receiver_name}.");
        if let Some(stripped) = name.strip_prefix(&qualified_prefix) {
            return stripped.to_string();
        }
    }
    name.to_string()
}

fn render_statement(program: &HirProgram, statement: &HirStmt) -> String {
    match statement {
        HirStmt::Let(binding) => format!(
            "let {}: {} = {}",
            program.symbol_name(binding.symbol),
            binding.ty.display_name(),
            render_expr(program, &binding.value)
        ),
        HirStmt::Const(binding) => format!(
            "const {}: {} = {}",
            program.symbol_name(binding.symbol),
            binding.ty.display_name(),
            render_expr(program, &binding.value)
        ),
        HirStmt::For(for_stmt) => format!(
            "for {} in {} -> {}",
            program.symbol_name(for_stmt.binding),
            render_expr(program, &for_stmt.iterable),
            for_stmt.binding_ty.display_name()
        ),
        HirStmt::While(while_stmt) => {
            format!("while {}", render_expr(program, &while_stmt.condition))
        }
        HirStmt::Return(Some(expr), _) => format!("return {}", render_expr(program, expr)),
        HirStmt::Return(None, _) => "return".to_string(),
        HirStmt::Expr(expr) => render_expr(program, expr),
    }
}

fn render_expr(program: &HirProgram, expr: &HirExpr) -> String {
    match &expr.kind {
        HirExprKind::Literal(value) => format!("{value}: {}", expr.ty.display_name()),
        HirExprKind::EnumVariant {
            enum_id,
            variant_id,
            discriminant,
        } => format!(
            "{}.{}#{discriminant}: {}",
            program.symbol_name(*enum_id),
            program.symbol_name(*variant_id),
            expr.ty.display_name()
        ),
        HirExprKind::Path(symbol) => {
            format!("{}: {}", program.symbol_name(*symbol), expr.ty.display_name())
        }
        HirExprKind::Array(items) => {
            let items = items
                .iter()
                .map(|item| render_expr(program, item))
                .collect::<Vec<_>>()
                .join(", ");
            format!("[{items}]: {}", expr.ty.display_name())
        }
        HirExprKind::RepeatArray { value, length } => {
            format!(
                "[{}; {length}]: {}",
                render_expr(program, value),
                expr.ty.display_name()
            )
        }
        HirExprKind::Cast { expr: inner } => {
            format!(
                "({} as {}): {}",
                render_expr(program, inner),
                expr.ty.display_name(),
                expr.ty.display_name()
            )
        }
        HirExprKind::Unary { op, expr: inner } => {
            format!(
                "({op} {}): {}",
                render_expr(program, inner),
                expr.ty.display_name()
            )
        }
        HirExprKind::Binary { op, left, right } => format!(
            "({} {op} {}): {}",
            render_expr(program, left),
            render_expr(program, right),
            expr.ty.display_name()
        ),
        HirExprKind::Call { callee, args } => {
            let args = args
                .iter()
                .map(|arg| render_expr(program, arg))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "{}({args}): {}",
                render_expr(program, callee),
                expr.ty.display_name()
            )
        }
        HirExprKind::Field { base, field } => {
            format!(
                "{}.{}: {}",
                render_expr(program, base),
                program.symbol_name(*field),
                expr.ty.display_name()
            )
        }
        HirExprKind::Index { target, index } => format!(
            "{}[{}]: {}",
            render_expr(program, target),
            render_expr(program, index),
            expr.ty.display_name()
        ),
        HirExprKind::StructLiteral { struct_id, .. } => {
            format!(
                "{} {{...}}: {}",
                program.symbol_name(*struct_id),
                expr.ty.display_name()
            )
        }
        HirExprKind::If { .. } => format!("if ...: {}", expr.ty.display_name()),
        HirExprKind::Match { .. } => format!("match ...: {}", expr.ty.display_name()),
        HirExprKind::Block(block) => format!("block -> {}", block.ty.display_name()),
        HirExprKind::Try(inner) => {
            format!("{}?: {}", render_expr(program, inner), expr.ty.display_name())
        }
    }
}
