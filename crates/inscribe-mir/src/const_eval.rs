use crate::nodes::{BasicBlockData, Constant, ConstantValue, Operand, Rvalue, StatementKind};

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
