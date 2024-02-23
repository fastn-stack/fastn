use crate::auth::email_password::{confirmation_link, confirmation_mail_body, generate_key};

// This will create db entries needed to dispatch email
// Mail workers will carry on the actual email sending process
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
                fastn_core::schema::fastn_email_confirmation::sent_at.eq(&now), // todo: change this to none
                fastn_core::schema::fastn_email_confirmation::created_at.eq(&now),
                fastn_core::schema::fastn_email_confirmation::key.eq(key),
            ))
            .returning(fastn_core::schema::fastn_email_confirmation::key)
            .get_result(conn)
            .await?;

    let confirmation_link = confirmation_link(&req_config.request, stored_key, next);

    Ok((confirmation_link, session_id))
}
