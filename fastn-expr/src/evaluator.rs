use fastn_expr::parser::{parse, ExprNode, ParseError};

#[derive(thiserror::Error, Debug)]
pub enum EvalError {
    #[error("Could not parse the expression: {0}")]
    ParseError(#[from] ParseError),
}

pub fn eval(input: &str) -> Result<String, EvalError> {
    let expr = parse(input)?;

    eval_expr(expr)
}

pub fn eval_expr(_expr: ExprNode) -> Result<String, EvalError> {
    todo!()
}
