#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct Value {
    pub value: Option<String>,
    pub properties: Vec<PropertyWithPattern>,
    pub line_number: Option<usize>,
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub struct PropertyWithPattern {
    pub property: ftd::interpreter2::Property,
    pub pattern_with_eval: Option<(String, bool)>,
}

impl PropertyWithPattern {
    fn new(
        property: ftd::interpreter2::Property,
        pattern_with_eval: Option<(String, bool)>,
    ) -> PropertyWithPattern {
        PropertyWithPattern {
            property,
            pattern_with_eval,
        }
    }
}

impl Value {
    pub fn from_string(value: &str) -> Value {
        Value {
            value: Some(value.to_string()),
            properties: vec![],
            line_number: None,
        }
    }

    pub fn from_executor_value<T>(
        value: Option<String>,
        exec_value: ftd::executor::Value<T>,
        pattern_with_eval: Option<(String, bool)>,
        doc_id: &str,
    ) -> Value {
        use itertools::Itertools;

        let properties = if pattern_with_eval.is_some() {
            exec_value
                .properties
                .into_iter()
                .map(|v| PropertyWithPattern::new(v, pattern_with_eval.clone()))
                .collect_vec()
        } else {
            let mut properties = vec![];
            for property in exec_value.properties {
                let pattern = property
                    .value
                    .kind()
                    .pattern(doc_id)
                    .or(pattern_with_eval.clone());
                properties.push(PropertyWithPattern::new(property, pattern));
            }
            properties
        };
        /*if properties.len() == 1 {
            let property = properties.first().unwrap();
            if property.value.is_value() && property.condition.is_none() {
                properties = vec![]
            }
        }*/

        Value {
            value,
            properties,
            line_number: exec_value.line_number,
        }
    }
}

impl ftd::interpreter2::Kind {
    fn pattern(&self, doc_id: &str) -> Option<(String, bool)> {
        match self {
            ftd::interpreter2::Kind::OrType {
                name,
                variant: Some(variant),
            } if name.eq(ftd::interpreter2::FTD_LENGTH) => {
                ftd::executor::Length::pattern_from_variant_str(variant.as_str(), doc_id, 0)
                    .ok()
                    .map(|v| (v.to_string(), false))
            }
            _ => None,
        }
    }
}
