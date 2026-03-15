use std::collections::HashMap;

use crate::nodes::{ConstantValue, MirFunction, MirProgram, Operand, TerminatorKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeterminismIssue {
    pub function: String,
    pub callee: String,
}

pub fn find_nondeterministic_calls(program: &MirProgram) -> Vec<DeterminismIssue> {
    let nondeterministic = nondeterministic_functions(program);
    let mut issues = Vec::new();
    for function in &program.functions {
        issues.extend(find_in_function(function, &nondeterministic));
    }
    issues
}

fn find_in_function(
    function: &MirFunction,
    nondeterministic: &HashMap<String, bool>,
) -> Vec<DeterminismIssue> {
    let mut issues = Vec::new();
    for block in &function.blocks {
        if let TerminatorKind::Call { callee, .. } = &block.terminator {
            if let Some(name) = callee_name(callee) {
                if nondeterministic
                    .get(name.as_str())
                    .copied()
                    .unwrap_or_else(|| declaration_breaks_determinism(name.as_str()))
                {
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

fn nondeterministic_functions(program: &MirProgram) -> HashMap<String, bool> {
    let names = program
        .functions
        .iter()
        .map(callable_name)
        .collect::<Vec<_>>();
    let mut indices = HashMap::with_capacity(names.len());
    for (index, name) in names.iter().enumerate() {
        indices.insert(name.clone(), index);
    }

    let mut reverse_edges = vec![Vec::new(); names.len()];
    let mut nondeterministic = vec![false; names.len()];
    let mut stack = Vec::new();

    for (caller_index, function) in program.functions.iter().enumerate() {
        if function.is_declaration && declaration_breaks_determinism(function.name.as_str()) {
            nondeterministic[caller_index] = true;
            stack.push(caller_index);
        }

        for block in &function.blocks {
            let TerminatorKind::Call { callee, .. } = &block.terminator else {
                continue;
            };
            let Some(callee) = callee_name(callee) else {
                continue;
            };

            if let Some(&callee_index) = indices.get(callee.as_str()) {
                reverse_edges[callee_index].push(caller_index);
            } else if declaration_breaks_determinism(callee.as_str())
                && !nondeterministic[caller_index]
            {
                nondeterministic[caller_index] = true;
                stack.push(caller_index);
            }
        }
    }

    while let Some(callee_index) = stack.pop() {
        for &caller_index in &reverse_edges[callee_index] {
            if !nondeterministic[caller_index] {
                nondeterministic[caller_index] = true;
                stack.push(caller_index);
            }
        }
    }

    names.into_iter().zip(nondeterministic).collect()
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

fn callable_name(function: &MirFunction) -> String {
    match &function.receiver {
        Some(receiver) => format!("{receiver}.{}", function.name),
        None => function.name.clone(),
    }
}

fn declaration_breaks_determinism(name: &str) -> bool {
    runtime_capabilities(name)
        .iter()
        .any(|capability| capability.breaks_determinism())
}

fn runtime_capabilities(name: &str) -> &'static [Capability] {
    match name {
        "print_int" | "print_bool" | "print_string" | "print_newline" | "flush_stdout" => {
            &[Capability::Filesystem]
        }
        _ => &[Capability::ForeignHost],
    }
}

#[derive(Clone, Copy)]
enum Capability {
    Filesystem,
    ForeignHost,
}

impl Capability {
    fn breaks_determinism(self) -> bool {
        matches!(self, Self::ForeignHost)
    }
}
