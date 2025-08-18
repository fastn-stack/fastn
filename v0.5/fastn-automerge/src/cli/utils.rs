pub fn get_actor_id() -> String {
    // Try environment variable first
    if let Ok(actor_id) = std::env::var("FASTN_AUTOMERGE_ACTOR_ID") {
        return actor_id;
    }

    // Generate a simple default actor ID
    format!("cli-user-{}", 1)
}

pub fn open_db(db_path: &str) -> fastn_automerge::Result<fastn_automerge::Db> {
    let actor_id = get_actor_id();
    let path = std::path::Path::new(db_path);
    fastn_automerge::Db::open_with_actor(path, actor_id)
}

pub fn json_error(msg: String) -> Box<fastn_automerge::Error> {
    Box::new(fastn_automerge::Error::Database(
        rusqlite::Error::InvalidColumnType(0, msg, rusqlite::types::Type::Text),
    ))
}

pub fn read_json_file(file_path: &str) -> fastn_automerge::Result<String> {
    std::fs::read_to_string(file_path)
        .map_err(|e| json_error(format!("Failed to read file {file_path}: {e}")))
}

pub fn parse_json(json_str: &str) -> fastn_automerge::Result<serde_json::Value> {
    serde_json::from_str(json_str).map_err(|e| json_error(format!("JSON parse error: {e}")))
}

pub fn confirm_action(message: &str) -> bool {
    print!("{message} (y/N): ");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    input.trim().to_lowercase().starts_with('y')
}
