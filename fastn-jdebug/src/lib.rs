pub trait JDebug {
    fn debug(&self, source: &str) -> serde_json::Value;
}
