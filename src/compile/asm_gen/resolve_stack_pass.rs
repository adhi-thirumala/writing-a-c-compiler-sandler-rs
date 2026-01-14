use std::mem::take;
use std::sync::atomic::Ordering;

use super::replace_pseudoregisters_pass::STACK_COUNTER;

use super::{BinaryOperator, FunctionDefinition, Instruction, Operand, Program, Register};

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
            old_instructions
                .into_iter()
                .for_each(|instruction| parse_instruction(instruction, instructions));
        }
    }
}

fn parse_instruction(instruction: Instruction, new_instructions: &mut Vec<Instruction>) {
    match instruction {
        Instruction::Mov {
            src: src @ Operand::Stack(_),
            dst: dst @ Operand::Stack(_),
        } => {
            new_instructions.push(Instruction::Mov {
                src: src,
                dst: Operand::Register(Register::R10),
            });
            new_instructions.push(Instruction::Mov {
                src: Operand::Register(Register::R10),
                dst: dst,
            });
        }
        Instruction::Binary {
            left_operand: left_operand @ Operand::Stack(_),
            right_operand: right_operand @ Operand::Stack(_),
            binary_operator:
                binary_operator @ (BinaryOperator::Add
                | BinaryOperator::Sub
                | BinaryOperator::BitwiseOr
                | BinaryOperator::BitwiseAnd
                | BinaryOperator::BitwiseXor),
        } => {
            new_instructions.push(Instruction::Mov {
                src: left_operand,
                dst: Operand::Register(Register::R10),
            });
            new_instructions.push(Instruction::Binary {
                binary_operator: binary_operator,
                left_operand: Operand::Register(Register::R10),
                right_operand: right_operand,
            });
        }
        Instruction::Binary {
            left_operand,
            right_operand: right_operand @ Operand::Stack(_),
            binary_operator: binary_operator @ BinaryOperator::Mult,
        } => {
            new_instructions.push(Instruction::Mov {
                src: right_operand.clone(),
                dst: Operand::Register(Register::R11),
            });
            new_instructions.push(Instruction::Binary {
                binary_operator: binary_operator,
                left_operand: left_operand,
                right_operand: Operand::Register(Register::R11),
            });
            new_instructions.push(Instruction::Mov {
                src: Operand::Register(Register::R11),
                dst: right_operand,
            });
        }
        Instruction::Idiv(operand @ Operand::Imm(_)) => {
            new_instructions.push(Instruction::Mov {
                src: operand,
                dst: Operand::Register(Register::R10),
            });
            new_instructions.push(Instruction::Idiv(Operand::Register(Register::R10)));
        }
        instr => new_instructions.push(instr),
    }
}
