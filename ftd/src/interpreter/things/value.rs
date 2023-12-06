#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum PropertyValue {
    Value {
        value: ftd::interpreter::Value,
        is_mutable: bool,
        line_number: usize,
    },
    Reference {
        name: String,
        kind: ftd::interpreter::KindData,
        source: ftd::interpreter::PropertyValueSource,
        is_mutable: bool,
        line_number: usize,
    },
    Clone {
        name: String,
        kind: ftd::interpreter::KindData,
        source: ftd::interpreter::PropertyValueSource,
        is_mutable: bool,
        line_number: usize,
    },
    FunctionCall(ftd::interpreter::FunctionCall),
}

impl PropertyValue {
    pub(crate) fn is_mutable(&self) -> bool {
        match self {
            PropertyValue::Value { is_mutable, .. }
            | PropertyValue::Reference { is_mutable, .. }
            | PropertyValue::Clone { is_mutable, .. }
            | PropertyValue::FunctionCall(ftd::interpreter::FunctionCall { is_mutable, .. }) => {
                *is_mutable
            }
        }
    }

    pub(crate) fn set_mutable(&mut self, mutable: bool) {
        match self {
            PropertyValue::Value { is_mutable, .. }
            | PropertyValue::Reference { is_mutable, .. }
            | PropertyValue::Clone { is_mutable, .. }
            | PropertyValue::FunctionCall(ftd::interpreter::FunctionCall { is_mutable, .. }) => {
                *is_mutable = mutable;
            }
        }
    }

