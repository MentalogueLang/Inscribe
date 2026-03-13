use crate::nodes::{MirFunction, Place, StatementKind};

// TODO: Replace this with real aliasing and lifetime analysis when references exist in MIR.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BorrowIssue {
    pub local: String,
    pub message: String,
}

pub fn check_mutable_assignments(function: &MirFunction) -> Vec<BorrowIssue> {
    let mut issues = Vec::new();

    for block in &function.blocks {
        for statement in &block.statements {
            if let StatementKind::Assign(place, _) = &statement.kind {
                check_place(function, place, &mut issues);
            }
        }
    }

    issues
}

fn check_place(function: &MirFunction, place: &Place, issues: &mut Vec<BorrowIssue>) {
    let Some(local) = function.locals.get(place.local.0) else {
        return;
    };

    if !local.mutable && local.name != "_return" {
        issues.push(BorrowIssue {
            local: local.name.clone(),
            message: format!("assignment to immutable local `{}`", local.name),
        });
    }
}
