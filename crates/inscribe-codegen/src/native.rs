use std::collections::HashMap;

use inscribe_mir::{
    optimize_program, BasicBlockId, ConstantValue, MirFunction, MirProgram, Operand, Place,
    ProjectionElem, Rvalue, StatementKind, TerminatorKind,
};
use inscribe_typeck::Type;

use crate::targets::{Architecture, OperatingSystem, Target};
use crate::CodegenError;

pub fn emit_assembly(program: &MirProgram, target: Target) -> Result<String, CodegenError> {
    let lowered = lower_program(program, target)?;
    Ok(render_assembly(&lowered, target))
}

pub fn emit_executable(program: &MirProgram, target: Target) -> Result<Vec<u8>, CodegenError> {
    let lowered = lower_program(program, target)?;
    match target.os {
        OperatingSystem::Linux => emit_elf(&lowered),
        OperatingSystem::Windows => emit_pe(&lowered),
    }
}

fn lower_program(program: &MirProgram, target: Target) -> Result<LoweredProgram, CodegenError> {
    if target.arch != Architecture::X86_64 {
        return Err(CodegenError::new(
            "only x86-64 native codegen is currently implemented",
        ));
    }

    let mut program = program.clone();
    optimize_program(&mut program);

    let Some(main_index) = program
        .functions
        .iter()
        .position(|function| function.receiver.is_none() && function.name == "main")
    else {
        return Err(CodegenError::new(
            "native codegen requires a top-level `main` function",
        ));
    };

    let mut instructions = Vec::new();
    let labels = program
        .functions
        .iter()
        .map(|function| (callable_name(function), function_label(function)))
        .collect::<HashMap<_, _>>();

    emit_entry_wrapper(&program.functions[main_index], target, &mut instructions);

    for function in &program.functions {
        lower_function(function, &labels, target, &mut instructions)?;
    }

    Ok(LoweredProgram {
        entry_label: target.entry_symbol().to_string(),
        instructions,
    })
}

fn lower_function(
    function: &MirFunction,
    labels: &HashMap<String, String>,
    target: Target,
    instructions: &mut Vec<Instruction>,
) -> Result<(), CodegenError> {
    let stack = StackLayout::new(function, target)?;
    instructions.push(Instruction::Label(function_label(function)));
    if stack.total_size > 0 {
        instructions.push(Instruction::SubRsp(stack.total_size as u32));
    }
    spill_params(function, &stack, target, instructions)?;
    instructions.push(Instruction::Jump(block_label(function, function.entry)));

    for block in &function.blocks {
        instructions.push(Instruction::Label(block_label(function, block.id)));
        lower_block(function, block, &stack, labels, target, instructions)?;
    }

    Ok(())
}

fn lower_block(
    function: &MirFunction,
    block: &inscribe_mir::BasicBlockData,
    stack: &StackLayout,
    labels: &HashMap<String, String>,
    codegen_target: Target,
    instructions: &mut Vec<Instruction>,
) -> Result<(), CodegenError> {
    for statement in &block.statements {
        match &statement.kind {
            StatementKind::StorageLive(_)
            | StatementKind::StorageDead(_)
            | StatementKind::Drop(_)
            | StatementKind::Nop => {}
            StatementKind::Assign(place, value) => {
                lower_assign(function, place, value, stack, instructions)?
            }
        }
    }

    match &block.terminator {
        TerminatorKind::Goto { target } => {
            instructions.push(Instruction::Jump(block_label(function, *target)));
        }
        TerminatorKind::Branch {
            condition,
            then_bb,
            else_bb,
        } => {
            load_operand(condition, Register::Rax, stack, instructions)?;
            instructions.push(Instruction::CmpRegImm(Register::Rax, 0));
            instructions.push(Instruction::JumpIf(
                Condition::NotEqual,
                block_label(function, *then_bb),
            ));
            instructions.push(Instruction::Jump(block_label(function, *else_bb)));
        }
        TerminatorKind::Return => emit_function_return(function, stack, instructions)?,
        TerminatorKind::Unreachable => instructions.push(Instruction::Ud2),
        TerminatorKind::Match { .. } => {
            return Err(CodegenError::new(
                "native codegen does not yet support MIR `match` terminators",
            ))
        }
        TerminatorKind::Call {
            callee,
            args,
            destination,
            target: next_block,
        } => lower_call_terminator(
            function,
            callee,
            args,
            destination.as_ref(),
            *next_block,
            stack,
            labels,
            codegen_target,
            instructions,
        )?,
        TerminatorKind::IterNext { .. } => {
            return Err(CodegenError::new(
                "native codegen does not yet support MIR `for` iterators",
            ))
        }
        TerminatorKind::Try { .. } => {
            return Err(CodegenError::new(
                "native codegen does not yet support MIR `?` lowering",
            ))
        }
    }

    Ok(())
}

fn lower_assign(
    function: &MirFunction,
    place: &Place,
    value: &Rvalue,
    stack: &StackLayout,
    instructions: &mut Vec<Instruction>,
) -> Result<(), CodegenError> {
    if !place.projection.is_empty() {
        return Err(CodegenError::new(
            "native codegen does not yet support field projections",
        ));
    }

    ensure_supported_local_type(function, place.local.0)?;
    lower_rvalue(value, stack, instructions)?;
    instructions.push(Instruction::MovStackReg(
        stack.offset_for(place.local.0)?,
        Register::Rax,
    ));
    Ok(())
}

