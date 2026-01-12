use std::mem::take;
use std::sync::atomic::Ordering;

use super::replace_pseudoregisters_pass::STACK_COUNTER;

use super::{FunctionDefinition, Instruction, Operand, Program, Register};

pub(super) fn resolve_stack(ast: &mut Program) {
    parse_program(ast)
}

fn parse_program(program: &mut Program) {
    match program {
        Program::Program(function_definition) => parse_function(function_definition),
    }
}

fn parse_function(function: &mut FunctionDefinition) {
    match function {
        FunctionDefinition::Function { instructions, .. } => {
            let old_instructions = take(instructions);
            instructions.push(Instruction::AllocateStack(
                STACK_COUNTER.load(Ordering::Relaxed) * -1,
            ));
            instructions.extend(old_instructions.into_iter().flat_map(|instruction| {
                let (first, second) = parse_instruction(instruction);
                std::iter::once(first).chain(second)
            }));
        }
    }
}

fn parse_instruction(instruction: Instruction) -> (Instruction, Option<Instruction>) {
    if let Instruction::Mov {
        src: src @ Operand::Stack(_),
        dst: dst @ Operand::Stack(_),
    } = instruction
    {
        (
            Instruction::Mov {
                src: src,
                dst: Operand::Register(Register::R10),
            },
            Some(Instruction::Mov {
                src: Operand::Register(Register::R10),
                dst: dst,
            }),
        )
    } else {
        (instruction, None)
    }
}
