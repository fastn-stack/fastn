pub fn validate_strong_password(password: &str) -> Result<(), validator::ValidationError> {
    let mut has_uppercase = false;
    let mut has_lowercase = false;
    let mut has_number = false;
    let mut has_special = false;

    for c in password.chars() {
        if c.is_ascii_uppercase() {
            has_uppercase = true;
        } else if c.is_ascii_lowercase() {
            has_lowercase = true;
        } else if c.is_ascii_digit() {
            has_number = true;
        } else if c.is_ascii_punctuation() {
            has_special = true;
        }
    }

    let mut error = validator::ValidationError::new("password validation error");

    if !has_uppercase {
        error.add_param(
            "uppercase".into(),
            &std::borrow::Cow::from("password must contain at least one uppercase letter"),
        )
    }

    if !has_lowercase {
        error.add_param(
            "lowercase".into(),
            &std::borrow::Cow::from("password must contain at least one lowercase letter"),
        );
    }

    if !has_number {
        error.add_param(
            "number".into(),
            &std::borrow::Cow::from("password must contain at least one number"),
        );
    }

    if !has_special {
        error.add_param(
            "special".into(),
            &std::borrow::Cow::from(
                "password must contain at least one special character (!@#$%^&*()_+{}|:<>?~)",
            ),
        );
    }

    if error.params.is_empty() {
        Ok(())
    } else {
        Err(error)
    }
}
