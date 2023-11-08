pub async fn main() -> fastn_core::Result<()> {
    println!("starting TUTOR mode");
    std::env::set_current_dir(std::env::current_dir()?.join(".tutor"))?;
    fastn_core::listen(
        "127.0.0.1",
        Some(2000),
        None,
        Some("2023".to_string()),
        vec![],
        vec![],
        vec![],
        vec![],
        "the-tutor".to_string(),
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

async fn set_tutorial(t: Option<Tutorial>) -> fastn_core::Result<fastn_core::http::Response> {
    if !is_tutor() {
        return Ok(fastn_core::not_found!("this only works in tutor mode"));
    }

    *CURRENT_TUTORIAL.write().await = t;
    fastn_core::http::api_ok("done")
}

pub async fn start(t: Tutorial) -> fastn_core::Result<fastn_core::http::Response> {
    set_tutorial(Some(t)).await
}

pub async fn stop() -> fastn_core::Result<fastn_core::http::Response> {
    set_tutorial(None).await
}

static CURRENT_TUTORIAL: once_cell::sync::Lazy<async_lock::RwLock<Option<Tutorial>>> =
    once_cell::sync::Lazy::new(|| async_lock::RwLock::new(None));

#[derive(serde::Deserialize)]
pub struct Tutorial {
    id: String,
    data: fastn_core::commands::serve::AppData,
}

pub(crate) async fn config(
    app_data: &fastn_core::commands::serve::AppData,
) -> fastn_core::Result<(fastn_core::Config, String)> {
    let (root, app_data) = match CURRENT_TUTORIAL.read().await.as_ref() {
        Some(context) => (Some(context.id.clone()), context.data.clone()),
        None => (None, app_data.clone()),
    };

    let config = fastn_core::Config::read(root, false)
        .await
        .unwrap()
        .add_edition(app_data.edition)?
        .add_external_js(app_data.external_js)
        .add_inline_js(app_data.inline_js)
        .add_external_css(app_data.external_css)
        .add_inline_css(app_data.inline_css);

    Ok((config, app_data.package_name))
}

/// tutor-data $processor$
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

    let fs_state: TutorStateFS =
        match tokio::fs::read(dirs::home_dir().unwrap().join(".fastn").join("tutor.json")).await {
            Ok(v) => serde_json::from_slice(&v)?,
            Err(e) => match dbg!(e.kind()) {
                std::io::ErrorKind::NotFound => {
                    println!("not found, using default");
                    TutorStateFS::default()
                }
                _ => {
                    println!("error: {:?}, {:?}", e, e.kind());
                    return Err(e.into());
                }
            },
        };

    let state = TutorState {
        done: fs_state.done,
        current: CURRENT_TUTORIAL.read().await.as_ref().map(|t| t.id.clone()),
    };

    doc.from_json(&state, &kind, &value)
}

#[derive(Debug, Default, serde::Serialize)]
struct TutorState {
    done: Vec<String>,
    current: Option<String>,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct TutorStateFS {
    done: Vec<String>,
}

pub fn is_tutor() -> bool {
    // https://github.com/orgs/fastn-stack/discussions/1414
    // with either of these are passed we allow APIs like /-/shutdown/, `/-/start/` etc
    std::env::args().any(|e| e == "tutor")
}
