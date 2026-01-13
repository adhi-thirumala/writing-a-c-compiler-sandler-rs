use std::iter::Peekable;

use crate::compile::lexer::Token;
use crate::error::{Error, Result};

#[derive(Debug)]
pub(super) enum Program {
    Program(FunctionDefinition),
}

#[derive(Debug)]
pub(super) enum FunctionDefinition {
    Function { name: String, body: Statement },
}

#[derive(Debug)]
pub(super) enum Statement {
    Return(Expression),
}

#[derive(Debug)]
pub(super) enum Expression {
    IntConstant(i64),
    Unary {
        unary_operator: UnaryOperator,
        expression: Box<Expression>,
    },
    Binary {
        binary_operator: BinaryOperator,
        left_expression: Box<Expression>,
        right_expression: Box<Expression>,
    },
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
}

impl BinaryOperator {
    pub(super) fn precedence(&self) -> i64 {
        match self {
            BinaryOperator::Add | BinaryOperator::Subtract => 45,
            BinaryOperator::Multiply | BinaryOperator::Divide | BinaryOperator::Remainder => 50,
        }
    }
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
    let mut iter = toks.into_iter().peekable();
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
fn parse_program(iter: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Program> {
    Ok(Program::Program(parse_function(iter)?))
}

fn parse_function(iter: &mut Peekable<impl Iterator<Item = Token>>) -> Result<FunctionDefinition> {
    expect!(iter, Token::Int => ())?;
    let identifier = expect!(iter, Token::Identifier(id) => id)?;
    expect!(iter, Token::OpenParenthesis => ())?;
    expect!(iter, Token::Void => ())?;
    expect!(iter, Token::ClosedParenthesis => ())?;
    expect!(iter, Token::OpenBrace => ())?;
    let statement = parse_statement(iter)?;
    expect!(iter, Token::ClosedBrace => ())?;
    Ok(FunctionDefinition::Function {
        name: identifier,
        body: statement,
    })
}

fn parse_statement(iter: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Statement> {
    expect!(iter, Token::Return => ())?;
    let expression = parse_expression(iter, 0)?;
    expect!(iter, Token::Semicolon => ())?;
    Ok(Statement::Return(expression))
}

fn parse_expression(
    iter: &mut Peekable<impl Iterator<Item = Token>>,
    min_precedence: i64,
) -> Result<Expression> {
    let mut left = parse_factor(iter)?;
    while let Ok(binop) = parse_binary(iter)
        && binop.precedence() >= min_precedence
    {
        iter.next();
        left = Expression::Binary {
            left_expression: Box::new(left),
            right_expression: Box::new(parse_expression(iter, binop.precedence() + 1)?),
            binary_operator: binop,
        };
    }
    Ok(left)
}

fn parse_factor(iter: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Expression> {
    match iter.peek().ok_or_else(|| Error::ParserError {
        expected: "factor".to_string(),
        found: "end of string".to_string(),
    })? {
        Token::IntegerConstant(val) => {
            let val = *val;
            iter.next();
            Ok(Expression::IntConstant(val))
        }
        Token::OpenParenthesis => {
            iter.next();
            let inner = parse_expression(iter, 0)?;
            expect!(iter, Token::ClosedParenthesis => ())?;
            Ok(inner)
        }
        Token::Tilde | Token::Hyphen => Ok(Expression::Unary {
            unary_operator: parse_unary(iter)?,
            expression: Box::new(parse_factor(iter)?),
        }),
        tok => Err(Error::ParserError {
            expected: "beginning of factor".to_string(),
            found: tok.to_string(),
        }),
    }
}

fn parse_unary(iter: &mut Peekable<impl Iterator<Item = Token>>) -> Result<UnaryOperator> {
    match iter.next() {
        Some(Token::Hyphen) => Ok(UnaryOperator::Negate),
        Some(Token::Tilde) => Ok(UnaryOperator::Complement),
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

fn parse_binary(iter: &mut Peekable<impl Iterator<Item = Token>>) -> Result<BinaryOperator> {
    match iter.peek() {
        Some(Token::Hyphen) => Ok(BinaryOperator::Subtract),
        Some(Token::Plus) => Ok(BinaryOperator::Add),
        Some(Token::Asterisk) => Ok(BinaryOperator::Multiply),
        Some(Token::ForwardSlash) => Ok(BinaryOperator::Divide),
        Some(Token::Percent) => Ok(BinaryOperator::Remainder),
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
