use std::collections::HashMap;

use inscribe_ast::span::Span;
use inscribe_hir::nodes::{
    HirBlock, HirExpr, HirExprKind, HirFunction, HirItem, HirProgram, HirStmt,
};
use inscribe_typeck::Type;

use crate::nodes::{
    BasicBlockData, BasicBlockId, Constant, ConstantValue, LocalDecl, LocalId, MatchTarget,
    MirFunction, MirProgram, Operand, Place, ProjectionElem, Rvalue, Statement, StatementKind,
    TerminatorKind,
};

// TODO: Lower this into SSA-style data flow once optimization and register allocation need it.

pub fn lower_program(program: &HirProgram) -> MirProgram {
    let functions = program
        .items
        .iter()
        .filter_map(|item| match item {
            HirItem::Function(function) => Some(lower_function(function)),
            HirItem::Import(_) | HirItem::Struct(_) => None,
        })
        .collect();

    MirProgram {
        functions,
        span: program.span,
    }
}

fn lower_function(function: &HirFunction) -> MirFunction {
    let mut lowerer = FunctionLowerer::new(function);
    lowerer.lower_body(function.body.as_ref());
    lowerer.finish()
}

struct FunctionLowerer<'a> {
    function: &'a HirFunction,
    locals: Vec<LocalDecl>,
    blocks: Vec<BasicBlockData>,
    scopes: Vec<HashMap<String, LocalId>>,
    entry: BasicBlockId,
    return_local: LocalId,
}

impl<'a> FunctionLowerer<'a> {
    fn new(function: &'a HirFunction) -> Self {
        let mut lowerer = Self {
            function,
            locals: Vec::new(),
            blocks: Vec::new(),
            scopes: vec![HashMap::new()],
            entry: BasicBlockId(0),
            return_local: LocalId(0),
        };

        let entry = lowerer.new_block();
        lowerer.entry = entry;
        let return_local = lowerer.alloc_local(
            "_return".to_string(),
            (*function.signature.return_type).clone(),
            true,
            true,
            function.span,
        );
        lowerer.return_local = return_local;
        lowerer.emit(
            entry,
            StatementKind::StorageLive(return_local),
            function.span,
        );

        for param in &function.params {
            let local = lowerer.alloc_local(
                param.name.clone(),
                param.ty.clone(),
                false,
                false,
                param.span,
            );
            lowerer.define_binding(param.name.clone(), local);
            lowerer.emit(entry, StatementKind::StorageLive(local), param.span);
        }

        lowerer
    }

    fn finish(self) -> MirFunction {
        MirFunction {
            receiver: self.function.receiver.clone(),
            name: self.function.name.clone(),
            signature: self.function.signature.clone(),
            is_declaration: self.function.is_declaration,
            locals: self.locals,
            blocks: self.blocks,
            entry: self.entry,
            return_local: self.return_local,
            span: self.function.span,
        }
    }

    fn lower_body(&mut self, body: Option<&HirBlock>) {
        let current = self.entry;
        let final_block = if let Some(body) = body {
            let (block, value) = self.lower_block_value(body, current);
            if let Some(value) = value {
                self.emit_assign(
                    block,
                    Place::new(self.return_local),
                    Rvalue::Use(value),
                    body.span,
                );
            }
            block
        } else {
            current
        };

        if self.is_open(final_block) {
            self.set_terminator(final_block, TerminatorKind::Return);
        }
    }

    fn lower_block_value(
        &mut self,
        block: &HirBlock,
        mut current: BasicBlockId,
    ) -> (BasicBlockId, Option<Operand>) {
        self.push_scope();
        let last_index = block.statements.len().saturating_sub(1);
        let wants_value = !matches!(block.ty, Type::Unit);
        let mut result = None;

        for (index, statement) in block.statements.iter().enumerate() {
            let capture = wants_value && index == last_index;
            let (next, value) = self.lower_statement(statement, current, capture);
            current = next;
            if capture {
                result = value;
            }
        }

        self.pop_scope();
        (current, result)
    }