fn lower_rvalue(
    value: &Rvalue,
    stack: &StackLayout,
    instructions: &mut Vec<Instruction>,
) -> Result<(), CodegenError> {
    match value {
        Rvalue::Use(operand) => load_operand(operand, Register::Rax, stack, instructions),
        Rvalue::UnaryOp { op, operand } => {
            load_operand(operand, Register::Rax, stack, instructions)?;
            match op.as_str() {
                "Negate" => instructions.push(Instruction::NegReg(Register::Rax)),
                "Not" => {
                    instructions.push(Instruction::CmpRegImm(Register::Rax, 0));
                    instructions.push(Instruction::SetCondAl(Condition::Equal));
                    instructions.push(Instruction::MovzxEaxAl);
                }
                _ => {
                    return Err(CodegenError::new(format!(
                        "unsupported unary operator `{op}`"
                    )))
                }
            }
            Ok(())
        }
        Rvalue::BinaryOp { op, left, right } => {
            load_operand(left, Register::Rax, stack, instructions)?;
            load_operand(right, Register::Rcx, stack, instructions)?;
            match op.as_str() {
                "Add" => instructions.push(Instruction::AddRegReg(Register::Rax, Register::Rcx)),
                "Subtract" => {
                    instructions.push(Instruction::SubRegReg(Register::Rax, Register::Rcx))
                }
                "Multiply" => {
                    instructions.push(Instruction::IMulRegReg(Register::Rax, Register::Rcx))
                }
                "Divide" => {
                    instructions.push(Instruction::Cqo);
                    instructions.push(Instruction::IDivReg(Register::Rcx));
                }
                "And" => instructions.push(Instruction::AndRegReg(Register::Rax, Register::Rcx)),
                "Or" => instructions.push(Instruction::OrRegReg(Register::Rax, Register::Rcx)),
                "Equal" => emit_compare(Condition::Equal, instructions),
                "NotEqual" => emit_compare(Condition::NotEqual, instructions),
                "Less" => emit_compare(Condition::Less, instructions),
                "LessEqual" => emit_compare(Condition::LessEqual, instructions),
                "Greater" => emit_compare(Condition::Greater, instructions),
                "GreaterEqual" => emit_compare(Condition::GreaterEqual, instructions),
                other => {
                    return Err(CodegenError::new(format!(
                        "native codegen does not yet support binary operator `{other}`"
                    )))
                }
            }
            Ok(())
        }
        Rvalue::AggregateStruct { .. } => Err(CodegenError::new(
            "native codegen does not yet support struct aggregates",
        )),
        Rvalue::ResultOk(_) | Rvalue::ResultErr(_) => Err(CodegenError::new(
            "native codegen does not yet support result aggregates",
        )),
    }
}

fn lower_call_terminator(
    function: &MirFunction,
    callee: &Operand,
    args: &[Operand],
    destination: Option<&Place>,
    target: BasicBlockId,
    stack: &StackLayout,
    labels: &HashMap<String, String>,
    target_info: Target,
    instructions: &mut Vec<Instruction>,
) -> Result<(), CodegenError> {
    let Some(callee_name) = direct_callee_name(callee) else {
        return Err(CodegenError::new(
            "native codegen only supports direct function calls for now",
        ));
    };

    let Some(label) = labels.get(callee_name) else {
        return Err(CodegenError::new(format!(
            "native codegen could not find callee `{callee_name}`"
        )));
    };

    let arg_registers = argument_registers(target_info);
    let register_arg_count = args.len().min(arg_registers.len());
    for (operand, register) in args.iter().take(register_arg_count).zip(arg_registers.iter()) {
        load_operand(operand, *register, stack, instructions)?;
    }
    for (stack_index, operand) in args.iter().skip(register_arg_count).enumerate() {
        load_operand(operand, Register::Rax, stack, instructions)?;
        instructions.push(Instruction::MovStackReg(
            stack.outgoing_arg_offset(target_info, stack_index)?,
            Register::Rax,
        ));
    }

    instructions.push(Instruction::Call(label.clone()));

    if let Some(place) = destination {
        ensure_supported_local_type(function, place.local.0)?;
        instructions.push(Instruction::MovStackReg(
            stack.offset_for(place.local.0)?,
            Register::Rax,
        ));
    }

    instructions.push(Instruction::Jump(block_label(function, target)));
    Ok(())
}

fn emit_compare(condition: Condition, instructions: &mut Vec<Instruction>) {
    instructions.push(Instruction::CmpRegReg(Register::Rax, Register::Rcx));
    instructions.push(Instruction::SetCondAl(condition));
    instructions.push(Instruction::MovzxEaxAl);
}

fn emit_entry_wrapper(main: &MirFunction, target: Target, instructions: &mut Vec<Instruction>) {
    instructions.push(Instruction::Label(target.entry_symbol().to_string()));

    match target.os {
        OperatingSystem::Linux => {
            instructions.push(Instruction::Call(function_label(main)));
            instructions.push(Instruction::MovRegReg(Register::Rdi, Register::Rax));
            instructions.push(Instruction::MovRegImm64(Register::Rax, 60));
            instructions.push(Instruction::Syscall);
            instructions.push(Instruction::Ud2);
        }
        OperatingSystem::Windows => {
            instructions.push(Instruction::SubRsp(40));
            instructions.push(Instruction::Call(function_label(main)));
            instructions.push(Instruction::AddRsp(40));
            instructions.push(Instruction::Ret);
        }
    }
}

