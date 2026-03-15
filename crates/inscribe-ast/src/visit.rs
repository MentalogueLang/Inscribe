use crate::nodes::{
    Block, Expr, ExprKind, FunctionDecl, Item, MatchArm, Module, Pattern, PatternKind, Stmt,
};

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

pub trait VisitorMut {
    fn visit_module(&mut self, module: &mut Module) {
        walk_module_mut(self, module);
    }

    fn visit_item(&mut self, item: &mut Item) {
        walk_item_mut(self, item);
    }

    fn visit_function(&mut self, function: &mut FunctionDecl) {
        walk_function_mut(self, function);
    }

    fn visit_block(&mut self, block: &mut Block) {
        walk_block_mut(self, block);
    }

    fn visit_stmt(&mut self, stmt: &mut Stmt) {
        walk_stmt_mut(self, stmt);
    }

    fn visit_expr(&mut self, expr: &mut Expr) {
        walk_expr_mut(self, expr);
    }

    fn visit_pattern(&mut self, pattern: &mut Pattern) {
        walk_pattern_mut(self, pattern);
    }
}

pub trait ResultVisitor {
    type Error;

    fn visit_module(&mut self, module: &Module) -> Result<(), Self::Error> {
        walk_module_result(self, module)
    }

    fn visit_item(&mut self, item: &Item) -> Result<(), Self::Error> {
        walk_item_result(self, item)
    }

    fn visit_function(&mut self, function: &FunctionDecl) -> Result<(), Self::Error> {
        walk_function_result(self, function)
    }

    fn visit_block(&mut self, block: &Block) -> Result<(), Self::Error> {
        walk_block_result(self, block)
    }

    fn visit_stmt(&mut self, stmt: &Stmt) -> Result<(), Self::Error> {
        walk_stmt_result(self, stmt)
    }

    fn visit_expr(&mut self, expr: &Expr) -> Result<(), Self::Error> {
        walk_expr_result(self, expr)
    }

    fn visit_pattern(&mut self, pattern: &Pattern) -> Result<(), Self::Error> {
        walk_pattern_result(self, pattern)
    }
}

pub fn walk_module<V: Visitor + ?Sized>(visitor: &mut V, module: &Module) {
    for item in &module.items {
        visitor.visit_item(item);
    }
}

pub fn walk_module_mut<V: VisitorMut + ?Sized>(visitor: &mut V, module: &mut Module) {
    for item in &mut module.items {
        visitor.visit_item(item);
    }
}

pub fn walk_module_result<V: ResultVisitor + ?Sized>(
    visitor: &mut V,
    module: &Module,
) -> Result<(), V::Error> {
    for item in &module.items {
        visitor.visit_item(item)?;
    }
    Ok(())
}

pub fn walk_item<V: Visitor + ?Sized>(visitor: &mut V, item: &Item) {
    match item {
        Item::Function(function) => visitor.visit_function(function),
        Item::Import(_) | Item::Struct(_) | Item::Enum(_) => {}
    }
}

pub fn walk_item_mut<V: VisitorMut + ?Sized>(visitor: &mut V, item: &mut Item) {
    match item {
        Item::Function(function) => visitor.visit_function(function),
        Item::Import(_) | Item::Struct(_) | Item::Enum(_) => {}
    }
}

pub fn walk_item_result<V: ResultVisitor + ?Sized>(
    visitor: &mut V,
    item: &Item,
) -> Result<(), V::Error> {
    if let Item::Function(function) = item {
        visitor.visit_function(function)?;
    }
    Ok(())
}

pub fn walk_function<V: Visitor + ?Sized>(visitor: &mut V, function: &FunctionDecl) {
    if let Some(body) = &function.body {
        visitor.visit_block(body);
    }
}

pub fn walk_function_mut<V: VisitorMut + ?Sized>(visitor: &mut V, function: &mut FunctionDecl) {
    if let Some(body) = &mut function.body {
        visitor.visit_block(body);
    }
}

pub fn walk_function_result<V: ResultVisitor + ?Sized>(
    visitor: &mut V,
    function: &FunctionDecl,
) -> Result<(), V::Error> {
    if let Some(body) = &function.body {
        visitor.visit_block(body)?;
    }
    Ok(())
}

pub fn walk_block<V: Visitor + ?Sized>(visitor: &mut V, block: &Block) {
    for stmt in &block.statements {
        visitor.visit_stmt(stmt);
    }
}

pub fn walk_block_mut<V: VisitorMut + ?Sized>(visitor: &mut V, block: &mut Block) {
    for stmt in &mut block.statements {
        visitor.visit_stmt(stmt);
    }
}

