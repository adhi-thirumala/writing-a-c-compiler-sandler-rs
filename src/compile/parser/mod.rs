use itertools::PeekNth;
use itertools::peek_nth;

use crate::compile::lexer::Token;
use crate::error::{Error, Result};

type TokenStream = PeekNth<std::vec::IntoIter<Token>>;

#[derive(Debug)]
pub(super) enum Program {
    Program(FunctionDefinition),
}

#[derive(Debug)]
pub(super) enum FunctionDefinition {
    Function { name: String, body: Block },
}

#[derive(Debug)]
pub(super) enum BlockItem {
    S(Statement),
    D(Declaration),
}

#[derive(Debug)]
pub(super) enum Block {
    Block(Vec<BlockItem>),
}

#[derive(Debug)]
pub(super) enum Statement {
    Return(Expression),
    Expression(Expression),
    If {
        condition: Expression,
        then_statement: Box<Statement>,
        else_statement: Option<Box<Statement>>,
    },
    Compound(Block),
    Goto(String),
    Label(String),
    Null,
}

#[derive(Debug)]
pub(super) enum Declaration {
    Declaration {
        name: String,
        init: Option<Expression>,
    },
}

#[derive(Debug)]
pub(super) enum Expression {
    IntConstant(i32),
    Unary {
        unary_operator: UnaryOperator,
        expression: Box<Expression>,
    },
    Binary {
        binary_operator: BinaryOperator,
        left_expression: Box<Expression>,
        right_expression: Box<Expression>,
    },
    Var(String),
    Assignment {
        left_expression: Box<Expression>,
        right_expression: Box<Expression>,
        operator: Option<BinaryOperator>,
    },
    Postfix {
        postfix_operator: PostfixOperator,
        expression: Box<Expression>,
    },
    Conditional {
        condition: Box<Expression>,
        true_case: Box<Expression>,
        false_case: Box<Expression>,
    },
}

#[derive(Debug)]
pub(super) enum PostfixOperator {
    Increment,
    Decrement,
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
    And,
    Or,
    Equal,
    NotEqual,
    LessThan,
    Leq,
    GreaterThan,
    Geq,
    Assigmnent,
    CompoundAssignment(Box<BinaryOperator>),
    Ternary,
}

macro_rules! expect {
    ($iter:expr, $pat:pat => $val:expr) => {
        match $iter.next() {
            Some($pat) => Ok($val),
            Some(other) => Err(Error::ParserError {
                expected: stringify!($pat).to_string(),
                found: other.to_string(),
            }),
            None => Err(Error::ParserError {
                expected: stringify!($pat).to_string(),
                found: "end of file".to_string(),
            }),
        }
    };
}

pub(super) fn parser(toks: Vec<Token>) -> Result<Program> {
    let mut iter = peek_nth(toks.into_iter());
    let program = parse_program(&mut iter)?;
    match iter.next() {
        Some(tok) => Err(Error::ParserError {
            expected: "nothing, end of file".to_string(),
            found: tok.to_string(),
        }),
        None => Ok(program),
    }
}

// <program> ::= <function>
fn parse_program(iter: &mut TokenStream) -> Result<Program> {
    Ok(Program::Program(parse_function(iter)?))
}

fn parse_function(iter: &mut TokenStream) -> Result<FunctionDefinition> {
    expect!(iter, Token::Int => ())?;
    let name = expect!(iter, Token::Identifier(id) => id)?;
    expect!(iter, Token::OpenParenthesis => ())?;
    expect!(iter, Token::Void => ())?;
    expect!(iter, Token::ClosedParenthesis => ())?;
    let body = parse_block(iter)?;
    Ok(FunctionDefinition::Function { name, body })
}

fn parse_block(iter: &mut TokenStream) -> Result<Block> {
    expect!(iter, Token::OpenBrace => ())?;
    let mut block = Vec::new();
    while !matches!(iter.peek(), Some(Token::ClosedBrace)) {
        block.push(parse_block_item(iter)?);
    }
    Ok(Block::Block(block))
}

