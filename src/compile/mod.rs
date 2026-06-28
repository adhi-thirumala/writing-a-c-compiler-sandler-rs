mod asm_gen;
mod code_emission;
mod lexer;
mod parser;
mod semantic_analysis;
mod tacky;

use super::error::Result;
use asm_gen::asm_gen;
use code_emission::code_emission;
use lexer::Lex;
use parser::parser;
use semantic_analysis::semantic_analysis;
use std::io::Write;
use tacky::tacky_gen;

pub(crate) fn compile(
    writer: &mut impl Write,
    code: String,
    lex: bool,
    parse: bool,
    codegen: bool,
    tacky: bool,
    validate: bool,
) -> Result<()> {
    let toks = code.lex();
    if lex {
        println!("{:#?}", toks.collect::<Result<Vec<_>>>()?);
        return Ok(());
    }

    let ast = parser(toks)?;
    if parse {
        println!("{:#?}", ast);
        return Ok(());
    }

    let validated_ast = semantic_analysis(ast)?;
    if validate {
        println!("{:#?}", validated_ast);
        return Ok(());
    }
    let tacky_ast = tacky_gen(validated_ast)?;
    if tacky {
        println!("{:#?}", tacky_ast);
        return Ok(());
    }
    let asm_ast = asm_gen(tacky_ast)?;
    if codegen {
        println!("{:#?}", asm_ast);
        return Ok(());
    }

    code_emission(writer, asm_ast)
}
