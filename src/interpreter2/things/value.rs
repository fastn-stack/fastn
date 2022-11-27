#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum PropertyValue {
    Value {
        value: ftd::interpreter2::Value,
        is_mutable: bool,
        line_number: usize,
    },
    Reference {
        name: String,
        kind: ftd::interpreter2::KindData,
        source: ftd::interpreter2::PropertyValueSource,
        is_mutable: bool,
        line_number: usize,
    },
    Clone {
        name: String,
        kind: ftd::interpreter2::KindData,
        source: ftd::interpreter2::PropertyValueSource,
        is_mutable: bool,
        line_number: usize,
    },
    FunctionCall(ftd::interpreter2::FunctionCall),
}

impl PropertyValue {
    pub(crate) fn is_mutable(&self) -> bool {
        match self {
            PropertyValue::Value { is_mutable, .. }
            | PropertyValue::Reference { is_mutable, .. }
            | PropertyValue::Clone { is_mutable, .. }
            | PropertyValue::FunctionCall(ftd::interpreter2::FunctionCall { is_mutable, .. }) => {
                *is_mutable
            }
        }
    }

    pub(crate) fn is_static(&self, doc: &ftd::interpreter2::TDoc) -> bool {
        match self {
            PropertyValue::Clone { .. } => true,
            PropertyValue::Reference { is_mutable, .. } if *is_mutable => true,
            PropertyValue::Reference {
                name, line_number, ..
            } => doc
                .get_variable(name, *line_number)
                .map(|v| v.is_static())
                .unwrap_or(true),
            PropertyValue::Value { value, .. } => value.is_static(doc),
            PropertyValue::FunctionCall(f) => {
                let mut is_static = true;
                for d in f.values.values() {
                    if !d.is_static(doc) {
                        is_static = false;
                        break;
                    }
                }
                is_static
            }
        }
    }

    pub(crate) fn line_number(&self) -> usize {
        match self {
            PropertyValue::Value { line_number, .. }
            | PropertyValue::Reference { line_number, .. }
            | PropertyValue::Clone { line_number, .. }
            | PropertyValue::FunctionCall(ftd::interpreter2::FunctionCall {
                line_number, ..
            }) => *line_number,
        }
    }

