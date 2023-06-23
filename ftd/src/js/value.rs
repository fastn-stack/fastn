pub enum Value {
    Value(ftd::interpreter::Value),
    Reference(String),
    Formula(Vec<ftd::interpreter::Property>),
}

impl Value {
    pub(crate) fn to_set_property_value(&self) -> fastn_js::SetPropertyValue {
        match self {
            Value::Value(value) => fastn_js::SetPropertyValue::Value(value.to_fastn_js_value()),
            Value::Reference(name) => fastn_js::SetPropertyValue::Reference(name.to_string()),
            _ => todo!(),
        }
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
                    return Some(Value::Value(value.to_owned()))
                }
                ftd::interpreter::PropertyValue::Reference { ref name, .. } => {
                    return Some(Value::Reference(name.to_owned()))
                }
                _ => todo!(),
            }
        }
    }
    todo!()
}

impl ftd::interpreter::Value {
    pub(crate) fn to_fastn_js_value(&self) -> fastn_js::Value {
        match self {
            ftd::interpreter::Value::String { text } => fastn_js::Value::String(text.to_string()),
            _ => todo!(),
        }
    }
}
