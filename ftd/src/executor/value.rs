#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct Value<T> {
    pub value: T,
    pub line_number: Option<usize>,
    pub properties: Vec<ftd::interpreter::Property>,
}

impl<T> Value<T> {
    pub fn new(
        value: T,
        line_number: Option<usize>,
        properties: Vec<ftd::interpreter::Property>,
    ) -> Value<T> {
        Value {
            value,
            line_number,
            properties,
        }
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Value<U> {
        Value {
            value: f(self.value),
            line_number: self.line_number,
            properties: self.properties,
        }
    }
}

pub(crate) fn get_value_from_properties_using_key_and_arguments(
    key: &str,
    component_name: &str,
    properties: &[ftd::interpreter::Property],
    arguments: &[ftd::interpreter::Argument],
    doc: &ftd::executor::TDoc,
    line_number: usize,
) -> ftd::executor::Result<ftd::executor::Value<Option<fastn_type::Value>>> {
    get_value_from_properties_using_key_and_arguments_dummy(
        key,
        component_name,
        properties,
        arguments,
        doc,
        line_number,
        false,
        &Default::default(),
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn get_value_from_properties_using_key_and_arguments_dummy(
    key: &str,
    component_name: &str,
    properties: &[ftd::interpreter::Property],
    arguments: &[ftd::interpreter::Argument],
    doc: &ftd::executor::TDoc,
    line_number: usize,
    is_dummy: bool,
    inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
) -> ftd::executor::Result<ftd::executor::Value<Option<fastn_type::Value>>> {
    let argument =
        arguments
            .iter()
            .find(|v| v.name.eq(key))
            .ok_or(ftd::executor::Error::ParseError {
                message: format!("Cannot find `{}` argument", key),
                doc_id: doc.name.to_string(),
                line_number,
            })?;
    let sources = argument.to_sources();

    let ftd::executor::Value {
        line_number: v_line_number,
        properties,
        value,
    } = find_value_by_argument(
        component_name,
        sources.as_slice(),
        properties,
        doc,
        argument,
        arguments,
        line_number,
        is_dummy,
        inherited_variables,
    )?;
    let expected_kind = value.as_ref().map(|v| v.kind());
    if !expected_kind
        .as_ref()
        .map_or(true, |v| v.is_same_as(&argument.kind.kind))
    {
        return ftd::executor::utils::parse_error(
            format!(
                "Expected kind {:?}, found: `{:?}`",
                expected_kind, argument.kind.kind
            ),
            doc.name,
            line_number,
        );
    }

    Ok(ftd::executor::Value::new(value, v_line_number, properties))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn find_value_by_argument(
    component_name: &str,
    source: &[ftd::interpreter::PropertySource],
    properties: &[ftd::interpreter::Property],
    doc: &ftd::executor::TDoc,
    target_argument: &ftd::interpreter::Argument,
    arguments: &[ftd::interpreter::Argument],
    line_number: usize,
    is_dummy: bool,
    inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
) -> ftd::executor::Result<ftd::executor::Value<Option<fastn_type::Value>>> {
    let properties = {
        let new_properties = ftd::interpreter::utils::find_properties_by_source(
            source,
            properties,
            doc.name,
            target_argument,
            line_number,
        )?;

        let mut evaluated_property = vec![];

        for p in new_properties.iter() {
            if let Some(property) = ftd::executor::utils::get_evaluated_property(
                p,
                properties,
                arguments,
                component_name,
                doc.name,
                p.line_number,
            )? {
                evaluated_property.push(property);
            }
        }

        evaluated_property
    };

    let mut value = None;
    let mut line_number = None;
    if !is_dummy {
        for p in properties.iter() {
            if let Some(v) = p.resolve(&doc.itdoc(), inherited_variables)? {
                value = Some(v);
                line_number = Some(p.line_number);
                if p.condition.is_some() {
                    break;
                }
            }
        }
    } else {
        for p in properties.iter() {
            if let Ok(Some(v)) = p.resolve(&doc.itdoc(), inherited_variables) {
                value = Some(v);
                line_number = Some(p.line_number);
                if p.condition.is_some() {
                    break;
                }
            } else if p.condition.is_none() {
                if let Some(v) = p.value.get_reference_or_clone() {
                    value = Some(fastn_type::Value::new_string(format!("{{{}}}", v).as_str()));
                    line_number = Some(p.line_number);
                }
            }
        }
    }

    Ok(ftd::executor::Value::new(value, line_number, properties))
}

pub fn string_list(
    key: &str,
    component_name: &str,
    properties: &[ftd::interpreter::Property],
    arguments: &[ftd::interpreter::Argument],
    doc: &ftd::executor::TDoc,
    line_number: usize,
    inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
) -> ftd::executor::Result<ftd::executor::Value<Vec<String>>> {
    let value = get_value_from_properties_using_key_and_arguments_dummy(
        key,
        component_name,
        properties,
        arguments,
        doc,
        line_number,
        false,
        inherited_variables,
    )?;

    match value.value.and_then(|v| v.inner()) {
        Some(fastn_type::Value::List { data, kind }) if kind.is_string() => {
            let mut values = vec![];
            for d in data {
                values.push(
                    d.resolve(&doc.itdoc(), line_number)?
                        .string(doc.name, line_number)?,
                );
            }
            Ok(ftd::executor::Value::new(
                values,
                value.line_number,
                value.properties,
            ))
        }
        None => Ok(ftd::executor::Value::new(
            vec![],
            value.line_number,
            value.properties,
        )),
        t => ftd::executor::utils::parse_error(
            format!("Expected value of type string list, found: {:?}", t),
            doc.name,
            line_number,
        ),
    }
}

#[allow(dead_code)]
pub fn string(
    key: &str,
    component_name: &str,
    properties: &[ftd::interpreter::Property],
    arguments: &[ftd::interpreter::Argument],
    doc: &ftd::executor::TDoc,
    line_number: usize,
) -> ftd::executor::Result<ftd::executor::Value<String>> {
    let value = get_value_from_properties_using_key_and_arguments(
        key,
        component_name,
        properties,
        arguments,
        doc,
        line_number,
    )?;

    match value.value.and_then(|v| v.inner()) {
        Some(fastn_type::Value::String { text }) => Ok(ftd::executor::Value::new(
            text,
            value.line_number,
            value.properties,
        )),
        t => ftd::executor::utils::parse_error(
            format!("Expected value of type string, found: {:?}", t),
            doc.name,
            line_number,
        ),
    }
}

pub fn record(
    key: &str,
    component_name: &str,
    properties: &[ftd::interpreter::Property],
    arguments: &[ftd::interpreter::Argument],
    doc: &ftd::executor::TDoc,
    line_number: usize,
    rec_name: &str,
) -> ftd::executor::Result<ftd::executor::Value<ftd::Map<ftd::interpreter::PropertyValue>>> {
    let value = get_value_from_properties_using_key_and_arguments(
        key,
        component_name,
        properties,
        arguments,
        doc,
        line_number,
    )?;

    match value.value.and_then(|v| v.inner()) {
        Some(fastn_type::Value::Record { name, fields }) if name.eq(rec_name) => Ok(
            ftd::executor::Value::new(fields, value.line_number, value.properties),
        ),
        t => ftd::executor::utils::parse_error(
            format!(
                "Expected value of type record `{}`, found: {:?}",
                rec_name, t
            ),
            doc.name,
            line_number,
        ),
    }
}

pub fn i64(
    key: &str,
    component_name: &str,
    properties: &[ftd::interpreter::Property],
    arguments: &[ftd::interpreter::Argument],
    doc: &ftd::executor::TDoc,
    line_number: usize,
) -> ftd::executor::Result<ftd::executor::Value<i64>> {
    let value = get_value_from_properties_using_key_and_arguments(
        key,
        component_name,
        properties,
        arguments,
        doc,
        line_number,
    )?;

    match value.value.and_then(|v| v.inner()) {
        Some(fastn_type::Value::Integer { value: v }) => Ok(ftd::executor::Value::new(
            v,
            value.line_number,
            value.properties,
        )),
        t => ftd::executor::utils::parse_error(
            format!("Expected value of type integer, found: {:?}", t),
            doc.name,
            line_number,
        ),
    }
}

pub fn f64(
    key: &str,
    component_name: &str,
    properties: &[ftd::interpreter::Property],
    arguments: &[ftd::interpreter::Argument],
    doc: &ftd::executor::TDoc,
    line_number: usize,
) -> ftd::executor::Result<ftd::executor::Value<f64>> {
    let value = get_value_from_properties_using_key_and_arguments(
        key,
        component_name,
        properties,
        arguments,
        doc,
        line_number,
    )?;

    match value.value.and_then(|v| v.inner()) {
        Some(fastn_type::Value::Decimal { value: v }) => Ok(ftd::executor::Value::new(
            v,
            value.line_number,
            value.properties,
        )),
        t => ftd::executor::utils::parse_error(
            format!("Expected value of type decimal, found: {:?}", t),
            doc.name,
            line_number,
        ),
    }
}

pub fn bool(
    key: &str,
    component_name: &str,
    properties: &[ftd::interpreter::Property],
    arguments: &[ftd::interpreter::Argument],
    doc: &ftd::executor::TDoc,
    line_number: usize,
) -> ftd::executor::Result<ftd::executor::Value<bool>> {
    let value = get_value_from_properties_using_key_and_arguments(
        key,
        component_name,
        properties,
        arguments,
        doc,
        line_number,
    )?;

    match value.value.and_then(|v| v.inner()) {
        Some(fastn_type::Value::Boolean { value: v }) => Ok(ftd::executor::Value::new(
            v,
            value.line_number,
            value.properties,
        )),
        t => ftd::executor::utils::parse_error(
            format!("Expected value of type boolean, found: {:?}", t),
            doc.name,
            line_number,
        ),
    }
}

pub fn bool_with_default(
    key: &str,
    component_name: &str,
    properties: &[ftd::interpreter::Property],
    arguments: &[ftd::interpreter::Argument],
    default: bool,
    doc: &ftd::executor::TDoc,
    line_number: usize,
) -> ftd::executor::Result<ftd::executor::Value<bool>> {
    let value = get_value_from_properties_using_key_and_arguments(
        key,
        component_name,
        properties,
        arguments,
        doc,
        line_number,
    )?;

    match value.value.and_then(|v| v.inner()) {
        Some(fastn_type::Value::Boolean { value: b }) => Ok(ftd::executor::Value::new(
            b,
            value.line_number,
            value.properties,
        )),
        None => Ok(ftd::executor::Value::new(
            default,
            value.line_number,
            value.properties,
        )),
        t => ftd::executor::utils::parse_error(
            format!("Expected value of type optional bool, found: {:?}", t),
            doc.name,
            line_number,
        ),
    }
}

#[allow(dead_code)]
pub fn optional_i64(
    key: &str,
    component_name: &str,
    properties: &[ftd::interpreter::Property],
    arguments: &[ftd::interpreter::Argument],
    doc: &ftd::executor::TDoc,
    line_number: usize,
    inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
) -> ftd::executor::Result<ftd::executor::Value<Option<i64>>> {
    let value = get_value_from_properties_using_key_and_arguments_dummy(
        key,
        component_name,
        properties,
        arguments,
        doc,
        line_number,
        false,
        inherited_variables,
    )?;

    match value.value.and_then(|v| v.inner()) {
        Some(fastn_type::Value::Integer { value: v }) => Ok(ftd::executor::Value::new(
            Some(v),
            value.line_number,
            value.properties,
        )),
        None => Ok(ftd::executor::Value::new(
            None,
            value.line_number,
            value.properties,
        )),
        t => ftd::executor::utils::parse_error(
            format!("Expected value of type optional integer, found: {:?}", t),
            doc.name,
            line_number,
        ),
    }
}

pub fn string_with_default(
    key: &str,
    component_name: &str,
    properties: &[ftd::interpreter::Property],
    arguments: &[ftd::interpreter::Argument],
    default: &str,
    doc: &ftd::executor::TDoc,
    line_number: usize,
) -> ftd::executor::Result<ftd::executor::Value<String>> {
    let value = get_value_from_properties_using_key_and_arguments(
        key,
        component_name,
        properties,
        arguments,
        doc,
        line_number,
    )?;

    match value.value.and_then(|v| v.inner()) {
        Some(fastn_type::Value::String { text }) => Ok(ftd::executor::Value::new(
            text,
            value.line_number,
            value.properties,
        )),
        None => Ok(ftd::executor::Value::new(
            default.to_string(),
            value.line_number,
            value.properties,
        )),
        t => ftd::executor::utils::parse_error(
            format!("Expected value of type optional string, found: {:?}", t),
            doc.name,
            line_number,
        ),
    }
}

pub fn optional_string(
    key: &str,
    component_name: &str,
    properties: &[ftd::interpreter::Property],
    arguments: &[ftd::interpreter::Argument],
    doc: &ftd::executor::TDoc,
    line_number: usize,
) -> ftd::executor::Result<ftd::executor::Value<Option<String>>> {
    let value = get_value_from_properties_using_key_and_arguments(
        key,
        component_name,
        properties,
        arguments,
        doc,
        line_number,
    )?;

    match value.value.and_then(|v| v.inner()) {
        Some(fastn_type::Value::String { text }) => Ok(ftd::executor::Value::new(
            Some(text),
            value.line_number,
            value.properties,
        )),
        None => Ok(ftd::executor::Value::new(
            None,
            value.line_number,
            value.properties,
        )),
        t => ftd::executor::utils::parse_error(
            format!("Expected value of type optional string, found: {:?}", t),
            doc.name,
            line_number,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn dummy_optional_string(
    key: &str,
    component_name: &str,
    properties: &[ftd::interpreter::Property],
    arguments: &[ftd::interpreter::Argument],
    doc: &ftd::executor::TDoc,
    is_dummy: bool,
    line_number: usize,
    inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
) -> ftd::executor::Result<ftd::executor::Value<Option<String>>> {
    let value = get_value_from_properties_using_key_and_arguments_dummy(
        key,
        component_name,
        properties,
        arguments,
        doc,
        line_number,
        is_dummy,
        inherited_variables,
    )?;

    match value.value.and_then(|v| v.inner()) {
        Some(fastn_type::Value::String { text }) => Ok(ftd::executor::Value::new(
            Some(text),
            value.line_number,
            value.properties,
        )),
        None => Ok(ftd::executor::Value::new(
            None,
            value.line_number,
            value.properties,
        )),
        t => ftd::executor::utils::parse_error(
            format!("Expected value of type optional string, found: {:?}", t),
            doc.name,
            line_number,
        ),
    }
}

pub fn optional_bool(
    key: &str,
    component_name: &str,
    properties: &[ftd::interpreter::Property],
    arguments: &[ftd::interpreter::Argument],
    doc: &ftd::executor::TDoc,
    line_number: usize,
    inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
) -> ftd::executor::Result<ftd::executor::Value<Option<bool>>> {
    let value = get_value_from_properties_using_key_and_arguments_dummy(
        key,
        component_name,
        properties,
        arguments,
        doc,
        line_number,
        false,
        inherited_variables,
    )?;

    match value.value.and_then(|v| v.inner()) {
        Some(fastn_type::Value::Boolean { value: v }) => Ok(ftd::executor::Value::new(
            Some(v),
            value.line_number,
            value.properties,
        )),
        None => Ok(ftd::executor::Value::new(
            None,
            value.line_number,
            value.properties,
        )),
        t => ftd::executor::utils::parse_error(
            format!("Expected value of type optional boolean, found: {:?}", t),
            doc.name,
            line_number,
        ),
    }
}

#[allow(dead_code)]
pub fn optional_f64(
    key: &str,
    component_name: &str,
    properties: &[ftd::interpreter::Property],
    arguments: &[ftd::interpreter::Argument],
    doc: &ftd::executor::TDoc,
    line_number: usize,
) -> ftd::executor::Result<ftd::executor::Value<Option<f64>>> {
    let value = get_value_from_properties_using_key_and_arguments(
        key,
        component_name,
        properties,
        arguments,
        doc,
        line_number,
    )?;

    match value.value.and_then(|v| v.inner()) {
        Some(fastn_type::Value::Decimal { value: v }) => Ok(ftd::executor::Value::new(
            Some(v),
            value.line_number,
            value.properties,
        )),
        None => Ok(ftd::executor::Value::new(
            None,
            value.line_number,
            value.properties,
        )),
        t => ftd::executor::utils::parse_error(
            format!("Expected value of type optional decimal, found: {:?}", t),
            doc.name,
            line_number,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn optional_record_inherited(
    key: &str,
    component_name: &str,
    properties: &[ftd::interpreter::Property],
    arguments: &[ftd::interpreter::Argument],
    doc: &ftd::executor::TDoc,
    line_number: usize,
    rec_name: &str,
    inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
) -> ftd::executor::Result<ftd::executor::Value<Option<ftd::Map<ftd::interpreter::PropertyValue>>>>
{
    let value = get_value_from_properties_using_key_and_arguments_dummy(
        key,
        component_name,
        properties,
        arguments,
        doc,
        line_number,
        false,
        inherited_variables,
    )?;

    match value.value.and_then(|v| v.inner()) {
        Some(fastn_type::Value::Record { name, fields }) if name.eq(rec_name) => Ok(
            ftd::executor::Value::new(Some(fields), value.line_number, value.properties),
        ),
        None => Ok(ftd::executor::Value::new(
            None,
            value.line_number,
            value.properties,
        )),
        t => ftd::executor::utils::parse_error(
            format!(
                "Expected value of type record `{}`, found: {:?}",
                rec_name, t
            ),
            doc.name,
            line_number,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn optional_or_type(
    key: &str,
    component_name: &str,
    properties: &[ftd::interpreter::Property],
    arguments: &[ftd::interpreter::Argument],
    doc: &ftd::executor::TDoc,
    line_number: usize,
    rec_name: &str,
    inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
) -> ftd::executor::Result<ftd::executor::Value<Option<(String, ftd::interpreter::PropertyValue)>>>
{
    let value = get_value_from_properties_using_key_and_arguments_dummy(
        key,
        component_name,
        properties,
        arguments,
        doc,
        line_number,
        false,
        inherited_variables,
    )?;

    match value.value.and_then(|v| v.inner()) {
        Some(fastn_type::Value::OrType {
            name,
            value: property_value,
            variant,
            ..
        }) if name.eq(rec_name) => Ok(ftd::executor::Value::new(
            Some((variant, property_value.as_ref().to_owned())),
            value.line_number,
            value.properties,
        )),
        None => Ok(ftd::executor::Value::new(
            None,
            value.line_number,
            value.properties,
        )),
        t => ftd::executor::utils::parse_error(
            format!(
                "Expected value of type or-type `{}`, found: {:?}",
                rec_name, t
            ),
            doc.name,
            line_number,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn optional_or_type_list(
    key: &str,
    component_name: &str,
    properties: &[ftd::interpreter::Property],
    arguments: &[ftd::interpreter::Argument],
    doc: &ftd::executor::TDoc,
    line_number: usize,
    rec_name: &str,
    inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
) -> ftd::executor::Result<ftd::executor::Value<Vec<(String, ftd::interpreter::PropertyValue)>>> {
    let value = get_value_from_properties_using_key_and_arguments_dummy(
        key,
        component_name,
        properties,
        arguments,
        doc,
        line_number,
        false,
        inherited_variables,
    )?;

    match value.value.and_then(|v| v.inner()) {
        Some(fastn_type::Value::List { data, kind }) if kind.is_or_type() => {
            let mut values = vec![];
            for d in data {
                let resolved_value = d.resolve(&doc.itdoc(), line_number)?;
                if let fastn_type::Value::OrType { variant, value, .. } = resolved_value {
                    values.push((variant.clone(), *value));
                }
            }
            Ok(ftd::executor::Value::new(
                values,
                value.line_number,
                value.properties,
            ))
        }
        None => Ok(ftd::executor::Value::new(
            vec![],
            value.line_number,
            value.properties,
        )),
        t => ftd::executor::utils::parse_error(
            format!(
                "Expected value of type or-type `{}`, found: {:?}",
                rec_name, t
            ),
            doc.name,
            line_number,
        ),
    }
}
