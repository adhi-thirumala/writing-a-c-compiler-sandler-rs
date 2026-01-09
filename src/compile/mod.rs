mod asm_gen;
mod code_emission;
mod lexer;
mod parser;

use crate::{compile::code_emission::code_emission, error::Result};
use asm_gen::asm_gen;
use lexer::lexer;
use parser::parser;
use std::io::Write;

pub(crate) fn compile(
    writer: &mut impl Write,
    code: &str,
    lex: bool,
    parse: bool,
    codegen: bool,
) -> Result<()> {
    let toks = lexer(code)?;
    if lex {
        println!("{:?}", toks);
        return Ok(());
    }

    let ast = parser(toks)?;
    if parse {
        println!("{:?}", ast);
        return Ok(());
    }

    let asm_ast = asm_gen(ast)?;
    if codegen {
        println!("{:?}", asm_ast);
        return Ok(());
    }
    code_emission(writer, asm_ast)?;
    Ok(())
}
