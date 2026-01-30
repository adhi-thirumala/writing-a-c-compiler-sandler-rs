use super::{Error, Result};

use super::TACKY_COUNTER;
use super::parser;
use std::collections::HashMap;

struct MapEntry {
    name: String,
    from_current_block: bool,
}

pub(super) fn variable_resolution(ast: &mut parser::Program) -> Result<()> {
    let mut variable_map = HashMap::<String, MapEntry>::new();
    resolve_program(ast, &mut variable_map)
}

fn resolve_program(
    program: &mut parser::Program,
    variable_map: &mut HashMap<String, MapEntry>,
) -> Result<()> {
    match program {
        parser::Program::Program(function_definition) => {
            resolve_function(function_definition, variable_map)
        }
    }
}

fn resolve_function(
    function: &mut parser::FunctionDefinition,
    variable_map: &mut HashMap<String, MapEntry>,
) -> Result<()> {
    match function {
        parser::FunctionDefinition::Function { body, .. } => resolve_block(body, variable_map),
    }
}

fn resolve_block(
    block: &mut parser::Block,
    variable_map: &mut HashMap<String, MapEntry>,
) -> Result<()> {
    let parser::Block::Block(body) = block;
    body.iter_mut()
        .try_for_each(|block_item| resolve_block_item(block_item, variable_map))
}

fn resolve_block_item(
    block_item: &mut parser::BlockItem,
    variable_map: &mut HashMap<String, MapEntry>,
) -> Result<()> {
    match block_item {
        parser::BlockItem::D(declaration) => resolve_declaration(declaration, variable_map),
        parser::BlockItem::S(statement) => resolve_statement(statement, variable_map),
    }
}

fn resolve_declaration(
    declaration: &mut parser::Declaration,
    variable_map: &mut HashMap<String, MapEntry>,
) -> Result<()> {
    let parser::Declaration::Declaration { name, init } = declaration;
    if let Some(variable) = variable_map.get(name)
        && variable.from_current_block == true
    {
        Err(Error::SemanticError("duplicate declaration"))
    } else {
        let unique_name = make_temporary_name(name);
        variable_map.insert(
            name.clone(),
            MapEntry {
                name: unique_name.clone(),
                from_current_block: true,
            },
        );
        if let Some(expression) = init {
            resolve_expression(expression, variable_map)?;
        }
        *name = unique_name;
        Ok(())
    }
}

fn resolve_statement(
    statement: &mut parser::Statement,
    variable_map: &mut HashMap<String, MapEntry>,
) -> Result<()> {
    match statement {
        parser::Statement::Return(expression) | parser::Statement::Expression(expression) => {
            resolve_expression(expression, variable_map)
        }
        parser::Statement::If {
            condition,
            then_statement,
            else_statement,
        } => {
            resolve_expression(condition, variable_map)?;
            resolve_statement(&mut **then_statement, variable_map)?;
            if let Some(statement) = else_statement {
                resolve_statement(&mut **statement, variable_map)?;
            }
            Ok(())
        }
        parser::Statement::Goto(_) | parser::Statement::Label(_) | parser::Statement::Null => {
            Ok(())
        }
        parser::Statement::Compound(block) => {
            let mut new_variable_map = copy_variable_map(variable_map);
            resolve_block(block, &mut new_variable_map)
        }
    }
}

fn resolve_expression(
    expression: &mut parser::Expression,
    variable_map: &mut HashMap<String, MapEntry>,
) -> Result<()> {
    match expression {
        parser::Expression::Assignment {
            left_expression,
            right_expression,
            ..
        } => {
            resolve_expression(left_expression, variable_map)?;
            resolve_expression(right_expression, variable_map)?;
        }
        parser::Expression::Var(identifier) => match variable_map.get(identifier) {
            Some(val) => *identifier = val.name.to_string(),
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

        parser::Expression::Postfix { expression, .. } => {
            resolve_expression(expression, variable_map)?
        }
        parser::Expression::Conditional {
            condition,
            true_case,
            false_case,
        } => {
            resolve_expression(condition, variable_map)?;
            resolve_expression(true_case, variable_map)?;
            resolve_expression(false_case, variable_map)?;
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

fn copy_variable_map(variable_map: &HashMap<String, MapEntry>) -> HashMap<String, MapEntry> {
    variable_map
        .iter()
        .map(|(k, v)| {
            (
                k.clone(),
                MapEntry {
                    name: v.name.clone(),
                    from_current_block: false,
                },
            )
        })
        .collect()
}
