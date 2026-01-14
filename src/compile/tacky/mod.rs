use crate::compile::parser;
use crate::error::Result;
use std::sync::atomic::AtomicI64;
use std::sync::atomic::Ordering::Relaxed;

#[derive(Debug)]
pub(super) enum Program {
    Program(FunctionDefinition),
}

#[derive(Debug)]
pub(super) enum FunctionDefinition {
    Function {
        identifier: String,
        body: Vec<Instruction>,
    },
}

#[derive(Debug)]
pub(super) enum Instruction {
    Return(Value),
    UnaryOperator {
        unary_operator: UnaryOperator,
        src: Value,
        dst: Value,
    },
    BinaryOperator {
        binary_operator: BinaryOperator,
        src1: Value,
        src2: Value,
        dst: Value,
    },
}

#[derive(Debug, Clone)]
pub(super) enum Value {
    Constant(i32),
    Var(String),
}

#[derive(Debug)]
pub(super) enum UnaryOperator {
    Complement,
    Negate,
}

#[derive(Debug)]
pub(super) enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    LeftShift,
    RightShift,
}

static COUNTER: AtomicI64 = AtomicI64::new(0);

pub(super) fn tacky_gen(ast: parser::Program) -> Result<Program> {
    Ok(parse_program(ast))
}

fn parse_program(program: parser::Program) -> Program {
    match program {
        parser::Program::Program(function_definition) => {
            Program::Program(parse_function(function_definition))
        }
    }
}

fn parse_function(function: parser::FunctionDefinition) -> FunctionDefinition {
    match function {
        parser::FunctionDefinition::Function { name, body } => FunctionDefinition::Function {
            body: match body {
                parser::Statement::Return(expression) => {
                    let mut instructions = Vec::new();
                    let dst = parse_expression_to_tacky(&name, expression, &mut instructions);
                    instructions.push(Instruction::Return(dst));
                    instructions
                }
            },
            identifier: name,
        },
    }
}

fn parse_expression_to_tacky(
    function_name: &str,
    expression: parser::Expression,
    instructions: &mut Vec<Instruction>,
) -> Value {
    match expression {
        parser::Expression::IntConstant(val) => Value::Constant(val),
        parser::Expression::Unary {
            unary_operator,
            expression,
        } => {
            let src = parse_expression_to_tacky(&function_name, *expression, instructions);
            let dst = Value::Var(make_temp_name(function_name));
            instructions.push(Instruction::UnaryOperator {
                unary_operator: parse_unary_operator(unary_operator),
                src: src,
                dst: dst.clone(),
            });
            dst
        }
        parser::Expression::Binary {
            binary_operator,
            left_expression,
            right_expression,
        } => {
            let left_val =
                parse_expression_to_tacky(&function_name, *left_expression, instructions);
            let right_val =
                parse_expression_to_tacky(&function_name, *right_expression, instructions);
            let dst = Value::Var(make_temp_name(function_name));
            instructions.push(Instruction::BinaryOperator {
                binary_operator: parse_binary_operator(binary_operator),
                src1: left_val,
                src2: right_val,
                dst: dst.clone(),
            });
            dst
        }
    }
}

fn parse_unary_operator(unary_operator: parser::UnaryOperator) -> UnaryOperator {
    match unary_operator {
        parser::UnaryOperator::Complement => UnaryOperator::Complement,
        parser::UnaryOperator::Negate => UnaryOperator::Negate,
    }
}

fn parse_binary_operator(binary_operator: parser::BinaryOperator) -> BinaryOperator {
    match binary_operator {
        parser::BinaryOperator::Add => BinaryOperator::Add,
        parser::BinaryOperator::Subtract => BinaryOperator::Subtract,
        parser::BinaryOperator::Multiply => BinaryOperator::Multiply,
        parser::BinaryOperator::Divide => BinaryOperator::Divide,
        parser::BinaryOperator::Remainder => BinaryOperator::Remainder,
        parser::BinaryOperator::BitwiseAnd => BinaryOperator::BitwiseAnd,
        parser::BinaryOperator::BitwiseOr => BinaryOperator::BitwiseOr,
        parser::BinaryOperator::BitwiseXor => BinaryOperator::BitwiseXor,
        parser::BinaryOperator::LeftShift => BinaryOperator::LeftShift,
        parser::BinaryOperator::RightShift => BinaryOperator::RightShift,
    }
}

fn make_temp_name(function_name: &str) -> String {
    let temp_name = format!("{}-tmp.{:?}", function_name, COUNTER);
    COUNTER.fetch_add(1, Relaxed);
    temp_name
}
