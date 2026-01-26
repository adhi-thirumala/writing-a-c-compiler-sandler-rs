use super::parser;
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
    Copy {
        src: Value,
        dst: Value,
    },
    Jump(String),
    JumpIfZero {
        target: String,
        condition: Value,
    },
    JumpIfNotZero {
        target: String,
        condition: Value,
    },
    Label(String),
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
    Not,
    Increment,
    Decrement,
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
    Equal,
    NotEqual,
    LessThan,
    Leq,
    GreaterThan,
    Geq,
}

pub(super) static TACKY_COUNTER: AtomicI64 = AtomicI64::new(0);

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
    let parser::FunctionDefinition::Function { name, body } = function;
    let mut instructions = Vec::new();
    body.into_iter()
        .for_each(|block_item| parse_block_item(&name, block_item, &mut instructions));
    instructions.push(Instruction::Return(Value::Constant(0)));
    FunctionDefinition::Function {
        identifier: name,
        body: instructions,
    }
}

fn parse_block_item(
    function_name: &str,
    block_item: parser::BlockItem,
    instructions: &mut Vec<Instruction>,
) {
    match block_item {
        parser::BlockItem::S(statement) => match statement {
            parser::Statement::Return(expression) => {
                let ret = Instruction::Return(parse_expression_to_tacky(
                    function_name,
                    expression,
                    instructions,
                ));
                instructions.push(ret);
            }
            parser::Statement::Expression(expression) => {
                parse_expression_to_tacky(function_name, expression, instructions);
            }
            parser::Statement::Null => (),
            parser::Statement::If {
                condition,
                then_statement,
                else_statement,
            } => {
                let cond = parse_expression_to_tacky(function_name, condition, instructions);
                let else_label = make_temp_label(function_name);
                instructions.push(Instruction::JumpIfZero {
                    target: else_label.clone(),
                    condition: cond,
                });
                parse_block_item(
                    function_name,
                    parser::BlockItem::S(*then_statement),
                    instructions,
                );
                if let Some(statement) = else_statement {
                    let end_label = make_temp_label(function_name);
                    instructions.push(Instruction::Jump(end_label.clone()));
                    instructions.push(Instruction::Label(else_label));
                    parse_block_item(
                        function_name,
                        parser::BlockItem::S(*statement),
                        instructions,
                    );
                    instructions.push(Instruction::Label(end_label));
                } else {
                    instructions.push(Instruction::Label(else_label));
                }
            }
            parser::Statement::Goto(label) => instructions.push(Instruction::Jump(label)),
            parser::Statement::Label(label) => instructions.push(Instruction::Label(label)),
        },
        parser::BlockItem::D(declaration) => {
            parse_declaration(function_name, declaration, instructions)
        }
    }
}

