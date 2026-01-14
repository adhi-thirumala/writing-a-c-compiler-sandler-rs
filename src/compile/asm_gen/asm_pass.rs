use crate::compile::asm_gen::BinaryOperator;

use super::tacky;
use super::{FunctionDefinition, Instruction, Operand, Program, Register, UnaryOperator};

pub(super) fn parse_program(program: tacky::Program) -> Program {
    Program::Program(match program {
        tacky::Program::Program(func) => parse_function(func),
    })
}
fn parse_function(function: tacky::FunctionDefinition) -> FunctionDefinition {
    match function {
        tacky::FunctionDefinition::Function { identifier, body } => {
            let mut instructions = Vec::new();
            body.into_iter()
                .for_each(|instruction| parse_instruction(instruction, &mut instructions));
            FunctionDefinition::Function {
                name: identifier,
                instructions: instructions,
            }
        }
    }
}

fn parse_instruction(instruction: tacky::Instruction, instructions: &mut Vec<Instruction>) {
    match instruction {
        tacky::Instruction::Return(value) => {
            instructions.push(Instruction::Mov {
                src: parse_operand(value),
                dst: Operand::Register(Register::AX),
            });
            instructions.push(Instruction::Ret);
        }
        tacky::Instruction::UnaryOperator {
            unary_operator,
            src,
            dst,
        } => {
            let dst = parse_operand(dst);
            instructions.push(Instruction::Mov {
                src: parse_operand(src),
                dst: dst.clone(),
            });
            instructions.push(Instruction::Unary {
                unary_operator: parse_unary(unary_operator),
                operand: dst,
            });
        }
        tacky::Instruction::BinaryOperator {
            binary_operator,
            src1,
            src2,
            dst,
        } => {
            let dst = parse_operand(dst);
            match binary_operator {
                tacky::BinaryOperator::Add => {
                    instructions.push(Instruction::Mov {
                        src: parse_operand(src1),
                        dst: dst.clone(),
                    });
                    instructions.push(Instruction::Binary {
                        binary_operator: BinaryOperator::Add,
                        left_operand: parse_operand(src2),
                        right_operand: dst,
                    });
                }
                tacky::BinaryOperator::Subtract => {
                    instructions.push(Instruction::Mov {
                        src: parse_operand(src1),
                        dst: dst.clone(),
                    });
                    instructions.push(Instruction::Binary {
                        binary_operator: BinaryOperator::Sub,
                        left_operand: parse_operand(src2),
                        right_operand: dst,
                    });
                }
                tacky::BinaryOperator::Multiply => {
                    instructions.push(Instruction::Mov {
                        src: parse_operand(src1),
                        dst: dst.clone(),
                    });
                    instructions.push(Instruction::Binary {
                        binary_operator: BinaryOperator::Mult,
                        left_operand: parse_operand(src2),
                        right_operand: dst,
                    });
                }
                tacky::BinaryOperator::Divide => {
                    instructions.push(Instruction::Mov {
                        src: parse_operand(src1),
                        dst: Operand::Register(Register::AX),
                    });
                    instructions.push(Instruction::Cdq);
                    instructions.push(Instruction::Idiv(parse_operand(src2)));
                    instructions.push(Instruction::Mov {
                        src: Operand::Register(Register::AX),
                        dst: dst,
                    });
                }
                tacky::BinaryOperator::Remainder => {
                    instructions.push(Instruction::Mov {
                        src: parse_operand(src1),
                        dst: Operand::Register(Register::AX),
                    });
                    instructions.push(Instruction::Cdq);
                    instructions.push(Instruction::Idiv(parse_operand(src2)));
                    instructions.push(Instruction::Mov {
                        src: Operand::Register(Register::DX),
                        dst: dst,
                    });
                }
                tacky::BinaryOperator::BitwiseAnd => {
                    instructions.push(Instruction::Mov {
                        src: parse_operand(src1),
                        dst: dst.clone(),
                    });
                    instructions.push(Instruction::Binary {
                        binary_operator: BinaryOperator::BitwiseAnd,
                        left_operand: parse_operand(src2),
                        right_operand: dst,
                    });
                }
                tacky::BinaryOperator::BitwiseOr => {
                    instructions.push(Instruction::Mov {
                        src: parse_operand(src1),
                        dst: dst.clone(),
                    });
                    instructions.push(Instruction::Binary {
                        binary_operator: BinaryOperator::BitwiseOr,
                        left_operand: parse_operand(src2),
                        right_operand: dst,
                    });
                }

                tacky::BinaryOperator::BitwiseXor => {
                    instructions.push(Instruction::Mov {
                        src: parse_operand(src1),
                        dst: dst.clone(),
                    });
                    instructions.push(Instruction::Binary {
                        binary_operator: BinaryOperator::BitwiseXor,
                        left_operand: parse_operand(src2),
                        right_operand: dst,
                    });
                }
                tacky::BinaryOperator::LeftShift => {
                    //left is value being shifted
                    //right is shift count
                    //dst is where we want it to do
                    //move left into dst
                    //move right into cl
                    //left shift cl
                    instructions.push(Instruction::Mov {
                        src: parse_operand(src1),
                        dst: dst.clone(),
                    });
                    instructions.push(Instruction::Mov {
                        src: parse_operand(src2),
                        dst: Operand::Register(Register::CX),
                    });
                    instructions.push(Instruction::Binary {
                        binary_operator: BinaryOperator::LeftShift,
                        right_operand: dst,
                        left_operand: Operand::Register(Register::CL),
                    });
                }
                tacky::BinaryOperator::RightShift => {
                    instructions.push(Instruction::Mov {
                        src: parse_operand(src1),
                        dst: dst.clone(),
                    });
                    instructions.push(Instruction::Mov {
                        src: parse_operand(src2),
                        dst: Operand::Register(Register::CX),
                    });
                    instructions.push(Instruction::Binary {
                        binary_operator: BinaryOperator::RightShift,
                        right_operand: dst,
                        left_operand: Operand::Register(Register::CL),
                    });
                }
            }
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
