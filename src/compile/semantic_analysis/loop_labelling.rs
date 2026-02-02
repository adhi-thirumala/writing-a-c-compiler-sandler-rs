use super::{Error, Result};

use super::super::tacky::make_temp_label;
use super::parser;

pub(super) fn loop_label(ast: &mut parser::Program) -> Result<()> {
    resolve_program(ast)
}

fn resolve_program(program: &mut parser::Program) -> Result<()> {
    let parser::Program::Program(function) = program;
    resolve_function(function)
}

fn resolve_function(function: &mut parser::FunctionDefinition) -> Result<()> {
    let parser::FunctionDefinition::Function {
        body: parser::Block::Block(body),
        name,
    } = function;
    body.iter_mut()
        .try_for_each(|block_item| resolve_block_item(block_item, None, &name))
}

fn resolve_block_item(
    block_item: &mut parser::BlockItem,
    loop_id: Option<&str>,
    function_name: &str,
) -> Result<()> {
    let parser::BlockItem::S(statement) = block_item else {
        return Ok(());
    };
    resolve_statement(statement, loop_id, function_name)
}

fn resolve_statement(
    statement: &mut parser::Statement,
    loop_id: Option<&str>,
    function_name: &str,
) -> Result<()> {
    match statement {
        parser::Statement::Break(label) => match loop_id {
            Some(current_label) => *label = Some(current_label.to_string()),
            None => return Err(Error::SemanticError("break outside of loop")),
        },
        parser::Statement::Continue(label) => match loop_id {
            Some(current_label) => *label = Some(current_label.to_string()),
            None => return Err(Error::SemanticError("break outside of loop")),
        },
        parser::Statement::DoWhile { body, label, .. } => {
            let new_label = make_temp_label(function_name);
            resolve_statement(body, Some(&new_label), function_name)?;
            *label = Some(new_label);
        }
        parser::Statement::For { body, label, .. } => {
            let new_label = make_temp_label(function_name);
            resolve_statement(body, Some(&new_label), function_name)?;
            *label = Some(new_label);
        }
        parser::Statement::While { body, label, .. } => {
            let new_label = make_temp_label(function_name);
            resolve_statement(body, Some(&new_label), function_name)?;
            *label = Some(new_label);
        }
        parser::Statement::If {
            then_statement,
            else_statement,
            ..
        } => {
            resolve_statement(then_statement, loop_id, function_name)?;
            if let Some(statement) = else_statement {
                resolve_statement(statement, loop_id, function_name)?
            }
        }
        parser::Statement::Compound(parser::Block::Block(body)) => body
            .iter_mut()
            .try_for_each(|block_item| resolve_block_item(block_item, loop_id, function_name))?,

        parser::Statement::Return(_)
        | parser::Statement::Expression(_)
        | parser::Statement::Goto(_)
        | parser::Statement::Label(_)
        | parser::Statement::Null => (),
    }
    Ok(())
}
