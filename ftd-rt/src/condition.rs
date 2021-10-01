#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Condition {
    pub variable: String,
    pub value: String,
}

impl Condition {
    pub fn is_true(&self, data: &ftd_rt::Map, locals: &ftd_rt::Map) -> bool {
        match self.variable.strip_prefix('@') {
            Some(val) => {
                if let Some(v) = locals.get(val) {
                    return v == &self.value;
                }
            }
            None => {
                if let Some(v) = data.get(self.variable.as_str()) {
                    return v == &self.value;
                }
            }
        }

        true
    }
}