fn parse_block_item(iter: &mut TokenStream) -> Result<BlockItem> {
    if let Some(Token::Int) = iter.peek() {
        Ok(BlockItem::D(parse_declaration(iter)?))
    } else {
        Ok(BlockItem::S(parse_statement(iter)?))
    }
}

fn parse_declaration(iter: &mut TokenStream) -> Result<Declaration> {
    expect!(iter, Token::Int => ())?;
    let identifier = expect!(iter, Token::Identifier(id) => id)?;
    let init = if let Some(Token::Equal) = iter.peek() {
        iter.next();
        Some(parse_expression(iter, 0)?)
    } else {
        None
    };
    expect!(iter, Token::Semicolon => ())?;
    Ok(Declaration::Declaration {
        name: identifier,
        init: init,
    })
}

fn parse_statement(iter: &mut TokenStream) -> Result<Statement> {
    match iter.peek() {
        Some(Token::Return) => {
            iter.next();
            let expression = parse_expression(iter, 0)?;
            expect!(iter, Token::Semicolon => ())?;
            Ok(Statement::Return(expression))
        }
        Some(Token::Semicolon) => {
            iter.next();
            Ok(Statement::Null)
        }
        Some(Token::If) => {
            iter.next();
            expect!(iter, Token::OpenParenthesis => ())?;
            let condition = parse_expression(iter, 0)?;
            expect!(iter, Token::ClosedParenthesis => ())?;
            let then_statement = Box::new(parse_statement(iter)?);
            let else_statement = if let Some(Token::Else) = iter.peek() {
                iter.next();
                Some(Box::new(parse_statement(iter)?))
            } else {
                None
            };
            Ok(Statement::If {
                condition,
                then_statement,
                else_statement,
            })
        }
        Some(Token::Goto) => {
            iter.next();
            let label = expect!(iter, Token::Identifier(label) => label)?;
            expect!(iter, Token::Semicolon => ())?;
            Ok(Statement::Goto(label))
        }
        Some(Token::OpenBrace) => Ok(Statement::Compound(parse_block(iter)?)),
        Some(Token::Identifier(_)) => {
            //check if the next one is a colon, else just parse expression
            match iter.peek_nth(1) {
                Some(Token::Colon) => {
                    let Some(Token::Identifier(label)) = iter.next() else {
                        unreachable!("alr verified above")
                    };
                    iter.next();
                    Ok(Statement::Label(label))
                }
                Some(_) | None => {
                    let expression = parse_expression(iter, 0)?;
                    expect!(iter, Token::Semicolon => ())?;
                    Ok(Statement::Expression(expression))
                }
            }
        }
        Some(_) => {
            let expression = parse_expression(iter, 0)?;
            expect!(iter, Token::Semicolon => ())?;
            Ok(Statement::Expression(expression))
        }
        None => Err(Error::ParserError {
            expected: "beginning of stateent".to_string(),
            found: "end of file".to_string(),
        }),
    }
}

fn parse_expression(iter: &mut TokenStream, min_precedence: i64) -> Result<Expression> {
    let mut left = parse_factor(iter)?;
    while let Ok(binop) = parse_binary(iter)
        && let curr_precedence = binop.precedence()
        && curr_precedence >= min_precedence
    {
        let curr_precedence = binop.precedence();
        iter.next();
        if let BinaryOperator::Assigmnent = binop {
            left = Expression::Assignment {
                left_expression: Box::new(left),
                right_expression: Box::new(parse_expression(iter, curr_precedence)?),
                operator: None,
            };
        } else if let BinaryOperator::CompoundAssignment(operator) = binop {
            left = Expression::Assignment {
                left_expression: Box::new(left),
                right_expression: Box::new(parse_expression(iter, curr_precedence)?),
                operator: Some(*operator),
            };
        } else if let BinaryOperator::Ternary = binop {
            let middle = parse_expression(iter, 0)?;
            if let Some(Token::Colon) = iter.peek() {
                iter.next();
                let right = parse_expression(iter, curr_precedence)?;
                left = Expression::Conditional {
                    condition: Box::new(left),
                    true_case: Box::new(middle),
                    false_case: Box::new(right),
                };
            } else {
                return Err(Error::ParserError {
                    expected: "colon".to_string(),
                    found: iter
                        .peek()
                        .map_or("end of string".to_string(), |tok| tok.to_string()),
                });
            }
        } else {
            left = Expression::Binary {
                left_expression: Box::new(left),
                right_expression: Box::new(parse_expression(iter, curr_precedence + 1)?),
                binary_operator: binop,
            };
        }
    }
    Ok(left)
}

