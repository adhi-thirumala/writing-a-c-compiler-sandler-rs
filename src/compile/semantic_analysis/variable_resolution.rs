use super::{Error, Result};

use super::TACKY_COUNTER;
use super::parser;
use std::collections::HashMap;

pub(super) fn variable_resolution(ast: &mut parser::Program) -> Result<()> {
    let mut variable_map = HashMap::<String, String>::new();
    resolve_program(ast, &mut variable_map)
}

fn resolve_program(
    program: &mut parser::Program,
    variable_map: &mut HashMap<String, String>,
) -> Result<()> {
    match program {
        parser::Program::Program(function_definition) => {
            resolve_function(function_definition, variable_map)
        }
    }
}

fn resolve_function(
    function: &mut parser::FunctionDefinition,
    variable_map: &mut HashMap<String, String>,
) -> Result<()> {
    match function {
        parser::FunctionDefinition::Function { body, .. } => body
            .iter_mut()
            .try_for_each(|block_item| resolve_block_item(block_item, variable_map)),
    }
}

fn resolve_block_item(
    block_item: &mut parser::BlockItem,
    variable_map: &mut HashMap<String, String>,
) -> Result<()> {
    match block_item {
        parser::BlockItem::D(declaration) => resolve_declaration(declaration, variable_map),
        parser::BlockItem::S(statement) => resolve_statement(statement, variable_map),
    }
}

fn resolve_declaration(
    declaration: &mut parser::Declaration,
    variable_map: &mut HashMap<String, String>,
) -> Result<()> {
    let parser::Declaration::Declaration { name, init } = declaration;
    if variable_map.contains_key(name) {
        Err(Error::SemanticError("duplicate declaration"))
    } else {
        let unique_name = make_temporary_name(name);
        variable_map.insert(name.clone(), unique_name.clone());
        if let Some(expression) = init {
            resolve_expression(expression, variable_map)?;
        }
        *name = unique_name;
        Ok(())
    }
}

fn resolve_statement(
    statement: &mut parser::Statement,
    variable_map: &mut HashMap<String, String>,
) -> Result<()> {
    match statement {
        parser::Statement::Return(expression) | parser::Statement::Expression(expression) => {
            resolve_expression(expression, variable_map)
        }
        parser::Statement::Null => Ok(()),
    }
}

fn resolve_expression(
    expression: &mut parser::Expression,
    variable_map: &mut HashMap<String, String>,
) -> Result<()> {
    match expression {
        parser::Expression::Assignment {
            left_expression,
            right_expression,
            ..
        } => {
            if let parser::Expression::Var(_) = **left_expression {
                resolve_expression(left_expression, variable_map)?;
                resolve_expression(right_expression, variable_map)?;
            } else {
                return Err(Error::SemanticError("invalid lvalue"));
            }
        }
        parser::Expression::Var(identifier) => match variable_map.get(identifier) {
            Some(val) => *identifier = val.to_string(),
            None => return Err(Error::SemanticError("undeclared variable")),
        },
        parser::Expression::Unary { expression, .. } => {
            resolve_expression(expression, variable_map)?
        }
        parser::Expression::Binary {
            left_expression,
            right_expression,
            ..
        } => {
            resolve_expression(left_expression, variable_map)?;
            resolve_expression(right_expression, variable_map)?;
        }
        parser::Expression::IntConstant(_) => (),
    }
    Ok(())
}

fn make_temporary_name(name: &str) -> String {
    let temp_name = format!("tmp.{}.{:?}", name, TACKY_COUNTER);
    TACKY_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    temp_name
}
