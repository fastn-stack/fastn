use fastn_grammar::expr::tokenizer::{tokenize, Operator, Token, TokenizerError};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ParseError {
    #[error("UnexpectedEndOfInput")]
    UnexpectedEndOfInput,
    #[error("Unexpected token '{:?}'", _0)]
    UnexpectedToken(Token),
    #[error("Tokenizer Error: {0}")]
    TokenizerError(#[from] TokenizerError),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ExprNode {
    Identifier(String),
    Literal(String),
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
                    Token::Literal(literal) => ExprNode::Literal(literal.to_string()),
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
                Box::new(ExprNode::Literal(String::from("\"127.0.0.1:8000\""))),
                Operator::Or,
                Box::new(ExprNode::Literal(String::from("\"127.0.0.1:7999\""))),
            ))
        )
    );
    assert_eq!(
        parse(r#"env.ENDPOINT or "#).unwrap_err(),
        ParseError::UnexpectedEndOfInput
    );
}
