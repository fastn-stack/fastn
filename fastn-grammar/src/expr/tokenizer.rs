#[derive(thiserror::Error, Debug, PartialEq)]
pub enum TokenizerError {
    #[error("Unexpected token '{token}' at position {token}")]
    UnexpectedToken { token: char, position: usize },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Identifier(String),
    Operator(Operator),
    Literal(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operator {
    Or,
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, TokenizerError> {
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
        } else if c.is_whitespace() {
            if !current_token.is_empty() {
                tokens.push(get_token(&current_token));
                current_token.clear();
            }
        } else if (c == '.' && !current_token.is_empty()) || c.is_alphanumeric() {
            current_token.push(c);
        } else if c == '-' && current_token.is_empty() {
            current_token.push(c);
        } else if c == '"' {
            in_string_literal = true;
            current_token.push(c);
        } else if !current_token.is_empty() {
            tokens.push(get_token(&current_token));
            current_token.clear();
        } else {
            return Err(TokenizerError::UnexpectedToken {
                token: c,
                position: i,
            });
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
        _ => {
            if token_str.parse::<f64>().is_ok() {
                Token::Literal(token_str.to_string())
            } else {
                Token::Identifier(token_str.to_string())
            }
        }
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
    );
    assert_eq!(
        tokenize(r#"env.ENDPOINT or "or 127.0.0.1:8000""#).unwrap(),
        vec![
            Token::Identifier(String::from("env.ENDPOINT")),
            Token::Operator(Operator::Or),
            Token::Literal(String::from("\"or 127.0.0.1:8000\""))
        ]
    );
    assert_eq!(
        tokenize(r#"-100"#).unwrap(),
        vec![
            Token::Literal(String::from("-100"))
        ]
    );
}