fn spill_params(
    function: &MirFunction,
    stack: &StackLayout,
    target: Target,
    instructions: &mut Vec<Instruction>,
) -> Result<(), CodegenError> {
    let arg_registers = argument_registers(target);
    let register_arg_count = function.signature.params.len().min(arg_registers.len());
    for (index, register) in arg_registers.iter().take(register_arg_count).enumerate() {
        let local_index = 1 + index;
        if local_index >= function.locals.len() {
            break;
        }
        instructions.push(Instruction::MovStackReg(
            stack.offset_for(local_index)?,
            *register,
        ));
    }
    for index in register_arg_count..function.signature.params.len() {
        let local_index = 1 + index;
        if local_index >= function.locals.len() {
            break;
        }
        instructions.push(Instruction::MovRegStack(
            Register::Rax,
            stack.incoming_arg_offset(target, index - register_arg_count)?,
        ));
        instructions.push(Instruction::MovStackReg(
            stack.offset_for(local_index)?,
            Register::Rax,
        ));
    }
    Ok(())
}

fn emit_function_return(
    function: &MirFunction,
    stack: &StackLayout,
    instructions: &mut Vec<Instruction>,
) -> Result<(), CodegenError> {
    match function.signature.return_type.as_ref() {
        Type::Int | Type::Bool => instructions.push(Instruction::MovRegStack(
            Register::Rax,
            stack.offset_for(function.return_local.0)?,
        )),
        Type::Unit => instructions.push(Instruction::MovRegImm64(Register::Rax, 0)),
        other => {
            return Err(CodegenError::new(format!(
                "native codegen currently only supports direct returns of `int`, `bool`, or `()`, found `{}`",
                other.display_name()
            )))
        }
    }

    if stack.total_size > 0 {
        instructions.push(Instruction::AddRsp(stack.total_size as u32));
    }
    instructions.push(Instruction::Ret);
    Ok(())
}

fn direct_callee_name(callee: &Operand) -> Option<&str> {
    match callee {
        Operand::Constant(inscribe_mir::Constant {
            value: ConstantValue::Function(name),
            ..
        }) => Some(name.as_str()),
        _ => None,
    }
}

fn argument_registers(target: Target) -> &'static [Register] {
    match target.os {
        OperatingSystem::Linux => &[
            Register::Rdi,
            Register::Rsi,
            Register::Rdx,
            Register::Rcx,
            Register::R8,
            Register::R9,
        ][..],
        OperatingSystem::Windows => &[
            Register::Rcx,
            Register::Rdx,
            Register::R8,
            Register::R9,
        ][..],
    }
}

fn load_operand(
    operand: &Operand,
    destination: Register,
    stack: &StackLayout,
    instructions: &mut Vec<Instruction>,
) -> Result<(), CodegenError> {
    match operand {
        Operand::Copy(place) | Operand::Move(place) => {
            if place
                .projection
                .iter()
                .any(|projection| matches!(projection, ProjectionElem::Field(_)))
            {
                return Err(CodegenError::new(
                    "native codegen does not yet support field projections",
                ));
            }
            instructions.push(Instruction::MovRegStack(
                destination,
                stack.offset_for(place.local.0)?,
            ));
            Ok(())
        }
        Operand::Constant(constant) => load_constant(&constant.value, destination, instructions),
    }
}

fn load_constant(
    value: &ConstantValue,
    destination: Register,
    instructions: &mut Vec<Instruction>,
) -> Result<(), CodegenError> {
    let immediate = match value {
        ConstantValue::Unit => 0,
        ConstantValue::Integer(value) => value
            .parse::<i64>()
            .map_err(|_| CodegenError::new(format!("invalid integer literal `{value}` in MIR")))?,
        ConstantValue::Bool(value) => i64::from(*value),
        ConstantValue::Float(_) => {
            return Err(CodegenError::new(
                "native x86-64 codegen does not yet support floating-point constants",
            ))
        }
        ConstantValue::String(_) => {
            return Err(CodegenError::new(
                "native x86-64 codegen does not yet support string constants",
            ))
        }
        ConstantValue::Function(name) => {
            return Err(CodegenError::new(format!(
                "native x86-64 codegen cannot materialize function value `{name}` yet"
            )))
        }
    };

    instructions.push(Instruction::MovRegImm64(destination, immediate));
    Ok(())
}

fn ensure_supported_local_type(function: &MirFunction, local: usize) -> Result<(), CodegenError> {
    let Some(local_decl) = function.locals.get(local) else {
        return Err(CodegenError::new(format!(
            "MIR local `{local}` does not exist"
        )));
    };

    if is_supported_scalar_type(&local_decl.ty) {
        Ok(())
    } else {
        Err(CodegenError::new(format!(
            "native codegen does not yet support local `{}` of type `{}`",
            local_decl.name,
            local_decl.ty.display_name()
        )))
    }
}

fn is_supported_scalar_type(ty: &Type) -> bool {
    matches!(ty, Type::Int | Type::Bool | Type::Unit)
}