    fn lower_statement(
        &mut self,
        statement: &HirStmt,
        current: BasicBlockId,
        capture_value: bool,
    ) -> (BasicBlockId, Option<Operand>) {
        match statement {
            HirStmt::Let(binding) => {
                let local = self.alloc_local(
                    binding.name.clone(),
                    binding.ty.clone(),
                    true,
                    false,
                    binding.span,
                );
                self.define_binding(binding.name.clone(), local);
                self.emit(current, StatementKind::StorageLive(local), binding.span);
                let (block, value) = self.lower_expr(&binding.value, current);
                self.emit_assign(block, Place::new(local), Rvalue::Use(value), binding.span);
                (block, None)
            }
            HirStmt::Const(binding) => {
                let local = self.alloc_local(
                    binding.name.clone(),
                    binding.ty.clone(),
                    false,
                    false,
                    binding.span,
                );
                self.define_binding(binding.name.clone(), local);
                self.emit(current, StatementKind::StorageLive(local), binding.span);
                let (block, value) = self.lower_expr(&binding.value, current);
                self.emit_assign(block, Place::new(local), Rvalue::Use(value), binding.span);
                (block, None)
            }
            HirStmt::For(for_stmt) => {
                let iter_local = self.alloc_local(
                    format!("{}_iter", for_stmt.binding),
                    for_stmt.iterable.ty.clone(),
                    true,
                    true,
                    for_stmt.span,
                );
                self.emit(
                    current,
                    StatementKind::StorageLive(iter_local),
                    for_stmt.span,
                );
                let (block, iterable) = self.lower_expr(&for_stmt.iterable, current);
                self.emit_assign(
                    block,
                    Place::new(iter_local),
                    Rvalue::Use(iterable),
                    for_stmt.span,
                );

                let binding_local = self.alloc_local(
                    for_stmt.binding.clone(),
                    for_stmt.binding_ty.clone(),
                    true,
                    false,
                    for_stmt.span,
                );
                let head = self.new_block();
                let body = self.new_block();
                let exit = self.new_block();
                self.set_terminator(block, TerminatorKind::Goto { target: head });
                self.set_terminator(
                    head,
                    TerminatorKind::IterNext {
                        iterator: Place::new(iter_local),
                        binding: binding_local,
                        loop_body: body,
                        exit,
                    },
                );

                self.push_scope();
                self.define_binding(for_stmt.binding.clone(), binding_local);
                self.emit(
                    body,
                    StatementKind::StorageLive(binding_local),
                    for_stmt.span,
                );
                let (body_exit, _) = self.lower_block_value(&for_stmt.body, body);
                self.pop_scope();
                if self.is_open(body_exit) {
                    self.set_terminator(body_exit, TerminatorKind::Goto { target: head });
                }

                (exit, None)
            }
            HirStmt::While(while_stmt) => {
                let head = self.new_block();
                let body = self.new_block();
                let exit = self.new_block();
                self.set_terminator(current, TerminatorKind::Goto { target: head });
                let (cond_block, condition) = self.lower_expr(&while_stmt.condition, head);
                self.set_terminator(
                    cond_block,
                    TerminatorKind::Branch {
                        condition,
                        then_bb: body,
                        else_bb: exit,
                    },
                );
                let (body_exit, _) = self.lower_block_value(&while_stmt.body, body);
                if self.is_open(body_exit) {
                    self.set_terminator(body_exit, TerminatorKind::Goto { target: head });
                }
                (exit, None)
            }
            HirStmt::Return(value, span) => {
                let block = if let Some(expr) = value {
                    let (block, operand) = self.lower_expr(expr, current);
                    self.emit_assign(
                        block,
                        Place::new(self.return_local),
                        Rvalue::Use(operand),
                        *span,
                    );
                    block
                } else {
                    current
                };
                self.set_terminator(block, TerminatorKind::Return);
                (self.new_block(), None)
            }
            HirStmt::Expr(expr) => {
                let (block, value) = self.lower_expr(expr, current);
                if capture_value {
                    (block, Some(value))
                } else {
                    (block, None)
                }
            }
        }
    }

