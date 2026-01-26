use super::{Error, Result};

use super::parser;
use std::collections::HashSet;

pub(super) fn duplicate_labels_resolution(ast: &mut parser::Program) -> Result<()> {
    resolve_program(ast)
}

fn resolve_program(program: &mut parser::Program) -> Result<()> {
    let parser::Program::Program(function_definition) = program;
    resolve_function(function_definition)
}

fn resolve_function(function: &mut parser::FunctionDefinition) -> Result<()> {
    let parser::FunctionDefinition::Function { body, .. } = function;
    let mut labels = HashSet::new();
    body.iter_mut()
        .try_for_each(|block_item| resolve_block_item(block_item, &mut labels))
}

fn resolve_block_item(
    block_item: &mut parser::BlockItem,
    labels: &mut HashSet<String>,
) -> Result<()> {
    let parser::BlockItem::S(statement) = block_item else {
        return Ok(());
    };
    resolve_statement(statement, labels)
}

fn resolve_statement(
    statement: &mut parser::Statement,
    labels: &mut HashSet<String>,
) -> Result<()> {
    let parser::Statement::Label(label) = statement else {
        return Ok(());
    };
    if labels.contains(label) {
        return Err(Error::SemanticError("duplicate label"));
    } else {
        labels.insert(label.clone());
        Ok(())
    }
}
