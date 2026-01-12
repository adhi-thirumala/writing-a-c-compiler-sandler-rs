mod asm_pass;
mod replace_pseudoregisters_pass;
mod resolve_stack_pass;

use super::Result;
use super::tacky;
use asm_pass::parse_program;
use replace_pseudoregisters_pass::replace_psuedoregisters;
use resolve_stack_pass::resolve_stack;

#[derive(Debug)]
pub(super) enum Program {
    Program(FunctionDefinition),
}

#[derive(Debug)]
pub(super) enum FunctionDefinition {
    Function {
        name: String,
        instructions: Vec<Instruction>,
    },
}

#[derive(Debug)]
pub(super) enum Instruction {
    Mov {
        src: Operand,
        dst: Operand,
    },
    Ret,
    AllocateStack(i64),
    Unary {
        unary_operator: UnaryOperator,
        operand: Operand,
    },
}

#[derive(Debug)]
pub(super) enum UnaryOperator {
    Neg,
    Not,
}

#[derive(Debug, Clone)]
pub(super) enum Operand {
    Imm(i64),
    Register(Register),
    Psuedo(String),
    Stack(i64),
}

#[derive(Debug, Clone)]
pub(super) enum Register {
    AX,
    R10,
}

pub(super) fn asm_gen(ast: tacky::Program) -> Result<Program> {
    let mut program = parse_program(ast);
    replace_psuedoregisters(&mut program);
    resolve_stack(&mut program);
    Ok(program)
}
