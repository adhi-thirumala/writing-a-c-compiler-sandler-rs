mod asm_pass;
use crate::compile::tacky;
use crate::error::{Error, Result};

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
    let program = asm_pass::parse_program(ast);

    Ok(program)
}
