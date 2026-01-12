use super::tacky;
use super::{FunctionDefinition, Instruction, Operand, Program, Register, UnaryOperator};

pub(super) fn parse_program(program: tacky::Program) -> Program {
    Program::Program(match program {
        tacky::Program::Program(func) => parse_function(func),
    })
}
fn parse_function(function: tacky::FunctionDefinition) -> FunctionDefinition {
    match function {
        tacky::FunctionDefinition::Function { identifier, body } => FunctionDefinition::Function {
            name: identifier,
            instructions: body
                .into_iter()
                .flat_map(|instruction| parse_instruction(instruction))
                .collect(),
        },
    }
}

fn parse_instruction(instruction: tacky::Instruction) -> Vec<Instruction> {
    match instruction {
        tacky::Instruction::Return(value) => vec![
            Instruction::Mov {
                src: parse_operand(value),
                dst: Operand::Register(Register::AX),
            },
            Instruction::Ret,
        ],
        tacky::Instruction::UnaryOperator {
            unary_operator,
            src,
            dst,
        } => {
            let dst = parse_operand(dst);
            vec![
                Instruction::Mov {
                    src: parse_operand(src),
                    dst: dst.clone(),
                },
                Instruction::Unary {
                    unary_operator: parse_unary(unary_operator),
                    operand: dst,
                },
            ]
        }
    }
}

fn parse_operand(value: tacky::Value) -> Operand {
    match value {
        tacky::Value::Constant(num) => Operand::Imm(num),
        tacky::Value::Var(identifier) => Operand::Psuedo(identifier),
    }
}

fn parse_unary(unary_operator: tacky::UnaryOperator) -> UnaryOperator {
    match unary_operator {
        tacky::UnaryOperator::Complement => UnaryOperator::Not,
        tacky::UnaryOperator::Negate => UnaryOperator::Neg,
    }
}
