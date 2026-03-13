use crate::nodes::{
    Block, Expr, ExprKind, FunctionDecl, Item, MatchArm, Module, Pattern, PatternKind, Stmt,
};

// TODO: Add mutable visitors and result-carrying traversals once analysis passes need them.

pub trait Visitor {
    fn visit_module(&mut self, module: &Module) {
        walk_module(self, module);
    }

    fn visit_item(&mut self, item: &Item) {
        walk_item(self, item);
    }

    fn visit_function(&mut self, function: &FunctionDecl) {
        walk_function(self, function);
    }

    fn visit_block(&mut self, block: &Block) {
        walk_block(self, block);
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        walk_stmt(self, stmt);
    }

    fn visit_expr(&mut self, expr: &Expr) {
        walk_expr(self, expr);
    }

    fn visit_pattern(&mut self, pattern: &Pattern) {
        walk_pattern(self, pattern);
    }
}

pub fn walk_module<V: Visitor + ?Sized>(visitor: &mut V, module: &Module) {
    for item in &module.items {
        visitor.visit_item(item);
    }
}

pub fn walk_item<V: Visitor + ?Sized>(visitor: &mut V, item: &Item) {
    if let Item::Function(function) = item {
        visitor.visit_function(function);
    }
}

pub fn walk_function<V: Visitor + ?Sized>(visitor: &mut V, function: &FunctionDecl) {
    if let Some(body) = &function.body {
        visitor.visit_block(body);
    }
}

pub fn walk_block<V: Visitor + ?Sized>(visitor: &mut V, block: &Block) {
    for stmt in &block.statements {
        visitor.visit_stmt(stmt);
    }
}

pub fn walk_stmt<V: Visitor + ?Sized>(visitor: &mut V, stmt: &Stmt) {
    match stmt {
        Stmt::Let(stmt) => visitor.visit_expr(&stmt.value),
        Stmt::Const(stmt) => visitor.visit_expr(&stmt.value),
        Stmt::For(stmt) => {
            visitor.visit_pattern(&stmt.pattern);
            visitor.visit_expr(&stmt.iterable);
            visitor.visit_block(&stmt.body);
        }
        Stmt::While(stmt) => {
            visitor.visit_expr(&stmt.condition);
            visitor.visit_block(&stmt.body);
        }
        Stmt::Return(stmt) => {
            if let Some(expr) = &stmt.value {
                visitor.visit_expr(expr);
            }
        }
        Stmt::Expr(expr) => visitor.visit_expr(expr),
    }
}

pub fn walk_expr<V: Visitor + ?Sized>(visitor: &mut V, expr: &Expr) {
    match &expr.kind {
        ExprKind::Literal(_) | ExprKind::Path(_) => {}
        ExprKind::Unary { expr, .. } | ExprKind::Try(expr) => visitor.visit_expr(expr),
        ExprKind::Binary { left, right, .. } => {
            visitor.visit_expr(left);
            visitor.visit_expr(right);
        }
        ExprKind::Call { callee, args } => {
            visitor.visit_expr(callee);
            for arg in args {
                visitor.visit_expr(arg);
            }
        }
        ExprKind::Field { base, .. } => visitor.visit_expr(base),
        ExprKind::StructLiteral { fields, .. } => {
            for field in fields {
                visitor.visit_expr(&field.value);
            }
        }
        ExprKind::If {
            condition,
            then_block,
            else_branch,
        } => {
            visitor.visit_expr(condition);
            visitor.visit_block(then_block);
            if let Some(else_branch) = else_branch {
                visitor.visit_expr(else_branch);
            }
        }
        ExprKind::Match { value, arms } => {
            visitor.visit_expr(value);
            for MatchArm { pattern, value, .. } in arms {
                visitor.visit_pattern(pattern);
                visitor.visit_expr(value);
            }
        }
        ExprKind::Block(block) => visitor.visit_block(block),
    }
}

pub fn walk_pattern<V: Visitor + ?Sized>(visitor: &mut V, pattern: &Pattern) {
    if let PatternKind::Constructor { arguments, .. } = &pattern.kind {
        for argument in arguments {
            visitor.visit_pattern(argument);
        }
    }
}
