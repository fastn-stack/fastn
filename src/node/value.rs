#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct Value {
    pub value: Option<String>,
    pub properties: Vec<PropertyWithPattern>,
    pub line_number: Option<usize>,
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub struct PropertyWithPattern {
    pub property: ftd::interpreter2::Property,
    pub pattern: Option<String>,
}

impl PropertyWithPattern {
    fn new(property: ftd::interpreter2::Property, pattern: Option<String>) -> PropertyWithPattern {
        PropertyWithPattern { property, pattern }
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
        pattern: Option<String>,
        doc_id: &str,
    ) -> Value {
        use itertools::Itertools;

        let properties = if pattern.is_some() {
            exec_value
                .properties
                .into_iter()
                .map(|v| PropertyWithPattern::new(v, pattern.clone()))
                .collect_vec()
        } else {
            let mut properties = vec![];
            for property in exec_value.properties {
                let mut pattern = pattern.clone();
                match property.value.kind() {
                    ftd::interpreter2::Kind::OrType {
                        name,
                        variant: Some(variant),
                    } if name.eq(ftd::interpreter2::FTD_LENGTH) => {
                        pattern = ftd::executor::Length::pattern_from_variant_str(
                            variant.as_str(),
                            doc_id,
                            0,
                        )
                        .ok()
                        .map(ToString::to_string);
                    }
                    _ => {}
                }
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
