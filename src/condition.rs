#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Condition {
    pub variable: String,
    pub value: String,
}

impl Condition {
    pub fn is_true(&self, data: &ftd::DataDependenciesMap) -> bool {
        if let Some(ftd::Data { value, .. }) = data.get(self.variable.as_str()) {
            let v = value.replace("\"", "");
            return if self.value.eq("$IsNull$") {
                v.is_empty() || v.eq("null")
            } else if self.value.eq("$IsNotNull$") {
                !v.is_empty() && !v.eq("null")
            } else {
                v == self.value
            };
        }

        true
    }
}