pub fn walk_block_result<V: ResultVisitor + ?Sized>(
    visitor: &mut V,
    block: &Block,
) -> Result<(), V::Error> {
    for stmt in &block.statements {
        visitor.visit_stmt(stmt)?;
    }
    Ok(())
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

pub fn walk_stmt_mut<V: VisitorMut + ?Sized>(visitor: &mut V, stmt: &mut Stmt) {
    match stmt {
        Stmt::Let(stmt) => visitor.visit_expr(&mut stmt.value),
        Stmt::Const(stmt) => visitor.visit_expr(&mut stmt.value),
        Stmt::For(stmt) => {
            visitor.visit_pattern(&mut stmt.pattern);
            visitor.visit_expr(&mut stmt.iterable);
            visitor.visit_block(&mut stmt.body);
        }
        Stmt::While(stmt) => {
            visitor.visit_expr(&mut stmt.condition);
            visitor.visit_block(&mut stmt.body);
        }
        Stmt::Return(stmt) => {
            if let Some(expr) = &mut stmt.value {
                visitor.visit_expr(expr);
            }
        }
        Stmt::Expr(expr) => visitor.visit_expr(expr),
    }
}

pub fn walk_stmt_result<V: ResultVisitor + ?Sized>(
    visitor: &mut V,
    stmt: &Stmt,
) -> Result<(), V::Error> {
    match stmt {
        Stmt::Let(stmt) => visitor.visit_expr(&stmt.value),
        Stmt::Const(stmt) => visitor.visit_expr(&stmt.value),
        Stmt::For(stmt) => {
            visitor.visit_pattern(&stmt.pattern)?;
            visitor.visit_expr(&stmt.iterable)?;
            visitor.visit_block(&stmt.body)
        }
        Stmt::While(stmt) => {
            visitor.visit_expr(&stmt.condition)?;
            visitor.visit_block(&stmt.body)
        }
        Stmt::Return(stmt) => {
            if let Some(expr) = &stmt.value {
                visitor.visit_expr(expr)?;
            }
            Ok(())
        }
        Stmt::Expr(expr) => visitor.visit_expr(expr),
    }
}

pub fn walk_expr<V: Visitor + ?Sized>(visitor: &mut V, expr: &Expr) {
    match &expr.kind {
        ExprKind::Literal(_) | ExprKind::Path(_) => {}
        ExprKind::Array(items) => {
            for item in items {
                visitor.visit_expr(item);
            }
        }
        ExprKind::RepeatArray { value, .. } => visitor.visit_expr(value),
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
        ExprKind::Index { target, index } => {
            visitor.visit_expr(target);
            visitor.visit_expr(index);
        }
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

pub fn walk_expr_mut<V: VisitorMut + ?Sized>(visitor: &mut V, expr: &mut Expr) {
    match &mut expr.kind {
        ExprKind::Literal(_) | ExprKind::Path(_) => {}
        ExprKind::Array(items) => {
            for item in items {
                visitor.visit_expr(item);
            }
        }
        ExprKind::RepeatArray { value, .. } => visitor.visit_expr(value),
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
        ExprKind::Index { target, index } => {
            visitor.visit_expr(target);
            visitor.visit_expr(index);
        }
        ExprKind::StructLiteral { fields, .. } => {
            for field in fields {
                visitor.visit_expr(&mut field.value);
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

pub fn walk_expr_result<V: ResultVisitor + ?Sized>(
    visitor: &mut V,
    expr: &Expr,
) -> Result<(), V::Error> {
    match &expr.kind {
        ExprKind::Literal(_) | ExprKind::Path(_) => Ok(()),
        ExprKind::Array(items) => {
            for item in items {
                visitor.visit_expr(item)?;
            }
            Ok(())
        }
        ExprKind::RepeatArray { value, .. } => visitor.visit_expr(value),
        ExprKind::Unary { expr, .. } | ExprKind::Try(expr) => visitor.visit_expr(expr),
        ExprKind::Binary { left, right, .. } => {
            visitor.visit_expr(left)?;
            visitor.visit_expr(right)
        }
        ExprKind::Call { callee, args } => {
            visitor.visit_expr(callee)?;
            for arg in args {
                visitor.visit_expr(arg)?;
            }
            Ok(())
        }
        ExprKind::Field { base, .. } => visitor.visit_expr(base),
        ExprKind::Index { target, index } => {
            visitor.visit_expr(target)?;
            visitor.visit_expr(index)
        }
        ExprKind::StructLiteral { fields, .. } => {
            for field in fields {
                visitor.visit_expr(&field.value)?;
            }
            Ok(())
        }
        ExprKind::If {
            condition,
            then_block,
            else_branch,
        } => {
            visitor.visit_expr(condition)?;
            visitor.visit_block(then_block)?;
            if let Some(else_branch) = else_branch {
                visitor.visit_expr(else_branch)?;
            }
            Ok(())
        }
        ExprKind::Match { value, arms } => {
            visitor.visit_expr(value)?;
            for MatchArm { pattern, value, .. } in arms {
                visitor.visit_pattern(pattern)?;
                visitor.visit_expr(value)?;
            }
            Ok(())
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

pub fn walk_pattern_mut<V: VisitorMut + ?Sized>(visitor: &mut V, pattern: &mut Pattern) {
    if let PatternKind::Constructor { arguments, .. } = &mut pattern.kind {
        for argument in arguments {
            visitor.visit_pattern(argument);
        }
    }
}

pub fn walk_pattern_result<V: ResultVisitor + ?Sized>(
    visitor: &mut V,
    pattern: &Pattern,
) -> Result<(), V::Error> {
    if let PatternKind::Constructor { arguments, .. } = &pattern.kind {
        for argument in arguments {
            visitor.visit_pattern(argument)?;
        }
    }
    Ok(())
}
