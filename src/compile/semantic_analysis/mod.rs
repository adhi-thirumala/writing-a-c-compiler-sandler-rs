mod variable_resolution;

use super::super::error::Error;
use super::Result;
use super::parser;
use super::tacky::TACKY_COUNTER;

pub(super) fn semantic_analysis(ast: parser::Program) -> Result<parser::Program> {
    let mut ast = ast;
    variable_resolution::variable_resolution(&mut ast)?;
    Ok(ast)
}
