/// returns details of the logged in user
pub async fn process(
    value: ftd_ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    req_config: &fastn_core::RequestConfig,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    match ud(
        &req_config.config.ds,
        req_config.config.get_db_url().await.as_str(),
        req_config
            .request
            .cookie(fastn_core::http::SESSION_COOKIE_NAME),
    )
    .await
    {
        Ok(ud) => doc.from_json(&ud, &kind, &value),
        Err(e) => ftd::interpreter::utils::e2(
            format!("failed to get user data: {e:?}"),
            doc.name,
            value.line_number(),
        ),
    }
}

#[derive(thiserror::Error, Debug)]
pub enum UserDataError {
    #[error("multiple rows found: {0} {1}")]
    MultipleRowsFound(String, usize),
    #[error("serde error: {0}: {1}")]
    SerdeError(String, serde_json::Error),
    #[error("sql error: {0}")]
    SqlError(#[from] fastn_utils::SqlError),
}

pub async fn ud(
    ds: &fastn_ds::DocumentStore,
    db_url: &str,
    sid: Option<String>,
) -> Result<Option<ft_sys_shared::UserData>, UserDataError> {
    if let Ok(v) = ds.env("DEBUG_LOGGED_IN").await {
        let mut v = v.splitn(4, ' ');
        return Ok(Some(ft_sys_shared::UserData {
            id: v.next().unwrap().parse().unwrap(),
            identity: v.next().unwrap_or_default().to_string(),
            name: v.next().map(|v| v.to_string()).unwrap_or_default(),
            email: v.next().map(|v| v.to_string()).unwrap_or_default(),
            verified_email: true,
        }));
    }

    let sid = match sid {
        Some(v) => v,
        None => return Ok(None),
    };

    let mut rows = ds.sql_query(
        db_url,
        r#"
            SELECT
                fastn_user.id as id,
                fastn_user.identity as identity,
                fastn_user.name as name,
                json_extract(fastn_user.data, '$.email.emails[0]') as email,
                json_array_length(fastn_user.data, '$.email.verified_emails') as verified_email_count
            FROM fastn_user
            JOIN fastn_session
            WHERE
                fastn_session.id = $1
                AND fastn_user.id = fastn_session.uid
            "#,
        vec![sid.as_str().into()],
    ).await?;

    let mut row = match rows.len() {
        1 => rows.pop().unwrap(),
        0 => return Ok(None),
        n => return Err(UserDataError::MultipleRowsFound(sid, n)),
    };

    Ok(Some(ft_sys_shared::UserData {
        verified_email: serde_json::from_value::<i32>(row.pop().unwrap())
            .map_err(|e| UserDataError::SerdeError(sid.clone(), e))?
            > 0,
        email: serde_json::from_value(row.pop().unwrap())
            .map_err(|e| UserDataError::SerdeError(sid.clone(), e))?,
        name: serde_json::from_value(row.pop().unwrap())
            .map_err(|e| UserDataError::SerdeError(sid.clone(), e))?,
        identity: serde_json::from_value(row.pop().unwrap())
            .map_err(|e| UserDataError::SerdeError(sid.clone(), e))?,
        id: serde_json::from_value(row.pop().unwrap())
            .map_err(|e| UserDataError::SerdeError(sid, e))?,
    }))
}
