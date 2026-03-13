use std::collections::VecDeque;

use crate::nodes::{
    BasicBlockData, BasicBlockId, Constant, ConstantValue, MirFunction, Operand, Place, Rvalue,
    StatementKind, TerminatorKind,
};

// TODO: Extend this beyond simple scalar folding once aggregate and control-flow evaluation lands.

pub fn fold_block_constants(block: &mut BasicBlockData) {
    for statement in &mut block.statements {
        if let StatementKind::Assign(_, value) = &mut statement.kind {
            if let Some(constant) = evaluate_constant_rvalue(value) {
                *value = Rvalue::Use(Operand::Constant(constant));
            }
        }
    }
}

pub fn fold_function_constants(function: &mut MirFunction) {
    let local_count = function.locals.len();
    let mut incoming = vec![None; function.blocks.len()];
    let mut queue = VecDeque::new();

    incoming[function.entry.0] = Some(vec![None; local_count]);
    queue.push_back(function.entry);

    while let Some(block_id) = queue.pop_front() {
        let Some(mut env) = incoming[block_id.0].clone() else {
            continue;
        };

        let block = &mut function.blocks[block_id.0];
        fold_block_with_env(block, &mut env);

        for successor in successor_blocks(&block.terminator) {
            if merge_env(&mut incoming[successor.0], &env) {
                queue.push_back(successor);
            }
        }
    }
}

pub fn evaluate_constant_rvalue(rvalue: &Rvalue) -> Option<Constant> {
    match rvalue {
        Rvalue::Use(Operand::Constant(constant)) => Some(constant.clone()),
        Rvalue::UnaryOp { op, operand } => {
            let constant = operand_constant(operand)?;
            match (&constant.value, op.as_str()) {
                (ConstantValue::Integer(value), "Negate") => Some(Constant {
                    ty: constant.ty,
                    value: ConstantValue::Integer(format!("-{value}")),
                }),
                (ConstantValue::Bool(value), "Not") => Some(Constant {
                    ty: constant.ty,
                    value: ConstantValue::Bool(!value),
                }),
                _ => None,
            }
        }
        Rvalue::BinaryOp { op, left, right } => {
            let left = operand_constant(left)?;
            let right = operand_constant(right)?;
            fold_binary(op, &left, &right)
        }
        Rvalue::AggregateStruct { .. } | Rvalue::ResultOk(_) | Rvalue::ResultErr(_) => None,
        Rvalue::Use(Operand::Copy(_)) | Rvalue::Use(Operand::Move(_)) => None,
    }
}

fn fold_block_with_env(block: &mut BasicBlockData, env: &mut [Option<Constant>]) {
    for statement in &mut block.statements {
        match &mut statement.kind {
            StatementKind::Assign(place, value) => {
                let rewritten = rewrite_rvalue(value, env);
                let constant = evaluate_constant_rvalue(&rewritten);
                *value = constant
                    .clone()
                    .map(|constant| Rvalue::Use(Operand::Constant(constant)))
                    .unwrap_or(rewritten);
                update_place(place, constant, env);
            }
            StatementKind::StorageLive(local)
            | StatementKind::StorageDead(local)
            | StatementKind::Drop(local) => env[local.0] = None,
            StatementKind::Nop => {}
        }
    }

    fold_terminator(&mut block.terminator, env);
}

fn rewrite_rvalue(rvalue: &Rvalue, env: &[Option<Constant>]) -> Rvalue {
    match rvalue {
        Rvalue::Use(operand) => Rvalue::Use(rewrite_operand(operand, env)),
        Rvalue::UnaryOp { op, operand } => Rvalue::UnaryOp {
            op: op.clone(),
            operand: rewrite_operand(operand, env),
        },
        Rvalue::BinaryOp { op, left, right } => Rvalue::BinaryOp {
            op: op.clone(),
            left: rewrite_operand(left, env),
            right: rewrite_operand(right, env),
        },
        Rvalue::AggregateStruct { path, fields } => Rvalue::AggregateStruct {
            path: path.clone(),
            fields: fields
                .iter()
                .map(|(name, operand)| (name.clone(), rewrite_operand(operand, env)))
                .collect(),
        },
        Rvalue::ResultOk(operand) => Rvalue::ResultOk(rewrite_operand(operand, env)),
        Rvalue::ResultErr(operand) => Rvalue::ResultErr(rewrite_operand(operand, env)),
    }
}

fn rewrite_operand(operand: &Operand, env: &[Option<Constant>]) -> Operand {
    match operand {
        Operand::Copy(place) | Operand::Move(place) => lookup_place_constant(place, env)
            .map(Operand::Constant)
            .unwrap_or_else(|| operand.clone()),
        Operand::Constant(_) => operand.clone(),
    }
}

fn lookup_place_constant(place: &Place, env: &[Option<Constant>]) -> Option<Constant> {
    if place.projection.is_empty() {
        env.get(place.local.0).cloned().flatten()
    } else {
        None
    }
}

fn update_place(place: &Place, constant: Option<Constant>, env: &mut [Option<Constant>]) {
    env[place.local.0] = if place.projection.is_empty() {
        constant
    } else {
        None
    };
}