    fn lower_expr(&mut self, expr: &HirExpr, current: BasicBlockId) -> (BasicBlockId, Operand) {
        match &expr.kind {
            HirExprKind::Literal(value) => (current, literal_operand(value, &expr.ty)),
            HirExprKind::Path(segments) => {
                let operand = self.path_operand(segments, &expr.ty).unwrap_or_else(|| {
                    Operand::Constant(Constant {
                        ty: expr.ty.clone(),
                        value: ConstantValue::Function(segments.join(".")),
                    })
                });
                (current, operand)
            }
            HirExprKind::Unary { op, expr: inner } => {
                let (block, operand) = self.lower_expr(inner, current);
                let temp = self.alloc_temp(expr.ty.clone(), expr.span);
                self.emit_assign(
                    block,
                    Place::new(temp),
                    Rvalue::UnaryOp {
                        op: op.clone(),
                        operand,
                    },
                    expr.span,
                );
                (block, Operand::Move(Place::new(temp)))
            }
            HirExprKind::Binary { op, left, right } => {
                if op == "Assign" {
                    return self.lower_assignment(left, right, current, expr.span);
                }
                let (block, left_op) = self.lower_expr(left, current);
                let (block, right_op) = self.lower_expr(right, block);
                let temp = self.alloc_temp(expr.ty.clone(), expr.span);
                self.emit_assign(
                    block,
                    Place::new(temp),
                    Rvalue::BinaryOp {
                        op: op.clone(),
                        left: left_op,
                        right: right_op,
                    },
                    expr.span,
                );
                (block, Operand::Move(Place::new(temp)))
            }
            HirExprKind::Call { callee, args } => self.lower_call(expr, callee, args, current),
            HirExprKind::Field { base, field } => {
                let place = self
                    .expr_place(base)
                    .map(|mut place| {
                        place.projection.push(ProjectionElem::Field(field.clone()));
                        place
                    })
                    .unwrap_or_else(|| {
                        let temp = self.alloc_temp(expr.ty.clone(), expr.span);
                        Place::new(temp)
                    });
                (current, Operand::Copy(place))
            }
            HirExprKind::StructLiteral { path, fields } => {
                let mut block = current;
                let lowered_fields = fields
                    .iter()
                    .map(|(name, value)| {
                        let (next, operand) = self.lower_expr(value, block);
                        block = next;
                        (name.clone(), operand)
                    })
                    .collect::<Vec<_>>();
                let temp = self.alloc_temp(expr.ty.clone(), expr.span);
                self.emit_assign(
                    block,
                    Place::new(temp),
                    Rvalue::AggregateStruct {
                        path: path.clone(),
                        fields: lowered_fields,
                    },
                    expr.span,
                );
                (block, Operand::Move(Place::new(temp)))
            }
            HirExprKind::If {
                condition,
                then_block,
                else_branch,
            } => self.lower_if_expr(expr, condition, then_block, else_branch.as_deref(), current),
            HirExprKind::Match { value, arms } => {
                self.lower_match_expr(expr, value, arms.as_slice(), current)
            }
            HirExprKind::Block(block) => {
                let (block, value) = self.lower_block_value(block, current);
                (block, value.unwrap_or_else(unit_operand))
            }
            HirExprKind::Try(inner) => self.lower_try_expr(expr, inner, current),
        }
    }

    fn lower_assignment(
        &mut self,
        left: &HirExpr,
        right: &HirExpr,
        current: BasicBlockId,
        span: Span,
    ) -> (BasicBlockId, Operand) {
        let Some(place) = self.expr_place(left) else {
            let temp = self.alloc_temp(Type::Unknown, span);
            return (current, Operand::Move(Place::new(temp)));
        };
        let (block, value) = self.lower_expr(right, current);
        self.emit_assign(block, place.clone(), Rvalue::Use(value), span);
        (block, Operand::Copy(place))
    }

    fn lower_call(
        &mut self,
        expr: &HirExpr,
        callee: &HirExpr,
        args: &[HirExpr],
        current: BasicBlockId,
    ) -> (BasicBlockId, Operand) {
        let mut block = current;
        let mut lowered_args = Vec::new();
        let callee_operand = if let HirExprKind::Field { base, field } = &callee.kind {
            let (next, receiver) = self.lower_expr(base, block);
            block = next;
            lowered_args.push(receiver);
            Operand::Constant(Constant {
                ty: callee.ty.clone(),
                value: ConstantValue::Function(format!("{}.{}", base.ty.display_name(), field)),
            })
        } else {
            let (next, operand) = self.lower_expr(callee, block);
            block = next;
            operand
        };

        for arg in args {
            let (next, operand) = self.lower_expr(arg, block);
            block = next;
            lowered_args.push(operand);
        }

        let destination = if matches!(expr.ty, Type::Unit) {
            None
        } else {
            let temp = self.alloc_temp(expr.ty.clone(), expr.span);
            Some(Place::new(temp))
        };
        let target = self.new_block();
        self.set_terminator(
            block,
            TerminatorKind::Call {
                callee: callee_operand,
                args: lowered_args,
                destination: destination.clone(),
                target,
            },
        );

        let value = destination.map(Operand::Move).unwrap_or_else(unit_operand);
        (target, value)
    }

