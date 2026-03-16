use inscribe_mir::{
    BasicBlockId, MatchTarget, MirFunction, MirProgram, Operand, Place, ProjectionElem, Rvalue,
    StatementKind, TerminatorKind,
};
use std::fmt;
use std::sync::Arc;

use crate::boundary::{
    constant_to_value, ComptimeError, ComptimeResult, ComptimeValue, RangeValue, StructValue,
};
use crate::reflect::{qualified_function_name, MirReflection};
use crate::runtime::Runtime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InterpreterConfig {
    pub max_steps: usize,
    pub max_call_depth: usize,
}

impl Default for InterpreterConfig {
    fn default() -> Self {
        Self {
            max_steps: 100_000,
            max_call_depth: 128,
        }
    }
}

pub struct Interpreter<'a> {
    reflection: MirReflection<'a>,
    config: InterpreterConfig,
    runtime: Option<Arc<dyn Runtime>>,
}

impl<'a> fmt::Debug for Interpreter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Interpreter")
            .field("reflection", &self.reflection)
            .field("config", &self.config)
            .field("runtime", &self.runtime.as_ref().map(|_| "<runtime>"))
            .finish()
    }
}

impl<'a> Interpreter<'a> {
    pub fn new(program: &'a MirProgram) -> Self {
        Self {
            reflection: MirReflection::new(program),
            config: InterpreterConfig::default(),
            runtime: None,
        }
    }

    pub fn with_config(program: &'a MirProgram, config: InterpreterConfig) -> Self {
        Self {
            reflection: MirReflection::new(program),
            config,
            runtime: None,
        }
    }

    pub fn with_runtime(program: &'a MirProgram, runtime: Arc<dyn Runtime>) -> Self {
        Self {
            reflection: MirReflection::new(program),
            config: InterpreterConfig::default(),
            runtime: Some(runtime),
        }
    }

