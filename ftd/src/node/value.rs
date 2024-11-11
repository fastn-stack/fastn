#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct Value {
    pub value: Option<String>,
    pub properties: Vec<PropertyWithPattern>,
    pub line_number: Option<usize>,
    pub default: Option<String>,
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub struct PropertyWithPattern {
    pub property: ftd::interpreter::Property,
    pub pattern_with_eval: Option<(String, bool)>,
}

impl PropertyWithPattern {
    fn new(
        property: ftd::interpreter::Property,
        pattern_with_eval: Option<(String, bool)>,
    ) -> PropertyWithPattern {
        PropertyWithPattern {
            property,
            pattern_with_eval,
        }
    }
}

impl Value {
    pub fn from_string<T: ToString>(value: T) -> Value {
        Value {
            value: Some(value.to_string()),
            properties: vec![],
            line_number: None,
            default: None,
        }
    }

    pub fn from_executor_value<T>(
        value: Option<String>,
        exec_value: ftd::executor::Value<T>,
        pattern_with_eval: Option<(String, bool)>,
        doc_id: &str,
    ) -> Value {
        Value::from_executor_value_with_default(value, exec_value, pattern_with_eval, doc_id, None)
    }

    pub fn from_executor_value_with_default<T>(
        value: Option<String>,
        exec_value: ftd::executor::Value<T>,
        pattern_with_eval: Option<(String, bool)>,
        doc_id: &str,
        default: Option<String>,
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
                    .or_else(|| pattern_with_eval.clone());
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
            default,
        }
    }
}

impl fastn_type::Kind {
    fn pattern(&self, doc_id: &str) -> Option<(String, bool)> {
        match self {
            fastn_type::Kind::OrType { name, .. } if name.eq(ftd::interpreter::FTD_LENGTH) => None,
            fastn_type::Kind::OrType {
                name,
                variant: Some(variant),
                full_variant,
            } if name.eq(ftd::interpreter::FTD_RESIZING) => {
                ftd::executor::Resizing::get_pattern_from_variant_str(
                    variant,
                    full_variant.as_ref().unwrap_or(variant),
                    doc_id,
                    0,
                )
                .ok()
                .map(|v| (v.0.to_string(), v.1))
            }
            _ => None,
        }
    }
}
