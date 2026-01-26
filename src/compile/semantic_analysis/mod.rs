mod duplicate_labels;
mod lvalue_check;
mod statement_after_labels;
mod variable_resolution;

use super::super::error::Error;
use super::Result;
use super::parser;
use super::tacky::TACKY_COUNTER;
use duplicate_labels::duplicate_labels_resolution;
use lvalue_check::check_lvalue;
use statement_after_labels::statement_after_labels_resolution;
use variable_resolution::variable_resolution;

pub(super) fn semantic_analysis(ast: parser::Program) -> Result<parser::Program> {
    let mut ast = ast;
    variable_resolution(&mut ast)?;
    check_lvalue(&ast)?;
    duplicate_labels_resolution(&mut ast)?;
    statement_after_labels_resolution(&ast)?;
    Ok(ast)
}
