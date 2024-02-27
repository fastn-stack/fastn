use crate::auth::email_password::{confirmation_link, confirmation_mail_body, generate_key};

pub(crate) async fn create_and_send_confirmation_email(
    email: String,
    conn: &mut fastn_core::db::Conn,
    req_config: &mut fastn_core::RequestConfig,
    next: String,
) -> fastn_core::Result<(String, i64)> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    let key = generate_key(64);
    let now = chrono::Utc::now();

    let query_result: Result<(i64, i64), _> = fastn_core::schema::fastn_user_email::table
        .select((
            fastn_core::schema::fastn_user_email::id,
            fastn_core::schema::fastn_user_email::user_id,
        ))
        .filter(
            fastn_core::schema::fastn_user_email::email
                .eq(fastn_core::utils::citext(email.as_str())),
        )
        .first(conn)
        .await;

    if let Err(e) = query_result {
        tracing::error!("failed to get email_id and user_id from db: {:?}", e);
        return Err(fastn_core::error::Error::generic("Bad request"));
    }

    let (email_id, user_id) = query_result.unwrap();

    // create a non active fastn_auth_session entry for auto login
    let session_id: i64 = diesel::insert_into(fastn_core::schema::fastn_auth_session::table)
        .values((
            fastn_core::schema::fastn_auth_session::user_id.eq(&user_id),
            fastn_core::schema::fastn_auth_session::created_at.eq(&now),
            fastn_core::schema::fastn_auth_session::updated_at.eq(&now),
        ))
        .returning(fastn_core::schema::fastn_auth_session::id)
        .get_result(conn)
        .await?;

    let stored_key: String =
        diesel::insert_into(fastn_core::schema::fastn_email_confirmation::table)
            .values((
                fastn_core::schema::fastn_email_confirmation::email_id.eq(email_id),
                fastn_core::schema::fastn_email_confirmation::session_id.eq(&session_id),
                fastn_core::schema::fastn_email_confirmation::sent_at.eq(&now),
                fastn_core::schema::fastn_email_confirmation::created_at.eq(&now),
                fastn_core::schema::fastn_email_confirmation::key.eq(key),
            ))
            .returning(fastn_core::schema::fastn_email_confirmation::key)
            .get_result(conn)
            .await?;

    let confirmation_link = confirmation_link(&req_config.request, stored_key, next);

    let name: String = fastn_core::schema::fastn_user::table
        .select(fastn_core::schema::fastn_user::name)
        .filter(fastn_core::schema::fastn_user::id.eq(user_id))
        .first(conn)
        .await?;

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
        "{}/{}#confirmation-mail-html",
        req_config.config.package.name, path
    );

    let html: String = main_ftd_doc.get(&html_email_templ).unwrap();

    tracing::info!("confirmation link: {}", &confirmation_link);

    req_config
        .config
        .ds
        .send_email(
            (&name, &email),
            "Verify your email",
            confirmation_mail_body(html, &confirmation_link),
        )
        .await
        .map_err(|e| fastn_core::Error::generic(format!("failed to send email: {e}")))?;

    Ok((confirmation_link, session_id))
}