fn parse_expression_to_tacky(
    function_name: &str,
    expression: parser::Expression,
    instructions: &mut Vec<Instruction>,
) -> Value {
    match expression {
        parser::Expression::IntConstant(val) => Value::Constant(val),
        parser::Expression::Var(val) => Value::Var(val),
        parser::Expression::Unary {
            unary_operator: parser::UnaryOperator::Increment,
            expression,
        } => parse_expression_to_tacky(
            function_name,
            parser::Expression::Assignment {
                left_expression: expression,
                right_expression: Box::new(parser::Expression::IntConstant(1)),
                operator: Some(parser::BinaryOperator::Add),
            },
            instructions,
        ),

        parser::Expression::Unary {
            unary_operator: parser::UnaryOperator::Decrement,
            expression,
        } => parse_expression_to_tacky(
            function_name,
            parser::Expression::Assignment {
                left_expression: expression,
                right_expression: Box::new(parser::Expression::IntConstant(1)),
                operator: Some(parser::BinaryOperator::Subtract),
            },
            instructions,
        ),
        parser::Expression::Unary {
            unary_operator,
            expression,
        } => {
            let src = parse_expression_to_tacky(&function_name, *expression, instructions);
            let dst = Value::Var(make_temp_identifier(function_name));
            instructions.push(Instruction::UnaryOperator {
                unary_operator: parse_unary_operator(unary_operator),
                src: src,
                dst: dst.clone(),
            });
            dst
        }
        parser::Expression::Binary {
            binary_operator: parser::BinaryOperator::And,
            left_expression,
            right_expression,
        } => {
            let left_val =
                parse_expression_to_tacky(&function_name, *left_expression, instructions);
            let temp_label = make_temp_label(&function_name);
            let dst = Value::Var(make_temp_identifier(&function_name));
            instructions.push(Instruction::JumpIfZero {
                target: temp_label.clone(),
                condition: left_val,
            });
            let right_val =
                parse_expression_to_tacky(&function_name, *right_expression, instructions);
            instructions.push(Instruction::JumpIfZero {
                target: temp_label.clone(),
                condition: right_val,
            });
            instructions.push(Instruction::Copy {
                src: Value::Constant(1),
                dst: dst.clone(),
            });
            let end = make_temp_label(function_name);
            instructions.push(Instruction::Jump(end.clone()));
            instructions.push(Instruction::Label(temp_label));
            instructions.push(Instruction::Copy {
                src: Value::Constant(0),
                dst: dst.clone(),
            });

            instructions.push(Instruction::Label(end));
            dst
        }
        parser::Expression::Binary {
            binary_operator: parser::BinaryOperator::Or,
            left_expression,
            right_expression,
        } => {
            let left_val =
                parse_expression_to_tacky(&function_name, *left_expression, instructions);
            let temp_label = make_temp_label(&function_name);
            let dst = Value::Var(make_temp_identifier(&function_name));
            instructions.push(Instruction::JumpIfNotZero {
                target: temp_label.clone(),
                condition: left_val,
            });
            let right_val =
                parse_expression_to_tacky(&function_name, *right_expression, instructions);
            instructions.push(Instruction::JumpIfNotZero {
                target: temp_label.clone(),
                condition: right_val,
            });
            instructions.push(Instruction::Copy {
                src: Value::Constant(0),
                dst: dst.clone(),
            });
            let end = make_temp_label(function_name);
            instructions.push(Instruction::Jump(end.clone()));
            instructions.push(Instruction::Label(temp_label));
            instructions.push(Instruction::Copy {
                src: Value::Constant(1),
                dst: dst.clone(),
            });

            instructions.push(Instruction::Label(end));
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
            let dst = Value::Var(make_temp_identifier(function_name));
            instructions.push(Instruction::BinaryOperator {
                binary_operator: parse_binary_operator(binary_operator),
                src1: left_val,
                src2: right_val,
                dst: dst.clone(),
            });
            dst
        }
        parser::Expression::Assignment {
            left_expression,
            right_expression,
            operator,
        } => {
            if let parser::Expression::Var(val) = *left_expression {
                let src = parse_expression_to_tacky(function_name, *right_expression, instructions);
                let dst = Value::Var(val);
                if let Some(operator) = operator {
                    instructions.push(Instruction::BinaryOperator {
                        binary_operator: parse_binary_operator(operator),
                        src1: dst.clone(),
                        src2: src,
                        dst: dst.clone(),
                    });
                } else {
                    instructions.push(Instruction::Copy {
                        src: src,
                        dst: dst.clone(),
                    });
                }
                dst
            } else {
                unreachable!("semantic analysis checked")
            }
        }
        parser::Expression::Postfix {
            postfix_operator,
            expression,
        } => {
            let binop = parse_postfix_operator(postfix_operator);
            let src = parse_expression_to_tacky(function_name, *expression, instructions);
            let dst = Value::Var(make_temp_identifier(function_name));
            instructions.push(Instruction::Copy {
                src: src.clone(),
                dst: dst.clone(),
            });
            instructions.push(Instruction::BinaryOperator {
                binary_operator: binop,
                src1: src.clone(),
                src2: Value::Constant(1),
                dst: src,
            });
            dst
        }
        parser::Expression::Conditional {
            condition,
            true_case,
            false_case,
        } => {
            let c = parse_expression_to_tacky(function_name, *condition, instructions);
            let end = make_temp_label(function_name);
            let e2_label = make_temp_label(function_name);
            instructions.push(Instruction::JumpIfZero {
                target: e2_label.clone(),
                condition: c,
            });
            let v1 = parse_expression_to_tacky(function_name, *true_case, instructions);
            let result = Value::Var(make_temp_identifier(function_name));
            instructions.push(Instruction::Copy {
                src: v1,
                dst: result.clone(),
            });
            instructions.push(Instruction::Jump(end.clone()));
            instructions.push(Instruction::Label(e2_label));
            let v2 = parse_expression_to_tacky(function_name, *false_case, instructions);
            instructions.push(Instruction::Copy {
                src: v2,
                dst: result.clone(),
            });
            instructions.push(Instruction::Label(end));
            result
        }
    }
}

fn parse_declaration(
    function_name: &str,
    declaration: parser::Declaration,
    instructions: &mut Vec<Instruction>,
) {
    let parser::Declaration::Declaration { init, name } = declaration;
    if let Some(expression) = init {
        let val = parse_expression_to_tacky(function_name, expression, instructions);
        instructions.push(Instruction::Copy {
            src: val,
            dst: Value::Var(name),
        });
    }
}

fn parse_postfix_operator(postfix_operator: parser::PostfixOperator) -> BinaryOperator {
    match postfix_operator {
        parser::PostfixOperator::Increment => BinaryOperator::Add,
        parser::PostfixOperator::Decrement => BinaryOperator::Subtract,
    }
}

fn parse_unary_operator(unary_operator: parser::UnaryOperator) -> UnaryOperator {
    match unary_operator {
        parser::UnaryOperator::Complement => UnaryOperator::Complement,
        parser::UnaryOperator::Negate => UnaryOperator::Negate,
        parser::UnaryOperator::Not => UnaryOperator::Not,
        parser::UnaryOperator::Increment => UnaryOperator::Increment,
        parser::UnaryOperator::Decrement => UnaryOperator::Decrement,
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
        parser::BinaryOperator::Equal => BinaryOperator::Equal,
        parser::BinaryOperator::NotEqual => BinaryOperator::NotEqual,
        parser::BinaryOperator::LessThan => BinaryOperator::LessThan,
        parser::BinaryOperator::Leq => BinaryOperator::Leq,
        parser::BinaryOperator::GreaterThan => BinaryOperator::GreaterThan,
        parser::BinaryOperator::Geq => BinaryOperator::Geq,
        parser::BinaryOperator::And
        | parser::BinaryOperator::Or
        | parser::BinaryOperator::Assigmnent
        | parser::BinaryOperator::CompoundAssignment(_)
        | parser::BinaryOperator::Ternary => {
            unreachable!("unconstructable")
        }
    }
}

fn make_temp_identifier(function_name: &str) -> String {
    let temp_name = format!("{}-tmp.{:?}", function_name, TACKY_COUNTER);
    TACKY_COUNTER.fetch_add(1, Relaxed);
    temp_name
}

fn make_temp_label(function_name: &str) -> String {
    let temp_name = format!("{}_tmp_label.{:?}", function_name, TACKY_COUNTER);
    TACKY_COUNTER.fetch_add(1, Relaxed);
    temp_name
}
