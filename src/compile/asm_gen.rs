use crate::compile::parser;
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
    Mov { src: Operand, dst: Operand },
    Ret,
}

#[derive(Debug)]
pub(super) enum Operand {
    Imm(i64),
    Register,
}

pub(super) fn asm_gen(ast: parser::Program) -> Result<Program> {
    parse_program(ast)
}

fn parse_program(root: parser::Program) -> Result<Program> {
    Ok(Program::Program(match root {
        parser::Program::Program(func) => parse_function(func)?,
    }))
}
fn parse_function(root: parser::FunctionDefinition) -> Result<FunctionDefinition> {
    match root {
        parser::FunctionDefinition::Function { name, body } => Ok(FunctionDefinition::Function {
            name: name,
            instructions: parse_instruction(body)?,
        }),
    }
}

fn parse_instruction(root: parser::Statement) -> Result<Vec<Instruction>> {
    match root {
        parser::Statement::Return(val) => Ok(vec![
            Instruction::Mov {
                src: parse_operand(val)?,
                dst: Operand::Register,
            },
            Instruction::Ret,
        ]),
    }
}

fn parse_operand(root: parser::Expression) -> Result<Operand> {
    match root {
        parser::Expression::IntConstant(num) => Ok(Operand::Imm(num)),
    }
}
