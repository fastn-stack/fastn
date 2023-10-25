pub async fn pwd() -> fastn_core::Result<fastn_core::http::Response> {
    if !is_tutor() {
        return Ok(fastn_core::not_found!("this only works in tutor mode"));
    }

    fastn_core::http::api_ok(std::env::current_dir()?.to_string_lossy())
}

pub async fn shutdown() -> fastn_core::Result<fastn_core::http::Response> {
    if !is_tutor() {
        return Ok(fastn_core::not_found!("this only works in tutor mode"));
    }

    println!("/-/shutdown/ called, shutting down");
    std::process::exit(0);
}

pub async fn process(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    if !fastn_core::tutor::is_tutor() {
        return Err(ftd::interpreter::Error::OtherError(
            "tutor process only works in tutor mode".to_string(),
        ));
    }

    let state = match tokio::fs::read(
        std::path::PathBuf::from(dirs::home_dir().unwrap())
            .join(".fastn")
            .join("tutor.json"),
    )
    .await
    {
        Ok(v) => serde_json::from_slice(&v)?,
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => TutorState::default(),
            _ => {
                return Err(ftd::interpreter::Error::OtherError(format!(
                    "tutor error: {}",
                    e.to_string()
                )))
            }
        },
    };

    doc.from_json(&state, &kind, &value)
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
struct TutorState {
    done: Vec<String>,
    current: String,
}

pub fn is_tutor() -> bool {
    // https://github.com/orgs/fastn-stack/discussions/1414
    // with either of these are passed we allow APIs like /-/shutdown/, `/-/start/` etc
    std::env::args().any(|e| e == "tutor" || e == "--tutor")
}
