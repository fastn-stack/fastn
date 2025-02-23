use fastn_expr::tokenizer::{Operator, Token, TokenizerError, tokenize};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ParseError {
    #[error("Unexpected end of input while parsing expression")]
    UnexpectedEndOfInput,
    #[error("Unexpected token '{:?}'", _0)]
    UnexpectedToken(Token),
    #[error("Tokenizer Error: {0}")]
    TokenizerError(#[from] TokenizerError),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ExprNode {
    Identifier(String),
    StringLiteral(String),
    Integer(i64),
    Decimal(f64),
    Binary(Box<ExprNode>, Operator, Box<ExprNode>),
}

#[derive(Debug)]
pub enum State {
    InMain,
    InBinary(Box<ExprNode>, Operator),
}

pub fn parse(input: &str) -> Result<ExprNode, ParseError> {
    let tokens = tokenize(input)?;
    let mut tokens_iter = tokens.iter().peekable();

    parse_expr(&mut tokens_iter)
}

pub fn parse_expr(
    tokens: &mut std::iter::Peekable<std::slice::Iter<'_, Token>>,
) -> Result<ExprNode, ParseError> {
    let mut state = State::InMain;

    while let Some(token) = tokens.next() {
        match state {
            State::InMain => {
                let left_expr = match token {
                    Token::Identifier(identifier) => ExprNode::Identifier(identifier.to_string()),
                    Token::StringLiteral(value) => ExprNode::StringLiteral(value.to_string()),
                    Token::Integer(value) => ExprNode::Integer(*value),
                    Token::Decimal(value) => ExprNode::Decimal(*value),
                    _ => return Err(ParseError::UnexpectedToken(token.clone())),
                };

                if let Some(Token::Operator(op)) = tokens.peek() {
                    state = State::InBinary(Box::new(left_expr), op.clone());
                    continue;
                }

                return Ok(left_expr);
            }
            State::InBinary(left, op) => {
                let right = parse_expr(tokens)?;
                return Ok(ExprNode::Binary(left, op, Box::new(right)));
            }
        }
    }

    Err(ParseError::UnexpectedEndOfInput)
}

#[test]
fn test_parser() {
    assert_eq!(
        parse(r#"env.ENDPOINT or "127.0.0.1:8000" or "127.0.0.1:7999""#).unwrap(),
        ExprNode::Binary(
            Box::new(ExprNode::Identifier(String::from("env.ENDPOINT"))),
            Operator::Or,
            Box::new(ExprNode::Binary(
                Box::new(ExprNode::StringLiteral(String::from("127.0.0.1:8000"))),
                Operator::Or,
                Box::new(ExprNode::StringLiteral(String::from("127.0.0.1:7999"))),
            ))
        )
    );
    assert_eq!(
        parse(r#"env.ENDPOINT or "#).unwrap_err(),
        ParseError::UnexpectedEndOfInput
    );
}