fn parse_factor(iter: &mut TokenStream) -> Result<Expression> {
    match iter.peek().ok_or_else(|| Error::ParserError {
        expected: "factor".to_string(),
        found: "end of string".to_string(),
    })? {
        Token::IntegerConstant(val) => {
            let val = *val;
            iter.next();
            Ok(Expression::IntConstant(val))
        }

        //only valid l value as of now
        Token::Identifier(id) => {
            let id = id.clone();
            iter.next();
            let next = iter.peek();
            if let Some(Token::DoublePlus) = next {
                iter.next();
                Ok(Expression::Postfix {
                    postfix_operator: PostfixOperator::Increment,
                    expression: Box::new(Expression::Var(id)),
                })
            } else if let Some(Token::DoubleHyphen) = next {
                iter.next();
                Ok(Expression::Postfix {
                    postfix_operator: PostfixOperator::Decrement,
                    expression: Box::new(Expression::Var(id)),
                })
            } else {
                Ok(Expression::Var(id))
            }
        }
        Token::OpenParenthesis => {
            iter.next();
            let inner = parse_expression(iter, 0)?;
            expect!(iter, Token::ClosedParenthesis => ())?;
            let next = iter.peek();
            if let Some(Token::DoublePlus) = next {
                iter.next();
                Ok(Expression::Postfix {
                    postfix_operator: PostfixOperator::Increment,
                    expression: Box::new(inner),
                })
            } else if let Some(Token::DoubleHyphen) = next {
                iter.next();
                Ok(Expression::Postfix {
                    postfix_operator: PostfixOperator::Decrement,
                    expression: Box::new(inner),
                })
            } else {
                Ok(inner)
            }
        }

        Token::Tilde
        | Token::Hyphen
        | Token::Exclamation
        | Token::DoublePlus
        | Token::DoubleHyphen => Ok(Expression::Unary {
            unary_operator: parse_unary(iter)?,
            expression: Box::new(parse_factor(iter)?),
        }),

        tok => Err(Error::ParserError {
            expected: "beginning of factor".to_string(),
            found: tok.to_string(),
        }),
    }
}

fn parse_unary(iter: &mut TokenStream) -> Result<UnaryOperator> {
    match iter.next() {
        Some(Token::Hyphen) => Ok(UnaryOperator::Negate),
        Some(Token::Tilde) => Ok(UnaryOperator::Complement),
        Some(Token::Exclamation) => Ok(UnaryOperator::Not),
        Some(Token::DoublePlus) => Ok(UnaryOperator::Increment),
        Some(Token::DoubleHyphen) => Ok(UnaryOperator::Decrement),
        Some(tok) => Err(Error::ParserError {
            expected: "unary operator".to_string(),
            found: tok.to_string(),
        }),
        None => Err(Error::ParserError {
            expected: "beginning of unary expression".to_string(),
            found: "end of string".to_string(),
        }),
    }
}

