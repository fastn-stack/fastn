pub enum Action {
    Read,
    Write,
}

pub enum OutputRequested {
    UI,
    Data,
}

async fn serve(
    _config: fastn_core::Config,
    _path: &str,
    _data: serde_json::Value,
) -> fastn_lang::Output {
    todo!()
}
