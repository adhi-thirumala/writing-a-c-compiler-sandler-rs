use crate::error::{Error, Result};
use regex::Regex;
use strum_macros::{Display, EnumString};

#[derive(Debug, Display, EnumString)]
pub(super) enum Token {
    #[strum(to_string = "Identifier: {0}")]
    Identifier(String),
    #[strum(to_string = "Integer Constant: {0}")]
    IntegerConstant(i64),
    Int,
    Void,
    Return,
    OpenParenthesis,
    ClosedParenthesis,
    OpenBrace,
    ClosedBrace,
    Semicolon,
    Tilde,
    Hyphen,
    DoubleHyphen,
    Plus,
    Asterisk,
    ForwardSlash,
    Percent,
}

pub(super) fn lexer(mut input: &str) -> Result<Vec<Token>> {
    let mut toks = Vec::new();

    //initialize all regexes
    let id_regex = Regex::new(r"^[a-zA-Z_]\w*\b").expect("failed to compile identifier regex");
    let const_regex = Regex::new(r"^[0-9]+\b").expect("failed to compile constant regex");

    input = input.trim_start();
    while !input.is_empty() {
        let tok = id_regex.find(input);
        match tok {
            Some(m) => {
                // all keywords
                toks.push(match m.as_str() {
                    "int" => Token::Int,
                    "void" => Token::Void,
                    "return" => Token::Return,
                    s => Token::Identifier(s.to_string()),
                });
                input = &input[m.len()..];
                input = input.trim_start();
                continue;
            }
            None => (),
        }

        let tok = const_regex.find(input);
        match tok {
            Some(m) => {
                toks.push(Token::IntegerConstant(m.as_str().parse().expect(
                    "regex mandates this is an int so it must be overflow if not",
                )));
                input = &input[m.len()..];
                input = input.trim_start();
                continue;
            }
            None => (),
        }

        // all 1 char toks - if we're here, we've failed to match all longer possibilities, there
        // next character MUST be a non alphanumeric or underscore char
        // cant be multiple tying 1 char regexes
        toks.push(
            match &input
                .chars()
                .nth(0)
                .expect("string cant be empty, alr checked")
            {
                '(' => Token::OpenParenthesis,
                ')' => Token::ClosedParenthesis,
                '{' => Token::OpenBrace,
                '}' => Token::ClosedBrace,
                ';' => Token::Semicolon,
                '-' => match &input.chars().nth(1) {
                    Some('-') => {
                        input = &input[2..];
                        input = input.trim_start();
                        Token::DoubleHyphen
                    }
                    Some(_) => Token::Hyphen,
                    None => Token::Hyphen,
                },
                '~' => Token::Tilde,
                '+' => Token::Plus,
                '*' => Token::Asterisk,
                '/' => Token::ForwardSlash,
                '%' => Token::Percent,
                c => {
                    return Err(Error::LexerError { char: *c });
                }
            },
        );
        input = &input[1..];
        input = input.trim_start();
        //keeping it at the end allows us to trim, and then immediately
        //check for emptiness such that empty strings dont cause lexer
        //error
    }
    Ok(toks)
}
