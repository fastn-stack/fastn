/// WARNING: This generates a DUMMY entity ID for CLI testing only!
/// Real applications should use actual entity ID52 values.
/// This function exists only for CLI convenience and should NOT be used in production code.
#[track_caller]
pub fn get_dummy_cli_entity_id() -> String {
    // Try environment variable first (for testing)
    if let Ok(entity_id) = std::env::var("FASTN_AUTOMERGE_ENTITY_ID") {
        return entity_id;
    }

    // Generate a dummy entity ID52 for CLI testing (must be exactly 52 chars)
    "clitempdum000000000000000000000000000000000000000000".to_string()
}

pub fn read_json_file(file_path: &str) -> eyre::Result<String> {
    std::fs::read_to_string(file_path)
        .map_err(|e| eyre::eyre!("Failed to read file {file_path}: {e}"))
}

pub fn parse_json(json_str: &str) -> eyre::Result<serde_json::Value> {
    serde_json::from_str(json_str).map_err(|e| eyre::eyre!("JSON parse error: {e}"))
}

pub fn confirm_action(message: &str) -> bool {
    print!("{message} (y/N): ");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    input.trim().to_lowercase().starts_with('y')
}
