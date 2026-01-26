use super::{Error, Result};

use super::parser;

//stupid c17 language makes it such that statements must come after all labels not declarations -
//we dont change this ever - we can only ever fail at this step - so no need to take a mut ref
pub(super) fn statement_after_labels_resolution(ast: &parser::Program) -> Result<()> {
    resolve_program(ast)
}

fn resolve_program(program: &parser::Program) -> Result<()> {
    let parser::Program::Program(function_definition) = program;
    resolve_function(function_definition)
}

fn resolve_function(function: &parser::FunctionDefinition) -> Result<()> {
    let parser::FunctionDefinition::Function { body, .. } = function;
    for window in body.windows(2) {
        if let [
            parser::BlockItem::S(parser::Statement::Label(_)),
            parser::BlockItem::D(_),
        ] = window
        {
            return Err(Error::SemanticError("must have statement after goto label"));
        }
    }
    if body.len() > 0
        && let parser::BlockItem::S(parser::Statement::Label(_)) = body[body.len() - 1]
    {
        return Err(Error::SemanticError("must have statement after goto label"));
    }
    Ok(())
}
