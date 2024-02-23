/* ------------------------------------------------------------------------------------------------
// ------------------------------ Mail Entry worker -------------------------------------------
// - Will create mail table entries for mail dispatch worker to work upon
// ------------------------------------------------------------------------------------------------
 */

pub async fn mail_entry_worker(mut req_config: fastn_core::RequestConfig) {
    let pool = fastn_core::db::pool(&req_config.config.ds)
        .await
        .as_ref()
        .map_err(|e| fastn_core::Error::DatabaseError {
            message: format!("Failed to get connection to db. {:?}", e),
        })?;

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        match mail_entry_once(&mut req_config, &pool).await {
            Ok(()) => {}
            Err(e) => tracing::error!("error creating mail entry (worker): {:?}", e),
        }
    }
}

#[tracing::instrument(skip(pool))]
pub async fn mail_entry_once(
    _req_config: &mut fastn_core::RequestConfig,
    pool: &fastn_core::db::PgPool,
) -> fastn_core::Result<()> {
    use ft_db::prelude::*;
    use ft_db::schema::ft_domain;

    tracing::info!("mail dispatch worker initialized");
    let mut conn = pool.get().await?;

    Ok(())
}

/* ------------------------------------------------------------------------------------------------
// ------------------------------ Mail dispatch worker -------------------------------------------
// - This worker will pick job from mail table entries created by mail entry worker
// - and send mail corresponding to those entries
// ------------------------------------------------------------------------------------------------
 */

pub async fn mail_dispatch_worker(mut req_config: fastn_core::RequestConfig) {
    let pool = fastn_core::db::pool(&req_config.config.ds)
        .await
        .as_ref()
        .map_err(|e| fastn_core::Error::DatabaseError {
            message: format!("Failed to get connection to db. {:?}", e),
        })?;

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        match mail_dispatch_once(&mut req_config, &pool).await {
            Ok(()) => {}
            Err(e) => tracing::error!("error dispatching mail (worker): {:?}", e),
        }
    }
}

// todo: fix this function
#[tracing::instrument(skip(pool))]
pub async fn mail_dispatch_once(
    req_config: &mut fastn_core::RequestConfig,
    pool: &fastn_core::db::PgPool,
) -> fastn_core::Result<()> {
    use ft_db::prelude::*;
    use ft_db::schema::ft_domain;

    tracing::info!("mail dispatch worker initialized");
    let mut conn = pool.get().await?;

    let mailer = match fastn_core::mail::Mailer::from_env(ds).await {
        Ok(mailer) => mailer,
        Err(_) => {
            return Err(fastn_core::Error::generic(
                "Creating mailer requires the following environment variables: \
                \tFASTN_SMTP_USERNAME \
                \tFASTN_SMTP_PASSWORD \
                \tFASTN_SMTP_HOST \
                \tFASTN_SMTP_SENDER_EMAIL \
                \tFASTN_SMTP_SENDER_NAME",
            ))
        }
    };

    let mailer = mailer.unwrap();

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

    mailer
        .send_raw(
            req_config
                .config
                .ds
                .env_bool("FASTN_ENABLE_EMAIL", true)
                .await?,
            format!("{} <{}>", name, email)
                .parse::<lettre::message::Mailbox>()
                .unwrap(),
            "Verify your email",
            confirmation_mail_body(html, &confirmation_link),
        )
        .await
        .map_err(|e| fastn_core::Error::generic(format!("failed to send email: {e}")))?;

    Ok(())
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = fastn_core::schema::fastn_mail_request)]
pub struct FastnMailRequest {
    pub user_id: i64,
    pub ekind: String,
    pub priority: String, // todo: make this enum
    pub email: fastn_core::utils::CiString,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub sent_at: Option<chrono::DateTime<chrono::Utc>>,
}
