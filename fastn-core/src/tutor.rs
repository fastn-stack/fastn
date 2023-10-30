pub async fn main(package_name: String) -> fastn_core::Result<()> {
    println!("starting TUTOR mode");
    std::env::set_current_dir(&std::env::current_dir()?.join(".tutor"))?;
    fastn_core::listen(
        "127.0.0.1",
        Some(2000),
        None,
        Some("2023".to_string()),
        vec!["/-/tutor.js".to_string()],
        vec![],
        vec![],
        vec![],
        package_name,
    )
    .await
}
pub async fn pwd() -> fastn_core::Result<fastn_core::http::Response> {
    if !is_tutor() {
        return Ok(fastn_core::not_found!("this only works in tutor mode"));
    }

    fastn_core::http::api_ok(std::env::current_dir()?.to_string_lossy())
}

pub async fn js() -> fastn_core::Result<fastn_core::http::Response> {
    Ok(actix_web::HttpResponse::Ok().body(include_bytes!("../tutor.js").to_vec()))
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

    dbg!("tutor process called");
    let state =
        match tokio::fs::read(dirs::home_dir().unwrap().join(".fastn").join("tutor.json")).await {
            Ok(v) => serde_json::from_slice(&v)?,
            Err(e) => match dbg!(e.kind()) {
                std::io::ErrorKind::NotFound => {
                    println!("not found, using default");
                    TutorState::default()
                }
                _ => {
                    println!("error: {:?}, {:?}", e, e.kind());
                    return Err(e.into());
                }
            },
        };
    dbg!(&state);

    doc.from_json(&state, &kind, &value)
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct TutorState {
    done: Vec<String>,
    current: String,
}

pub fn is_tutor() -> bool {
    // https://github.com/orgs/fastn-stack/discussions/1414
    // with either of these are passed we allow APIs like /-/shutdown/, `/-/start/` etc
    std::env::args().any(|e| e == "tutor" || e == "--tutor")
}
