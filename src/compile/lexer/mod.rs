mod tokens;

use crate::error::{Error, Result};
use regex::Regex;

pub use tokens::Token;

pub(super) trait Lex {
    fn lex(self) -> Lexer;
}

pub(super) struct Lexer {
    input: String,
    pos: usize,
    id: Regex,
    cnst: Regex,
}

impl Lexer {
    fn new(input: String) -> Self {
        Lexer {
            input,
            id: Regex::new(r"^[a-zA-Z_]\w*\b").expect("this is constant, will work"),
            cnst: Regex::new(r"^[0-9]+\b").expect("this is constant"),
            pos: 0,
        }
    }

    fn trim(&mut self) {
        while self.pos < self.input.len()
            && self
                .input
                .chars()
                .nth(self.pos)
                .expect("already bounds checked")
                .is_whitespace()
        {
            self.pos += 1
        }
    }

    fn next_keyword(&self) -> Option<(Token, usize)> {
        match self.id.find(&self.input[self.pos..]) {
            Some(m) => {
                let str = m.as_str();
                // all keywords
                Some((
                    match str {
                        "if" => Token::If,
                        "do" => Token::Do,
                        "while" => Token::While,
                        "for" => Token::For,
                        "break" => Token::Break,
                        "continue" => Token::Continue,
                        "int" => Token::Int,
                        "goto" => Token::Goto,
                        "else" => Token::Else,
                        "void" => Token::Void,
                        "return" => Token::Return,
                        "switch" => Token::Switch,
                        "case" => Token::Case,
                        "default" => Token::Default,
                        s => Token::Identifier(s.to_string()),
                    },
                    self.pos + str.len(),
                ))
            }
            None => None,
        }
    }

    fn next_constant(&self) -> Option<(Token, usize)> {
        match self.cnst.find(&self.input[self.pos..]) {
            Some(m) => {
                let str = m.as_str();
                Some((
                    Token::IntegerConstant(
                        str.parse::<i32>()
                            .expect("numbers are not bigger than i32 raw"),
                    ),
                    self.pos + str.len(),
                ))
            }
            None => None,
        }
    }

    fn next_operator(&self) -> Result<(Token, usize)> {
        let mut iter = self.input.chars();
        let first_char = iter.nth(self.pos).expect("string checked to be non empty");
        let second_char = iter.nth(0);
        let third_char = iter.nth(0);
        let new_size_1 = self.pos + 1;
        let new_size_2 = self.pos + 2;
        let new_size_3 = self.pos + 3;
        Ok(match first_char {
            '(' => (Token::OpenParenthesis, new_size_1),
            ')' => (Token::ClosedParenthesis, new_size_1),
            '{' => (Token::OpenBrace, new_size_1),
            '}' => (Token::ClosedBrace, new_size_1),
            ';' => (Token::Semicolon, new_size_1),
            '?' => (Token::QuestionMark, new_size_1),
            ':' => (Token::Colon, new_size_1),
            '~' => (Token::Tilde, new_size_1),
            '-' => match second_char {
                Some('-') => (Token::DoubleHyphen, new_size_2),
                Some('=') => (Token::MinusEqual, new_size_2),
                Some(_) | None => (Token::Hyphen, new_size_1),
            },
            '+' => match second_char {
                Some('+') => (Token::DoublePlus, new_size_2),
                Some('=') => (Token::PlusEqual, new_size_2),
                Some(_) | None => (Token::Plus, new_size_1),
            },
            '*' => match second_char {
                Some('=') => (Token::AsteriskEqual, new_size_2),
                Some(_) | None => (Token::Asterisk, new_size_1),
            },
            '/' => match second_char {
                Some('=') => (Token::ForwardSlashEqual, new_size_2),
                Some(_) | None => (Token::ForwardSlash, new_size_1),
            },
            '%' => match second_char {
                Some('=') => (Token::PercentEqual, new_size_2),
                Some(_) | None => (Token::Percent, new_size_1),
            },

            '&' => match second_char {
                Some('&') => (Token::DoubleAmpersand, new_size_2),
                Some('=') => (Token::AmpersandEqual, new_size_2),
                Some(_) | None => (Token::Ampersand, new_size_1),
            },
            '|' => match second_char {
                Some('|') => (Token::DoublePipe, new_size_2),
                Some('=') => (Token::PipeEqual, new_size_2),
                Some(_) | None => (Token::Pipe, new_size_1),
            },
            '^' => match second_char {
                Some('=') => (Token::CarrotEqual, new_size_2),
                Some(_) | None => (Token::Carrot, new_size_1),
            },

            '>' => match second_char {
                Some('>') => match third_char {
                    Some('=') => (Token::GtGtEqual, new_size_3),
                    Some(_) | None => (Token::RightShift, new_size_2),
                },
                Some('=') => (Token::Geq, new_size_2),
                Some(_) | None => (Token::GreaterThan, new_size_1),
            },
            '<' => match second_char {
                Some('<') => match third_char {
                    Some('=') => (Token::LtLtEqual, new_size_3),
                    Some(_) | None => (Token::LeftShift, new_size_2),
                },
                Some('=') => (Token::Leq, new_size_2),
                Some(_) | None => (Token::LessThan, new_size_1),
            },
            '!' => match second_char {
                Some('=') => (Token::NotEqual, new_size_2),
                Some(_) | None => (Token::Exclamation, new_size_1),
            },
            '=' => match second_char {
                Some('=') => (Token::DoubleEqual, new_size_2),
                Some(_) | None => (Token::Equal, new_size_1),
            },
            c => {
                return Err(Error::LexerError { char: c });
            }
        })
    }
}

impl Iterator for Lexer {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        self.trim();
        if self.pos >= self.input.len() {
            return None;
        }

        match self.next_keyword() {
            Some((token, pos)) => {
                self.pos = pos;
                return Some(Ok(token));
            }
            None => (),
        };

        match self.next_constant() {
            Some((token, pos)) => {
                self.pos = pos;
                return Some(Ok(token));
            }
            None => (),
        };

        match self.next_operator() {
            Ok((token, pos)) => {
                self.pos = pos;
                Some(Ok(token))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

impl Lex for String {
    fn lex(self) -> Lexer {
        Lexer::new(self)
    }
}
