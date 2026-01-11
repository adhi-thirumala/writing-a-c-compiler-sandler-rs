mod asm_gen;
mod code_emission;
mod lexer;
mod parser;
mod tacky;

use crate::{compile::code_emission::code_emission, error::Result};
use asm_gen::asm_gen;
use lexer::lexer;
use parser::parser;
use std::io::Write;
use tacky::tacky_gen;

pub(crate) fn compile(
    writer: &mut impl Write,
    code: &str,
    lex: bool,
    parse: bool,
    codegen: bool,
    tacky: bool,
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

    let tacky_ast = tacky_gen(ast)?;
    if tacky {
        println!("{:?}", tacky_ast);
        return Ok(());
    }
    let asm_ast = asm_gen(tacky_ast)?;
    if codegen {
        println!("{:?}", asm_ast);
        return Ok(());
    }
    todo!("fix code emission")
    //code_emission(writer, asm_ast)?;
}
