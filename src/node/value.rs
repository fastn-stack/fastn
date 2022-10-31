#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct Value {
    pub value: Option<String>,
    pub properties: Vec<ftd::interpreter2::Property>,
    pub pattern: Option<String>,
    pub line_number: Option<usize>,
}

impl Value {
    pub fn from_string(value: &str) -> Value {
        Value {
            value: Some(value.to_string()),
            properties: vec![],
            pattern: None,
            line_number: None,
        }
    }

    pub fn from_executor_value<T>(
        value: Option<String>,
        exec_value: ftd::executor::Value<T>,
        pattern: Option<String>,
    ) -> Value {
        let mut properties = exec_value.properties;
        if properties.len() == 1 {
            let property = properties.first().unwrap();
            if property.value.is_value() && property.condition.is_none() {
                properties = vec![]
            }
        }

        Value {
            value,
            properties,
            pattern,
            line_number: exec_value.line_number,
        }
    }
}
