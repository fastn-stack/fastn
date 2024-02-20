/// validate password strength using zxcvbn
/// arg: (String, String, String)
/// arg.0: username
/// arg.1: email
/// arg.2: full name
pub fn validate_strong_password(
    password: &str,
    arg: (&str, &str, &str),
) -> Result<(), validator::ValidationError> {
    let entropy = zxcvbn::zxcvbn(password, &[arg.0, arg.1, arg.2]).map_err(|e| match e {
        zxcvbn::ZxcvbnError::BlankPassword => {
            let mut error = validator::ValidationError::new("password validation error");

            error.add_param(
                "password".into(),
                &std::borrow::Cow::from("password is blank"),
            );
            error
        }
        zxcvbn::ZxcvbnError::DurationOutOfRange => {
            let mut error = validator::ValidationError::new("password validation error");

            error.add_param(
                "password".into(),
                &std::borrow::Cow::from("password is too long"),
            );
            error
        }
    })?;

    // from zxcvbn docs:
    // Overall strength score from 0-4. Any score less than 3 should be considered too weak.
    if entropy.score() < 3 {
        let mut error = validator::ValidationError::new("password validation error");

        error.add_param(
            "password".into(),
            &std::borrow::Cow::from("password is too weak"),
        );

        if let Some(feedback) = entropy.feedback() {
            if let Some(warning) = feedback.warning() {
                error.add_param(
                    "warning".into(),
                    &std::borrow::Cow::from(format!("{}", warning)),
                );
            }

            feedback
                .suggestions()
                .iter()
                .enumerate()
                .for_each(|(idx, suggestion)| {
                    error.add_param(
                        format!("suggestion{}", idx).into(),
                        &std::borrow::Cow::from(format!("{}", suggestion)),
                    );
                });
        }

        Err(error)
    } else {
        Ok(())
    }
}

pub fn accept_terms(
    val: &bool,
) -> Result<(), validator::ValidationError> {
    if *val {
        return Ok(());
    }

    let error = validator::ValidationError::new("accept terms validation error");

    Err(error)
}