    pub fn reflection(&self) -> &MirReflection<'a> {
        &self.reflection
    }

    pub fn run_main(&self) -> ComptimeResult<ComptimeValue> {
        self.run_function("main", &[])
    }

    pub fn run_function(
        &self,
        name: &str,
        args: &[ComptimeValue],
    ) -> ComptimeResult<ComptimeValue> {
        let function = self
            .reflection
            .function(name)
            .ok_or_else(|| ComptimeError::new(format!("unknown compile-time function `{name}`")))?;
        let mut steps_left = self.config.max_steps;
        self.execute_function(function, args, 0, &mut steps_left)
    }

    fn execute_function(
        &self,
        function: &MirFunction,
        args: &[ComptimeValue],
        call_depth: usize,
        steps_left: &mut usize,
    ) -> ComptimeResult<ComptimeValue> {
        if function.is_declaration {
            if let Some(runtime) = &self.runtime {
                return runtime.call(&qualified_function_name(function), args);
            }
            return Err(ComptimeError::new(format!(
                "cannot execute declaration-only function `{}` at compile time",
                qualified_function_name(function)
            )));
        }
        if call_depth >= self.config.max_call_depth {
            return Err(ComptimeError::new(format!(
                "compile-time call depth exceeded while executing `{}`",
                qualified_function_name(function)
            )));
        }
        if function.signature.params.len() != args.len() {
            return Err(ComptimeError::new(format!(
                "function `{}` expected {} arguments but received {}",
                qualified_function_name(function),
                function.signature.params.len(),
                args.len()
            )));
        }

        let mut frame = Frame::new(function.locals.len());
        let param_base = function.return_local.0 + 1;
        for (index, value) in args.iter().enumerate() {
            frame.locals[param_base + index] = Some(value.clone());
        }

        let mut current = function.entry;
        loop {
            if *steps_left == 0 {
                return Err(ComptimeError::new(format!(
                    "compile-time execution step budget exhausted in `{}`",
                    qualified_function_name(function)
                )));
            }
            *steps_left -= 1;

            let block = &function.blocks[current.0];
            for statement in &block.statements {
                self.execute_statement(&mut frame, &statement.kind)?;
            }

            current = match &block.terminator {
                TerminatorKind::Goto { target } => *target,
                TerminatorKind::Branch {
                    condition,
                    then_bb,
                    else_bb,
                } => {
                    if self.eval_operand(&frame, condition)?.expect_bool()? {
                        *then_bb
                    } else {
                        *else_bb
                    }
                }
                TerminatorKind::Match {
                    discriminant,
                    arms,
                    otherwise,
                } => {
                    let value = self.eval_operand(&frame, discriminant)?;
                    self.select_match_target(&value, arms).unwrap_or(*otherwise)
                }
                TerminatorKind::Call {
                    callee,
                    args,
                    destination,
                    target,
                } => {
                    let callee_name = self.eval_callee(&frame, callee)?;
                    let arg_values = args
                        .iter()
                        .map(|arg| self.eval_operand(&frame, arg))
                        .collect::<ComptimeResult<Vec<_>>>()?;
                    let result = match callee_name.as_str() {
                        "Ok" => {
                            let payload = arg_values.into_iter().next().ok_or_else(|| {
                                ComptimeError::new("constructor `Ok` expects 1 argument")
                            })?;
                            ComptimeValue::ResultOk(Box::new(payload))
                        }
                        "Err" => {
                            let payload = arg_values.into_iter().next().ok_or_else(|| {
                                ComptimeError::new("constructor `Err` expects 1 argument")
                            })?;
                            ComptimeValue::ResultErr(Box::new(payload))
                        }
                        _ => {
                            let callee = self.reflection.function(&callee_name).ok_or_else(|| {
                                ComptimeError::new(format!(
                                    "unknown compile-time callee `{callee_name}`"
                                ))
                            })?;
                            self.execute_function(callee, &arg_values, call_depth + 1, steps_left)?
                        }
                    };
                    if let Some(destination) = destination {
                        assign_place(&mut frame, destination, result)?;
                    }
                    *target
                }
                TerminatorKind::IterNext {
                    iterator,
                    binding,
                    loop_body,
                    exit,
                } => {
                    if let Some(next) = self.advance_iterator(&mut frame, iterator)? {
                        frame.locals[binding.0] = Some(next);
                        *loop_body
                    } else {
                        *exit
                    }
                }
                TerminatorKind::Try {
                    operand,
                    ok_local,
                    err_local,
                    ok_target,
                    err_target,
                } => match self.eval_operand(&frame, operand)? {
                    ComptimeValue::ResultOk(value) => {
                        frame.locals[ok_local.0] = Some(*value);
                        *ok_target
                    }
                    ComptimeValue::ResultErr(value) => {
                        frame.locals[err_local.0] = Some(*value);
                        *err_target
                    }
                    other => {
                        return Err(ComptimeError::new(format!(
                            "try terminator expected Result, found {}",
                            other.kind_name()
                        )));
                    }
                },
                TerminatorKind::Return => {
                    return Ok(frame.locals[function.return_local.0]
                        .clone()
                        .unwrap_or(ComptimeValue::Unit));
                }
                TerminatorKind::Unreachable => {
                    return Err(ComptimeError::new(format!(
                        "entered unreachable block in `{}`",
                        qualified_function_name(function)
                    )));
                }
            };
        }
    }

    fn execute_statement(&self, frame: &mut Frame, statement: &StatementKind) -> ComptimeResult<()> {
        match statement {
            StatementKind::StorageLive(_) | StatementKind::Nop => Ok(()),
            StatementKind::StorageDead(local) | StatementKind::Drop(local) => {
                frame.locals[local.0] = None;
                Ok(())
            }
            StatementKind::Assign(place, value) => {
                let value = self.eval_rvalue(frame, value)?;
                assign_place(frame, place, value)
            }
        }
    }

    fn eval_callee(&self, frame: &Frame, operand: &Operand) -> ComptimeResult<String> {
        match self.eval_operand(frame, operand)? {
            ComptimeValue::Function(name) => Ok(name),
            other => Err(ComptimeError::new(format!(
                "expected function operand, found {}",
                other.kind_name()
            ))),
        }
    }

    fn eval_rvalue(&self, frame: &Frame, rvalue: &Rvalue) -> ComptimeResult<ComptimeValue> {
        match rvalue {
            Rvalue::Use(operand) => self.eval_operand(frame, operand),
            Rvalue::UnaryOp { op, operand } => {
                let value = self.eval_operand(frame, operand)?;
                apply_unary(op, value)
            }
            Rvalue::BinaryOp { op, left, right } => {
                let left = self.eval_operand(frame, left)?;
                let right = self.eval_operand(frame, right)?;
                apply_binary(op, left, right)
            }
            Rvalue::AggregateStruct { path, fields } => Ok(ComptimeValue::Struct(StructValue::new(
                path.clone(),
                fields
                    .iter()
                    .map(|(name, value)| Ok((name.clone(), self.eval_operand(frame, value)?)))
                    .collect::<ComptimeResult<Vec<_>>>()?,
            ))),
            Rvalue::AggregateArray { .. } | Rvalue::RepeatArray { .. } => Err(ComptimeError::new(
                "compile-time execution does not yet support array aggregates",
            )),
            Rvalue::ResultOk(operand) => {
                Ok(ComptimeValue::ResultOk(Box::new(self.eval_operand(frame, operand)?)))
            }
            Rvalue::ResultErr(operand) => {
                Ok(ComptimeValue::ResultErr(Box::new(self.eval_operand(frame, operand)?)))
            }
        }
    }

    fn eval_operand(&self, frame: &Frame, operand: &Operand) -> ComptimeResult<ComptimeValue> {
        match operand {
            Operand::Copy(place) | Operand::Move(place) => read_place(frame, place),
            Operand::Constant(constant) => Ok(constant_to_value(constant)),
        }
    }

    fn select_match_target(
        &self,
        value: &ComptimeValue,
        arms: &[MatchTarget],
    ) -> Option<BasicBlockId> {
        arms.iter()
            .find_map(|arm| pattern_matches(value, &arm.pattern).then_some(arm.target))
    }

    fn advance_iterator(
        &self,
        frame: &mut Frame,
        iterator: &Place,
    ) -> ComptimeResult<Option<ComptimeValue>> {
        if !iterator.projection.is_empty() {
            return Err(ComptimeError::new(
                "projected iterators are not supported at compile time",
            ));
        }

        let slot = frame
            .locals
            .get_mut(iterator.local.0)
            .and_then(Option::as_mut)
            .ok_or_else(|| ComptimeError::new("iterator local is uninitialized"))?;
        match slot {
            ComptimeValue::Range(range) => {
                if range.next >= range.end {
                    Ok(None)
                } else {
                    let value = range.next;
                    range.next += 1;
                    Ok(Some(ComptimeValue::Integer(value)))
                }
            }
            other => Err(ComptimeError::new(format!(
                "cannot iterate over {}",
                other.kind_name()
            ))),
        }
    }
}

