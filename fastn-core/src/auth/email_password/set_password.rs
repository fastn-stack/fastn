/// GET | POST /-/auth/forgot-password/
/// POST forgot_password_request: send email with a link containing a key to reset password
/// for unauthenticated users
async fn forgot_password_request(
    req: &fastn_core::http::Request,
    req_config: &mut fastn_core::RequestConfig,
    db_pool: &fastn_core::db::PgPool,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    if req.method() == "GET" {
        let main = fastn_core::Document {
            package_name: req_config.config.package.name.clone(),
            id: "/-/password-reset-request-sent".to_string(),
            content: email_confirmation_sent_ftd().to_string(),
            parent_path: fastn_ds::Path::new("/"),
        };

        let resp = fastn_core::package::package_doc::read_ftd(req_config, &main, "/", false, false)
            .await?;

        return Ok(resp.into());
    }

    if req.method() != "POST" {
        return Ok(fastn_core::not_found!("invalid route"));
    }

    #[derive(serde::Deserialize, Debug)]
    struct Payload {
        #[serde(rename = "username")]
        email_or_username: String,
    }

    let payload = req.json::<Payload>();

    if let Err(e) = payload {
        return fastn_core::http::user_err(
            vec![
                ("payload".into(), vec![format!("invalid payload: {:?}", e)]),
                (
                    "username".into(),
                    vec!["username/email is required".to_string()],
                ),
            ],
            fastn_core::http::StatusCode::OK,
        );
    }

    let payload = payload.unwrap();

    if payload.email_or_username.is_empty() {
        return fastn_core::http::user_err(
            vec![(
                "username".into(),
                vec!["username/email is required".to_string()],
            )],
            fastn_core::http::StatusCode::OK,
        );
    }

    let mut conn = db_pool
        .get()
        .await
        .map_err(|e| fastn_core::Error::DatabaseError {
            message: format!("Failed to get connection to db. {:?}", e),
        })?;

    let query = fastn_core::schema::fastn_user::table
        .inner_join(fastn_core::schema::fastn_user_email::table)
        .filter(fastn_core::schema::fastn_user::username.eq(&payload.email_or_username))
        .or_filter(
            fastn_core::schema::fastn_user_email::email
                .eq(fastn_core::utils::citext(&payload.email_or_username)),
        )
        .select((
            fastn_core::auth::FastnUser::as_select(),
            fastn_core::schema::fastn_user_email::email,
        ));

    let user: Option<(fastn_core::auth::FastnUser, fastn_core::utils::CiString)> =
        query.first(&mut conn).await.optional()?;

    if user.is_none() {
        return fastn_core::http::user_err(
            vec![(
                "username".into(),
                vec!["invalid email/username".to_string()],
            )],
            fastn_core::http::StatusCode::OK,
        );
    }

    let (user, email) = user.expect("expected user to be Some");

    let key = generate_key(64);

    diesel::insert_into(fastn_core::schema::fastn_password_reset::table)
        .values((
            fastn_core::schema::fastn_password_reset::user_id.eq(&user.id),
            fastn_core::schema::fastn_password_reset::key.eq(&key),
            fastn_core::schema::fastn_password_reset::sent_at.eq(chrono::offset::Utc::now()),
        ))
        .execute(&mut conn)
        .await?;

    let mailer = get_mailer(&req_config.config.ds).await?;

    let reset_link = format!(
        "{}://{}/-/auth/reset-password/?code={key}",
        req.connection_info.scheme(),
        req.connection_info.host(),
        key = key,
    );

    // To use auth. The package has to have auto import with alias `auth` setup
    let path = req_config
        .config
        .package
        .eval_auto_import("auth")
        .unwrap()
        .to_owned();

    let path = path
        .strip_prefix(format!("{}/", req_config.config.package.name).as_str())
        .unwrap();

    let content = req_config
        .config
        .ds
        .read_to_string(&fastn_ds::Path::new(format!("{}.ftd", path)))
        .await?;

    let auth_doc = fastn_core::Document {
        package_name: req_config.config.package.name.clone(),
        id: path.to_string(),
        content,
        parent_path: fastn_ds::Path::new("/"),
    };

    let main_ftd_doc = fastn_core::doc::interpret_helper(
        auth_doc.id_with_package().as_str(),
        auth_doc.content.as_str(),
        req_config,
        "/",
        false,
        0,
    )
    .await?;

    let html_email_templ = format!(
        "{}/{}#reset-password-request-mail-html",
        req_config.config.package.name, path
    );

    let html: String = main_ftd_doc.get(&html_email_templ).unwrap();
    let html = html.replace("{{link}}", &reset_link);

    tracing::info!("confirmation link: {}", &reset_link);

    mailer
        .send_raw(
            format!("{} <{}>", user.name, email.0)
                .parse::<lettre::message::Mailbox>()
                .unwrap(),
            "Reset your password",
            html,
        )
        .await
        .map_err(|e| fastn_core::Error::generic(format!("failed to send email: {e}")))?;

    let resp_body = serde_json::json!({
        "success": true,
        "redirect": redirect_url_from_next(req, "/-/auth/forgot-password/".to_string()),
    });

    let mut resp = actix_web::HttpResponse::Ok();

    if req_config.config.test_command_running {
        resp.insert_header(("X-Fastn-Test", "true"))
            .insert_header(("X-Fastn-Test-Email-Confirmation-Link", reset_link));
    }

    Ok(resp.json(resp_body))
}

/// GET | POST /-/auth/reset-password/
/// reset_password_request -> authenticated user -> construct url to set_password_with_key -> forward them to set_password
/// supposed to be used when the user is already authenticated and wants to change their password
async fn reset_password_request(
    req: &fastn_core::http::Request,
    req_config: &mut fastn_core::RequestConfig,
    db_pool: &fastn_core::db::PgPool,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    // check if I'm authenticated -> I forward to set_password with ?code
    todo!("reset_password_request");
}

// both forgot_password_request and reset_password_request will set some secure cookie that'll contain the fastn_target_user_id
// use sameSite: strict cookies for state

// set_password: will read the cookie and set the password
async fn set_password(
    req: &fastn_core::http::Request,
    req_config: &mut fastn_core::RequestConfig,
    db_pool: &fastn_core::db::PgPool,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    todo!("set_password");
}
