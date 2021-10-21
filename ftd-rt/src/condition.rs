#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Condition {
    pub variable: String,
    pub value: String,
}

impl Condition {
    pub fn is_true(&self, data: &ftd_rt::DataDependenciesMap) -> bool {
        if let Some(ftd_rt::Data { value: v, .. }) = data.get(self.variable.as_str()) {
            return v == &self.value;
        }

        true
    }
}
