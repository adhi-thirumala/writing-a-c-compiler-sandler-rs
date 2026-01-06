use std::fs;

use regex::Regex;

use crate::error::Error;

pub(crate) enum Tokens {
    Identifier(String),
    IntConstant(i64),
    Int,
    Void,
    Return,
    OpenParen,
    ClosedParen,
    OpenBrace,
    ClosedBrace,
    Semicolon,
}

pub(crate) fn compile(path: &str, lex: bool, parse: bool, codegen: bool) -> Result<(), Error> {
    let mut code = fs::read_to_string(path).expect("failed to read in file");
    let toks = lexer(&code)?;
    if lex {
        return Ok(());
    }
    Ok(())
}

fn lexer(mut input: &str) -> Result<Vec<Tokens>, Error> {
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
                    "int" => Tokens::Int,
                    "void" => Tokens::Void,
                    "return" => Tokens::Return,
                    s => Tokens::Identifier(s.to_string()),
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
                toks.push(Tokens::IntConstant(m.as_str().parse().expect(
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
                '(' => Tokens::OpenParen,
                ')' => Tokens::ClosedParen,
                '{' => Tokens::OpenBrace,
                '}' => Tokens::ClosedBrace,
                ';' => Tokens::Semicolon,
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
