use std::{
    collections::HashMap,
    sync::atomic::{AtomicI64, Ordering},
};

use super::{FunctionDefinition, Instruction, Operand, Program};

type IdMap = HashMap<String, i64>;

pub(super) static STACK_COUNTER: AtomicI64 = AtomicI64::new(-4);

pub(super) fn replace_psuedoregisters(ast: &mut Program) {
    let mut map: IdMap = Default::default();
    parse_program(ast, &mut map)
}

fn parse_program(ast: &mut Program, map: &mut IdMap) {
    match ast {
        Program::Program(function_definition) => parse_function(function_definition, map),
    }
}

fn parse_function(function: &mut FunctionDefinition, map: &mut IdMap) {
    match function {
        FunctionDefinition::Function {
            name: _,
            instructions,
        } => instructions
            .iter_mut()
            .for_each(|instruction| parse_instruction(instruction, map)),
    };
}

fn parse_instruction(instruction: &mut Instruction, map: &mut IdMap) {
    match instruction {
        Instruction::Mov { src, dst } => {
            parse_operand(src, map);
            parse_operand(dst, map);
        }
        Instruction::Unary {
            unary_operator: _,
            operand,
        } => parse_operand(operand, map),
        Instruction::Binary {
            binary_operator: _,
            left_operand,
            right_operand,
        } => {
            parse_operand(left_operand, map);
            parse_operand(right_operand, map);
        }
        Instruction::Idiv(operand) => {
            parse_operand(operand, map);
        }
        Instruction::Cmp {
            left_operand,
            right_operand,
        } => {
            parse_operand(left_operand, map);
            parse_operand(right_operand, map);
        }

        Instruction::SetCC { operand, .. } => parse_operand(operand, map),
        Instruction::Jmp(_)
        | Instruction::JmpCC { .. }
        | Instruction::Label(_)
        | Instruction::Cdq
        | Instruction::Ret
        | Instruction::AllocateStack(_) => (),
    }
}

fn parse_operand(operand: &mut Operand, map: &mut IdMap) {
    if let Operand::Psuedo(id) = operand {
        *operand = if map.contains_key(id) {
            Operand::Stack(map[id])
        } else {
            let offset = STACK_COUNTER.load(Ordering::Relaxed);
            let op = Operand::Stack(offset);
            STACK_COUNTER.fetch_sub(4, Ordering::Relaxed);
            map.insert(id.to_string(), offset);
            op
        }
    }
}