fn callable_name(function: &MirFunction) -> String {
    function
        .receiver
        .as_ref()
        .map(|receiver| format!("{receiver}.{}", function.name))
        .unwrap_or_else(|| function.name.clone())
}

fn function_label(function: &MirFunction) -> String {
    format!("__ml_fn_{}", sanitize_symbol(&callable_name(function)))
}

fn block_label(function: &MirFunction, block: BasicBlockId) -> String {
    format!("{}.Lbb{}", function_label(function), block.0)
}

fn sanitize_symbol(name: &str) -> String {
    name.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn render_assembly(program: &LoweredProgram, _target: Target) -> String {
    let mut out = String::new();
    out.push_str(".intel_syntax noprefix\n");
    out.push_str(".text\n");
    out.push_str(".global ");
    out.push_str(&program.entry_label);
    out.push('\n');

    for instruction in &program.instructions {
        match instruction {
            Instruction::Label(name) => {
                out.push_str(name);
                out.push_str(":\n");
            }
            _ => {
                out.push_str("    ");
                out.push_str(&instruction.render());
                out.push('\n');
            }
        }
    }

    out
}

fn emit_elf(program: &LoweredProgram) -> Result<Vec<u8>, CodegenError> {
    let code_offset = 0x1000usize;
    let base_vaddr = 0x400000u64;
    let code = encode(program, EncodeContext {})?;
    let entry = base_vaddr + code_offset as u64 + program.entry_offset() as u64;
    let file_size = code_offset + code.len();

    let mut bytes = Vec::with_capacity(file_size);
    bytes.extend_from_slice(b"\x7FELF");
    bytes.push(2);
    bytes.push(1);
    bytes.push(1);
    bytes.push(0);
    bytes.extend_from_slice(&[0; 8]);
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&62u16.to_le_bytes());
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&entry.to_le_bytes());
    bytes.extend_from_slice(&64u64.to_le_bytes());
    bytes.extend_from_slice(&0u64.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&64u16.to_le_bytes());
    bytes.extend_from_slice(&56u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());

    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&5u32.to_le_bytes());
    bytes.extend_from_slice(&0u64.to_le_bytes());
    bytes.extend_from_slice(&base_vaddr.to_le_bytes());
    bytes.extend_from_slice(&base_vaddr.to_le_bytes());
    bytes.extend_from_slice(&(file_size as u64).to_le_bytes());
    bytes.extend_from_slice(&(file_size as u64).to_le_bytes());
    bytes.extend_from_slice(&0x1000u64.to_le_bytes());

    bytes.resize(code_offset, 0);
    bytes.extend_from_slice(&code);
    Ok(bytes)
}

