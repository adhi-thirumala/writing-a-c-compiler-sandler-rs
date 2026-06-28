mod tokens;

use crate::error::{Error, Result};
use regex::Regex;

pub use tokens::Token;

pub(super) trait Lex {
    fn lex(self) -> Lexer;
}

pub(super) struct Lexer {
    input: String,
    chars: Vec<u8>,
    pos: usize,
    id: Regex,
    cnst: Regex,
}

impl Lexer {
    fn new(input: String) -> Self {
        Lexer {
            input: input.clone(),
            chars: input.into_bytes(),
            id: Regex::new(r"^[a-zA-Z_]\w*\b").expect("this is constant, will work"),
            cnst: Regex::new(r"^[0-9]+\b").expect("this is constant"),
            pos: 0,
        }
    }
    fn trim(&mut self) {
        while self.pos < self.input.len() && self.chars[self.pos].is_ascii_whitespace() {
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
        let one_more = self.pos + 1;
        let two_more = self.pos + 2;
        let three_more = self.pos + 3;
        let first_char = self.chars[self.pos] as char;
        let second_char = self.chars.get(one_more).map(|x| *x as char);
        let third_char = self.chars.get(two_more).map(|x| *x as char);
        Ok(match first_char {
            '(' => (Token::OpenParenthesis, one_more),
            ')' => (Token::ClosedParenthesis, one_more),
            '{' => (Token::OpenBrace, one_more),
            '}' => (Token::ClosedBrace, one_more),
            ';' => (Token::Semicolon, one_more),
            '?' => (Token::QuestionMark, one_more),
            ':' => (Token::Colon, one_more),
            '~' => (Token::Tilde, one_more),
            '-' => match second_char {
                Some('-') => (Token::DoubleHyphen, two_more),
                Some('=') => (Token::MinusEqual, two_more),
                Some(_) | None => (Token::Hyphen, one_more),
            },
            '+' => match second_char {
                Some('+') => (Token::DoublePlus, two_more),
                Some('=') => (Token::PlusEqual, two_more),
                Some(_) | None => (Token::Plus, one_more),
            },
            '*' => match second_char {
                Some('=') => (Token::AsteriskEqual, two_more),
                Some(_) | None => (Token::Asterisk, one_more),
            },
            '/' => match second_char {
                Some('=') => (Token::ForwardSlashEqual, two_more),
                Some(_) | None => (Token::ForwardSlash, one_more),
            },
            '%' => match second_char {
                Some('=') => (Token::PercentEqual, two_more),
                Some(_) | None => (Token::Percent, one_more),
            },

            '&' => match second_char {
                Some('&') => (Token::DoubleAmpersand, two_more),
                Some('=') => (Token::AmpersandEqual, two_more),
                Some(_) | None => (Token::Ampersand, one_more),
            },
            '|' => match second_char {
                Some('|') => (Token::DoublePipe, two_more),
                Some('=') => (Token::PipeEqual, two_more),
                Some(_) | None => (Token::Pipe, one_more),
            },
            '^' => match second_char {
                Some('=') => (Token::CarrotEqual, two_more),
                Some(_) | None => (Token::Carrot, one_more),
            },

            '>' => match second_char {
                Some('>') => match third_char {
                    Some('=') => (Token::GtGtEqual, three_more),
                    Some(_) | None => (Token::RightShift, two_more),
                },
                Some('=') => (Token::Geq, two_more),
                Some(_) | None => (Token::GreaterThan, one_more),
            },
            '<' => match second_char {
                Some('<') => match third_char {
                    Some('=') => (Token::LtLtEqual, three_more),
                    Some(_) | None => (Token::LeftShift, two_more),
                },
                Some('=') => (Token::Leq, two_more),
                Some(_) | None => (Token::LessThan, one_more),
            },
            '!' => match second_char {
                Some('=') => (Token::NotEqual, two_more),
                Some(_) | None => (Token::Exclamation, one_more),
            },
            '=' => match second_char {
                Some('=') => (Token::DoubleEqual, two_more),
                Some(_) | None => (Token::Equal, one_more),
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
