#[derive(Debug)]
pub enum Value {
    Data(ftd::interpreter::Value),
    Reference(String),
    Formula(Vec<ftd::interpreter::Property>),
}

impl Value {
    pub(crate) fn to_set_property_value(&self) -> fastn_js::SetPropertyValue {
        match self {
            Value::Data(value) => value.to_fastn_js_value(),
            Value::Reference(name) => fastn_js::SetPropertyValue::Reference(name.to_string()),
            Value::Formula(formulas) => {
                fastn_js::SetPropertyValue::Formula(formulas_to_fastn_js_value(formulas))
            }
        }
    }

    pub(crate) fn to_set_property(
        &self,
        kind: fastn_js::PropertyKind,
        element_name: &str,
    ) -> fastn_js::SetProperty {
        fastn_js::SetProperty {
            kind,
            value: self.to_set_property_value(),
            element_name: element_name.to_string(),
        }
    }
}

fn formulas_to_fastn_js_value(properties: &[ftd::interpreter::Property]) -> fastn_js::Formula {
    let mut deps = vec![];
    let mut conditional_values = vec![];
    for property in properties {
        if let Some(reference) = property.value.get_reference_or_clone() {
            deps.push(reference.to_owned());
        }
        conditional_values.push(fastn_js::ConditionalValue {
            condition: property
                .condition
                .as_ref()
                .map(|condition| ftd::html::utils::get_condition_string_(condition, false)),
            expression: property.value.to_fastn_js_value(),
        });
    }

    fastn_js::Formula {
        deps,
        conditional_values,
    }
}

pub(crate) fn get_properties(
    key: &str,
    properties: &[ftd::interpreter::Property],
    arguments: &[ftd::interpreter::Argument],
    // doc_name: &str,
    // line_number: usize
) -> Option<Value> {
    let argument = arguments.iter().find(|v| v.name.eq(key)).unwrap();
    let sources = argument.to_sources();
    let properties = ftd::interpreter::utils::find_properties_by_source(
        sources.as_slice(),
        properties,
        "", // doc_name
        argument,
        0, // line_number
    )
    .unwrap();

    if properties.len() == 1 {
        let property = properties.first().unwrap();
        if property.condition.is_none() {
            match property.value {
                ftd::interpreter::PropertyValue::Value { ref value, .. } => {
                    return Some(Value::Data(value.to_owned()))
                }
                ftd::interpreter::PropertyValue::Reference { ref name, .. } => {
                    return Some(Value::Reference(name.to_owned()))
                }
                _ => todo!(),
            }
        }
    }

    // Todo: For more than one properties
    None
}

impl ftd::interpreter::PropertyValue {
    pub(crate) fn to_fastn_js_value(&self) -> fastn_js::SetPropertyValue {
        match self {
            ftd::interpreter::PropertyValue::Value { ref value, .. } => value.to_fastn_js_value(),
            ftd::interpreter::PropertyValue::Reference { ref name, .. } => {
                fastn_js::SetPropertyValue::Reference(name.to_string())
            }
            _ => todo!(),
        }
    }
}

impl ftd::interpreter::Value {
    pub(crate) fn to_fastn_js_value(&self) -> fastn_js::SetPropertyValue {
        match self {
            ftd::interpreter::Value::String { text } => {
                fastn_js::SetPropertyValue::Value(fastn_js::Value::String(text.to_string()))
            }
            ftd::interpreter::Value::Integer { value } => {
                fastn_js::SetPropertyValue::Value(fastn_js::Value::Integer(*value))
            }
            ftd::interpreter::Value::Decimal { value } => {
                fastn_js::SetPropertyValue::Value(fastn_js::Value::Decimal(*value))
            }
            ftd::interpreter::Value::OrType {
                name,
                variant,
                value,
                ..
            } => {
                let (js_variant, has_value) = ftd_to_js_variant(name, variant);
                if has_value {
                    return fastn_js::SetPropertyValue::Value(fastn_js::Value::OrType {
                        variant: js_variant,
                        value: Some(Box::new(value.to_fastn_js_value())),
                    });
                }
                fastn_js::SetPropertyValue::Value(fastn_js::Value::OrType {
                    variant: js_variant,
                    value: None,
                })
            }
            _ => todo!(),
        }
    }
}

fn ftd_to_js_variant(name: &str, variant: &str) -> (String, bool) {
    // returns (JSVariant, has_value)
    let variant = variant.strip_prefix(format!("{}.", name).as_str()).unwrap();
    match name {
        "ftd#resizing" => {
            let js_variant = resizing_variants(variant);
            (format!("fastn_dom.Resizing.{}", js_variant.0), js_variant.1)
        }
        "ftd#length" => {
            let js_variant = length_variants(variant);
            (format!("fastn_dom.Length.{}", js_variant), true)
        }
        _ => todo!(),
    }
}

// Returns the corresponding js string and has_value
// Todo: Remove has_value flag
fn resizing_variants(name: &str) -> (&'static str, bool) {
    match name {
        "fixed" => ("Fixed", true),
        "fill-container" => ("FillContainer", false),
        _ => todo!(),
    }
}

fn length_variants(name: &str) -> &'static str {
    match name {
        "px" => "Px",
        "em" => "Em",
        "rem" => "Rem",
        "percent" => "Percent",
        "vh" => "Vh",
        "vw" => "Vw",
        "calc" => "Calc",
        _ => todo!(),
    }
}
