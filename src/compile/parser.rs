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
    let mut iter = toks.into_iter();
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
fn parse_program(iter: &mut impl Iterator<Item = Token>) -> Result<Program> {
    Ok(Program::Program(parse_function(iter)?))
}

fn parse_function(iter: &mut impl Iterator<Item = Token>) -> Result<FunctionDefinition> {
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

fn parse_statement(iter: &mut impl Iterator<Item = Token>) -> Result<Statement> {
    expect!(iter, Token::Return => ())?;
    let expression = parse_expression(iter)?;
    expect!(iter, Token::Semicolon => ())?;
    Ok(Statement::Return(expression))
}

fn parse_expression(iter: &mut impl Iterator<Item = Token>) -> Result<Expression> {
    Ok(Expression::IntConstant(
        expect!(iter, Token::IntegerConstant(num) => num)?,
    ))
}
