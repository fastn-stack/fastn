#[derive(thiserror::Error, Debug)]
pub enum ExprError {
    #[error("Unexpected token '{token}' at position {token}")]
    UnexpectedToken { token: char, position: usize },
}

#[derive(Debug, PartialEq)]
enum Token {
    Identifier(String),
    Operator(Operator),
    Literal(String),
}

#[derive(Debug, PartialEq)]
enum Operator {
    Or,
}

fn tokenize(input: &str) -> Result<Vec<Token>, ExprError> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();
    let mut in_string_literal = false;
    let mut escaped = false;

    for (i, c) in input.chars().enumerate() {
        if in_string_literal {
            if escaped {
                current_token.push(c);
                escaped = false;
            } else if c == '\\' {
                escaped = true;
                current_token.push(c);
            } else if c == '"' {
                in_string_literal = false;
                current_token.push(c);
                tokens.push(Token::Literal(current_token.clone()));
                current_token.clear();
            } else {
                current_token.push(c);
            }
        } else {
            if c.is_whitespace() {
                if !current_token.is_empty() {
                    tokens.push(get_token(&current_token));
                    current_token.clear();
                }
            } else if (c == '.' && !current_token.is_empty()) || c.is_alphanumeric() {
                current_token.push(c);
            } else if c == '"' {
                in_string_literal = true;
                current_token.push(c);
            } else if !current_token.is_empty() {
                tokens.push(get_token(&current_token));
                current_token.clear();
            } else {
                return Err(ExprError::UnexpectedToken {
                    token: c,
                    position: i,
                });
            }
        }
    }

    if !current_token.is_empty() {
        tokens.push(get_token(&current_token));
    }

    Ok(tokens)
}

fn get_token(token_str: &str) -> Token {
    match token_str {
        "or" => Token::Operator(Operator::Or),
        _ => Token::Identifier(token_str.to_string()),
    }
}

#[test]
fn test_expr() {
    assert_eq!(
        tokenize(r#"env.ENDPOINT or "127.0.0.1:8000""#).unwrap(),
        vec![
            Token::Identifier(String::from("env.ENDPOINT")),
            Token::Operator(Operator::Or),
            Token::Literal(String::from("\"127.0.0.1:8000\""))
        ]
    )
}
