use crate::error::{Error, Result};
use regex::Regex;
use strum_macros::{Display, EnumString};

#[derive(Debug, Display, EnumString)]
pub(super) enum Token {
    #[strum(to_string = "Identifier: {0}")]
    Identifier(String),
    #[strum(to_string = "Integer Constant: {0}")]
    IntegerConstant(i32),
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
    Plus,
    Asterisk,
    ForwardSlash,
    Percent,
    Ampersand,
    Pipe,
    Carrot,
    LeftShift,
    RightShift,
    Exclamation,
    DoubleAmpersand,
    DoublePipe,
    DoubleEqual,
    NotEqual,
    LessThan,
    GreaterThan,
    Leq,
    Geq,
    Equal,
    PlusEqual,
    MinusEqual,
    AsteriskEqual,
    ForwardSlashEqual,
    PercentEqual,
    AmpersandEqual,
    PipeEqual,
    CarrotEqual,
    LtLtEqual,
    GtGtEqual,
    DoubleHyphen,
    DoublePlus,
    If,
    Else,
    QuestionMark,
    Colon,
    Goto,
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
                    "if" => Token::If,
                    "int" => Token::Int,
                    "goto" => Token::Goto,
                    "else" => Token::Else,
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
        let mut length = 1;
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
                        length = 2;
                        Token::DoubleHyphen
                    }
                    Some('=') => {
                        length = 2;
                        Token::MinusEqual
                    }
                    Some(_) | None => Token::Hyphen,
                },
                '~' => Token::Tilde,
                '+' => match &input.chars().nth(1) {
                    Some('+') => {
                        length = 2;
                        Token::DoublePlus
                    }
                    Some('=') => {
                        length = 2;
                        Token::PlusEqual
                    }
                    Some(_) | None => Token::Plus,
                },
                '*' => match &input.chars().nth(1) {
                    Some('=') => {
                        length = 2;
                        Token::AsteriskEqual
                    }
                    Some(_) | None => Token::Asterisk,
                },
                '/' => match &input.chars().nth(1) {
                    Some('=') => {
                        length = 2;
                        Token::ForwardSlashEqual
                    }
                    Some(_) | None => Token::ForwardSlash,
                },
                '%' => match &input.chars().nth(1) {
                    Some('=') => {
                        length = 2;
                        Token::PercentEqual
                    }
                    Some(_) | None => Token::Percent,
                },

                '&' => match &input.chars().nth(1) {
                    Some('&') => {
                        length = 2;
                        Token::DoubleAmpersand
                    }
                    Some('=') => {
                        length = 2;
                        Token::AmpersandEqual
                    }
                    Some(_) | None => Token::Ampersand,
                },
                '|' => match &input.chars().nth(1) {
                    Some('|') => {
                        length = 2;
                        Token::DoublePipe
                    }
                    Some('=') => {
                        length = 2;
                        Token::PipeEqual
                    }
                    Some(_) | None => Token::Pipe,
                },
                '^' => match &input.chars().nth(1) {
                    Some('=') => {
                        length = 2;
                        Token::CarrotEqual
                    }
                    Some(_) | None => Token::Carrot,
                },

                '>' => match &input.chars().nth(1) {
                    Some('>') => match &input.chars().nth(2) {
                        Some('=') => {
                            length = 3;
                            Token::GtGtEqual
                        }
                        Some(_) | None => {
                            length = 2;
                            Token::RightShift
                        }
                    },
                    Some('=') => {
                        length = 2;
                        Token::Geq
                    }
                    Some(_) | None => Token::GreaterThan,
                },
                '<' => match &input.chars().nth(1) {
                    Some('<') => match &input.chars().nth(2) {
                        Some('=') => {
                            length = 3;
                            Token::LtLtEqual
                        }
                        Some(_) | None => {
                            length = 2;
                            Token::LeftShift
                        }
                    },
                    Some('=') => {
                        length = 2;
                        Token::Leq
                    }
                    Some(_) | None => Token::LessThan,
                },
                '!' => match &input.chars().nth(1) {
                    Some('=') => {
                        length = 2;
                        Token::NotEqual
                    }
                    Some(_) | None => Token::Exclamation,
                },
                '=' => match &input.chars().nth(1) {
                    Some('=') => {
                        length = 2;
                        Token::DoubleEqual
                    }
                    Some(_) | None => Token::Equal,
                },
                '?' => Token::QuestionMark,
                ':' => Token::Colon,
                c => {
                    return Err(Error::LexerError { char: *c });
                }
            },
        );
        input = &input[length..];
        input = input.trim_start();
        //keeping it at the end allows us to trim, and then immediately
        //check for emptiness such that empty strings dont cause lexer
        //error
    }
    Ok(toks)
}
