use crate::compile::parser::{BlockItem, Statement};

use super::parser;
use super::{Error, Result};

enum CaseType {
    Case(i32),
    Default,
}

/// collect cases of all switches, make sure no dupes
pub(super) fn collect_cases(ast: &mut parser::Program) -> Result<()> {
    let parser::Program::Program(function_definition) = ast;
    resolve_function_defintion(function_definition)
}

fn resolve_function_defintion(function_definition: &mut parser::FunctionDefinition) -> Result<()> {
    let parser::FunctionDefinition::Function { body, .. } = function_definition;
    resolve_block(body).map(|_| ())
}

fn resolve_block(block: &mut parser::Block) -> Result<Vec<CaseType>> {
    let parser::Block::Block(block_items) = block;
    Ok(block_items
        .iter_mut()
        .map(|block_item| resolve_block_item(block_item))
        .collect::<Result<Vec<Vec<CaseType>>>>()?
        .into_iter()
        .flatten()
        .collect())
}

fn resolve_block_item(block_item: &mut parser::BlockItem) -> Result<Vec<CaseType>> {
    match block_item {
        BlockItem::S(statement) => resolve_statement(statement),
        BlockItem::D(_) => Ok(Vec::new()),
    }
}

fn resolve_statement(statement: &mut parser::Statement) -> Result<Vec<CaseType>> {
    match statement {
        Statement::Switch {
            body,
            case_expressions,
            default,
            ..
        } => {
            *case_expressions = resolve_statement(body)?.into_iter().try_fold(
                Vec::new(),
                |mut acc, case_type| match case_type {
                    CaseType::Case(val) => {
                        if acc.contains(&val) {
                            Err(Error::SemanticError("multiple of a case value"))
                        } else {
                            acc.push(val);
                            Ok(acc)
                        }
                    }
                    CaseType::Default => {
                        if *default {
                            Err(Error::SemanticError("multiple defaults"))
                        } else {
                            *default = true;
                            Ok(acc)
                        }
                    }
                },
            )?;
            Ok(Vec::new())
        }
        Statement::Case {
            condition, body, ..
        } => {
            let parser::Expression::IntConstant(val) = condition else {
                return Err(Error::SemanticError("case has non constant condition"));
            };
            let mut result = resolve_statement(body)?;
            result.push(CaseType::Case(*val));
            Ok(result)
        }
        Statement::Default { body, .. } => {
            let mut result = resolve_statement(body)?;
            result.push(CaseType::Default);
            Ok(result)
        }

        Statement::If {
            then_statement,
            else_statement,
            ..
        } => {
            let mut res = resolve_statement(then_statement)?;
            if let Some(es) = else_statement {
                res.extend(resolve_statement(es)?.into_iter());
            }
            Ok(res)
        }

        Statement::Compound(block) => resolve_block(block),

        Statement::For { body, .. }
        | Statement::While { body, .. }
        | Statement::DoWhile { body, .. } => resolve_statement(body),

        Statement::Label { body, .. } => resolve_statement(body),

        Statement::Goto(_)
        | Statement::Break(_)
        | Statement::Continue(_)
        | Statement::Return(_)
        | Statement::Expression(_)
        | Statement::Null => Ok(Vec::new()),
    }
}