#[derive(Debug)]
struct Frame {
    locals: Vec<Option<ComptimeValue>>,
}

impl Frame {
    fn new(local_count: usize) -> Self {
        Self {
            locals: vec![None; local_count],
        }
    }
}

fn read_place(frame: &Frame, place: &Place) -> ComptimeResult<ComptimeValue> {
    let value = frame
        .locals
        .get(place.local.0)
        .and_then(Option::as_ref)
        .ok_or_else(|| ComptimeError::new(format!("local {} is uninitialized", place.local.0)))?;
    project_value(value, &place.projection)
}

fn project_value(value: &ComptimeValue, projection: &[ProjectionElem]) -> ComptimeResult<ComptimeValue> {
    let mut current = value;
    for elem in projection {
        current = match (elem, current) {
            (ProjectionElem::Field(name), ComptimeValue::Struct(value)) => value.field(name).ok_or_else(
                || ComptimeError::new(format!("struct field `{name}` does not exist")),
            )?,
            (ProjectionElem::Index(_), _) => {
                return Err(ComptimeError::new(
                    "compile-time execution does not yet support indexed projections",
                ));
            }
            (ProjectionElem::Field(name), other) => {
                return Err(ComptimeError::new(format!(
                    "cannot access field `{name}` on {}",
                    other.kind_name()
                )));
            }
        };
    }
    Ok(current.clone())
}

fn assign_place(frame: &mut Frame, place: &Place, value: ComptimeValue) -> ComptimeResult<()> {
    let slot = frame
        .locals
        .get_mut(place.local.0)
        .ok_or_else(|| ComptimeError::new(format!("unknown local {}", place.local.0)))?;
    if place.projection.is_empty() {
        *slot = Some(value);
        return Ok(());
    }

    let root = slot
        .as_mut()
        .ok_or_else(|| ComptimeError::new(format!("local {} is uninitialized", place.local.0)))?;
    assign_projection(root, &place.projection, value)
}