    pub(crate) fn is_static(&self, doc: &ftd::interpreter::TDoc) -> bool {
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

    pub fn line_number(&self) -> usize {
        match self {
            PropertyValue::Value { line_number, .. }
            | PropertyValue::Reference { line_number, .. }
            | PropertyValue::Clone { line_number, .. }
            | PropertyValue::FunctionCall(ftd::interpreter::FunctionCall { line_number, .. }) => {
                *line_number
            }
        }
    }

    pub fn resolve(
        self,
        doc: &ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::Value> {
        self.resolve_with_inherited(doc, line_number, &Default::default())
    }

    pub(crate) fn resolve_with_inherited(
        self,
        doc: &ftd::interpreter::TDoc,
        line_number: usize,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::interpreter::Result<ftd::interpreter::Value> {
        match self {
            ftd::interpreter::PropertyValue::Value { value, .. } => Ok(value),
            ftd::interpreter::PropertyValue::Reference { name, kind, .. }
            | ftd::interpreter::PropertyValue::Clone { name, kind, .. } => {
                doc.resolve_with_inherited(name.as_str(), &kind, line_number, inherited_variables)
            }
            ftd::interpreter::PropertyValue::FunctionCall(ftd::interpreter::FunctionCall {
                name,
                kind,
                values,
                line_number,
                ..
            }) => {
                let function = doc.get_function(name.as_str(), line_number)?;
                function.resolve(&kind, &values, doc, line_number)?.ok_or(
                    ftd::interpreter::Error::ParseError {
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
        matches!(self, ftd::interpreter::PropertyValue::Value { .. })
    }

    pub fn into_property(
        &self,
        source: ftd::interpreter::PropertySource,
    ) -> ftd::interpreter::Property {
        ftd::interpreter::Property {
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
    ) -> ftd::interpreter::Result<&ftd::interpreter::Value> {
        match self {
            ftd::interpreter::PropertyValue::Value { value, .. } => Ok(value),
            t => ftd::interpreter::utils::e2(
                format!("Expected value found `{:?}`", t).as_str(),
                doc_id,
                line_number,
            ),
        }
    }

    pub(crate) fn value_mut(
        &mut self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<&mut ftd::interpreter::Value> {
        match self {
            ftd::interpreter::PropertyValue::Value { value, .. } => Ok(value),
            t => ftd::interpreter::utils::e2(
                format!("Expected value found `{:?}`", t).as_str(),
                doc_id,
                line_number,
            ),
        }
    }

    pub(crate) fn value_optional(&self) -> Option<&ftd::interpreter::Value> {
        self.value("", 0).ok()
    }

    pub(crate) fn reference_name(&self) -> Option<&String> {
        match self {
            PropertyValue::Reference { name, .. } => Some(name),
            _ => None,
        }
    }

    pub(crate) fn get_function(&self) -> Option<&ftd::interpreter::FunctionCall> {
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

    pub(crate) fn scan_string_with_argument(
        value: &str,
        doc: &mut ftd::interpreter::TDoc,
        line_number: usize,
        definition_name_with_arguments: Option<(&str, &[String])>,
        loop_object_name_and_kind: &Option<String>,
    ) -> ftd::interpreter::Result<()> {
        let value = ftd::ast::VariableValue::String {
            value: value.to_string(),
            line_number,
            source: ftd::ast::ValueSource::Default,
        };

        ftd::interpreter::PropertyValue::scan_ast_value_with_argument(
            value,
            doc,
            definition_name_with_arguments,
            loop_object_name_and_kind,
        )
    }

    pub(crate) fn from_string_with_argument(
        value: &str,
        doc: &mut ftd::interpreter::TDoc,
        expected_kind: Option<&ftd::interpreter::KindData>,
        mutable: bool,
        line_number: usize,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<ftd::interpreter::PropertyValue>>
    {
        let value = ftd::ast::VariableValue::String {
            value: value.to_string(),
            line_number,
            source: ftd::ast::ValueSource::Default,
        };

        ftd::interpreter::PropertyValue::from_ast_value_with_argument(
            value,
            doc,
            mutable,
            expected_kind,
            definition_name_with_arguments,
            loop_object_name_and_kind,
        )
    }

    pub(crate) fn scan_ast_value(
        value: ftd::ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()> {
        PropertyValue::scan_ast_value_with_argument(value, doc, None, &None)
    }

    pub(crate) fn scan_ast_value_with_argument(
        value: ftd::ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
        definition_name_with_arguments: Option<(&str, &[String])>,
        loop_object_name_and_kind: &Option<String>,
    ) -> ftd::interpreter::Result<()> {
        if PropertyValue::scan_reference_from_ast_value(
            value.clone(),
            doc,
            definition_name_with_arguments,
            loop_object_name_and_kind,
        )? {
            Ok(())
        } else {
            PropertyValue::scan_value_from_ast_value(
                value,
                doc,
                definition_name_with_arguments,
                loop_object_name_and_kind,
            )
        }
    }

    fn scan_reference_from_ast_value(
        value: ftd::ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
        definition_name_with_arguments: Option<(&str, &[String])>,
        loop_object_name_and_kind: &Option<String>,
    ) -> ftd::interpreter::Result<bool> {
        match value.string(doc.name) {
            Ok(expression)
                if expression.starts_with(ftd::interpreter::utils::REFERENCE)
                    && ftd::interpreter::utils::get_function_name(
                        expression.trim_start_matches(ftd::interpreter::utils::REFERENCE),
                        doc.name,
                        value.line_number(),
                    )
                    .is_ok() =>
            {
                let expression = expression
                    .trim_start_matches(ftd::interpreter::utils::REFERENCE)
                    .to_string();

                ftd::interpreter::FunctionCall::scan_string(
                    expression.as_str(),
                    doc,
                    definition_name_with_arguments,
                    loop_object_name_and_kind,
                    value.line_number(),
                )?;

                Ok(true)
            }
            Ok(reference)
                if reference.starts_with(ftd::interpreter::utils::CLONE)
                    || reference.starts_with(ftd::interpreter::utils::REFERENCE) =>
            {
                let reference = reference
                    .strip_prefix(ftd::interpreter::utils::REFERENCE)
                    .or_else(|| reference.strip_prefix(ftd::interpreter::utils::CLONE))
                    .map_or(reference.to_string(), ToString::to_string);

                let initial_kind_with_remaining_and_source =
                    ftd::interpreter::utils::is_argument_in_component_or_loop(
                        reference.as_str(),
                        doc,
                        definition_name_with_arguments,
                        loop_object_name_and_kind,
                    );

                if !initial_kind_with_remaining_and_source {
                    doc.scan_thing(reference.as_str(), value.line_number())?;
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn from_ast_value(
        value: ftd::ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
        mutable: bool,
        expected_kind: Option<&ftd::interpreter::KindData>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<ftd::interpreter::PropertyValue>>
    {
        PropertyValue::from_ast_value_with_argument(
            value,
            doc,
            mutable,
            expected_kind,
            &mut None,
            &None,
        )
    }

    pub(crate) fn from_ast_value_with_argument(
        value: ftd::ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
        is_mutable: bool,
        expected_kind: Option<&ftd::interpreter::KindData>,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<ftd::interpreter::PropertyValue>>
    {
        if let Some(reference) = try_ok_state!(PropertyValue::reference_from_ast_value(
            value.clone(),
            doc,
            is_mutable,
            expected_kind,
            definition_name_with_arguments,
            loop_object_name_and_kind,
        )?) {
            Ok(ftd::interpreter::StateWithThing::new_thing(reference))
        } else {
            PropertyValue::value_from_ast_value(
                value,
                doc,
                is_mutable,
                expected_kind,
                definition_name_with_arguments,
                loop_object_name_and_kind,
            )
        }
    }

    fn to_ui_value(
        key: &str,
        value: ftd::ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<ftd::interpreter::PropertyValue>>
    {
        let line_number = value.line_number();

        if key.eq("ftd.ui") {
            return ftd::interpreter::PropertyValue::from_ast_value_with_argument(
                value,
                doc,
                false,
                Some(&ftd::interpreter::Kind::ui().into_kind_data()),
                definition_name_with_arguments,
                loop_object_name_and_kind,
            );
        }
        let ast_component = ftd::ast::Component::from_variable_value(key, value, doc.name)?;
        let component = try_ok_state!(ftd::interpreter::Component::from_ast_component(
            ast_component,
            definition_name_with_arguments,
            doc,
        )?);

        Ok(ftd::interpreter::StateWithThing::new_thing(
            ftd::interpreter::PropertyValue::Value {
                value: ftd::interpreter::Value::UI {
                    name: component.name.to_string(),
                    kind: ftd::interpreter::Kind::ui().into_kind_data(),
                    component,
                },
                is_mutable: false,
                line_number,
            },
        ))
    }

    fn from_record(
        record: &ftd::interpreter::Record,
        value: ftd::ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
        is_mutable: bool,
        _expected_kind: &ftd::interpreter::KindData,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<ftd::interpreter::PropertyValue>>
    {
        if !(value.is_record() || value.is_string()) {
            return ftd::interpreter::utils::e2(
                format!("`{:?}` value is npt supported yet", value),
                doc.name,
                value.line_number(),
            );
        }
        let name = record.name.as_str();
        let (caption, headers, body, line_number) = if let Ok(val) = value.get_record(doc.name) {
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
        for field in record.fields.iter() {
            if field.is_caption() && caption.is_some() {
                let caption = caption.as_ref().unwrap().clone();
                let property_value = try_ok_state!(PropertyValue::from_ast_value_with_argument(
                    caption,
                    doc,
                    field.mutable || is_mutable,
                    Some(&field.kind),
                    definition_name_with_arguments,
                    loop_object_name_and_kind
                )?);
                result_field.insert(field.name.to_string(), property_value);
                continue;
            }
            if field.is_body() && body.is_some() {
                let body = body.as_ref().unwrap();
                let property_value = try_ok_state!(PropertyValue::from_ast_value_with_argument(
                    ftd::ast::VariableValue::String {
                        value: body.value.to_string(),
                        line_number: body.line_number,
                        source: ftd::ast::ValueSource::Body
                    },
                    doc,
                    field.mutable || is_mutable,
                    Some(&field.kind),
                    definition_name_with_arguments,
                    loop_object_name_and_kind
                )?);
                result_field.insert(field.name.to_string(), property_value);
                continue;
            }
            let headers =
                headers.get_by_key_optional(field.name.as_str(), doc.name, line_number)?;

            if headers.is_none() && field.kind.is_optional() {
                result_field.insert(
                    field.name.to_string(),
                    PropertyValue::Value {
                        value: ftd::interpreter::Value::Optional {
                            data: Box::new(None),
                            kind: field.kind.to_owned().inner(),
                        },
                        is_mutable: field.mutable || is_mutable,
                        line_number,
                    },
                );
                continue;
            }
            if field.kind.is_list() {
                let mut variable = ftd::ast::VariableValue::List {
                    value: vec![],
                    line_number: value.line_number(),
                };
                if let Some(header) = headers {
                    variable = header.value.clone();
                    variable.set_line_number(value.line_number())
                }
                let property_value = try_ok_state!(PropertyValue::from_ast_value_with_argument(
                    variable,
                    doc,
                    field.mutable || is_mutable,
                    Some(&field.kind),
                    definition_name_with_arguments,
                    loop_object_name_and_kind
                )?);
                result_field.insert(field.name.to_string(), property_value);
                continue;
            }

            if headers.is_none() && field.value.is_some() {
                let value = field.value.as_ref().unwrap();
                match value {
                    ftd::interpreter::PropertyValue::Reference {
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
                                .ok_or(ftd::interpreter::Error::ParseError {
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
            if headers.is_none() {
                return ftd::interpreter::utils::e2(
                    format!(
                        "Expected `{}` of type `{:?}`, found: `{:?}`",
                        field.name, field.kind, headers
                    ),
                    doc.name,
                    value.line_number(),
                );
            }
            let first_header = headers.unwrap();

            if field.mutable.ne(&first_header.mutable) {
                return ftd::interpreter::utils::e2(
                    format!(
                        "Mutability conflict in field `{}` for record `{}`",
                        field.name, name
                    ),
                    doc.name,
                    first_header.line_number,
                );
            }

            let mut field = field.to_owned();
            let remaining = first_header
                .key
                .trim_start_matches(format!("{}.", field.name).as_str())
                .trim_start_matches(field.name.as_str())
                .trim()
                .to_string();
            if !remaining.is_empty() {
                try_ok_state!(field.update_with_or_type_variant(
                    doc,
                    remaining.as_str(),
                    value.line_number()
                )?);
            }

            let property_value = try_ok_state!(PropertyValue::from_ast_value_with_argument(
                first_header.value.clone(),
                doc,
                field.mutable || is_mutable,
                Some(&field.kind),
                definition_name_with_arguments,
                loop_object_name_and_kind
            )?);
            result_field.insert(field.name.to_string(), property_value);
        }
        Ok(ftd::interpreter::StateWithThing::new_thing(
            PropertyValue::Value {
                value: ftd::interpreter::Value::Record {
                    name: name.to_string(),
                    fields: result_field,
                },
                is_mutable,
                line_number,
            },
        ))
    }

    fn scan_value_from_ast_value(
        value: ftd::ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
        definition_name_with_arguments: Option<(&str, &[String])>,
        loop_object_name_and_kind: &Option<String>,
    ) -> ftd::interpreter::Result<()> {
        match value {
            ftd::ast::VariableValue::Optional { value, .. } if value.is_some() => {
                PropertyValue::scan_ast_value_with_argument(
                    value.unwrap(),
                    doc,
                    definition_name_with_arguments,
                    loop_object_name_and_kind,
                )?;
            }
            ftd::ast::VariableValue::List { value, .. } => {
                for val in value {
                    PropertyValue::scan_ast_value_with_argument(
                        val.value,
                        doc,
                        definition_name_with_arguments,
                        loop_object_name_and_kind,
                    )?;
                }
            }
            ftd::ast::VariableValue::Record {
                caption,
                headers,
                body,
                values,
                ..
            } => {
                if let Some(caption) = caption.as_ref() {
                    PropertyValue::scan_ast_value_with_argument(
                        caption.to_owned(),
                        doc,
                        definition_name_with_arguments,
                        loop_object_name_and_kind,
                    )?;
                }
                for header in headers.0 {
                    PropertyValue::scan_ast_value_with_argument(
                        header.value,
                        doc,
                        definition_name_with_arguments,
                        loop_object_name_and_kind,
                    )?;
                    if let Some(condition) = header.condition {
                        ftd::interpreter::Expression::scan_ast_condition(
                            ftd::ast::Condition::new(condition.as_str(), header.line_number),
                            definition_name_with_arguments,
                            loop_object_name_and_kind,
                            doc,
                        )?
                    }
                }
                if let Some(body) = body {
                    PropertyValue::scan_string_with_argument(
                        body.value.as_str(),
                        doc,
                        body.line_number,
                        definition_name_with_arguments,
                        loop_object_name_and_kind,
                    )?;
                }

                for val in values {
                    PropertyValue::scan_ast_value_with_argument(
                        val.value,
                        doc,
                        definition_name_with_arguments,
                        loop_object_name_and_kind,
                    )?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn value_from_ast_value(
        value: ftd::ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
        is_mutable: bool,
        expected_kind: Option<&ftd::interpreter::KindData>,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<ftd::interpreter::PropertyValue>>
    {
        let expected_kind = expected_kind.ok_or(ftd::interpreter::Error::ParseError {
            message: "Need expected kind".to_string(),
            doc_id: doc.name.to_string(),
            line_number: value.line_number(),
        })?;
        return get_property_value(
            value,
            doc,
            is_mutable,
            expected_kind,
            definition_name_with_arguments,
            loop_object_name_and_kind,
        );

        fn get_property_value(
            value: ftd::ast::VariableValue,
            doc: &mut ftd::interpreter::TDoc,
            is_mutable: bool,
            expected_kind: &ftd::interpreter::KindData,
            definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
            loop_object_name_and_kind: &Option<(
                String,
                ftd::interpreter::Argument,
                Option<String>,
            )>,
        ) -> ftd::interpreter::Result<
            ftd::interpreter::StateWithThing<ftd::interpreter::PropertyValue>,
        > {
            Ok(match &expected_kind.kind.clone() {
                ftd::interpreter::Kind::Optional { kind } => {
                    let kind = kind.clone().into_kind_data();
                    value.is_null();
                    match value {
                        ftd::ast::VariableValue::Optional {
                            value: ref ivalue, ..
                        } => match ivalue.as_ref() {
                            None => ftd::interpreter::StateWithThing::new_thing(
                                ftd::interpreter::PropertyValue::Value {
                                    value: ftd::interpreter::Value::Optional {
                                        data: Box::new(None),
                                        kind,
                                    },
                                    is_mutable,
                                    line_number: value.line_number(),
                                },
                            ),
                            Some(value) => get_property_value(
                                value.to_owned(),
                                doc,
                                is_mutable,
                                &kind,
                                definition_name_with_arguments,
                                loop_object_name_and_kind,
                            )?,
                        },
                        _ => get_property_value(
                            value,
                            doc,
                            is_mutable,
                            &kind,
                            definition_name_with_arguments,
                            loop_object_name_and_kind,
                        )?,
                    }
                }
                ftd::interpreter::Kind::Constant { kind } => {
                    let kind = kind.clone().into_kind_data();
                    get_property_value(
                        value,
                        doc,
                        is_mutable,
                        &kind,
                        definition_name_with_arguments,
                        loop_object_name_and_kind,
                    )?
                }
                ftd::interpreter::Kind::String => {
                    ftd::interpreter::StateWithThing::new_thing(PropertyValue::Value {
                        value: Value::String {
                            text: value.string(doc.name)?,
                        },
                        is_mutable,
                        line_number: value.line_number(),
                    })
                }
                ftd::interpreter::Kind::Integer => {
                    ftd::interpreter::StateWithThing::new_thing(PropertyValue::Value {
                        value: Value::Integer {
                            value: value.string(doc.name)?.parse()?,
                        },
                        is_mutable,
                        line_number: value.line_number(),
                    })
                }
                ftd::interpreter::Kind::Decimal => {
                    ftd::interpreter::StateWithThing::new_thing(PropertyValue::Value {
                        value: Value::Decimal {
                            value: value.string(doc.name)?.parse()?,
                        },
                        is_mutable,
                        line_number: value.line_number(),
                    })
                }
                ftd::interpreter::Kind::Boolean => {
                    ftd::interpreter::StateWithThing::new_thing(PropertyValue::Value {
                        value: Value::Boolean {
                            value: value.string(doc.name)?.parse()?,
                        },
                        is_mutable,
                        line_number: value.line_number(),
                    })
                }
                ftd::interpreter::Kind::List { kind } => {
                    let line_number = value.line_number();
                    let value_list = value.into_list(doc.name, kind)?;
                    let mut values = vec![];
                    for (key, value) in value_list {
                        if !try_ok_state!(ftd::interpreter::utils::kind_eq(
                            key.as_str(),
                            kind,
                            doc,
                            value.line_number(),
                        )?) {
                            return ftd::interpreter::utils::e2(
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
                                loop_object_name_and_kind
                            )?)
                        } else {
                            try_ok_state!(PropertyValue::from_ast_value(
                                value,
                                doc,
                                is_mutable,
                                Some(&ftd::interpreter::KindData {
                                    kind: kind.as_ref().clone(),
                                    caption: expected_kind.caption,
                                    body: expected_kind.body,
                                }),
                            )?)
                        });
                    }
                    ftd::interpreter::StateWithThing::new_thing(PropertyValue::Value {
                        value: ftd::interpreter::Value::List {
                            data: values,
                            kind: expected_kind.clone().inner_list(),
                        },
                        is_mutable,
                        line_number,
                    })
                }
                ftd::interpreter::Kind::Record { name }
                    if value.is_record() || value.is_string() =>
                {
                    let record = try_ok_state!(doc.search_record(name, value.line_number())?);
                    ftd::interpreter::PropertyValue::from_record(
                        &record,
                        value,
                        doc,
                        is_mutable,
                        expected_kind,
                        definition_name_with_arguments,
                        loop_object_name_and_kind,
                    )?
                }
                ftd::interpreter::Kind::OrType { name, variant, .. } => {
                    let or_type = try_ok_state!(doc.search_or_type(name, value.line_number())?);
                    let line_number = value.line_number();
                    if let Some(variant_name) = variant {
                        let variant = or_type
                            .variants
                            .into_iter()
                            .find(|v| {
                                v.name().eq(variant_name)
                                    || variant_name.starts_with(format!("{}.", v.name()).as_str())
                            })
                            .ok_or(ftd::interpreter::Error::ParseError {
                                message: format!(
                                    "Expected variant `{}` in or-type `{}`",
                                    variant_name, name
                                ),
                                doc_id: doc.name.to_string(),
                                line_number: value.line_number(),
                            })?;
                        let value = match &variant {
                        ftd::interpreter::OrTypeVariant::Constant(c) => return ftd::interpreter::utils::e2(format!("Cannot pass constant variant as property, variant: `{}`. Help: Pass variant as value instead", c.name), doc.name, c.line_number),
                        ftd::interpreter::OrTypeVariant::AnonymousRecord(record) => try_ok_state!(ftd::interpreter::PropertyValue::from_record(
                            record,
                            value,
                            doc,
                            is_mutable,
                            expected_kind,
                            definition_name_with_arguments,
                            loop_object_name_and_kind,
                        )?),
                        ftd::interpreter::OrTypeVariant::Regular(regular) => {
                            let variant_name = variant_name.trim_start_matches(format!("{}.", variant.name()).as_str()).trim().to_string();
                            let kind = if regular.kind.kind.ref_inner().is_or_type() && !variant_name.is_empty() {
                                let (name, variant, _full_variant) = regular.kind.kind.get_or_type().unwrap();
                                let variant_name = format!("{}.{}", name, variant_name);
                                ftd::interpreter::Kind::or_type_with_variant(name.as_str(), variant.unwrap_or_else(|| variant_name.clone()).as_str(), variant_name.as_str()).into_kind_data()
                            } else {
                                regular.kind.to_owned()
                            };

                            /*try_ok_state!(
                                ftd::interpreter::PropertyValue::value_from_ast_value(
                                    value,
                                    doc,
                                    is_mutable,
                                    Some(&kind),
                                    definition_name_with_arguments,
                                    loop_object_name_and_kind
                                )?
                            );*/

                            try_ok_state!(
                                ftd::interpreter::PropertyValue::from_ast_value_with_argument(
                                    value,
                                    doc,
                                    is_mutable,
                                    Some(&kind),
                                    definition_name_with_arguments,
                                    loop_object_name_and_kind
                                )?
                            )
                        }
                    };
                        ftd::interpreter::StateWithThing::new_thing(
                            ftd::interpreter::Value::new_or_type(
                                name,
                                variant.name().as_str(),
                                variant_name,
                                value,
                            )
                            .into_property_value(false, line_number),
                        )
                    } else {
                        let value_str = format!("{}.{}", name, value.string(doc.name)?);
                        let (found_or_type_name, or_type_variant) = try_ok_state!(doc
                            .search_or_type_with_variant(
                                value_str.as_str(),
                                value.line_number()
                            )?);

                        if or_type.name.ne(&found_or_type_name) {
                            return ftd::interpreter::utils::e2(
                                format!(
                                    "Expected or-type is `{}`, found: `{}`",
                                    or_type.name, found_or_type_name
                                ),
                                doc.name,
                                value.line_number(),
                            );
                        }

                        let constant = or_type_variant.ok_constant(doc.name)?;
                        let value =
                            constant
                                .value
                                .clone()
                                .ok_or(ftd::interpreter::Error::ParseError {
                                    message: format!(
                                        "Expected value for constant variant `{}`",
                                        constant.name
                                    ),
                                    doc_id: doc.name.to_string(),
                                    line_number: constant.line_number,
                                })?;

                        ftd::interpreter::StateWithThing::new_thing(
                            ftd::interpreter::Value::new_or_type(
                                name,
                                constant.name.as_str(),
                                constant.name.as_str(),
                                value,
                            )
                            .into_property_value(false, line_number),
                        )
                    }
                }
                ftd::interpreter::Kind::Module => {
                    ftd::interpreter::StateWithThing::new_thing(PropertyValue::Value {
                        value: Value::Module {
                            name: doc.resolve_module_name(value.string(doc.name)?.as_str()),
                            things: Default::default(),
                        },
                        is_mutable,
                        line_number: value.line_number(),
                    })
                }
                t => {
                    unimplemented!("t::{:?}  {:?}", t, value)
                }
            })
        }
    }

    fn reference_from_ast_value(
        value: ftd::ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
        mutable: bool,
        expected_kind: Option<&ftd::interpreter::KindData>,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
    ) -> ftd::interpreter::Result<
        ftd::interpreter::StateWithThing<Option<ftd::interpreter::PropertyValue>>,
    > {
        match value.string(doc.name) {
            Ok(expression)
                if expression
                    .starts_with(format!("${}.", ftd::interpreter::FTD_INHERITED).as_str()) =>
            {
                let reference = expression
                    .trim_start_matches(ftd::interpreter::utils::REFERENCE)
                    .to_string();

                // Todo: remove it after 0.3
                if reference.starts_with("inherited.colors")
                    || reference.starts_with("inherited.types")
                {
                    let found_kind = doc
                        .get_kind_with_argument(
                            reference.as_str(),
                            value.line_number(),
                            definition_name_with_arguments,
                            loop_object_name_and_kind,
                        )
                        .ok()
                        .and_then(|v| v.into_optional().map(|v| v.1));

                    if let Some(found_kind) = found_kind {
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
                                    loop_object_name_and_kind,
                                )?
                                .map(Some));
                            }
                            Some(ekind) if !ekind.kind.is_same_as(&found_kind.kind) => {
                                return ftd::interpreter::utils::e2(
                                    format!(
                                        "3.1 Expected kind `{:?}`, found: `{:?}`",
                                        ekind, found_kind
                                    )
                                    .as_str(),
                                    doc.name,
                                    value.line_number(),
                                );
                            }
                            _ => {}
                        }
                        let kind = get_kind(expected_kind, &found_kind);

                        return Ok(ftd::interpreter::StateWithThing::new_thing(Some(
                            ftd::interpreter::PropertyValue::Reference {
                                name: reference,
                                kind: kind.to_owned(),
                                source: PropertyValueSource::Global,
                                is_mutable: false,
                                line_number: 0,
                            },
                        )));
                    }
                }
                if let Some(kind) = expected_kind {
                    Ok(ftd::interpreter::StateWithThing::new_thing(Some(
                        ftd::interpreter::PropertyValue::Reference {
                            name: expression.trim_start_matches('$').to_string(),
                            kind: kind.to_owned(),
                            source: PropertyValueSource::Global,
                            is_mutable: false,
                            line_number: 0,
                        },
                    )))
                } else {
                    ftd::interpreter::utils::e2("Kind not found", doc.name, value.line_number())
                }
            }
            Ok(expression) if expression.eq(ftd::interpreter::FTD_SPECIAL_VALUE) => {
                Ok(ftd::interpreter::StateWithThing::new_thing(Some(
                    ftd::interpreter::PropertyValue::Reference {
                        name: "VALUE".to_string(),
                        kind: ftd::interpreter::Kind::string()
                            .into_optional()
                            .into_kind_data(),
                        source: PropertyValueSource::Global,
                        is_mutable: false,
                        line_number: 0,
                    },
                )))
            }
            Ok(expression) if expression.eq(ftd::interpreter::FTD_SPECIAL_CHECKED) => {
                Ok(ftd::interpreter::StateWithThing::new_thing(Some(
                    ftd::interpreter::PropertyValue::Reference {
                        name: "CHECKED".to_string(),
                        kind: ftd::interpreter::Kind::boolean()
                            .into_optional()
                            .into_kind_data(),
                        source: PropertyValueSource::Global,
                        is_mutable: false,
                        line_number: 0,
                    },
                )))
            }
            Ok(expression)
                if expression.starts_with(ftd::interpreter::utils::REFERENCE)
                    && ftd::interpreter::utils::get_function_name(
                        expression.trim_start_matches(ftd::interpreter::utils::REFERENCE),
                        doc.name,
                        value.line_number(),
                    )
                    .is_ok() =>
            {
                let expression = expression
                    .trim_start_matches(ftd::interpreter::utils::REFERENCE)
                    .to_string();

                let mut function_call = try_ok_state!(ftd::interpreter::FunctionCall::from_string(
                    expression.as_str(),
                    doc,
                    mutable,
                    definition_name_with_arguments,
                    loop_object_name_and_kind,
                    value.line_number(),
                )?);
                let found_kind = &function_call.kind;

                match expected_kind {
                    _ if function_call.module_name.is_some() => {}
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
                            loop_object_name_and_kind,
                        )?
                        .map(Some));
                    }
                    Some(ekind) if !ekind.kind.is_same_as(&found_kind.kind) => {
                        return ftd::interpreter::utils::e2(
                            format!("Expected kind `{:?}`, found: `{:?}`", ekind, found_kind)
                                .as_str(),
                            doc.name,
                            value.line_number(),
                        )
                    }
                    _ => {}
                }

                function_call.kind = get_kind(expected_kind, found_kind);
                if function_call.module_name.is_some() {
                    let (function_name, _) =
                        ftd::interpreter::utils::get_function_name_and_properties(
                            expression.as_str(),
                            doc.name,
                            value.line_number(),
                        )?;

                    ftd::interpreter::utils::insert_module_thing(
                        &function_call.kind,
                        function_name.as_str(),
                        function_call.name.as_str(),
                        definition_name_with_arguments,
                        value.line_number(),
                        doc,
                    )?;
                }

                Ok(ftd::interpreter::StateWithThing::new_thing(Some(
                    ftd::interpreter::PropertyValue::FunctionCall(function_call),
                )))
            }
            Ok(reference) if reference.starts_with(ftd::interpreter::utils::CLONE) => {
                let reference = reference
                    .trim_start_matches(ftd::interpreter::utils::CLONE)
                    .to_string();

                if expected_kind
                    .map(|ekind| ekind.kind.is_list() && reference.contains(','))
                    .unwrap_or(false)
                {
                    return Ok(ftd::interpreter::StateWithThing::new_thing(None));
                }

                let (source, found_kind, _) = try_ok_state!(doc.get_kind_with_argument(
                    reference.as_str(),
                    value.line_number(),
                    definition_name_with_arguments,
                    loop_object_name_and_kind,
                )?);

                match expected_kind {
                    _ if found_kind.is_module() => {}
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
                            loop_object_name_and_kind,
                        )?
                        .map(Some));
                    }
                    Some(ekind) if !ekind.kind.is_same_as(&found_kind.kind) => {
                        return ftd::interpreter::utils::e2(
                            format!("Expected kind `{:?}`, found: `{:?}`", ekind, found_kind)
                                .as_str(),
                            doc.name,
                            value.line_number(),
                        )
                    }
                    _ => {}
                }

                let reference_full_name = source.get_reference_name(reference.as_str(), doc);
                let kind = get_kind(expected_kind, &found_kind);

                if found_kind.is_module() {
                    ftd::interpreter::utils::insert_module_thing(
                        &kind,
                        reference.as_str(),
                        reference_full_name.as_str(),
                        definition_name_with_arguments,
                        value.line_number(),
                        doc,
                    )?;
                }

                Ok(ftd::interpreter::StateWithThing::new_thing(Some(
                    PropertyValue::Clone {
                        name: reference_full_name,
                        kind,
                        source,
                        is_mutable: mutable,
                        line_number: value.line_number(),
                    },
                )))
            }
            Ok(reference) if reference.starts_with(ftd::interpreter::utils::REFERENCE) => {
                let reference = reference
                    .trim_start_matches(ftd::interpreter::utils::REFERENCE)
                    .to_string();

                if expected_kind
                    .map(|ekind| ekind.kind.is_list() && reference.contains(','))
                    .unwrap_or(false)
                {
                    return Ok(ftd::interpreter::StateWithThing::new_thing(None));
                }

                let (source, found_kind, _) = try_ok_state!(doc.get_kind_with_argument(
                    reference.as_str(),
                    value.line_number(),
                    definition_name_with_arguments,
                    loop_object_name_and_kind,
                )?);

                match expected_kind {
                    _ if found_kind.is_module() => {}
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
                            loop_object_name_and_kind,
                        )?
                        .map(Some));
                    }
                    Some(ekind)
                        if ekind.kind.is_list()
                            && ekind.kind.ref_inner_list().is_same_as(&found_kind.kind) =>
                    {
                        return Ok(ftd::interpreter::StateWithThing::new_thing(None));
                    }
                    Some(ekind) if !ekind.kind.is_same_as(&found_kind.kind) => {
                        return ftd::interpreter::utils::e2(
                            format!("3.2 Expected kind `{:?}`, found: `{:?}`", ekind, found_kind)
                                .as_str(),
                            doc.name,
                            value.line_number(),
                        )
                    }
                    _ => {}
                }

                if mutable && !found_kind.is_module() {
                    let is_variable_mutable = if source.is_global() {
                        try_ok_state!(doc.search_variable(reference.as_str(), value.line_number())?)
                            .mutable
                    } else {
                        ftd::interpreter::utils::get_argument_for_reference_and_remaining(
                            reference.as_str(),
                            doc,
                            definition_name_with_arguments,
                            loop_object_name_and_kind,
                            value.line_number(),
                        )?
                        .unwrap()
                        .0
                        .mutable
                    };

                    if !is_variable_mutable {
                        return ftd::interpreter::utils::e2(
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
                let kind = get_kind(expected_kind, &found_kind);

                if found_kind.is_module() {
                    ftd::interpreter::utils::insert_module_thing(
                        &kind,
                        reference.as_str(),
                        reference_full_name.as_str(),
                        definition_name_with_arguments,
                        value.line_number(),
                        doc,
                    )?;
                }

                Ok(ftd::interpreter::StateWithThing::new_thing(Some(
                    PropertyValue::Reference {
                        name: reference_full_name,
                        kind,
                        source,
                        is_mutable: mutable,
                        line_number: value.line_number(),
                    },
                )))
            }
            _ => Ok(ftd::interpreter::StateWithThing::new_thing(None)),
        }
    }

    pub(crate) fn kind(&self) -> ftd::interpreter::Kind {
        match self {
            PropertyValue::Value { value, .. } => value.kind(),
            PropertyValue::Reference { kind, .. } => kind.kind.to_owned(),
            PropertyValue::Clone { kind, .. } => kind.kind.to_owned(),
            PropertyValue::FunctionCall(ftd::interpreter::FunctionCall { kind, .. }) => {
                kind.kind.to_owned()
            }
        }
    }

    pub(crate) fn new_none(
        kind: ftd::interpreter::KindData,
        line_number: usize,
    ) -> ftd::interpreter::PropertyValue {
        ftd::interpreter::PropertyValue::Value {
            value: ftd::interpreter::Value::new_none(kind),
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

    pub fn get_name(&self) -> Option<String> {
        match self {
            PropertyValueSource::Local(s) | PropertyValueSource::Loop(s) => Some(s.to_owned()),
            _ => None,
        }
    }

    pub fn get_reference_name(&self, name: &str, doc: &ftd::interpreter::TDoc) -> String {
        let name = name
            .strip_prefix(ftd::interpreter::utils::REFERENCE)
            .or_else(|| name.strip_prefix(ftd::interpreter::utils::CLONE))
            .unwrap_or(name);
        match self {
            PropertyValueSource::Global => doc.resolve_name(name),
            PropertyValueSource::Local(_) | PropertyValueSource::Loop(_) => {
                if name.contains('#') {
                    name.to_string()
                } else {
                    format!("{}#{}", doc.name, name)
                }
            } //TODO: Some different name for loop
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
        full_variant: String,
        value: Box<PropertyValue>, // Todo: Make it optional
    },
    List {
        data: Vec<PropertyValue>,
        kind: ftd::interpreter::KindData,
    },
    Optional {
        data: Box<Option<Value>>,
        kind: ftd::interpreter::KindData,
    },
    UI {
        name: String,
        kind: ftd::interpreter::KindData,
        component: ftd::interpreter::Component,
    },
    Module {
        name: String,
        things: ftd::Map<ftd::interpreter::ModuleThing>,
    },
}

impl Value {
    pub fn is_null(&self) -> bool {
        if let Self::String { text, .. } = self {
            return text.is_empty();
        }
        if let Self::Optional { data, .. } = self {
            let value = if let Some(ftd::interpreter::Value::String { text, .. }) = data.as_ref() {
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

    pub fn is_or_type_variant(&self, or_variant: &str) -> bool {
        matches!(self, Self::OrType { variant, .. } if or_variant.eq(variant))
    }

    pub fn record_fields(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::Map<PropertyValue>> {
        match self {
            Self::Record { fields, .. } => Ok(fields.to_owned()),
            t => ftd::interpreter::utils::e2(
                format!("Expected record, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    pub fn get_or_type(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<(&String, &String, &ftd::interpreter::PropertyValue)> {
        match self {
            Self::OrType {
                name,
                variant,
                value,
                ..
            } => Ok((name, variant, value)),
            t => ftd::interpreter::utils::e2(
                format!("Expected or-type, found: `{:?}`", t),
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

    pub(crate) fn ref_inner(&self) -> Option<&Self> {
        match self {
            Value::Optional { data, .. } => data.as_ref().as_ref(),
            t => Some(t),
        }
    }

    pub(crate) fn is_static(&self, doc: &ftd::interpreter::TDoc) -> bool {
        match self {
            ftd::interpreter::Value::Optional { data, .. } if data.is_some() => {
                data.clone().unwrap().is_static(doc)
            }
            ftd::interpreter::Value::List { data, .. } => {
                let mut is_static = true;
                for d in data {
                    if !d.is_static(doc) {
                        is_static = false;
                        break;
                    }
                }
                is_static
            }
            ftd::interpreter::Value::Record { fields, .. }
            | ftd::interpreter::Value::Object { values: fields, .. } => {
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

    pub(crate) fn kind(&self) -> ftd::interpreter::Kind {
        match self {
            Value::String { .. } => ftd::interpreter::Kind::string(),
            Value::Integer { .. } => ftd::interpreter::Kind::integer(),
            Value::Decimal { .. } => ftd::interpreter::Kind::decimal(),
            Value::Boolean { .. } => ftd::interpreter::Kind::boolean(),
            Value::Object { .. } => ftd::interpreter::Kind::object(),
            Value::Record { name, .. } => ftd::interpreter::Kind::record(name),
            Value::List { kind, .. } => kind.kind.clone().into_list(),
            Value::Optional { kind, .. } => ftd::interpreter::Kind::Optional {
                kind: Box::new(kind.kind.clone()),
            },
            Value::UI { name, .. } => ftd::interpreter::Kind::ui_with_name(name),
            Value::OrType {
                name,
                variant,
                full_variant,
                ..
            } => ftd::interpreter::Kind::or_type_with_variant(name, variant, full_variant),
            Value::Module { .. } => ftd::interpreter::Kind::module(),
        }
    }

    pub(crate) fn to_evalexpr_value(
        &self,
        doc: &ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<fastn_grammar::evalexpr::Value> {
        Ok(match self {
            Value::String { text } => fastn_grammar::evalexpr::Value::String(text.to_string()),
            Value::Integer { value } => fastn_grammar::evalexpr::Value::Int(*value),
            Value::Decimal { value } => fastn_grammar::evalexpr::Value::Float(*value),
            Value::Boolean { value } => fastn_grammar::evalexpr::Value::Boolean(*value),
            Value::List { data, .. } => {
                let mut values = vec![];
                for value in data {
                    let v = value
                        .clone()
                        .resolve(doc, line_number)?
                        .to_evalexpr_value(doc, value.line_number())?;
                    values.push(v);
                }
                fastn_grammar::evalexpr::Value::Tuple(values)
            }
            Value::Optional { data, .. } => {
                if let Some(data) = data.as_ref() {
                    data.to_evalexpr_value(doc, line_number)?
                } else {
                    fastn_grammar::evalexpr::Value::Empty
                }
            }
            t => unimplemented!("{:?}", t),
        })
    }

    pub(crate) fn from_evalexpr_value(
        value: fastn_grammar::evalexpr::Value,
        expected_kind: &ftd::interpreter::Kind,
        doc_name: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<Value> {
        Ok(match value {
            fastn_grammar::evalexpr::Value::String(text) if expected_kind.is_string() => {
                Value::String { text }
            }
            fastn_grammar::evalexpr::Value::Float(value) if expected_kind.is_decimal() => {
                Value::Decimal { value }
            }
            fastn_grammar::evalexpr::Value::Int(value) if expected_kind.is_integer() => {
                Value::Integer { value }
            }
            fastn_grammar::evalexpr::Value::Boolean(value) if expected_kind.is_boolean() => {
                Value::Boolean { value }
            }
            fastn_grammar::evalexpr::Value::Tuple(data) if expected_kind.is_list() => {
                let mut values = vec![];
                let val_kind = expected_kind.list_type(doc_name, line_number)?;
                for val in data {
                    values.push(ftd::interpreter::PropertyValue::Value {
                        value: Value::from_evalexpr_value(val, &val_kind, doc_name, line_number)?,
                        is_mutable: false,
                        line_number,
                    });
                }
                Value::List {
                    data: values,
                    kind: ftd::interpreter::KindData::new(val_kind),
                }
            }
            fastn_grammar::evalexpr::Value::Empty if expected_kind.is_optional() => {
                Value::Optional {
                    data: Box::new(None),
                    kind: ftd::interpreter::KindData::new(expected_kind.clone()),
                }
            }
            t => {
                return ftd::interpreter::utils::e2(
                    format!("Expected kind: `{:?}`, found: `{:?}`", expected_kind, t),
                    doc_name,
                    line_number,
                )
            }
        })
    }

    pub(crate) fn new_none(kind: ftd::interpreter::KindData) -> ftd::interpreter::Value {
        ftd::interpreter::Value::Optional {
            data: Box::new(None),
            kind,
        }
    }

    pub(crate) fn new_string(text: &str) -> ftd::interpreter::Value {
        ftd::interpreter::Value::String {
            text: text.to_string(),
        }
    }

    pub(crate) fn new_or_type(
        name: &str,
        variant: &str,
        full_variant: &str,
        value: ftd::interpreter::PropertyValue,
    ) -> ftd::interpreter::Value {
        ftd::interpreter::Value::OrType {
            name: name.to_string(),
            variant: variant.to_string(),
            full_variant: full_variant.to_string(),
            value: Box::new(value),
        }
    }

    pub(crate) fn into_evalexpr_value(
        self,
        doc: &ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<fastn_grammar::evalexpr::Value> {
        match self {
            ftd::interpreter::Value::String { text } => {
                Ok(fastn_grammar::evalexpr::Value::String(text))
            }
            ftd::interpreter::Value::Integer { value } => {
                Ok(fastn_grammar::evalexpr::Value::Int(value))
            }
            ftd::interpreter::Value::Decimal { value } => {
                Ok(fastn_grammar::evalexpr::Value::Float(value))
            }
            ftd::interpreter::Value::Boolean { value } => {
                Ok(fastn_grammar::evalexpr::Value::Boolean(value))
            }
            ftd::interpreter::Value::Optional { data, .. } => {
                if let Some(data) = data.as_ref() {
                    data.clone().into_evalexpr_value(doc)
                } else {
                    Ok(fastn_grammar::evalexpr::Value::Empty)
                }
            }
            ftd::interpreter::Value::OrType { value, .. } => {
                let line_number = value.line_number();
                value.resolve(doc, line_number)?.into_evalexpr_value(doc)
            }
            ftd::interpreter::Value::Record { .. } => {
                if let Ok(Some(value)) = ftd::interpreter::utils::get_value(doc, &self) {
                    Ok(fastn_grammar::evalexpr::Value::String(value.to_string()))
                } else {
                    unimplemented!("{:?}", self)
                }
            }
            ftd::interpreter::Value::List { data, .. } => {
                let mut values = vec![];
                for item in data {
                    let line_number = item.line_number();
                    values.push(item.resolve(doc, line_number)?.into_evalexpr_value(doc)?);
                }
                Ok(fastn_grammar::evalexpr::Value::Tuple(values))
            }
            t => unimplemented!("{:?}", t),
        }
    }

    pub fn string(&self, doc_id: &str, line_number: usize) -> ftd::interpreter::Result<String> {
        match self {
            ftd::interpreter::Value::String { text } => Ok(text.to_string()),
            t => ftd::interpreter::utils::e2(
                format!("Expected String, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    pub fn string_list(
        &self,
        doc: &ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<Vec<String>> {
        match self {
            ftd::interpreter::Value::List { data, kind } if kind.is_string() => {
                let mut values = vec![];
                for item in data.iter() {
                    let line_number = item.line_number();
                    values.push(
                        item.to_owned()
                            .resolve(doc, line_number)?
                            .string(doc.name, line_number)?,
                    );
                }
                Ok(values)
            }
            ftd::interpreter::Value::String { text } => Ok(vec![text.to_string()]),
            t => ftd::interpreter::utils::e2(
                format!("Expected String list, found: `{:?}`", t),
                doc.name,
                line_number,
            ),
        }
    }

    pub fn ui(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::Component> {
        match self {
            ftd::interpreter::Value::UI { component, .. } => Ok(component.to_owned()),
            t => ftd::interpreter::utils::e2(
                format!("Expected UI, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    pub fn optional_string(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<Option<String>> {
        match self {
            ftd::interpreter::Value::Optional { data, kind } if kind.is_string() => {
                if let Some(data) = data.as_ref() {
                    data.optional_string(doc_id, line_number)
                } else {
                    Ok(None)
                }
            }
            ftd::interpreter::Value::String { text } => Ok(Some(text.to_string())),
            t => ftd::interpreter::utils::e2(
                format!("Expected Optional String, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    pub fn decimal(&self, doc_id: &str, line_number: usize) -> ftd::interpreter::Result<f64> {
        match self {
            ftd::interpreter::Value::Decimal { value } => Ok(*value),
            t => ftd::interpreter::utils::e2(
                format!("Expected Decimal, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    pub fn integer(&self, doc_id: &str, line_number: usize) -> ftd::interpreter::Result<i64> {
        match self {
            ftd::interpreter::Value::Integer { value } => Ok(*value),
            t => ftd::interpreter::utils::e2(
                format!("Expected Integer, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    pub fn bool(&self, doc_id: &str, line_number: usize) -> ftd::interpreter::Result<bool> {
        match self {
            ftd::interpreter::Value::Boolean { value } => Ok(*value),
            t => ftd::interpreter::utils::e2(
                format!("Expected Boolean, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    pub fn module_name_optional(&self) -> Option<String> {
        match self {
            ftd::interpreter::Value::Module { name, .. } => Some(name.to_string()),
            _ => None,
        }
    }

    pub fn module_thing_optional(&self) -> Option<&ftd::Map<ftd::interpreter::ModuleThing>> {
        match self {
            ftd::interpreter::Value::Module { things, .. } => Some(things),
            _ => None,
        }
    }

    pub fn mut_module_optional(
        &mut self,
    ) -> Option<(&str, &mut ftd::Map<ftd::interpreter::ModuleThing>)> {
        match self {
            ftd::interpreter::Value::Module { name, things } => Some((name, things)),
            _ => None,
        }
    }

    pub fn optional_integer(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<Option<i64>> {
        match self {
            ftd::interpreter::Value::Optional { data, kind } if kind.is_integer() => {
                if let Some(data) = data.as_ref() {
                    data.optional_integer(doc_id, line_number)
                } else {
                    Ok(None)
                }
            }
            ftd::interpreter::Value::Integer { value } => Ok(Some(*value)),
            t => ftd::interpreter::utils::e2(
                format!("Expected Optional Integer, found: `{:?}`", t),
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

    pub fn to_serde_value(&self) -> Option<serde_json::Value> {
        match self {
            Value::String { text, .. } => Some(serde_json::Value::String(text.to_string())),
            Value::Integer { value } => Some(serde_json::json!(value)),
            Value::Decimal { value } => Some(serde_json::json!(value)),
            Value::Boolean { value } => Some(serde_json::Value::Bool(value.to_owned())),
            Value::Optional { data, .. } => {
                if let Some(data) = data.as_ref() {
                    data.to_serde_value()
                } else {
                    Some(serde_json::Value::Null)
                }
            }
            Value::Object { values } => {
                let mut new_values: ftd::Map<serde_json::Value> = Default::default();
                for (k, v) in values {
                    if let ftd::interpreter::PropertyValue::Value { value, .. } = v {
                        if let Some(v) = value.to_serde_value() {
                            new_values.insert(k.to_owned(), v);
                        }
                    }
                }
                serde_json::to_value(&new_values).ok()
            }
            Value::Record { fields, .. } => {
                let mut new_values: ftd::Map<serde_json::Value> = Default::default();
                for (k, v) in fields {
                    if let ftd::interpreter::PropertyValue::Value { value, .. } = v {
                        if let Some(v) = value.to_serde_value() {
                            new_values.insert(k.to_owned(), v);
                        }
                    }
                }
                serde_json::to_value(&new_values).ok()
            }
            Value::List { data, .. } => {
                let mut new_values: Vec<serde_json::Value> = Default::default();
                for v in data {
                    if let ftd::interpreter::PropertyValue::Value { value, .. } = v {
                        if let Some(v) = value.to_serde_value() {
                            new_values.push(v);
                        }
                    }
                }
                serde_json::to_value(&new_values).ok()
            }
            _ => None,
        }
    }

    pub fn to_string(&self) -> Option<String> {
        match self {
            Value::String { text } => Some(text.to_string()),
            Value::Integer { value } => Some(value.to_string()),
            Value::Decimal { value } => Some(value.to_string()),
            Value::Boolean { value } => Some(value.to_string()),
            Value::Optional { data, .. } => {
                if let Some(data) = data.as_ref() {
                    data.to_string()
                } else {
                    Some("".to_string())
                }
            }
            Value::Object { .. } | Value::Record { .. } | Value::List { .. } => {
                serde_json::to_string(&self.to_serde_value()).ok()
            }
            _ => None,
        }
    }
}

fn get_kind(
    expected_kind: Option<&ftd::interpreter::KindData>,
    found_kind: &ftd::interpreter::KindData,
) -> ftd::interpreter::KindData {
    if let Some(expected_kind) = expected_kind {
        if expected_kind.kind.ref_inner_list().ref_inner().is_ui() {
            expected_kind.clone()
        } else {
            let mut expected_kind = expected_kind.clone();
            if !found_kind.is_module() {
                expected_kind.kind = found_kind.kind.clone();
            }
            expected_kind
        }
    } else {
        found_kind.clone()
    }
}