fn parse_binary(iter: &mut TokenStream) -> Result<BinaryOperator> {
    match iter.peek() {
        Some(Token::Hyphen) => Ok(BinaryOperator::Subtract),
        Some(Token::Plus) => Ok(BinaryOperator::Add),
        Some(Token::Asterisk) => Ok(BinaryOperator::Multiply),
        Some(Token::ForwardSlash) => Ok(BinaryOperator::Divide),
        Some(Token::Percent) => Ok(BinaryOperator::Remainder),
        Some(Token::Ampersand) => Ok(BinaryOperator::BitwiseAnd),
        Some(Token::LeftShift) => Ok(BinaryOperator::LeftShift),
        Some(Token::RightShift) => Ok(BinaryOperator::RightShift),
        Some(Token::Pipe) => Ok(BinaryOperator::BitwiseOr),
        Some(Token::Carrot) => Ok(BinaryOperator::BitwiseXor),
        Some(Token::DoubleAmpersand) => Ok(BinaryOperator::And),
        Some(Token::DoublePipe) => Ok(BinaryOperator::Or),
        Some(Token::DoubleEqual) => Ok(BinaryOperator::Equal),
        Some(Token::NotEqual) => Ok(BinaryOperator::NotEqual),
        Some(Token::LessThan) => Ok(BinaryOperator::LessThan),
        Some(Token::Leq) => Ok(BinaryOperator::Leq),
        Some(Token::GreaterThan) => Ok(BinaryOperator::GreaterThan),
        Some(Token::Geq) => Ok(BinaryOperator::Geq),
        Some(Token::Equal) => Ok(BinaryOperator::Assigmnent),
        Some(Token::QuestionMark) => Ok(BinaryOperator::Ternary),
        Some(Token::PlusEqual) => Ok(BinaryOperator::CompoundAssignment(Box::new(
            BinaryOperator::Add,
        ))),
        Some(Token::MinusEqual) => Ok(BinaryOperator::CompoundAssignment(Box::new(
            BinaryOperator::Subtract,
        ))),
        Some(Token::AsteriskEqual) => Ok(BinaryOperator::CompoundAssignment(Box::new(
            BinaryOperator::Multiply,
        ))),
        Some(Token::ForwardSlashEqual) => Ok(BinaryOperator::CompoundAssignment(Box::new(
            BinaryOperator::Divide,
        ))),
        Some(Token::PercentEqual) => Ok(BinaryOperator::CompoundAssignment(Box::new(
            BinaryOperator::Remainder,
        ))),
        Some(Token::AmpersandEqual) => Ok(BinaryOperator::CompoundAssignment(Box::new(
            BinaryOperator::BitwiseAnd,
        ))),
        Some(Token::PipeEqual) => Ok(BinaryOperator::CompoundAssignment(Box::new(
            BinaryOperator::BitwiseOr,
        ))),
        Some(Token::CarrotEqual) => Ok(BinaryOperator::CompoundAssignment(Box::new(
            BinaryOperator::BitwiseXor,
        ))),
        Some(Token::LtLtEqual) => Ok(BinaryOperator::CompoundAssignment(Box::new(
            BinaryOperator::LeftShift,
        ))),
        Some(Token::GtGtEqual) => Ok(BinaryOperator::CompoundAssignment(Box::new(
            BinaryOperator::RightShift,
        ))),
        Some(tok) => Err(Error::ParserError {
            expected: "binary operator".to_string(),
            found: tok.to_string(),
        }),
        None => Err(Error::ParserError {
            expected: "beginning of binary expression".to_string(),
            found: "end of string".to_string(),
        }),
    }
}

impl BinaryOperator {
    pub(super) fn precedence(&self) -> i64 {
        match self {
            BinaryOperator::Multiply | BinaryOperator::Divide | BinaryOperator::Remainder => 50,
            BinaryOperator::Add | BinaryOperator::Subtract => 45,
            BinaryOperator::RightShift | BinaryOperator::LeftShift => 40,
            BinaryOperator::LessThan
            | BinaryOperator::Leq
            | BinaryOperator::GreaterThan
            | BinaryOperator::Geq => 39,
            BinaryOperator::NotEqual | BinaryOperator::Equal => 38,
            BinaryOperator::BitwiseAnd => 35,
            BinaryOperator::BitwiseXor => 34,
            BinaryOperator::BitwiseOr => 33,
            BinaryOperator::And => 30,
            BinaryOperator::Or => 29,
            BinaryOperator::Ternary => 25,
            BinaryOperator::Assigmnent | BinaryOperator::CompoundAssignment(_) => 20,
        }
    }
}
