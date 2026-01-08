mod lexer;
mod parser;

use crate::error::Result;
use lexer::lexer;
use parser::parser;
use std::fs;

pub(crate) fn compile(path: &str, lex: bool, parse: bool, codegen: bool) -> Result<()> {
    let code = fs::read_to_string(path).expect("failed to read in file");
    let toks = lexer(&code)?;
    if lex {
        println!("{:?}", toks);
        return Ok(());
    }
    let ast = parser(toks)?;
    if parse {
        println!("{:?}", ast);
        return Ok(());
    }
    Ok(())
}
