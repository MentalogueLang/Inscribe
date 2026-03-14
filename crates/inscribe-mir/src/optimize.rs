use crate::const_eval::{evaluate_constant_rvalue, fold_function_constants};
use crate::nodes::{
    BasicBlockData, BasicBlockId, Constant, ConstantValue, MirFunction, MirProgram, Operand,
    Rvalue, StatementKind, TerminatorKind,
};

pub fn optimize_program(program: &mut MirProgram) {
    for function in &mut program.functions {
        optimize_function(function);
    }
}

pub fn optimize_function(function: &mut MirFunction) {
    fold_function_constants(function);
    simplify_algebraic_identities(function);
    fold_function_constants(function);
    remove_unreachable_blocks(function);
}

fn simplify_algebraic_identities(function: &mut MirFunction) {
    for block in &mut function.blocks {
        for statement in &mut block.statements {
            let StatementKind::Assign(_, value) = &mut statement.kind else {
                continue;
            };

            let simplified = simplify_rvalue(value);
            *value = evaluate_constant_rvalue(&simplified)
                .map(|constant| Rvalue::Use(Operand::Constant(constant)))
                .unwrap_or(simplified);
        }
    }
}

fn simplify_rvalue(value: &Rvalue) -> Rvalue {
    match value {
        Rvalue::BinaryOp { op, left, right } => simplify_binary(op, left, right),
        _ => value.clone(),
    }
}

fn simplify_binary(op: &str, left: &Operand, right: &Operand) -> Rvalue {
    match op {
        "Add" => {
            if is_int_constant(left, 0) {
                return Rvalue::Use(right.clone());
            }
            if is_int_constant(right, 0) {
                return Rvalue::Use(left.clone());
            }
        }
        "Subtract" => {
            if is_int_constant(right, 0) {
                return Rvalue::Use(left.clone());
            }
        }
        "Multiply" => {
            if is_int_constant(left, 0) || is_int_constant(right, 0) {
                return Rvalue::Use(int_constant_like(left, 0));
            }
            if is_int_constant(left, 1) {
                return Rvalue::Use(right.clone());
            }
            if is_int_constant(right, 1) {
                return Rvalue::Use(left.clone());
            }
        }
        "Divide" => {
            if is_int_constant(right, 1) {
                return Rvalue::Use(left.clone());
            }
        }
        "And" => {
            if is_bool_constant(left, false) || is_bool_constant(right, false) {
                return Rvalue::Use(bool_constant_like(left, false));
            }
            if is_bool_constant(left, true) {
                return Rvalue::Use(right.clone());
            }
            if is_bool_constant(right, true) {
                return Rvalue::Use(left.clone());
            }
        }
        "Or" => {
            if is_bool_constant(left, true) || is_bool_constant(right, true) {
                return Rvalue::Use(bool_constant_like(left, true));
            }
            if is_bool_constant(left, false) {
                return Rvalue::Use(right.clone());
            }
            if is_bool_constant(right, false) {
                return Rvalue::Use(left.clone());
            }
        }
        _ => {}
    }

    Rvalue::BinaryOp {
        op: op.to_string(),
        left: left.clone(),
        right: right.clone(),
    }
}

fn remove_unreachable_blocks(function: &mut MirFunction) {
    let mut reachable = vec![false; function.blocks.len()];
    mark_reachable(function.entry, &function.blocks, &mut reachable);

    if reachable.iter().all(|value| *value) {
        return;
    }

    let mut remap = vec![None; function.blocks.len()];
    let mut blocks = Vec::new();

    for (index, block) in function.blocks.iter().enumerate() {
        if !reachable[index] {
            continue;
        }

        let id = BasicBlockId(blocks.len());
        remap[index] = Some(id);

        let mut rewritten = block.clone();
        rewritten.id = id;
        blocks.push(rewritten);
    }

    for block in &mut blocks {
        rewrite_terminator_targets(&mut block.terminator, &remap);
    }

    function.entry = remap[function.entry.0].expect("entry block should remain reachable");
    function.blocks = blocks;
}

fn mark_reachable(block: BasicBlockId, blocks: &[BasicBlockData], reachable: &mut [bool]) {
    if reachable[block.0] {
        return;
    }

    reachable[block.0] = true;

    for successor in successor_blocks(&blocks[block.0].terminator) {
        mark_reachable(successor, blocks, reachable);
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
            let mut blocks = arms.iter().map(|arm| arm.target).collect::<Vec<_>>();
            blocks.push(*otherwise);
            blocks
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

fn rewrite_terminator_targets(terminator: &mut TerminatorKind, remap: &[Option<BasicBlockId>]) {
    match terminator {
        TerminatorKind::Goto { target } => *target = remap[target.0].expect("reachable goto"),
        TerminatorKind::Branch {
            then_bb, else_bb, ..
        } => {
            *then_bb = remap[then_bb.0].expect("reachable branch target");
            *else_bb = remap[else_bb.0].expect("reachable branch target");
        }
        TerminatorKind::Match {
            arms, otherwise, ..
        } => {
            for arm in arms {
                arm.target = remap[arm.target.0].expect("reachable match target");
            }
            *otherwise = remap[otherwise.0].expect("reachable match fallback");
        }
        TerminatorKind::Call { target, .. } => {
            *target = remap[target.0].expect("reachable call target");
        }
        TerminatorKind::IterNext {
            loop_body, exit, ..
        } => {
            *loop_body = remap[loop_body.0].expect("reachable loop body");
            *exit = remap[exit.0].expect("reachable loop exit");
        }
        TerminatorKind::Try {
            ok_target,
            err_target,
            ..
        } => {
            *ok_target = remap[ok_target.0].expect("reachable try ok target");
            *err_target = remap[err_target.0].expect("reachable try err target");
        }
        TerminatorKind::Return | TerminatorKind::Unreachable => {}
    }
}

fn is_int_constant(operand: &Operand, expected: i64) -> bool {
    matches!(
        operand,
        Operand::Constant(Constant {
            value: ConstantValue::Integer(value),
            ..
        }) if value.parse::<i64>().ok() == Some(expected)
    )
}

fn is_bool_constant(operand: &Operand, expected: bool) -> bool {
    matches!(
        operand,
        Operand::Constant(Constant {
            value: ConstantValue::Bool(value),
            ..
        }) if *value == expected
    )
}

fn int_constant_like(source: &Operand, value: i64) -> Operand {
    let ty = match source {
        Operand::Constant(constant) => constant.ty.clone(),
        Operand::Copy(place) | Operand::Move(place) => {
            let _ = place;
            inscribe_typeck::Type::Int
        }
    };

    Operand::Constant(Constant {
        ty,
        value: ConstantValue::Integer(value.to_string()),
    })
}

fn bool_constant_like(source: &Operand, value: bool) -> Operand {
    let ty = match source {
        Operand::Constant(constant) => constant.ty.clone(),
        Operand::Copy(place) | Operand::Move(place) => {
            let _ = place;
            inscribe_typeck::Type::Bool
        }
    };

    Operand::Constant(Constant {
        ty,
        value: ConstantValue::Bool(value),
    })
}
