#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Condition {
    pub variable: String,
    pub value: serde_json::Value,
}

impl Condition {
    pub fn is_true(&self, data: &ftd::DataDependenciesMap) -> bool {
        if let Some(ftd::Data { value, .. }) = data.get(self.variable.as_str()) {
            let v = value.to_string().replace('\"', "");
            return if self.value.eq("$IsNull$") {
                v.is_empty() || value.eq(&serde_json::Value::Null)
            } else if self.value.eq("$IsNotNull$") {
                !v.is_empty() && !value.eq(&serde_json::Value::Null)
            } else {
                self.value.eq(value)
            };
        }

        true
    }
}
