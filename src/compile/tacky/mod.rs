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
}

#[derive(Debug, Clone)]
pub(super) enum Value {
    Constant(i64),
    Var(String),
}

#[derive(Debug)]
pub(super) enum UnaryOperator {
    Complement,
    Negate,
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
                    let (mut instructions, dst) = parse_expression_to_tacky(&name, expression);
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
) -> (Vec<Instruction>, Value) {
    match expression {
        parser::Expression::IntConstant(val) => (Vec::new(), Value::Constant(val)),
        parser::Expression::Unary {
            unary_operator,
            expression,
        } => {
            let (mut instructions, src) = parse_expression_to_tacky(&function_name, *expression);
            let dst = Value::Var(make_temp_name(function_name));
            instructions.push(Instruction::UnaryOperator {
                unary_operator: parse_unary_operator(unary_operator),
                src: src,
                dst: dst.clone(),
            });
            (instructions, dst)
        }
    }
}

fn parse_unary_operator(unary_operator: parser::UnaryOperator) -> UnaryOperator {
    match unary_operator {
        parser::UnaryOperator::Complement => UnaryOperator::Complement,
        parser::UnaryOperator::Negate => UnaryOperator::Negate,
    }
}

fn make_temp_name(function_name: &str) -> String {
    let temp_name = format!("{}-tmp.{:?}", function_name, COUNTER);
    COUNTER.fetch_add(1, Relaxed);
    temp_name
}
