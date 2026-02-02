/*
 * scenarios in which we need lvalues
 * unary increment and decrement
 * postfix increment and decrement
 * assignment
 *
 *
 * lvalues that we have rn: unary inrement and decrement
 * variables
 */

use super::{Error, Result};

use super::parser;

pub(super) fn check_lvalue(ast: &parser::Program) -> Result<()> {
    resolve_program(ast)
}

fn resolve_program(program: &parser::Program) -> Result<()> {
    let parser::Program::Program(function) = program;
    resolve_function(function)
}

fn resolve_function(function: &parser::FunctionDefinition) -> Result<()> {
    let parser::FunctionDefinition::Function {
        body: parser::Block::Block(body),
        ..
    } = function;
    body.iter()
        .try_for_each(|block_item| resolve_block_item(block_item))
}

fn resolve_block_item(block_item: &parser::BlockItem) -> Result<()> {
    let parser::BlockItem::S(statement) = block_item else {
        return Ok(());
    };
    resolve_statement(statement)
}

fn resolve_statement(statement: &parser::Statement) -> Result<()> {
    match statement {
        parser::Statement::Return(expression) => resolve_expression(expression),
        parser::Statement::Expression(expression) => resolve_expression(expression),
        parser::Statement::If {
            condition,
            then_statement,
            else_statement,
        } => {
            resolve_expression(condition)?;
            resolve_statement(then_statement)?;
            if let Some(statement) = else_statement {
                resolve_statement(statement)
            } else {
                Ok(())
            }
        }

        parser::Statement::While {
            condition, body, ..
        } => {
            resolve_expression(condition)?;
            resolve_statement(body)
        }
        parser::Statement::DoWhile {
            condition, body, ..
        } => {
            resolve_expression(condition)?;
            resolve_statement(body)
        }
        parser::Statement::For {
            init,
            condition,
            post,
            body,
            ..
        } => {
            resolve_for_init(init)?;
            if let Some(expression) = condition {
                resolve_expression(expression)?;
            }
            if let Some(expression) = post {
                resolve_expression(expression)?;
            }
            resolve_statement(body)
        }

        parser::Statement::Compound(parser::Block::Block(body)) => body
            .iter()
            .try_for_each(|block_item| resolve_block_item(block_item)),
        parser::Statement::Break(_)
        | parser::Statement::Continue(_)
        | parser::Statement::Goto(_)
        | parser::Statement::Label(_)
        | parser::Statement::Null => Ok(()),
    }
}

fn resolve_for_init(for_init: &parser::ForInit) -> Result<()> {
    if let parser::ForInit::InitExp(Some(expression)) = for_init {
        resolve_expression(expression)
    } else {
        Ok(())
    }
}

fn resolve_expression(expression: &parser::Expression) -> Result<()> {
    match expression {
        parser::Expression::Unary {
            unary_operator: parser::UnaryOperator::Increment | parser::UnaryOperator::Decrement,
            expression,
        } => is_lvalue(expression),
        parser::Expression::Assignment {
            left_expression,
            right_expression,
            ..
        } => {
            is_lvalue(left_expression)?;
            resolve_expression(right_expression)
        }
        parser::Expression::Postfix { expression, .. } => is_lvalue(expression),
        parser::Expression::Binary {
            left_expression,
            right_expression,
            ..
        } => {
            resolve_expression(left_expression)?;
            resolve_expression(right_expression)
        }

        parser::Expression::Unary { expression, .. } => resolve_expression(expression),
        parser::Expression::Conditional {
            condition,
            true_case,
            false_case,
        } => {
            resolve_expression(condition)?;
            resolve_expression(true_case)?;
            resolve_expression(false_case)
        }

        parser::Expression::Var(_) | parser::Expression::IntConstant(_) => Ok(()),
    }
}

fn is_lvalue(expression: &parser::Expression) -> Result<()> {
    match expression {
        parser::Expression::Unary {
            unary_operator: parser::UnaryOperator::Increment | parser::UnaryOperator::Decrement,
            ..
        }
        | parser::Expression::Var(_) => Ok(()),
        _ => Err(Error::SemanticError("invalid lvalue")),
    }
}
