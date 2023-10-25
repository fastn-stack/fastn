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

    let state: TutorState =
        match tokio::fs::read(dirs::home_dir().unwrap().join(".fastn").join("tutor.json")).await {
            Ok(v) => serde_json::from_slice(&v)?,
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => TutorStateFS::default(),
                _ => {
                    return Err(ftd::interpreter::Error::OtherError(format!(
                        "tutor error: {}",
                        e
                    )))
                }
            },
        }
        .into();

    doc.from_json(&state, &kind, &value)
}

#[derive(Debug, Default, serde::Deserialize)]
struct TutorStateFS {
    // done: Vec<String>,
    // current: String,
}

#[derive(Debug, serde::Serialize)]
struct TutorState {
    workshops: Vec<Workshop>,
}

impl From<TutorStateFS> for TutorState {
    fn from(_s: TutorStateFS) -> Self {
        todo!()
    }
}

#[derive(Debug, serde::Serialize)]
struct Workshop {
    title: String,
    about: String,
    done: bool,
    current: bool,
    tutorials: Vec<Tutorial>,
}

#[derive(Debug, serde::Serialize)]
struct Tutorial {
    title: String,
    about: String,
    done: bool,
    current: bool,
}

pub fn is_tutor() -> bool {
    // https://github.com/orgs/fastn-stack/discussions/1414
    // with either of these are passed we allow APIs like /-/shutdown/, `/-/start/` etc
    std::env::args().any(|e| e == "tutor" || e == "--tutor")
}
