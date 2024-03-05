// route: /-/auth/logout/
pub async fn logout(
    req: &fastn_core::http::Request,
    ds: &fastn_ds::DocumentStore,
    db_pool: &fastn_core::db::PgPool,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    req.log(
        fastn_core::log::EventKind::Auth(fastn_core::log::AuthEvent::Logout),
        fastn_core::log::OutcomeKind::Info,
        file!(),
        line!(),
    );

    if let Some(session_data) = req.cookie(fastn_core::auth::SESSION_COOKIE_NAME) {
        let session_data = fastn_core::auth::utils::decrypt(ds, &session_data)
            .await
            .unwrap_or_default();

        #[derive(serde::Deserialize)]
        struct SessionData {
            session_id: i64,
        }

        if let Ok(data) = serde_json::from_str::<SessionData>(session_data.as_str()) {
            let session_id = data.session_id;

            let mut conn = db_pool
                .get()
                .await
                .map_err(|e| fastn_core::Error::DatabaseError {
                    message: format!("Failed to get connection to db. {:?}", e),
                })?;

            let affected = diesel::delete(fastn_core::schema::fastn_auth_session::table)
                .filter(fastn_core::schema::fastn_auth_session::id.eq(&session_id))
                .execute(&mut conn)
                .await?;

            tracing::info!("session destroyed for {session_id}. Rows affected {affected}.");
        }
    }

    Ok(actix_web::HttpResponse::Found()
        .cookie(
            actix_web::cookie::Cookie::build(fastn_core::auth::SESSION_COOKIE_NAME, "")
                .domain(fastn_core::auth::utils::domain(req.connection_info.host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .append_header((actix_web::http::header::LOCATION, next))
        .finish())
}