    fn lower_if_expr(
        &mut self,
        expr: &HirExpr,
        condition: &HirExpr,
        then_block: &HirBlock,
        else_branch: Option<&HirExpr>,
        current: BasicBlockId,
    ) -> (BasicBlockId, Operand) {
        let result = if matches!(expr.ty, Type::Unit) {
            None
        } else {
            Some(self.alloc_temp(expr.ty.clone(), expr.span))
        };

        let (cond_block, condition) = self.lower_expr(condition, current);
        let then_bb = self.new_block();
        let else_bb = self.new_block();
        let join_bb = self.new_block();
        self.set_terminator(
            cond_block,
            TerminatorKind::Branch {
                condition,
                then_bb,
                else_bb,
            },
        );

        let (then_exit, then_value) = self.lower_block_value(then_block, then_bb);
        if let (Some(local), Some(value)) = (result, then_value) {
            self.emit_assign(
                then_exit,
                Place::new(local),
                Rvalue::Use(value),
                then_block.span,
            );
        }
        if self.is_open(then_exit) {
            self.set_terminator(then_exit, TerminatorKind::Goto { target: join_bb });
        }

        let else_exit = if let Some(else_expr) = else_branch {
            let (else_exit, else_value) = self.lower_expr(else_expr, else_bb);
            if let Some(local) = result {
                self.emit_assign(
                    else_exit,
                    Place::new(local),
                    Rvalue::Use(else_value),
                    else_expr.span,
                );
            }
            else_exit
        } else {
            else_bb
        };

        if self.is_open(else_exit) {
            self.set_terminator(else_exit, TerminatorKind::Goto { target: join_bb });
        }

        let operand = result
            .map(|local| Operand::Move(Place::new(local)))
            .unwrap_or_else(unit_operand);
        (join_bb, operand)
    }

    fn lower_match_expr(
        &mut self,
        expr: &HirExpr,
        value: &HirExpr,
        arms: &[inscribe_hir::nodes::HirMatchArm],
        current: BasicBlockId,
    ) -> (BasicBlockId, Operand) {
        let result = if matches!(expr.ty, Type::Unit) {
            None
        } else {
            Some(self.alloc_temp(expr.ty.clone(), expr.span))
        };

        let (block, discriminant) = self.lower_expr(value, current);
        let join = self.new_block();
        let otherwise = join;
        let arm_targets = arms
            .iter()
            .map(|arm| MatchTarget {
                pattern: arm.pattern.clone(),
                target: self.new_block(),
            })
            .collect::<Vec<_>>();
        self.set_terminator(
            block,
            TerminatorKind::Match {
                discriminant,
                arms: arm_targets.clone(),
                otherwise,
            },
        );

        for (arm, target) in arms.iter().zip(arm_targets.iter()) {
            let (arm_exit, arm_value) = self.lower_expr(&arm.value, target.target);
            if let Some(local) = result {
                self.emit_assign(
                    arm_exit,
                    Place::new(local),
                    Rvalue::Use(arm_value),
                    arm.span,
                );
            }
            if self.is_open(arm_exit) {
                self.set_terminator(arm_exit, TerminatorKind::Goto { target: join });
            }
        }

        let operand = result
            .map(|local| Operand::Move(Place::new(local)))
            .unwrap_or_else(unit_operand);
        (join, operand)
    }