fn fold_terminator(terminator: &mut TerminatorKind, env: &[Option<Constant>]) {
    match terminator {
        TerminatorKind::Goto { .. } | TerminatorKind::Return | TerminatorKind::Unreachable => {}
        TerminatorKind::Branch {
            condition,
            then_bb,
            else_bb,
        } => {
            let then_target = *then_bb;
            let else_target = *else_bb;
            *condition = rewrite_operand(condition, env);
            let chosen = if let Operand::Constant(Constant {
                value: ConstantValue::Bool(value),
                ..
            }) = condition
            {
                Some(if *value { then_target } else { else_target })
            } else {
                None
            };

            if let Some(target) = chosen {
                *terminator = TerminatorKind::Goto { target };
            }
        }
        TerminatorKind::Match { discriminant, .. } => {
            *discriminant = rewrite_operand(discriminant, env);
        }
        TerminatorKind::Call { callee, args, .. } => {
            *callee = rewrite_operand(callee, env);
            for arg in args {
                *arg = rewrite_operand(arg, env);
            }
        }
        TerminatorKind::IterNext { .. } => {}
        TerminatorKind::Try { operand, .. } => {
            *operand = rewrite_operand(operand, env);
        }
    }
}

fn successor_blocks(terminator: &TerminatorKind) -> Vec<BasicBlockId> {
    match terminator {
        TerminatorKind::Goto { target } => vec![*target],
        TerminatorKind::Branch {
            then_bb, else_bb, ..
        } => vec![*then_bb, *else_bb],
        TerminatorKind::Match {
            arms, otherwise, ..
        } => {
            let mut successors = arms.iter().map(|arm| arm.target).collect::<Vec<_>>();
            successors.push(*otherwise);
            successors
        }
        TerminatorKind::Call { target, .. } => vec![*target],
        TerminatorKind::IterNext {
            loop_body, exit, ..
        } => vec![*loop_body, *exit],
        TerminatorKind::Try {
            ok_target,
            err_target,
            ..
        } => vec![*ok_target, *err_target],
        TerminatorKind::Return | TerminatorKind::Unreachable => Vec::new(),
    }
}

fn merge_env(slot: &mut Option<Vec<Option<Constant>>>, incoming: &[Option<Constant>]) -> bool {
    match slot {
        Some(existing) => {
            let mut changed = false;
            for (current, next) in existing.iter_mut().zip(incoming.iter()) {
                let merged = if *current == *next {
                    current.clone()
                } else {
                    None
                };
                if *current != merged {
                    *current = merged;
                    changed = true;
                }
            }
            changed
        }
        None => {
            *slot = Some(incoming.to_vec());
            true
        }
    }
}

fn operand_constant(operand: &Operand) -> Option<Constant> {
    match operand {
        Operand::Constant(constant) => Some(constant.clone()),
        Operand::Copy(_) | Operand::Move(_) => None,
    }
}

fn fold_binary(op: &str, left: &Constant, right: &Constant) -> Option<Constant> {
    match (&left.value, &right.value, op) {
        (ConstantValue::Integer(lhs), ConstantValue::Integer(rhs), "Add") => {
            let value = lhs.parse::<i64>().ok()? + rhs.parse::<i64>().ok()?;
            Some(Constant {
                ty: left.ty.clone(),
                value: ConstantValue::Integer(value.to_string()),
            })
        }
        (ConstantValue::Integer(lhs), ConstantValue::Integer(rhs), "Subtract") => {
            let value = lhs.parse::<i64>().ok()? - rhs.parse::<i64>().ok()?;
            Some(Constant {
                ty: left.ty.clone(),
                value: ConstantValue::Integer(value.to_string()),
            })
        }
        (ConstantValue::Integer(lhs), ConstantValue::Integer(rhs), "Multiply") => {
            let value = lhs.parse::<i64>().ok()? * rhs.parse::<i64>().ok()?;
            Some(Constant {
                ty: left.ty.clone(),
                value: ConstantValue::Integer(value.to_string()),
            })
        }
        (ConstantValue::Integer(lhs), ConstantValue::Integer(rhs), "Divide") => {
            let divisor = rhs.parse::<i64>().ok()?;
            if divisor == 0 {
                return None;
            }
            let value = lhs.parse::<i64>().ok()? / divisor;
            Some(Constant {
                ty: left.ty.clone(),
                value: ConstantValue::Integer(value.to_string()),
            })
        }
        (ConstantValue::Bool(lhs), ConstantValue::Bool(rhs), "And") => Some(Constant {
            ty: left.ty.clone(),
            value: ConstantValue::Bool(*lhs && *rhs),
        }),
        (ConstantValue::Bool(lhs), ConstantValue::Bool(rhs), "Or") => Some(Constant {
            ty: left.ty.clone(),
            value: ConstantValue::Bool(*lhs || *rhs),
        }),
        (ConstantValue::String(lhs), ConstantValue::String(rhs), "Add") => Some(Constant {
            ty: left.ty.clone(),
            value: ConstantValue::String(format!("{lhs}{rhs}")),
        }),
        _ => None,
    }
}
