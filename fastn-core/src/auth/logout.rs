// route: /-/auth/logout/
pub async fn logout(
    req: &fastn_core::http::Request,
    ds: &fastn_ds::DocumentStore,
    db_pool: &fastn_core::db::PgPool,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    // [INFO] logging: logout
    req.log(
        "logout",
        fastn_core::log::OutcomeKind::Info,
        file!(),
        line!(),
    );

    if let Some(session_data) = req.cookie(fastn_core::auth::SESSION_COOKIE_NAME) {
        let session_data = match fastn_core::auth::utils::decrypt(ds, &session_data).await {
            Ok(data) => data,
            Err(e) => {
                // [ERROR] logging (server-error: CookieError)
                let err_message = format!("Failed to decrypt session cookie. {:?}", &e);
                req.log(
                    "logout",
                    fastn_core::log::ServerErrorOutcome::CookieError {
                        message: err_message,
                    }
                    .into_kind(),
                    file!(),
                    line!(),
                );
                String::default()
            }
        };

        #[derive(serde::Deserialize)]
        struct SessionData {
            session_id: i64,
        }

        if let Ok(data) = serde_json::from_str::<SessionData>(session_data.as_str()) {
            let session_id = data.session_id;

            let mut conn = match db_pool.get().await {
                Ok(conn) => conn,
                Err(e) => {
                    // [ERROR] logging (server-error: PoolError)
                    let err_message = format!("Failed to get connection to db. {:?}", &e);
                    req.log(
                        "logout",
                        fastn_core::log::ServerErrorOutcome::PoolError {
                            message: err_message.clone(),
                        }
                        .into_kind(),
                        file!(),
                        line!(),
                    );
                    return Err(fastn_core::Error::DatabaseError {
                        message: err_message,
                    });
                }
            };

            let affected = match diesel::delete(fastn_core::schema::fastn_auth_session::table)
                .filter(fastn_core::schema::fastn_auth_session::id.eq(&session_id))
                .execute(&mut conn)
                .await
            {
                Ok(affected) => affected,
                Err(e) => {
                    // [ERROR] logging (server-error: DatabaseQueryError)
                    let err_message = format!("{:?}", &e);
                    req.log(
                        "logout",
                        fastn_core::log::ServerErrorOutcome::DatabaseQueryError {
                            message: err_message,
                        }
                        .into_kind(),
                        file!(),
                        line!(),
                    );
                    return Err(e.into());
                }
            };

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
