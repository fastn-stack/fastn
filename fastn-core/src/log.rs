#[derive(Debug, Default)]
pub struct Log {
    pub ekind: String,
    pub okind: String,
    pub outcome: String,
    pub outcome_data: serde_json::Value,
    pub input: serde_json::Value,
}
