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
                    fastn_expr::parser::ExprNode::Literal(default_value),
                ) => (
                    Some(var_name.clone()),
                    Some(trim_quotes(default_value.as_str())),
                ),
                (fastn_expr::parser::ExprNode::Literal(value), _) => {
                    return Ok((None, Some(trim_quotes(value.as_str()))))
                }
                _ => {
                    return Err(InterpolationError::CantInterpolate(
                        "Invalid expression".to_string(),
                    ))
                }
            };

            Ok((var_name, default_value))
        }
        fastn_expr::parser::ExprNode::Identifier(var_name) => Ok((Some(var_name), None)),
        fastn_expr::parser::ExprNode::Literal(value) => {
            Ok((None, Some(trim_quotes(value.as_str()))))
        }
    }
}

fn trim_quotes(s: &str) -> String {
    s.trim_matches('"').to_string()
}
