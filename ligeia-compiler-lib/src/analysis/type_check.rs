use std::collections::HashMap;

use thiserror::Error;

use crate::parser::ast;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TypeCheckError {
    #[error("Mismatched arguments (`{1}`, `{2}`) for operator `{0}`")]
    MismatchedOperatorArguments(String, ast::Type, ast::Type),
    #[error("The returned type `{1}` does not match declared (`{0}`)")]
    MismatchedReturnType(ast::Type, ast::Type),
}

pub(super) fn type_check(ast: &[ast::SourceFile]) -> Result<(), TypeCheckError> {
    for file in ast {
        for declaration in &file.declarations {
            match &declaration {
                ast::Declaration::Function(function) => type_check_function(function)?,
            }
        }
    }

    Ok(())
}

fn type_check_function(function: &ast::Function) -> Result<(), TypeCheckError> {
    let locals: HashMap<_, _> = function
        .arguments
        .iter()
        .map(|x| (&x.name, &x.r#type))
        .collect();

    match &function.body {
        ast::FunctionBody::Extern(_) => Ok(()),
        ast::FunctionBody::Statements(statements) => {
            for statement in statements {
                match statement {
                    ast::Statement::Expression(expression) => {
                        type_check_expression(expression, &locals)?;
                    }
                    ast::Statement::Return(expression) => {
                        // TODO verify that all code paths return a value
                        let return_type = determine_expression_type(expression, &locals);

                        if return_type != function.return_type {
                            return Err(TypeCheckError::MismatchedReturnType(
                                function.return_type,
                                return_type,
                            ));
                        }
                    }
                }
            }

            Ok(())
        }
    }
}

fn type_check_expression(
    expression: &ast::Expression,
    locals: &HashMap<&ast::Identifier, &ast::Type>,
) -> Result<(), TypeCheckError> {
    match expression {
        ast::Expression::Literal(_) | ast::Expression::VariableReference(_) => Ok(()),
        ast::Expression::Sum(left, right) => {
            // TODO check if the types actually have the operator defined
            let left_type = determine_expression_type(left, locals);
            let right_type = determine_expression_type(right, locals);

            if left_type != right_type {
                return Err(TypeCheckError::MismatchedOperatorArguments(
                    "+".to_string(),
                    left_type,
                    right_type,
                ));
            }

            Ok(())
        }
    }
}

fn determine_expression_type(
    expression: &ast::Expression,
    locals: &HashMap<&ast::Identifier, &ast::Type>,
) -> ast::Type {
    match expression {
        ast::Expression::Literal(literal) => match literal {
            ast::Literal::UnsignedInteger(_) => ast::Type::U64,
        },
        // TODO: return Err(...) if the variable does not exist
        ast::Expression::VariableReference(identifier) => **locals.get(identifier).unwrap(),
        // TODO: handle user-defined operator implementations, where the return type does not have
        // to match the arguments type
        ast::Expression::Sum(left, _right) => determine_expression_type(left, locals),
    }
}
