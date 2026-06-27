use super::{Error, Result};

use super::super::tacky::make_temp_label;
use super::parser;

#[derive(Clone)]
enum Inner {
    Switch,
    Loop,
    None,
}

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
        .try_for_each(|block_item| resolve_block_item(block_item, None, None, Inner::None, &name))
}

fn resolve_block_item(
    block_item: &mut parser::BlockItem,
    loop_id: Option<&str>,
    switch_id: Option<&str>,
    inner: Inner,
    function_name: &str,
) -> Result<()> {
    let parser::BlockItem::S(statement) = block_item else {
        return Ok(());
    };
    resolve_statement(statement, loop_id, switch_id, inner, function_name)
}

fn resolve_statement(
    statement: &mut parser::Statement,
    loop_id: Option<&str>,
    switch_id: Option<&str>,
    inner: Inner,
    function_name: &str,
) -> Result<()> {
    match statement {
        parser::Statement::Break(label) => match inner {
            Inner::Switch => {
                *label = Some(
                    switch_id
                        .map(|x| x.to_string())
                        .expect("enums mean we never hit this case"),
                )
            }
            Inner::Loop => {
                *label = Some(
                    loop_id
                        .map(|x| x.to_string())
                        .expect("enums mean we never hit this case"),
                )
            }
            Inner::None => return Err(Error::SemanticError("break outside of loop")),
        },

        parser::Statement::Default { body, label }
        | parser::Statement::Case { body, label, .. } => match switch_id {
            Some(current_label) => {
                *label = Some(current_label.to_string());
                resolve_statement(body, loop_id, switch_id, inner, function_name)?
            }
            None => return Err(Error::SemanticError("default or case outside of loop")),
        },
        parser::Statement::Continue(label) => match loop_id {
            Some(current_label) => *label = Some(current_label.to_string()),
            None => return Err(Error::SemanticError("continue outside of loop")),
        },

        parser::Statement::For { body, label, .. }
        | parser::Statement::While { body, label, .. }
        | parser::Statement::DoWhile { body, label, .. } => {
            let new_label = make_temp_label(function_name);
            resolve_statement(
                body,
                Some(&new_label),
                switch_id,
                Inner::Loop,
                function_name,
            )?;
            *label = Some(new_label);
        }

        parser::Statement::Switch { body, label, .. } => {
            let new_label = make_temp_label(function_name);
            resolve_statement(
                body,
                loop_id,
                Some(&new_label),
                Inner::Switch,
                function_name,
            )?;
            *label = Some(new_label);
        }

        parser::Statement::If {
            then_statement,
            else_statement,
            ..
        } => {
            resolve_statement(
                then_statement,
                loop_id,
                switch_id,
                inner.clone(),
                function_name,
            )?;
            if let Some(statement) = else_statement {
                resolve_statement(statement, loop_id, switch_id, inner, function_name)?
            }
        }
        parser::Statement::Compound(parser::Block::Block(body)) => {
            body.iter_mut().try_for_each(|block_item| {
                resolve_block_item(block_item, loop_id, switch_id, inner.clone(), function_name)
            })?
        }

        parser::Statement::Return(_)
        | parser::Statement::Expression(_)
        | parser::Statement::Goto(_)
        | parser::Statement::Label(_)
        | parser::Statement::Null => (),
    }
    Ok(())
}