fn emit_pe(program: &LoweredProgram) -> Result<Vec<u8>, CodegenError> {
    let code_rva = 0x1000u32;
    let headers_size = 0x200u32;
    let file_alignment = 0x200u32;
    let section_alignment = 0x1000u32;

    let code = encode(program, EncodeContext {})?;
    let entry_rva = code_rva + program.entry_offset() as u32;

    let text_raw_size = align_up(code.len() as u32, file_alignment);
    let text_virtual_size = code.len() as u32;
    let text_raw_ptr = headers_size;
    let size_of_image = align_up(code_rva + text_virtual_size, section_alignment);

    let mut bytes = vec![0; headers_size as usize];

    bytes[0] = b'M';
    bytes[1] = b'Z';
    write_u32(&mut bytes, 0x3c, 0x80);

    let mut cursor = 0x80usize;
    bytes[cursor..cursor + 4].copy_from_slice(b"PE\0\0");
    cursor += 4;

    bytes[cursor..cursor + 2].copy_from_slice(&0x8664u16.to_le_bytes());
    bytes[cursor + 2..cursor + 4].copy_from_slice(&1u16.to_le_bytes());
    bytes[cursor + 16..cursor + 18].copy_from_slice(&0xF0u16.to_le_bytes());
    bytes[cursor + 18..cursor + 20].copy_from_slice(&0x0023u16.to_le_bytes());
    cursor += 20;

    bytes[cursor..cursor + 2].copy_from_slice(&0x20Bu16.to_le_bytes());
    bytes[cursor + 2] = 1;
    bytes[cursor + 4..cursor + 8].copy_from_slice(&text_raw_size.to_le_bytes());
    bytes[cursor + 8..cursor + 12].copy_from_slice(&0u32.to_le_bytes());
    bytes[cursor + 16..cursor + 20].copy_from_slice(&entry_rva.to_le_bytes());
    bytes[cursor + 20..cursor + 24].copy_from_slice(&code_rva.to_le_bytes());
    bytes[cursor + 24..cursor + 32].copy_from_slice(&0x0000_0001_4000_0000u64.to_le_bytes());
    bytes[cursor + 32..cursor + 36].copy_from_slice(&section_alignment.to_le_bytes());
    bytes[cursor + 36..cursor + 40].copy_from_slice(&file_alignment.to_le_bytes());
    bytes[cursor + 40..cursor + 42].copy_from_slice(&6u16.to_le_bytes());
    bytes[cursor + 48..cursor + 50].copy_from_slice(&6u16.to_le_bytes());
    bytes[cursor + 56..cursor + 60].copy_from_slice(&size_of_image.to_le_bytes());
    bytes[cursor + 60..cursor + 64].copy_from_slice(&headers_size.to_le_bytes());
    bytes[cursor + 68..cursor + 70].copy_from_slice(&3u16.to_le_bytes());
    bytes[cursor + 72..cursor + 80].copy_from_slice(&0x0010_0000u64.to_le_bytes());
    bytes[cursor + 80..cursor + 88].copy_from_slice(&0x1000u64.to_le_bytes());
    bytes[cursor + 88..cursor + 96].copy_from_slice(&0x0010_0000u64.to_le_bytes());
    bytes[cursor + 96..cursor + 104].copy_from_slice(&0x1000u64.to_le_bytes());
    bytes[cursor + 104..cursor + 108].copy_from_slice(&0u32.to_le_bytes());
    bytes[cursor + 108..cursor + 112].copy_from_slice(&16u32.to_le_bytes());

    cursor += 0xF0;

    write_section_header(
        &mut bytes,
        cursor,
        b".text\0\0\0",
        text_virtual_size,
        code_rva,
        text_raw_size,
        text_raw_ptr,
        0x6000_0020,
    );

    bytes.resize(text_raw_ptr as usize, 0);
    bytes.extend_from_slice(&code);
    bytes.resize((text_raw_ptr + text_raw_size) as usize, 0);
    Ok(bytes)
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_section_header(
    bytes: &mut [u8],
    offset: usize,
    name: &[u8; 8],
    virtual_size: u32,
    virtual_address: u32,
    raw_size: u32,
    raw_ptr: u32,
    characteristics: u32,
) {
    bytes[offset..offset + 8].copy_from_slice(name);
    bytes[offset + 8..offset + 12].copy_from_slice(&virtual_size.to_le_bytes());
    bytes[offset + 12..offset + 16].copy_from_slice(&virtual_address.to_le_bytes());
    bytes[offset + 16..offset + 20].copy_from_slice(&raw_size.to_le_bytes());
    bytes[offset + 20..offset + 24].copy_from_slice(&raw_ptr.to_le_bytes());
    bytes[offset + 36..offset + 40].copy_from_slice(&characteristics.to_le_bytes());
}

fn align_up(value: u32, alignment: u32) -> u32 {
    if value == 0 {
        0
    } else {
        ((value + alignment - 1) / alignment) * alignment
    }
}

fn max_outgoing_call_area(function: &MirFunction, target: Target) -> usize {
    function
        .blocks
        .iter()
        .filter_map(|block| match &block.terminator {
            TerminatorKind::Call { args, .. } => {
                let stack_args = args.len().saturating_sub(argument_registers(target).len()) * 8;
                Some(stack_arg_base(target) + stack_args)
            }
            _ => None,
        })
        .max()
        .unwrap_or_else(|| match target.os {
            OperatingSystem::Linux => 0,
            OperatingSystem::Windows => 32,
        })
}

fn stack_arg_base(target: Target) -> usize {
    match target.os {
        OperatingSystem::Linux => 0,
        OperatingSystem::Windows => 32,
    }
}

fn incoming_stack_arg_base(target: Target) -> usize {
    match target.os {
        OperatingSystem::Linux => 8,
        OperatingSystem::Windows => 40,
    }
}

fn required_frame_alignment(function: &MirFunction, target: Target) -> usize {
    match target.os {
        OperatingSystem::Linux => 8,
        OperatingSystem::Windows => {
            if function.receiver.is_none() && function.name == "main" {
                0
            } else {
                8
            }
        }
    }
}

struct StackLayout {
    total_size: usize,
    outgoing_call_area: usize,
    offsets: Vec<i32>,
}

impl StackLayout {
    fn new(function: &MirFunction, target: Target) -> Result<Self, CodegenError> {
        for local in &function.locals {
            if !is_supported_scalar_type(&local.ty) {
                return Err(CodegenError::new(format!(
                    "native codegen does not yet support local `{}` of type `{}`",
                    local.name,
                    local.ty.display_name()
                )));
            }
        }

        let outgoing_call_area = max_outgoing_call_area(function, target);
        let frame_size = function.locals.len() * 8;
        let mut total_size = outgoing_call_area + frame_size;
        while total_size % 16 != required_frame_alignment(function, target) {
            total_size += 1;
        }

        let offsets = (0..function.locals.len())
            .map(|index| (outgoing_call_area + index * 8) as i32)
            .collect();

        Ok(Self {
            total_size,
            outgoing_call_area,
            offsets,
        })
    }

    fn offset_for(&self, local: usize) -> Result<i32, CodegenError> {
        self.offsets.get(local).copied().ok_or_else(|| {
            CodegenError::new(format!("stack slot for local `{local}` does not exist"))
        })
    }

    fn outgoing_arg_offset(&self, target: Target, stack_index: usize) -> Result<i32, CodegenError> {
        let offset = stack_arg_base(target)
            .checked_add(stack_index * 8)
            .ok_or_else(|| CodegenError::new("outgoing stack argument offset overflowed"))?;
        if offset >= self.outgoing_call_area {
            return Err(CodegenError::new(format!(
                "outgoing stack argument slot `{stack_index}` does not exist"
            )));
        }
        Ok(offset as i32)
    }

    fn incoming_arg_offset(&self, target: Target, stack_index: usize) -> Result<i32, CodegenError> {
        let base = self
            .total_size
            .checked_add(incoming_stack_arg_base(target))
            .and_then(|value| value.checked_add(stack_index * 8))
            .ok_or_else(|| CodegenError::new("incoming stack argument offset overflowed"))?;
        i32::try_from(base)
            .map_err(|_| CodegenError::new("incoming stack argument offset exceeds x86-64 frame limits"))
    }
}

#[derive(Debug, Clone)]
struct LoweredProgram {
    entry_label: String,
    instructions: Vec<Instruction>,
}

impl LoweredProgram {
    fn entry_offset(&self) -> usize {
        instruction_offsets(&self.instructions)
            .labels
            .get(&self.entry_label)
            .copied()
            .unwrap_or(0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Register {
    Rax,
    Rcx,
    Rdx,
    Rdi,
    Rsi,
    R8,
    R9,
}

impl Register {
    fn encoding(self) -> u8 {
        match self {
            Self::Rax => 0,
            Self::Rcx => 1,
            Self::Rdx => 2,
            Self::Rdi => 7,
            Self::Rsi => 6,
            Self::R8 => 8,
            Self::R9 => 9,
        }
    }

    fn low3(self) -> u8 {
        self.encoding() & 0b111
    }

    fn rex_r(self) -> u8 {
        u8::from((self.encoding() & 0b1000) != 0) << 2
    }

    fn rex_b(self) -> u8 {
        u8::from((self.encoding() & 0b1000) != 0)
    }

    fn name(self) -> &'static str {
        match self {
            Self::Rax => "rax",
            Self::Rcx => "rcx",
            Self::Rdx => "rdx",
            Self::Rdi => "rdi",
            Self::Rsi => "rsi",
            Self::R8 => "r8",
            Self::R9 => "r9",
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Condition {
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

impl Condition {
    fn setcc_opcode(self) -> u8 {
        match self {
            Self::Equal => 0x94,
            Self::NotEqual => 0x95,
            Self::Less => 0x9C,
            Self::LessEqual => 0x9E,
            Self::Greater => 0x9F,
            Self::GreaterEqual => 0x9D,
        }
    }

    fn jcc_opcode(self) -> u8 {
        match self {
            Self::Equal => 0x84,
            Self::NotEqual => 0x85,
            Self::Less => 0x8C,
            Self::LessEqual => 0x8E,
            Self::Greater => 0x8F,
            Self::GreaterEqual => 0x8D,
        }
    }

    fn mnemonic(self) -> &'static str {
        match self {
            Self::Equal => "je",
            Self::NotEqual => "jne",
            Self::Less => "jl",
            Self::LessEqual => "jle",
            Self::Greater => "jg",
            Self::GreaterEqual => "jge",
        }
    }

    fn set_mnemonic(self) -> &'static str {
        match self {
            Self::Equal => "sete",
            Self::NotEqual => "setne",
            Self::Less => "setl",
            Self::LessEqual => "setle",
            Self::Greater => "setg",
            Self::GreaterEqual => "setge",
        }
    }
}

#[derive(Debug, Clone)]
enum Instruction {
    Label(String),
    SubRsp(u32),
    AddRsp(u32),
    MovRegImm64(Register, i64),
    MovRegStack(Register, i32),
    MovStackReg(i32, Register),
    MovRegReg(Register, Register),
    AddRegReg(Register, Register),
    SubRegReg(Register, Register),
    AndRegReg(Register, Register),
    OrRegReg(Register, Register),
    IMulRegReg(Register, Register),
    Cqo,
    IDivReg(Register),
    NegReg(Register),
    CmpRegImm(Register, i32),
    CmpRegReg(Register, Register),
    SetCondAl(Condition),
    MovzxEaxAl,
    Call(String),
    Jump(String),
    JumpIf(Condition, String),
    Syscall,
    Ret,
    Ud2,
}

impl Instruction {
    fn len(&self) -> usize {
        match self {
            Self::Label(_) => 0,
            Self::SubRsp(value) => {
                if i8::try_from(*value).is_ok() {
                    4
                } else {
                    7
                }
            }
            Self::AddRsp(value) => {
                if i8::try_from(*value).is_ok() {
                    4
                } else {
                    7
                }
            }
            Self::MovRegImm64(_, _) => 10,
            Self::MovRegStack(_, offset) | Self::MovStackReg(offset, _) => stack_mem_len(*offset),
            Self::MovRegReg(_, _) => 3,
            Self::AddRegReg(_, _)
            | Self::SubRegReg(_, _)
            | Self::AndRegReg(_, _)
            | Self::OrRegReg(_, _)
            | Self::CmpRegReg(_, _) => 3,
            Self::IMulRegReg(_, _) => 4,
            Self::Cqo => 2,
            Self::IDivReg(_) | Self::NegReg(_) => 3,
            Self::CmpRegImm(_, value) => {
                if i8::try_from(*value).is_ok() {
                    4
                } else {
                    7
                }
            }
            Self::SetCondAl(_) => 3,
            Self::MovzxEaxAl => 3,
            Self::Call(_) => 5,
            Self::Jump(_) => 5,
            Self::JumpIf(_, _) => 6,
            Self::Syscall | Self::Ud2 => 2,
            Self::Ret => 1,
        }
    }

    fn render(&self) -> String {
        match self {
            Self::Label(name) => format!("{name}:"),
            Self::SubRsp(value) => format!("sub rsp, {value}"),
            Self::AddRsp(value) => format!("add rsp, {value}"),
            Self::MovRegImm64(reg, value) => format!("mov {}, {}", reg.name(), value),
            Self::MovRegStack(reg, offset) => {
                format!("mov {}, qword ptr [rsp + {offset}]", reg.name())
            }
            Self::MovStackReg(offset, reg) => {
                format!("mov qword ptr [rsp + {offset}], {}", reg.name())
            }
            Self::MovRegReg(dst, src) => format!("mov {}, {}", dst.name(), src.name()),
            Self::AddRegReg(dst, src) => format!("add {}, {}", dst.name(), src.name()),
            Self::SubRegReg(dst, src) => format!("sub {}, {}", dst.name(), src.name()),
            Self::AndRegReg(dst, src) => format!("and {}, {}", dst.name(), src.name()),
            Self::OrRegReg(dst, src) => format!("or {}, {}", dst.name(), src.name()),
            Self::IMulRegReg(dst, src) => format!("imul {}, {}", dst.name(), src.name()),
            Self::Cqo => "cqo".to_string(),
            Self::IDivReg(reg) => format!("idiv {}", reg.name()),
            Self::NegReg(reg) => format!("neg {}", reg.name()),
            Self::CmpRegImm(reg, value) => format!("cmp {}, {}", reg.name(), value),
            Self::CmpRegReg(left, right) => format!("cmp {}, {}", left.name(), right.name()),
            Self::SetCondAl(condition) => format!("{} al", condition.set_mnemonic()),
            Self::MovzxEaxAl => "movzx eax, al".to_string(),
            Self::Call(label) => format!("call {label}"),
            Self::Jump(label) => format!("jmp {label}"),
            Self::JumpIf(condition, label) => format!("{} {label}", condition.mnemonic()),
            Self::Syscall => "syscall".to_string(),
            Self::Ret => "ret".to_string(),
            Self::Ud2 => "ud2".to_string(),
        }
    }
}

struct EncodeContext {}

struct InstructionOffsets {
    labels: HashMap<String, usize>,
}

fn instruction_offsets(instructions: &[Instruction]) -> InstructionOffsets {
    let mut labels = HashMap::new();
    let mut cursor = 0usize;
    for instruction in instructions {
        match instruction {
            Instruction::Label(name) => {
                labels.insert(name.clone(), cursor);
            }
            _ => cursor += instruction.len(),
        }
    }
    InstructionOffsets { labels }
}

fn encode(program: &LoweredProgram, context: EncodeContext) -> Result<Vec<u8>, CodegenError> {
    let offsets = instruction_offsets(&program.instructions);
    let mut output = Vec::new();
    let mut cursor = 0usize;
    for instruction in &program.instructions {
        encode_instruction(instruction, &mut output, &offsets, &context, cursor)?;
        cursor += instruction.len();
    }
    Ok(output)
}

fn encode_instruction(
    instruction: &Instruction,
    output: &mut Vec<u8>,
    offsets: &InstructionOffsets,
    _context: &EncodeContext,
    cursor: usize,
) -> Result<(), CodegenError> {
    match instruction {
        Instruction::Label(_) => {}
        Instruction::SubRsp(value) => encode_sub_rsp(output, *value),
        Instruction::AddRsp(value) => encode_add_rsp(output, *value),
        Instruction::MovRegImm64(reg, value) => encode_mov_reg_imm64(output, *reg, *value),
        Instruction::MovRegStack(reg, offset) => encode_mov_reg_stack(output, *reg, *offset),
        Instruction::MovStackReg(offset, reg) => encode_mov_stack_reg(output, *offset, *reg),
        Instruction::MovRegReg(dst, src) => encode_mov_reg_reg(output, *dst, *src),
        Instruction::AddRegReg(dst, src) => encode_reg_reg(output, 0x01, *dst, *src),
        Instruction::SubRegReg(dst, src) => encode_reg_reg(output, 0x29, *dst, *src),
        Instruction::AndRegReg(dst, src) => encode_reg_reg(output, 0x21, *dst, *src),
        Instruction::OrRegReg(dst, src) => encode_reg_reg(output, 0x09, *dst, *src),
        Instruction::IMulRegReg(dst, src) => {
            output.extend_from_slice(&[
                0x48 | dst.rex_r() | src.rex_b(),
                0x0F,
                0xAF,
                modrm(0b11, dst.low3(), src.low3()),
            ]);
        }
        Instruction::Cqo => output.extend_from_slice(&[0x48, 0x99]),
        Instruction::IDivReg(reg) => {
            output.extend_from_slice(&[0x48 | reg.rex_b(), 0xF7, modrm(0b11, 7, reg.low3())]);
        }
        Instruction::NegReg(reg) => {
            output.extend_from_slice(&[0x48 | reg.rex_b(), 0xF7, modrm(0b11, 3, reg.low3())]);
        }
        Instruction::CmpRegImm(reg, value) => encode_cmp_reg_imm(output, *reg, *value),
        Instruction::CmpRegReg(left, right) => encode_reg_reg(output, 0x39, *left, *right),
        Instruction::SetCondAl(condition) => {
            output.extend_from_slice(&[0x0F, condition.setcc_opcode(), 0xC0]);
        }
        Instruction::MovzxEaxAl => output.extend_from_slice(&[0x0F, 0xB6, 0xC0]),
        Instruction::Call(label) => {
            output.push(0xE8);
            let target = offsets
                .labels
                .get(label)
                .copied()
                .ok_or_else(|| CodegenError::new(format!("unknown label `{label}`")))?;
            let rel = relative_displacement(cursor, instruction.len(), target)?;
            output.extend_from_slice(&rel.to_le_bytes());
        }
        Instruction::Jump(label) => {
            output.push(0xE9);
            let target = offsets
                .labels
                .get(label)
                .copied()
                .ok_or_else(|| CodegenError::new(format!("unknown label `{label}`")))?;
            let rel = relative_displacement(cursor, instruction.len(), target)?;
            output.extend_from_slice(&rel.to_le_bytes());
        }
        Instruction::JumpIf(condition, label) => {
            output.extend_from_slice(&[0x0F, condition.jcc_opcode()]);
            let target = offsets
                .labels
                .get(label)
                .copied()
                .ok_or_else(|| CodegenError::new(format!("unknown label `{label}`")))?;
            let rel = relative_displacement(cursor, instruction.len(), target)?;
            output.extend_from_slice(&rel.to_le_bytes());
        }
        Instruction::Syscall => output.extend_from_slice(&[0x0F, 0x05]),
        Instruction::Ret => output.push(0xC3),
        Instruction::Ud2 => output.extend_from_slice(&[0x0F, 0x0B]),
    }
    Ok(())
}

fn relative_displacement(from: usize, len: usize, target: usize) -> Result<i32, CodegenError> {
    (target as i64 - (from + len) as i64)
        .try_into()
        .map_err(|_| CodegenError::new("jump displacement does not fit in rel32"))
}

fn encode_sub_rsp(output: &mut Vec<u8>, value: u32) {
    if let Ok(value) = i8::try_from(value) {
        output.extend_from_slice(&[0x48, 0x83, 0xEC, value as u8]);
    } else {
        output.extend_from_slice(&[0x48, 0x81, 0xEC]);
        output.extend_from_slice(&value.to_le_bytes());
    }
}

fn encode_add_rsp(output: &mut Vec<u8>, value: u32) {
    if let Ok(value) = i8::try_from(value) {
        output.extend_from_slice(&[0x48, 0x83, 0xC4, value as u8]);
    } else {
        output.extend_from_slice(&[0x48, 0x81, 0xC4]);
        output.extend_from_slice(&value.to_le_bytes());
    }
}

fn encode_mov_reg_imm64(output: &mut Vec<u8>, reg: Register, value: i64) {
    output.push(0x48 | reg.rex_b());
    output.push(0xB8 + reg.low3());
    output.extend_from_slice(&value.to_le_bytes());
}

fn encode_mov_reg_stack(output: &mut Vec<u8>, reg: Register, offset: i32) {
    output.extend_from_slice(&[0x48 | reg.rex_r(), 0x8B]);
    encode_rsp_memory_operand(output, reg.low3(), offset);
}

fn encode_mov_stack_reg(output: &mut Vec<u8>, offset: i32, reg: Register) {
    output.extend_from_slice(&[0x48 | reg.rex_r(), 0x89]);
    encode_rsp_memory_operand(output, reg.low3(), offset);
}

fn encode_mov_reg_reg(output: &mut Vec<u8>, dst: Register, src: Register) {
    output.extend_from_slice(&[
        0x48 | src.rex_r() | dst.rex_b(),
        0x89,
        modrm(0b11, src.low3(), dst.low3()),
    ]);
}

fn encode_reg_reg(output: &mut Vec<u8>, opcode: u8, dst: Register, src: Register) {
    output.extend_from_slice(&[
        0x48 | src.rex_r() | dst.rex_b(),
        opcode,
        modrm(0b11, src.low3(), dst.low3()),
    ]);
}

fn encode_cmp_reg_imm(output: &mut Vec<u8>, reg: Register, value: i32) {
    if let Ok(value8) = i8::try_from(value) {
        output.extend_from_slice(&[
            0x48 | reg.rex_b(),
            0x83,
            modrm(0b11, 7, reg.low3()),
            value8 as u8,
        ]);
    } else {
        output.extend_from_slice(&[0x48 | reg.rex_b(), 0x81, modrm(0b11, 7, reg.low3())]);
        output.extend_from_slice(&value.to_le_bytes());
    }
}

fn encode_rsp_memory_operand(output: &mut Vec<u8>, reg: u8, offset: i32) {
    if let Ok(offset8) = i8::try_from(offset) {
        output.push(modrm(0b01, reg, 0b100));
        output.push(0x24);
        output.push(offset8 as u8);
    } else {
        output.push(modrm(0b10, reg, 0b100));
        output.push(0x24);
        output.extend_from_slice(&offset.to_le_bytes());
    }
}

fn stack_mem_len(offset: i32) -> usize {
    if i8::try_from(offset).is_ok() {
        5
    } else {
        8
    }
}

fn modrm(mode: u8, reg: u8, rm: u8) -> u8 {
    (mode << 6) | ((reg & 0b111) << 3) | (rm & 0b111)
}