fn assign_projection(
    current: &mut ComptimeValue,
    projection: &[ProjectionElem],
    value: ComptimeValue,
) -> ComptimeResult<()> {
    if projection.is_empty() {
        *current = value;
        return Ok(());
    }

    match (&projection[0], current) {
        (ProjectionElem::Field(name), ComptimeValue::Struct(struct_value)) => {
            let field = struct_value.field_mut(name).ok_or_else(|| {
                ComptimeError::new(format!("struct field `{name}` does not exist"))
            })?;
            assign_projection(field, &projection[1..], value)
        }
        (ProjectionElem::Index(_), _) => Err(ComptimeError::new(
            "compile-time execution does not yet support indexed assignment",
        )),
        (ProjectionElem::Field(name), other) => Err(ComptimeError::new(format!(
            "cannot assign field `{name}` on {}",
            other.kind_name()
        ))),
    }
}

fn apply_unary(op: &str, value: ComptimeValue) -> ComptimeResult<ComptimeValue> {
    match (op, value) {
        ("Negate", ComptimeValue::Integer(value)) => Ok(ComptimeValue::Integer(-value)),
        ("Negate", ComptimeValue::Float(value)) => Ok(ComptimeValue::Float(-value)),
        ("Not", ComptimeValue::Bool(value)) => Ok(ComptimeValue::Bool(!value)),
        (op, value) => Err(ComptimeError::new(format!(
            "unsupported unary operation `{op}` for {}",
            value.kind_name()
        ))),
    }
}

fn apply_binary(op: &str, left: ComptimeValue, right: ComptimeValue) -> ComptimeResult<ComptimeValue> {
    match op {
        "Add" => match (left, right) {
            (ComptimeValue::Integer(left), ComptimeValue::Integer(right)) => {
                Ok(ComptimeValue::Integer(left + right))
            }
            (ComptimeValue::Float(left), ComptimeValue::Float(right)) => {
                Ok(ComptimeValue::Float(left + right))
            }
            (ComptimeValue::String(left), ComptimeValue::String(right)) => {
                Ok(ComptimeValue::String(format!("{left}{right}")))
            }
            (left, right) => binary_type_error(op, &left, &right),
        },
        "Subtract" => match (left, right) {
            (ComptimeValue::Integer(left), ComptimeValue::Integer(right)) => {
                Ok(ComptimeValue::Integer(left - right))
            }
            (ComptimeValue::Float(left), ComptimeValue::Float(right)) => {
                Ok(ComptimeValue::Float(left - right))
            }
            (left, right) => binary_type_error(op, &left, &right),
        },
        "Multiply" => match (left, right) {
            (ComptimeValue::Integer(left), ComptimeValue::Integer(right)) => {
                Ok(ComptimeValue::Integer(left * right))
            }
            (ComptimeValue::Float(left), ComptimeValue::Float(right)) => {
                Ok(ComptimeValue::Float(left * right))
            }
            (left, right) => binary_type_error(op, &left, &right),
        },
        "Divide" => match (left, right) {
            (ComptimeValue::Integer(_), ComptimeValue::Integer(0)) => {
                Err(ComptimeError::new("division by zero"))
            }
            (ComptimeValue::Float(_), ComptimeValue::Float(0.0)) => {
                Err(ComptimeError::new("division by zero"))
            }
            (ComptimeValue::Integer(left), ComptimeValue::Integer(right)) => {
                Ok(ComptimeValue::Integer(left / right))
            }
            (ComptimeValue::Float(left), ComptimeValue::Float(right)) => {
                Ok(ComptimeValue::Float(left / right))
            }
            (left, right) => binary_type_error(op, &left, &right),
        },
        "Equal" => Ok(ComptimeValue::Bool(left == right)),
        "NotEqual" => Ok(ComptimeValue::Bool(left != right)),
        "Less" => compare_values(op, left, right, |left, right| left < right),
        "LessEqual" => compare_values(op, left, right, |left, right| left <= right),
        "Greater" => compare_values(op, left, right, |left, right| left > right),
        "GreaterEqual" => compare_values(op, left, right, |left, right| left >= right),
        "And" => match (left, right) {
            (ComptimeValue::Bool(left), ComptimeValue::Bool(right)) => {
                Ok(ComptimeValue::Bool(left && right))
            }
            (left, right) => binary_type_error(op, &left, &right),
        },
        "Or" => match (left, right) {
            (ComptimeValue::Bool(left), ComptimeValue::Bool(right)) => {
                Ok(ComptimeValue::Bool(left || right))
            }
            (left, right) => binary_type_error(op, &left, &right),
        },
        "Range" => match (left, right) {
            (ComptimeValue::Integer(start), ComptimeValue::Integer(end)) => {
                Ok(ComptimeValue::Range(RangeValue { next: start, end }))
            }
            (left, right) => {
                let start = coerce_int(&left);
                let end = coerce_int(&right);
                match (start, end) {
                    (Some(start), Some(end)) => Ok(ComptimeValue::Range(RangeValue {
                        next: start,
                        end,
                    })),
                    _ => binary_type_error(op, &left, &right),
                }
            }
        },
        _ => Err(ComptimeError::new(format!(
            "unsupported binary operation `{op}`"
        ))),
    }
}

