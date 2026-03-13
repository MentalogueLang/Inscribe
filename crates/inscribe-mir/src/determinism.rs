use crate::nodes::{ConstantValue, MirFunction, MirProgram, Operand, TerminatorKind};

// TODO: Replace the name-based heuristic with capability-aware determinism tracking.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeterminismIssue {
    pub function: String,
    pub callee: String,
}

pub fn find_nondeterministic_calls(program: &MirProgram) -> Vec<DeterminismIssue> {
    let mut issues = Vec::new();
    for function in &program.functions {
        issues.extend(find_in_function(function));
    }
    issues
}

fn find_in_function(function: &MirFunction) -> Vec<DeterminismIssue> {
    let mut issues = Vec::new();
    for block in &function.blocks {
        if let TerminatorKind::Call { callee, .. } = &block.terminator {
            if let Some(name) = callee_name(callee) {
                if is_nondeterministic(&name) {
                    issues.push(DeterminismIssue {
                        function: function.name.clone(),
                        callee: name,
                    });
                }
            }
        }
    }
    issues
}

fn callee_name(operand: &Operand) -> Option<String> {
    match operand {
        Operand::Constant(constant) => match &constant.value {
            ConstantValue::Function(name) => Some(name.clone()),
            _ => None,
        },
        Operand::Copy(_) | Operand::Move(_) => None,
    }
}

fn is_nondeterministic(name: &str) -> bool {
    let lowered = name.to_ascii_lowercase();
    ["random", "clock", "time", "uuid", "entropy"]
        .iter()
        .any(|needle| lowered.contains(needle))
}