    pub(crate) fn resolve(
        self,
        doc: &ftd::interpreter2::TDoc,
        line_number: usize,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::Value> {
        match self {
            ftd::interpreter2::PropertyValue::Value { value, .. } => Ok(value),
            ftd::interpreter2::PropertyValue::Reference { name, kind, .. }
            | ftd::interpreter2::PropertyValue::Clone { name, kind, .. } => {
                doc.resolve(name.as_str(), &kind, line_number)
            }
            ftd::interpreter2::PropertyValue::FunctionCall(ftd::interpreter2::FunctionCall {
                name,
                kind,
                values,
                line_number,
                ..
            }) => {
                let function = doc.get_function(name.as_str(), line_number)?;
                function.resolve(&kind, &values, doc, line_number)?.ok_or(
                    ftd::interpreter2::Error::ParseError {
                        message: format!(
                            "Expected return value of type {:?} for function {}",
                            kind, name
                        ),
                        doc_id: doc.name.to_string(),
                        line_number,
                    },
                )
            }
        }
    }

    pub fn is_value(&self) -> bool {
        matches!(self, ftd::interpreter2::PropertyValue::Value { .. })
    }

    pub fn into_property(
        &self,
        source: ftd::interpreter2::PropertySource,
    ) -> ftd::interpreter2::Property {
        ftd::interpreter2::Property {
            value: self.clone(),
            source,
            condition: None,
            line_number: self.line_number(),
        }
    }

    pub(crate) fn value(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter2::Result<&ftd::interpreter2::Value> {
        match self {
            ftd::interpreter2::PropertyValue::Value { value, .. } => Ok(value),
            t => ftd::interpreter2::utils::e2(
                format!("Expected value found `{:?}`", t).as_str(),
                doc_id,
                line_number,
            ),
        }
    }

    pub(crate) fn reference_name(&self) -> Option<&String> {
        match self {
            PropertyValue::Reference { name, .. } => Some(name),
            _ => None,
        }
    }

    pub(crate) fn get_function(&self) -> Option<&ftd::interpreter2::FunctionCall> {
        match self {
            PropertyValue::FunctionCall(f) => Some(f),
            _ => None,
        }
    }

    pub fn get_reference_or_clone(&self) -> Option<&String> {
        match self {
            PropertyValue::Reference { name, .. } | PropertyValue::Clone { name, .. } => Some(name),
            _ => None,
        }
    }

    pub fn set_reference_or_clone(&mut self, new_name: &str) {
        match self {
            PropertyValue::Reference { name, .. } | PropertyValue::Clone { name, .. } => {
                *name = new_name.to_string();
            }
            _ => {}
        }
    }

    pub(crate) fn from_string_with_argument(
        value: &str,
        doc: &ftd::interpreter2::TDoc,
        expected_kind: Option<&ftd::interpreter2::KindData>,
        mutable: bool,
        line_number: usize,
        definition_name_with_arguments: Option<(&str, &[ftd::interpreter2::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter2::Argument)>,
    ) -> ftd::interpreter2::Result<
        ftd::interpreter2::StateWithThing<ftd::interpreter2::PropertyValue>,
    > {
        let value = ftd::ast::VariableValue::String {
            value: value.to_string(),
            line_number,
        };

        PropertyValue::from_ast_value_with_argument(
            value,
            doc,
            mutable,
            expected_kind,
            definition_name_with_arguments,
            loop_object_name_and_kind,
        )
    }

    pub(crate) fn from_ast_value(
        value: ftd::ast::VariableValue,
        doc: &ftd::interpreter2::TDoc,
        mutable: bool,
        expected_kind: Option<&ftd::interpreter2::KindData>,
    ) -> ftd::interpreter2::Result<
        ftd::interpreter2::StateWithThing<ftd::interpreter2::PropertyValue>,
    > {
        PropertyValue::from_ast_value_with_argument(value, doc, mutable, expected_kind, None, &None)
    }

    pub(crate) fn from_ast_value_with_argument(
        value: ftd::ast::VariableValue,
        doc: &ftd::interpreter2::TDoc,
        is_mutable: bool,
        expected_kind: Option<&ftd::interpreter2::KindData>,
        definition_name_with_arguments: Option<(&str, &[ftd::interpreter2::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter2::Argument)>,
    ) -> ftd::interpreter2::Result<
        ftd::interpreter2::StateWithThing<ftd::interpreter2::PropertyValue>,
    > {
        if let Some(reference) = try_ok_state!(PropertyValue::reference_from_ast_value(
            value.clone(),
            doc,
            is_mutable,
            expected_kind,
            definition_name_with_arguments,
            loop_object_name_and_kind,
        )?) {
            Ok(ftd::interpreter2::StateWithThing::new_thing(reference))
        } else {
            PropertyValue::value_from_ast_value(
                value,
                doc,
                is_mutable,
                expected_kind,
                definition_name_with_arguments,
            )
        }
    }

    fn to_ui_value(
        key: &str,
        value: ftd::ast::VariableValue,
        doc: &ftd::interpreter2::TDoc,
        definition_name_with_arguments: Option<(&str, &[ftd::interpreter2::Argument])>,
    ) -> ftd::interpreter2::Result<
        ftd::interpreter2::StateWithThing<ftd::interpreter2::PropertyValue>,
    > {
        let line_number = value.line_number();
        let ast_component = ftd::ast::Component::from_variable_value(key, value, doc.name)?;
        let component = try_ok_state!(ftd::interpreter2::Component::from_ast_component(
            ast_component,
            definition_name_with_arguments,
            doc,
        )?);

        Ok(ftd::interpreter2::StateWithThing::new_thing(
            ftd::interpreter2::PropertyValue::Value {
                value: ftd::interpreter2::Value::UI {
                    name: component.name.to_string(),
                    kind: ftd::interpreter2::Kind::ui().into_kind_data(),
                    component,
                },
                is_mutable: false,
                line_number,
            },
        ))
    }

    fn value_from_ast_value(
        value: ftd::ast::VariableValue,
        doc: &ftd::interpreter2::TDoc,
        is_mutable: bool,
        expected_kind: Option<&ftd::interpreter2::KindData>,
        definition_name_with_arguments: Option<(&str, &[ftd::interpreter2::Argument])>,
    ) -> ftd::interpreter2::Result<
        ftd::interpreter2::StateWithThing<ftd::interpreter2::PropertyValue>,
    > {
        let expected_kind = expected_kind.ok_or(ftd::interpreter2::Error::ParseError {
            message: "Need expected kind".to_string(),
            doc_id: doc.name.to_string(),
            line_number: value.line_number(),
        })?;
        Ok(match &expected_kind.kind.clone().inner() {
            ftd::interpreter2::Kind::String => {
                ftd::interpreter2::StateWithThing::new_thing(PropertyValue::Value {
                    value: Value::String {
                        text: value.string(doc.name)?,
                    },
                    is_mutable,
                    line_number: value.line_number(),
                })
            }
            ftd::interpreter2::Kind::Integer => {
                ftd::interpreter2::StateWithThing::new_thing(PropertyValue::Value {
                    value: Value::Integer {
                        value: value.string(doc.name)?.parse()?,
                    },
                    is_mutable,
                    line_number: value.line_number(),
                })
            }
            ftd::interpreter2::Kind::Decimal => {
                ftd::interpreter2::StateWithThing::new_thing(PropertyValue::Value {
                    value: Value::Decimal {
                        value: value.string(doc.name)?.parse()?,
                    },
                    is_mutable,
                    line_number: value.line_number(),
                })
            }
            ftd::interpreter2::Kind::Boolean => {
                ftd::interpreter2::StateWithThing::new_thing(PropertyValue::Value {
                    value: Value::Boolean {
                        value: value.string(doc.name)?.parse()?,
                    },
                    is_mutable,
                    line_number: value.line_number(),
                })
            }
            ftd::interpreter2::Kind::List { kind } if value.is_list() => {
                let line_number = value.line_number();
                let value_list = value.into_list(doc.name)?;
                let mut values = vec![];
                for (key, value) in value_list {
                    if !try_ok_state!(ftd::interpreter2::utils::kind_eq(
                        key.as_str(),
                        kind,
                        doc,
                        value.line_number(),
                    )?) {
                        return ftd::interpreter2::utils::e2(
                            format!("Expected list of `{:?}`, found: `{}`", kind, key),
                            doc.name,
                            value.line_number(),
                        );
                    }
                    values.push(if kind.is_ui() {
                        try_ok_state!(PropertyValue::to_ui_value(
                            &key,
                            value,
                            doc,
                            definition_name_with_arguments,
                        )?)
                    } else {
                        try_ok_state!(PropertyValue::from_ast_value(
                            value,
                            doc,
                            is_mutable,
                            Some(&ftd::interpreter2::KindData {
                                kind: kind.as_ref().clone(),
                                caption: expected_kind.caption,
                                body: expected_kind.body,
                            }),
                        )?)
                    });
                }
                ftd::interpreter2::StateWithThing::new_thing(PropertyValue::Value {
                    value: ftd::interpreter2::Value::List {
                        data: values,
                        kind: expected_kind.clone().inner_list(),
                    },
                    is_mutable,
                    line_number,
                })
            }
            ftd::interpreter2::Kind::Record { name } if value.is_record() || value.is_string() => {
                let record = try_ok_state!(doc.search_record(name, value.line_number())?);
                let (caption, headers, body, line_number) =
                    if let Ok(val) = value.get_record(doc.name) {
                        (
                            val.1.as_ref().to_owned(),
                            val.2.to_owned(),
                            val.3.to_owned(),
                            val.5.to_owned(),
                        )
                    } else {
                        (
                            Some(value.clone()),
                            ftd::ast::HeaderValues::new(vec![]),
                            None,
                            value.line_number(),
                        )
                    };

                // TODO: Check if the record name and the value kind are same
                let mut result_field: ftd::Map<PropertyValue> = Default::default();
                for field in record.fields {
                    if field.is_caption() && caption.is_some() {
                        let caption = caption.as_ref().unwrap().clone();
                        let property_value = try_ok_state!(PropertyValue::from_ast_value(
                            caption,
                            doc,
                            field.mutable,
                            Some(&field.kind),
                        )?);
                        result_field.insert(field.name.to_string(), property_value);
                        continue;
                    }
                    if field.is_body() && body.is_some() {
                        let body = body.as_ref().unwrap();
                        let property_value = try_ok_state!(PropertyValue::from_ast_value(
                            ftd::ast::VariableValue::String {
                                value: body.value.to_string(),
                                line_number: body.line_number,
                            },
                            doc,
                            field.mutable,
                            Some(&field.kind),
                        )?);
                        result_field.insert(field.name.to_string(), property_value);
                        continue;
                    }
                    let headers = headers.get_by_key(field.name.as_str());
                    if headers.is_empty() && field.kind.is_optional() {
                        result_field.insert(
                            field.name.to_string(),
                            PropertyValue::Value {
                                value: ftd::interpreter2::Value::Optional {
                                    data: Box::new(None),
                                    kind: expected_kind.to_owned(),
                                },
                                is_mutable,
                                line_number,
                            },
                        );
                        continue;
                    }
                    if field.kind.is_list() {
                        let mut header_list = vec![];
                        for header in headers {
                            header_list.extend(match &header.value {
                                ftd::ast::VariableValue::List { value, .. } => value.to_owned(),
                                t => vec![(header.key.to_string(), t.to_owned())],
                            });
                        }
                        let property_value = try_ok_state!(PropertyValue::from_ast_value(
                            ftd::ast::VariableValue::List {
                                value: header_list,
                                line_number: value.line_number(),
                            },
                            doc,
                            field.mutable,
                            Some(&field.kind),
                        )?);
                        result_field.insert(field.name.to_string(), property_value);
                        continue;
                    }

                    if headers.is_empty() && field.value.is_some() {
                        let value = field.value.unwrap();
                        match &value {
                            ftd::interpreter2::PropertyValue::Reference {
                                name: refernence,
                                source,
                                ..
                            } if source.is_local(name) => {
                                if let Some(field_name) =
                                    refernence.strip_prefix(format!("{}.", name).as_str())
                                {
                                    // Todo: field_name is empty throw error
                                    let property_value = result_field
                                        .get(field_name)
                                        .ok_or(ftd::interpreter2::Error::ParseError {
                                            message: format!(
                                                "field `{}` not found in record: `{}`",
                                                field_name, name
                                            ),
                                            doc_id: doc.name.to_string(),
                                            line_number,
                                        })?
                                        .clone();
                                    result_field.insert(field.name.to_string(), property_value);
                                }
                            }
                            t => {
                                result_field.insert(field.name.to_string(), t.clone());
                            }
                        }
                        continue;
                    }
                    if headers.len() != 1 {
                        return ftd::interpreter2::utils::e2(
                            format!(
                                "Expected `{}` of type `{:?}`, found: `{:?}`",
                                field.name, field.kind, headers
                            ),
                            doc.name,
                            value.line_number(),
                        );
                    }
                    let first_header = headers.first().unwrap();
                    let property_value = try_ok_state!(PropertyValue::from_ast_value(
                        first_header.value.clone(),
                        doc,
                        first_header.mutable,
                        Some(&field.kind),
                    )?);
                    result_field.insert(field.name.to_string(), property_value);
                }
                ftd::interpreter2::StateWithThing::new_thing(PropertyValue::Value {
                    value: ftd::interpreter2::Value::Record {
                        name: name.to_string(),
                        fields: result_field,
                    },
                    is_mutable,
                    line_number,
                })
            }
            ftd::interpreter2::Kind::OrType { name, variant }
                if variant.is_some() && (value.is_record() || value.is_string()) =>
            {
                let or_type = try_ok_state!(doc.search_or_type(name, value.line_number())?);
                let (caption, headers, body, line_number) =
                    if let Ok(val) = value.get_record(doc.name) {
                        (
                            val.1.as_ref().to_owned(),
                            val.2.to_owned(),
                            val.3.to_owned(),
                            val.5.to_owned(),
                        )
                    } else {
                        (
                            Some(value.clone()),
                            ftd::ast::HeaderValues::new(vec![]),
                            None,
                            value.line_number(),
                        )
                    };
                let variant_name = variant.as_ref().unwrap().clone();
                let variant = or_type
                    .variants
                    .into_iter()
                    .find(|v| v.name.eq(&variant_name))
                    .ok_or(ftd::interpreter2::Error::ParseError {
                        message: format!(
                            "Expected variant `{}` in or-type `{}`",
                            variant_name, name
                        ),
                        doc_id: doc.name.to_string(),
                        line_number: value.line_number(),
                    })?;

                // TODO: Check if the record name and the value kind are same
                let mut result_field: ftd::Map<PropertyValue> = Default::default();
                for field in variant.fields {
                    if field.is_caption() && caption.is_some() {
                        let caption = caption.as_ref().unwrap().clone();
                        let property_value = try_ok_state!(PropertyValue::from_ast_value(
                            caption,
                            doc,
                            field.mutable,
                            Some(&field.kind),
                        )?);
                        result_field.insert(field.name.to_string(), property_value);
                        continue;
                    }
                    if field.is_body() && body.is_some() {
                        let body = body.as_ref().unwrap();
                        let property_value = try_ok_state!(PropertyValue::from_ast_value(
                            ftd::ast::VariableValue::String {
                                value: body.value.to_string(),
                                line_number: body.line_number,
                            },
                            doc,
                            field.mutable,
                            Some(&field.kind),
                        )?);
                        result_field.insert(field.name.to_string(), property_value);
                        continue;
                    }
                    let headers = headers.get_by_key(field.name.as_str());
                    if headers.is_empty() && field.kind.is_optional() {
                        result_field.insert(
                            field.name.to_string(),
                            PropertyValue::Value {
                                value: ftd::interpreter2::Value::Optional {
                                    data: Box::new(None),
                                    kind: expected_kind.to_owned(),
                                },
                                is_mutable,
                                line_number,
                            },
                        );
                        continue;
                    }
                    if field.kind.is_list() {
                        let mut header_list = vec![];
                        for header in headers {
                            header_list.extend(match &header.value {
                                ftd::ast::VariableValue::List { value, .. } => value.to_owned(),
                                t => vec![(header.key.to_string(), t.to_owned())],
                            });
                        }
                        let property_value = try_ok_state!(PropertyValue::from_ast_value(
                            ftd::ast::VariableValue::List {
                                value: header_list,
                                line_number: value.line_number(),
                            },
                            doc,
                            field.mutable,
                            Some(&field.kind),
                        )?);
                        result_field.insert(field.name.to_string(), property_value);
                        continue;
                    }

                    if headers.is_empty() && field.value.is_some() {
                        let value = field.value.unwrap();
                        match &value {
                            ftd::interpreter2::PropertyValue::Reference {
                                name: refernence,
                                source,
                                ..
                            } if source.is_local(name) => {
                                if let Some(field_name) =
                                    refernence.strip_prefix(format!("{}.", name).as_str())
                                {
                                    // Todo: field_name is empty throw error
                                    let property_value = result_field
                                        .get(field_name)
                                        .ok_or(ftd::interpreter2::Error::ParseError {
                                            message: format!(
                                                "field `{}` not found in record: `{}`",
                                                field_name, name
                                            ),
                                            doc_id: doc.name.to_string(),
                                            line_number,
                                        })?
                                        .clone();
                                    result_field.insert(field.name.to_string(), property_value);
                                }
                            }
                            t => {
                                result_field.insert(field.name.to_string(), t.clone());
                            }
                        }
                        continue;
                    }
                    if headers.len() != 1 {
                        return ftd::interpreter2::utils::e2(
                            format!(
                                "Expected `{}` of type `{:?}`, found: `{:?}`",
                                field.name, field.kind, headers
                            ),
                            doc.name,
                            value.line_number(),
                        );
                    }
                    let first_header = headers.first().unwrap();
                    let property_value = try_ok_state!(PropertyValue::from_ast_value(
                        first_header.value.clone(),
                        doc,
                        first_header.mutable,
                        Some(&field.kind),
                    )?);
                    result_field.insert(field.name.to_string(), property_value);
                }
                ftd::interpreter2::StateWithThing::new_thing(dbg!(PropertyValue::Value {
                    value: ftd::interpreter2::Value::OrType {
                        name: name.to_string(),
                        variant: variant_name,
                        fields: result_field,
                    },
                    is_mutable,
                    line_number,
                }))
            }
            t => {
                unimplemented!("t::{:?}  {:?}", t, value)
            }
        })
    }

    fn reference_from_ast_value(
        value: ftd::ast::VariableValue,
        doc: &ftd::interpreter2::TDoc,
        mutable: bool,
        expected_kind: Option<&ftd::interpreter2::KindData>,
        definition_name_with_arguments: Option<(&str, &[ftd::interpreter2::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter2::Argument)>,
    ) -> ftd::interpreter2::Result<
        ftd::interpreter2::StateWithThing<Option<ftd::interpreter2::PropertyValue>>,
    > {
        match value.string(doc.name) {
            Ok(expression)
                if expression.starts_with(ftd::interpreter2::utils::REFERENCE)
                    && ftd::interpreter2::utils::get_function_name(
                        expression.trim_start_matches(ftd::interpreter2::utils::REFERENCE),
                        doc.name,
                        value.line_number(),
                    )
                    .is_ok() =>
            {
                let expression = expression
                    .trim_start_matches(ftd::interpreter2::utils::REFERENCE)
                    .to_string();

                let function_call = try_ok_state!(ftd::interpreter2::FunctionCall::from_string(
                    expression.as_str(),
                    doc,
                    mutable,
                    definition_name_with_arguments,
                    loop_object_name_and_kind,
                    value.line_number(),
                )?);

                Ok(ftd::interpreter2::StateWithThing::new_thing(Some(
                    ftd::interpreter2::PropertyValue::FunctionCall(function_call),
                )))
            }
            Ok(reference) if reference.starts_with(ftd::interpreter2::utils::CLONE) => {
                let reference = reference
                    .trim_start_matches(ftd::interpreter2::utils::CLONE)
                    .to_string();

                let (source, found_kind) = try_ok_state!(doc.get_kind_with_argument(
                    reference.as_str(),
                    value.line_number(),
                    definition_name_with_arguments,
                    loop_object_name_and_kind,
                )?);

                match expected_kind {
                    Some(ekind) if !ekind.kind.is_same_as(&found_kind.kind) => {
                        return ftd::interpreter2::utils::e2(
                            format!("2 Expected kind `{:?}`, found: `{:?}`", ekind, found_kind)
                                .as_str(),
                            doc.name,
                            value.line_number(),
                        )
                    }
                    _ => {}
                }

                let reference_full_name = source.get_reference_name(reference.as_str(), doc);

                Ok(ftd::interpreter2::StateWithThing::new_thing(Some(
                    PropertyValue::Clone {
                        name: reference_full_name,
                        kind: found_kind,
                        source,
                        is_mutable: mutable,
                        line_number: value.line_number(),
                    },
                )))
            }
            Ok(reference) if reference.starts_with(ftd::interpreter2::utils::REFERENCE) => {
                let reference = reference
                    .trim_start_matches(ftd::interpreter2::utils::REFERENCE)
                    .to_string();

                let (source, found_kind) = try_ok_state!(doc.get_kind_with_argument(
                    reference.as_str(),
                    value.line_number(),
                    definition_name_with_arguments,
                    loop_object_name_and_kind,
                )?);

                match expected_kind {
                    Some(ekind)
                        if !ekind.kind.is_same_as(&found_kind.kind)
                            && (ekind.kind.ref_inner().is_record()
                                || ekind.kind.ref_inner().is_or_type()) =>
                    {
                        return Ok(PropertyValue::value_from_ast_value(
                            value,
                            doc,
                            mutable,
                            expected_kind,
                            definition_name_with_arguments,
                        )?
                        .map(Some));
                    }
                    Some(ekind) if !ekind.kind.is_same_as(&found_kind.kind) => {
                        return ftd::interpreter2::utils::e2(
                            format!("3 Expected kind `{:?}`, found: `{:?}`", ekind, found_kind)
                                .as_str(),
                            doc.name,
                            value.line_number(),
                        )
                    }
                    _ => {}
                }

                if mutable {
                    let is_variable_mutable = if source.is_global() {
                        try_ok_state!(doc.search_variable(reference.as_str(), value.line_number())?)
                            .mutable
                    } else {
                        ftd::interpreter2::utils::get_argument_for_reference_and_remaining(
                            reference.as_str(),
                            doc.name,
                            definition_name_with_arguments,
                            loop_object_name_and_kind,
                        )
                        .unwrap()
                        .0
                        .mutable
                    };

                    if !is_variable_mutable {
                        return ftd::interpreter2::utils::e2(
                            format!(
                                "Cannot have mutable reference of immutable variable `{}`",
                                reference
                            ),
                            doc.name,
                            value.line_number(),
                        );
                    }
                }

                let reference_full_name = source.get_reference_name(reference.as_str(), doc);

                Ok(ftd::interpreter2::StateWithThing::new_thing(Some(
                    PropertyValue::Reference {
                        name: reference_full_name,
                        kind: expected_kind.map(Clone::clone).unwrap_or(found_kind),
                        source,
                        is_mutable: mutable,
                        line_number: value.line_number(),
                    },
                )))
            }
            _ => Ok(ftd::interpreter2::StateWithThing::new_thing(None)),
        }
    }

    pub(crate) fn kind(&self) -> ftd::interpreter2::Kind {
        match self {
            PropertyValue::Value { value, .. } => value.kind(),
            PropertyValue::Reference { kind, .. } => kind.kind.to_owned(),
            PropertyValue::Clone { kind, .. } => kind.kind.to_owned(),
            PropertyValue::FunctionCall(ftd::interpreter2::FunctionCall { kind, .. }) => {
                kind.kind.to_owned()
            }
        }
    }

    pub(crate) fn new_none(
        kind: ftd::interpreter2::KindData,
        line_number: usize,
    ) -> ftd::interpreter2::PropertyValue {
        ftd::interpreter2::PropertyValue::Value {
            value: ftd::interpreter2::Value::new_none(kind),
            is_mutable: false,
            line_number,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum PropertyValueSource {
    Global,
    Local(String),
    Loop(String),
}

impl PropertyValueSource {
    fn is_global(&self) -> bool {
        PropertyValueSource::Global.eq(self)
    }

    pub fn is_local(&self, name: &str) -> bool {
        matches!(self, PropertyValueSource::Local(l_name) if l_name.eq(name))
    }

    pub fn get_reference_name(&self, name: &str, doc: &ftd::interpreter2::TDoc) -> String {
        let name = name
            .strip_prefix(ftd::interpreter2::utils::REFERENCE)
            .or_else(|| name.strip_prefix(ftd::interpreter2::utils::CLONE))
            .unwrap_or(name);
        match self {
            PropertyValueSource::Global | PropertyValueSource::Local(_) => doc.resolve_name(name),
            PropertyValueSource::Loop(_) => doc.resolve_name(name), //TODO: Some different name for loop
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Value {
    String {
        text: String,
    },
    Integer {
        value: i64,
    },
    Decimal {
        value: f64,
    },
    Boolean {
        value: bool,
    },
    Object {
        values: ftd::Map<PropertyValue>,
    },
    Record {
        name: String,
        fields: ftd::Map<PropertyValue>,
    },
    OrType {
        name: String,
        variant: String,
        fields: ftd::Map<PropertyValue>,
    },
    List {
        data: Vec<PropertyValue>,
        kind: ftd::interpreter2::KindData,
    },
    Optional {
        data: Box<Option<Value>>,
        kind: ftd::interpreter2::KindData,
    },
    UI {
        name: String,
        kind: ftd::interpreter2::KindData,
        component: ftd::interpreter2::Component,
    },
}

impl Value {
    pub fn is_null(&self) -> bool {
        if let Self::String { text, .. } = self {
            return text.is_empty();
        }
        if let Self::Optional { data, .. } = self {
            let value = if let Some(ftd::interpreter2::Value::String { text, .. }) = data.as_ref() {
                text.is_empty()
            } else {
                false
            };
            if data.as_ref().eq(&None) || value {
                return true;
            }
        }
        false
    }

    pub fn is_empty(&self) -> bool {
        if let Self::List { data, .. } = self {
            if data.is_empty() {
                return true;
            }
        }
        false
    }

    pub fn is_record(&self, rec_name: &str) -> bool {
        matches!(self, Self::Record { name, .. } if rec_name.eq(name))
    }

    pub fn record_fields(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter2::Result<ftd::Map<PropertyValue>> {
        match self {
            Self::Record { fields, .. } => Ok(fields.to_owned()),
            t => ftd::interpreter2::utils::e2(
                format!("Expected record, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    pub fn is_equal(&self, other: &Self) -> bool {
        match (self.to_owned().inner(), other.to_owned().inner()) {
            (Some(Value::String { text: ref a, .. }), Some(Value::String { text: ref b, .. })) => {
                a == b
            }
            (a, b) => a == b,
        }
    }

    pub(crate) fn inner(&self) -> Option<Self> {
        match self {
            Value::Optional { data, .. } => data.as_ref().to_owned(),
            t => Some(t.to_owned()),
        }
    }

    pub(crate) fn is_static(&self, doc: &ftd::interpreter2::TDoc) -> bool {
        match self {
            ftd::interpreter2::Value::Optional { data, .. } if data.is_some() => {
                data.clone().unwrap().is_static(doc)
            }
            ftd::interpreter2::Value::List { data, .. } => {
                let mut is_static = true;
                for d in data {
                    if !d.is_static(doc) {
                        is_static = false;
                        break;
                    }
                }
                is_static
            }
            ftd::interpreter2::Value::Record { fields, .. }
            | ftd::interpreter2::Value::Object { values: fields, .. } => {
                let mut is_static = true;
                for d in fields.values() {
                    if !d.is_static(doc) {
                        is_static = false;
                        break;
                    }
                }
                is_static
            }
            _ => true,
        }
    }

    pub(crate) fn kind(&self) -> ftd::interpreter2::Kind {
        match self {
            Value::String { .. } => ftd::interpreter2::Kind::string(),
            Value::Integer { .. } => ftd::interpreter2::Kind::integer(),
            Value::Decimal { .. } => ftd::interpreter2::Kind::decimal(),
            Value::Boolean { .. } => ftd::interpreter2::Kind::boolean(),
            Value::Object { .. } => ftd::interpreter2::Kind::object(),
            Value::Record { name, .. } => ftd::interpreter2::Kind::record(name),
            Value::List { kind, .. } => kind.kind.clone().into_list(),
            Value::Optional { kind, .. } => ftd::interpreter2::Kind::Optional {
                kind: Box::new(kind.kind.clone()),
            },
            Value::UI { name, .. } => ftd::interpreter2::Kind::ui_with_name(name),
            Value::OrType { name, .. } => ftd::interpreter2::Kind::or_type(name),
        }
    }

    pub(crate) fn to_string(
        &self,
        doc: &ftd::interpreter2::TDoc,
        line_number: usize,
        field: Option<String>,
    ) -> ftd::interpreter2::Result<String> {
        Ok(match self {
            Value::String { text } => format!("\"{}\"", text),
            Value::Integer { value } => value.to_string(),
            Value::Decimal { value } => value.to_string(),
            Value::Boolean { value } => value.to_string(),
            Value::List { data, .. } => {
                let mut values = vec![];
                for value in data {
                    let v = value.clone().resolve(doc, line_number)?.to_string(
                        doc,
                        value.line_number(),
                        None,
                    )?;
                    values.push(v);
                }
                format!("({:?})", values.join(","))
            }
            Value::Record { fields, .. }
                if field
                    .as_ref()
                    .map(|v| fields.contains_key(v))
                    .unwrap_or(false) =>
            {
                let property_value = fields.get(&field.unwrap()).unwrap();
                property_value
                    .clone()
                    .resolve(doc, line_number)?
                    .to_string(doc, property_value.line_number(), None)?
            }

            t => unimplemented!("{:?}", t),
        })
    }

    pub(crate) fn to_evalexpr_value(
        &self,
        doc: &ftd::interpreter2::TDoc,
        line_number: usize,
    ) -> ftd::interpreter2::Result<ftd::evalexpr::Value> {
        Ok(match self {
            Value::String { text } => ftd::evalexpr::Value::String(text.to_string()),
            Value::Integer { value } => ftd::evalexpr::Value::Int(*value),
            Value::Decimal { value } => ftd::evalexpr::Value::Float(*value),
            Value::Boolean { value } => ftd::evalexpr::Value::Boolean(*value),
            Value::List { data, .. } => {
                let mut values = vec![];
                for value in data {
                    let v = value
                        .clone()
                        .resolve(doc, line_number)?
                        .to_evalexpr_value(doc, value.line_number())?;
                    values.push(v);
                }
                ftd::evalexpr::Value::Tuple(values)
            }
            Value::Optional { data, .. } => {
                if let Some(data) = data.as_ref() {
                    data.to_evalexpr_value(doc, line_number)?
                } else {
                    ftd::evalexpr::Value::Empty
                }
            }
            t => unimplemented!("{:?}", t),
        })
    }

    pub(crate) fn from_evalexpr_value(
        value: ftd::evalexpr::Value,
        expected_kind: &ftd::interpreter2::Kind,
        doc_name: &str,
        line_number: usize,
    ) -> ftd::interpreter2::Result<Value> {
        Ok(match value {
            ftd::evalexpr::Value::String(text) if expected_kind.is_string() => {
                Value::String { text }
            }
            ftd::evalexpr::Value::Float(value) if expected_kind.is_decimal() => {
                Value::Decimal { value }
            }
            ftd::evalexpr::Value::Int(value) if expected_kind.is_integer() => {
                Value::Integer { value }
            }
            ftd::evalexpr::Value::Boolean(value) if expected_kind.is_boolean() => {
                Value::Boolean { value }
            }
            ftd::evalexpr::Value::Tuple(data) if expected_kind.is_list() => {
                let mut values = vec![];
                let val_kind = expected_kind.list_type(doc_name, line_number)?;
                for val in data {
                    values.push(ftd::interpreter2::PropertyValue::Value {
                        value: Value::from_evalexpr_value(val, &val_kind, doc_name, line_number)?,
                        is_mutable: false,
                        line_number,
                    });
                }
                Value::List {
                    data: values,
                    kind: ftd::interpreter2::KindData::new(val_kind),
                }
            }
            ftd::evalexpr::Value::Empty if expected_kind.is_optional() => Value::Optional {
                data: Box::new(None),
                kind: ftd::interpreter2::KindData::new(expected_kind.clone()),
            },
            t => {
                return ftd::interpreter2::utils::e2(
                    format!("Expected kind: `{:?}`, found: `{:?}`", expected_kind, t),
                    doc_name,
                    line_number,
                )
            }
        })
    }

    pub(crate) fn new_none(kind: ftd::interpreter2::KindData) -> ftd::interpreter2::Value {
        ftd::interpreter2::Value::Optional {
            data: Box::new(None),
            kind,
        }
    }

    pub(crate) fn into_evalexpr_value(self) -> ftd::evalexpr::Value {
        match self {
            ftd::interpreter2::Value::String { text } => ftd::evalexpr::Value::String(text),
            ftd::interpreter2::Value::Integer { value } => ftd::evalexpr::Value::Int(value),
            ftd::interpreter2::Value::Decimal { value } => ftd::evalexpr::Value::Float(value),
            ftd::interpreter2::Value::Boolean { value } => ftd::evalexpr::Value::Boolean(value),
            t => unimplemented!("{:?}", t),
        }
    }

    pub fn string(&self, doc_id: &str, line_number: usize) -> ftd::interpreter2::Result<String> {
        match self {
            ftd::interpreter2::Value::String { text } => Ok(text.to_string()),
            t => ftd::interpreter2::utils::e2(
                format!("Expected String, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    pub fn into_property_value(self, is_mutable: bool, line_number: usize) -> PropertyValue {
        PropertyValue::Value {
            value: self,
            is_mutable,
            line_number,
        }
    }
}