    fn lower_try_expr(
        &mut self,
        expr: &HirExpr,
        inner: &HirExpr,
        current: BasicBlockId,
    ) -> (BasicBlockId, Operand) {
        let (block, operand) = self.lower_expr(inner, current);
        let ok_local = self.alloc_temp(expr.ty.clone(), expr.span);
        let err_local = self.alloc_temp(Type::Error, expr.span);
        let ok_bb = self.new_block();
        let err_bb = self.new_block();
        let join_bb = self.new_block();
        self.set_terminator(
            block,
            TerminatorKind::Try {
                operand,
                ok_local,
                err_local,
                ok_target: ok_bb,
                err_target: err_bb,
            },
        );
        self.set_terminator(ok_bb, TerminatorKind::Goto { target: join_bb });
        self.emit_assign(
            err_bb,
            Place::new(self.return_local),
            Rvalue::ResultErr(Operand::Move(Place::new(err_local))),
            expr.span,
        );
        self.set_terminator(err_bb, TerminatorKind::Return);
        (join_bb, Operand::Move(Place::new(ok_local)))
    }

    fn expr_place(&self, expr: &HirExpr) -> Option<Place> {
        match &expr.kind {
            HirExprKind::Path(segments) => self.path_place(segments),
            HirExprKind::Field { base, field } => {
                let mut place = self.expr_place(base)?;
                place.projection.push(ProjectionElem::Field(field.clone()));
                Some(place)
            }
            _ => None,
        }
    }

    fn path_place(&self, segments: &[String]) -> Option<Place> {
        let (first, rest) = segments.split_first()?;
        let local = self.lookup_binding(first)?;
        let mut place = Place::new(local);
        for field in rest {
            place.projection.push(ProjectionElem::Field(field.clone()));
        }
        Some(place)
    }

    fn path_operand(&self, segments: &[String], ty: &Type) -> Option<Operand> {
        self.path_place(segments)
            .map(|place| Operand::Copy(place))
            .or_else(|| {
                if matches!(ty, Type::Function(_)) {
                    Some(Operand::Constant(Constant {
                        ty: ty.clone(),
                        value: ConstantValue::Function(segments.join(".")),
                    }))
                } else {
                    None
                }
            })
    }

    fn alloc_local(
        &mut self,
        name: String,
        ty: Type,
        mutable: bool,
        temp: bool,
        span: Span,
    ) -> LocalId {
        let id = LocalId(self.locals.len());
        self.locals.push(LocalDecl {
            id,
            name,
            ty,
            mutable,
            temp,
            span,
        });
        id
    }

    fn alloc_temp(&mut self, ty: Type, span: Span) -> LocalId {
        let local = self.alloc_local(format!("_tmp{}", self.locals.len()), ty, true, true, span);
        self.emit(self.entry, StatementKind::StorageLive(local), span);
        local
    }

    fn new_block(&mut self) -> BasicBlockId {
        let id = BasicBlockId(self.blocks.len());
        self.blocks.push(BasicBlockData {
            id,
            statements: Vec::new(),
            terminator: TerminatorKind::Unreachable,
        });
        id
    }

    fn emit(&mut self, block: BasicBlockId, kind: StatementKind, span: Span) {
        self.blocks[block.0]
            .statements
            .push(Statement { kind, span });
    }

    fn emit_assign(&mut self, block: BasicBlockId, place: Place, value: Rvalue, span: Span) {
        self.emit(block, StatementKind::Assign(place, value), span);
    }

    fn set_terminator(&mut self, block: BasicBlockId, terminator: TerminatorKind) {
        self.blocks[block.0].terminator = terminator;
    }

    fn is_open(&self, block: BasicBlockId) -> bool {
        matches!(self.blocks[block.0].terminator, TerminatorKind::Unreachable)
    }

    fn lookup_binding(&self, name: &str) -> Option<LocalId> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }

    fn define_binding(&mut self, name: String, local: LocalId) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, local);
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            let _ = self.scopes.pop();
        }
    }
}

fn unit_operand() -> Operand {
    Operand::Constant(Constant {
        ty: Type::Unit,
        value: ConstantValue::Unit,
    })
}

fn literal_operand(value: &str, ty: &Type) -> Operand {
    let constant = match ty {
        Type::Int => ConstantValue::Integer(value.to_string()),
        Type::Float => ConstantValue::Float(value.to_string()),
        Type::String => ConstantValue::String(value.trim_matches('"').to_string()),
        Type::Bool => ConstantValue::Bool(value == "true"),
        _ => ConstantValue::String(value.to_string()),
    };
    Operand::Constant(Constant {
        ty: ty.clone(),
        value: constant,
    })
}
