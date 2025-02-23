#[derive(thiserror::Error, Debug)]
pub enum InterpolationError {
    #[error("Failed to parse interpolation: {0}")]
    FailedToParse(#[from] fastn_expr::parser::ParseError),
    #[error("Failed to interpolate: {0}")]
    CantInterpolate(String),
}

pub fn get_var_name_and_default(
    key: &str,
) -> Result<(Option<String>, Option<String>), InterpolationError> {
    let result = fastn_expr::parser::parse(key)?;

    match result {
        fastn_expr::parser::ExprNode::Binary(
            boxed_lhs,
            fastn_expr::tokenizer::Operator::Or,
            boxed_rhs,
        ) => {
            let (var_name, default_value) = match (*boxed_lhs, *boxed_rhs) {
                (
                    fastn_expr::parser::ExprNode::Identifier(var_name),
                    fastn_expr::parser::ExprNode::StringLiteral(default_value),
                ) => (Some(var_name.clone()), Some(default_value)),
                (
                    fastn_expr::parser::ExprNode::Identifier(var_name),
                    fastn_expr::parser::ExprNode::Integer(default_value),
                ) => (Some(var_name.clone()), Some(default_value.to_string())),
                (
                    fastn_expr::parser::ExprNode::Identifier(var_name),
                    fastn_expr::parser::ExprNode::Decimal(default_value),
                ) => (Some(var_name.clone()), Some(default_value.to_string())),
                _ => {
                    return Err(InterpolationError::CantInterpolate(
                        "Invalid expression".to_string(),
                    ));
                }
            };

            Ok((var_name, default_value))
        }
        fastn_expr::parser::ExprNode::Identifier(var_name) => Ok((Some(var_name), None)),
        fastn_expr::parser::ExprNode::StringLiteral(value) => Ok((None, Some(value))),
        fastn_expr::parser::ExprNode::Integer(value) => Ok((None, Some(value.to_string()))),
        fastn_expr::parser::ExprNode::Decimal(value) => Ok((None, Some(value.to_string()))),
    }
}