fn compare_values<F>(
    op: &str,
    left: ComptimeValue,
    right: ComptimeValue,
    compare: F,
) -> ComptimeResult<ComptimeValue>
where
    F: FnOnce(f64, f64) -> bool,
{
    match (left, right) {
        (ComptimeValue::Integer(left), ComptimeValue::Integer(right)) => {
            Ok(ComptimeValue::Bool(compare(left as f64, right as f64)))
        }
        (ComptimeValue::Float(left), ComptimeValue::Float(right)) => {
            Ok(ComptimeValue::Bool(compare(left, right)))
        }
        (left, right) => binary_type_error(op, &left, &right),
    }
}

fn binary_type_error(
    op: &str,
    left: &ComptimeValue,
    right: &ComptimeValue,
) -> ComptimeResult<ComptimeValue> {
    Err(ComptimeError::new(format!(
        "unsupported binary operation `{op}` for {} and {}",
        left.kind_name(),
        right.kind_name()
    )))
}

fn coerce_int(value: &ComptimeValue) -> Option<i64> {
    match value {
        ComptimeValue::Integer(value) => Some(*value),
        ComptimeValue::String(value) => value.parse().ok(),
        _ => None,
    }
}

fn pattern_matches(value: &ComptimeValue, pattern: &str) -> bool {
    let pattern = pattern.trim();
    if pattern == "_" {
        return true;
    }
    if let Some((head, args)) = split_constructor_pattern(pattern) {
        return constructor_matches(value, head, &args);
    }
    if let Some(literal) = parse_pattern_literal(pattern) {
        return value == &literal;
    }
    if pattern.contains('.') {
        return value.display() == pattern;
    }
    true
}

fn constructor_matches(value: &ComptimeValue, head: &str, args: &[&str]) -> bool {
    match (head, value) {
        ("Ok", ComptimeValue::ResultOk(inner)) | ("Err", ComptimeValue::ResultErr(inner)) => {
            args.len() == 1 && pattern_matches(inner, args[0])
        }
        (head, ComptimeValue::Struct(value)) if value.type_name() == head => {
            args.len() == value.fields.len()
                && value
                    .fields
                    .iter()
                    .zip(args.iter())
                    .all(|((_, field), pattern)| pattern_matches(field, pattern))
        }
        _ => false,
    }
}

fn split_constructor_pattern(pattern: &str) -> Option<(&str, Vec<&str>)> {
    let open = pattern.find('(')?;
    if !pattern.ends_with(')') {
        return None;
    }
    let head = pattern[..open].trim();
    let inner = &pattern[open + 1..pattern.len() - 1];
    let args = if inner.trim().is_empty() {
        Vec::new()
    } else {
        inner.split(',').map(str::trim).collect()
    };
    Some((head, args))
}

fn parse_pattern_literal(pattern: &str) -> Option<ComptimeValue> {
    if pattern == "true" {
        return Some(ComptimeValue::Bool(true));
    }
    if pattern == "false" {
        return Some(ComptimeValue::Bool(false));
    }
    if pattern.starts_with('"') && pattern.ends_with('"') && pattern.len() >= 2 {
        return Some(ComptimeValue::String(pattern[1..pattern.len() - 1].to_string()));
    }
    if let Ok(value) = pattern.parse::<i64>() {
        return Some(ComptimeValue::Integer(value));
    }
    if let Ok(value) = pattern.parse::<f64>() {
        return Some(ComptimeValue::Float(value));
    }
    None
}
