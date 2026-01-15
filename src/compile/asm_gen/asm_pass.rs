use super::tacky;
use super::{
    BinaryOperator, CondCode, FunctionDefinition, Instruction, Operand, Program, Register,
    UnaryOperator,
};

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
            unary_operator: tacky::UnaryOperator::Not,
            src,
            dst,
        } => {
            let dst = parse_operand(dst);
            instructions.push(Instruction::Cmp {
                left_operand: Operand::Imm(0),
                right_operand: parse_operand(src),
            });
            instructions.push(Instruction::Mov {
                src: Operand::Imm(0),
                dst: dst.clone(),
            });
            instructions.push(Instruction::SetCC {
                cond_code: CondCode::E,
                operand: dst,
            });
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
                tacky::BinaryOperator::Add
                | tacky::BinaryOperator::Subtract
                | tacky::BinaryOperator::Multiply
                | tacky::BinaryOperator::BitwiseAnd
                | tacky::BinaryOperator::BitwiseOr
                | tacky::BinaryOperator::BitwiseXor => {
                    instructions.push(Instruction::Mov {
                        src: parse_operand(src1),
                        dst: dst.clone(),
                    });
                    instructions.push(Instruction::Binary {
                        binary_operator: parse_binary(binary_operator),
                        left_operand: parse_operand(src2),
                        right_operand: dst,
                    });
                }
                tacky::BinaryOperator::Divide | tacky::BinaryOperator::Remainder => {
                    instructions.push(Instruction::Mov {
                        src: parse_operand(src1),
                        dst: Operand::Register(Register::AX),
                    });
                    instructions.push(Instruction::Cdq);
                    instructions.push(Instruction::Idiv(parse_operand(src2)));
                    instructions.push(Instruction::Mov {
                        src: match binary_operator {
                            tacky::BinaryOperator::Divide => Operand::Register(Register::AX),
                            tacky::BinaryOperator::Remainder => Operand::Register(Register::DX),
                            _ => unreachable!("already checked to be divide or remainder"),
                        },
                        dst: dst,
                    });
                }

                tacky::BinaryOperator::LeftShift | tacky::BinaryOperator::RightShift => {
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
                        binary_operator: parse_binary(binary_operator),
                        right_operand: dst,
                        left_operand: Operand::Register(Register::CX),
                    });
                }

                tacky::BinaryOperator::Equal
                | tacky::BinaryOperator::NotEqual
                | tacky::BinaryOperator::LessThan
                | tacky::BinaryOperator::Leq
                | tacky::BinaryOperator::GreaterThan
                | tacky::BinaryOperator::Geq => {
                    instructions.push(Instruction::Cmp {
                        left_operand: parse_operand(src2),
                        right_operand: parse_operand(src1),
                    });
                    instructions.push(Instruction::Mov {
                        src: Operand::Imm(0),
                        dst: dst.clone(),
                    });
                    instructions.push(Instruction::SetCC {
                        cond_code: parse_relational_to_cc(binary_operator),
                        operand: dst,
                    });
                }
            }
        }
        tacky::Instruction::Copy { src, dst } => instructions.push(Instruction::Mov {
            src: parse_operand(src),
            dst: parse_operand(dst),
        }),
        tacky::Instruction::Jump(target) => instructions.push(Instruction::Jmp(target)),
        tacky::Instruction::JumpIfZero { target, condition } => {
            instructions.push(Instruction::Cmp {
                left_operand: Operand::Imm(0),
                right_operand: parse_operand(condition),
            });
            instructions.push(Instruction::JmpCC {
                cond_code: CondCode::E,
                identifier: target,
            });
        }
        tacky::Instruction::JumpIfNotZero { target, condition } => {
            instructions.push(Instruction::Cmp {
                left_operand: Operand::Imm(0),
                right_operand: parse_operand(condition),
            });
            instructions.push(Instruction::JmpCC {
                cond_code: CondCode::NE,
                identifier: target,
            });
        }
        tacky::Instruction::Label(s) => instructions.push(Instruction::Label(s)),
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
        tacky::UnaryOperator::Not => unreachable!("pattern matching resolves this before"),
    }
}

fn parse_binary(binary_operator: tacky::BinaryOperator) -> BinaryOperator {
    match binary_operator {
        tacky::BinaryOperator::Add => BinaryOperator::Add,
        tacky::BinaryOperator::Subtract => BinaryOperator::Sub,
        tacky::BinaryOperator::Multiply => BinaryOperator::Mult,
        tacky::BinaryOperator::BitwiseAnd => BinaryOperator::BitwiseAnd,
        tacky::BinaryOperator::BitwiseOr => BinaryOperator::BitwiseOr,
        tacky::BinaryOperator::BitwiseXor => BinaryOperator::BitwiseXor,
        tacky::BinaryOperator::LeftShift => BinaryOperator::LeftShift,
        tacky::BinaryOperator::RightShift => BinaryOperator::RightShift,
        tacky::BinaryOperator::Equal
        | tacky::BinaryOperator::NotEqual
        | tacky::BinaryOperator::LessThan
        | tacky::BinaryOperator::Leq
        | tacky::BinaryOperator::GreaterThan
        | tacky::BinaryOperator::Geq
        | tacky::BinaryOperator::Divide
        | tacky::BinaryOperator::Remainder => {
            unreachable!("dont need to come here")
        }
    }
}

fn parse_relational_to_cc(binary_operator: tacky::BinaryOperator) -> CondCode {
    match binary_operator {
        tacky::BinaryOperator::Equal => CondCode::E,
        tacky::BinaryOperator::NotEqual => CondCode::NE,
        tacky::BinaryOperator::LessThan => CondCode::L,
        tacky::BinaryOperator::Leq => CondCode::LE,
        tacky::BinaryOperator::GreaterThan => CondCode::G,
        tacky::BinaryOperator::Geq => CondCode::GE,
        tacky::BinaryOperator::Add
        | tacky::BinaryOperator::Subtract
        | tacky::BinaryOperator::Multiply
        | tacky::BinaryOperator::Divide
        | tacky::BinaryOperator::Remainder
        | tacky::BinaryOperator::BitwiseAnd
        | tacky::BinaryOperator::BitwiseOr
        | tacky::BinaryOperator::BitwiseXor
        | tacky::BinaryOperator::LeftShift
        | tacky::BinaryOperator::RightShift => unreachable!("checked above"),
    }
}
