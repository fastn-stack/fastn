use ftd::interpreter::expression::ExpressionExt;
use ftd::interpreter::things::function::FunctionCallExt;
use ftd::interpreter::things::or_type::OrTypeVariantExt;
use ftd::interpreter::things::record::FieldExt;
use ftd::interpreter::{FunctionExt, KindExt};

pub trait PropertyValueExt {
    fn resolve(
        self,
        doc: &ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<fastn_resolved::Value>;

    fn resolve_with_inherited(
        self,
        doc: &ftd::interpreter::TDoc,
        line_number: usize,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::interpreter::Result<fastn_resolved::Value>;

    fn from_ast_value(
        value: ftd_ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
        mutable: bool,
        expected_kind: Option<&fastn_resolved::KindData>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_resolved::PropertyValue>>;

    fn from_ast_value_with_argument(
        value: ftd_ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
        is_mutable: bool,
        expected_kind: Option<&fastn_resolved::KindData>,
        definition_name_with_arguments: &mut Option<(&str, &mut [fastn_resolved::Argument])>,
        loop_object_name_and_kind: &Option<(String, fastn_resolved::Argument, Option<String>)>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_resolved::PropertyValue>>;

    fn reference_from_ast_value(
        value: ftd_ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
        mutable: bool,
        expected_kind: Option<&fastn_resolved::KindData>,
        definition_name_with_arguments: &mut Option<(&str, &mut [fastn_resolved::Argument])>,
        loop_object_name_and_kind: &Option<(String, fastn_resolved::Argument, Option<String>)>,
    ) -> ftd::interpreter::Result<
        ftd::interpreter::StateWithThing<Option<fastn_resolved::PropertyValue>>,
    >;

    fn value_from_ast_value(
        value: ftd_ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
        is_mutable: bool,
        expected_kind: Option<&fastn_resolved::KindData>,
        definition_name_with_arguments: &mut Option<(&str, &mut [fastn_resolved::Argument])>,
        loop_object_name_and_kind: &Option<(String, fastn_resolved::Argument, Option<String>)>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_resolved::PropertyValue>>;

    fn value(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<&fastn_resolved::Value>;

    fn from_record(
        record: &fastn_resolved::Record,
        value: ftd_ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
        is_mutable: bool,
        _expected_kind: &fastn_resolved::KindData,
        definition_name_with_arguments: &mut Option<(&str, &mut [fastn_resolved::Argument])>,
        loop_object_name_and_kind: &Option<(String, fastn_resolved::Argument, Option<String>)>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_resolved::PropertyValue>>;

    fn scan_value_from_ast_value(
        value: ftd_ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
        definition_name_with_arguments: Option<(&str, &[String])>,
        loop_object_name_and_kind: &Option<String>,
    ) -> ftd::interpreter::Result<()>;

    fn scan_ast_value_with_argument(
        value: ftd_ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
        definition_name_with_arguments: Option<(&str, &[String])>,
        loop_object_name_and_kind: &Option<String>,
    ) -> ftd::interpreter::Result<()>;

    fn scan_string_with_argument(
        value: &str,
        doc: &mut ftd::interpreter::TDoc,
        line_number: usize,
        definition_name_with_arguments: Option<(&str, &[String])>,
        loop_object_name_and_kind: &Option<String>,
    ) -> ftd::interpreter::Result<()>;

    fn scan_reference_from_ast_value(
        value: ftd_ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
        definition_name_with_arguments: Option<(&str, &[String])>,
        loop_object_name_and_kind: &Option<String>,
    ) -> ftd::interpreter::Result<bool>;

    fn scan_ast_value(
        value: ftd_ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()>;

    fn from_string_with_argument(
        value: &str,
        doc: &mut ftd::interpreter::TDoc,
        expected_kind: Option<&fastn_resolved::KindData>,
        mutable: bool,
        line_number: usize,
        definition_name_with_arguments: &mut Option<(&str, &mut [fastn_resolved::Argument])>,
        loop_object_name_and_kind: &Option<(String, fastn_resolved::Argument, Option<String>)>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_resolved::PropertyValue>>;

    fn is_static(&self, doc: &ftd::interpreter::TDoc) -> bool;

    fn value_mut(
        &mut self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<&mut fastn_resolved::Value>;
    fn to_ui_value(
        key: &str,
        value: ftd_ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
        definition_name_with_arguments: &mut Option<(&str, &mut [fastn_resolved::Argument])>,
        loop_object_name_and_kind: &Option<(String, fastn_resolved::Argument, Option<String>)>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_resolved::PropertyValue>>;
}
impl PropertyValueExt for fastn_resolved::PropertyValue {
    fn resolve(
        self,
        doc: &ftd::interpreter::TDoc,
        line_number: usize, // Todo: Remove this line number instead use self.line_number()
    ) -> ftd::interpreter::Result<fastn_resolved::Value> {
        self.resolve_with_inherited(doc, line_number, &Default::default())
    }

    fn resolve_with_inherited(
        self,
        doc: &ftd::interpreter::TDoc,
        line_number: usize,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::interpreter::Result<fastn_resolved::Value> {
        match self {
            fastn_resolved::PropertyValue::Value { value, .. } => Ok(value),
            fastn_resolved::PropertyValue::Reference { name, kind, .. }
            | fastn_resolved::PropertyValue::Clone { name, kind, .. } => {
                doc.resolve_with_inherited(name.as_str(), &kind, line_number, inherited_variables)
            }
            fastn_resolved::PropertyValue::FunctionCall(fastn_resolved::FunctionCall {
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

    fn from_ast_value(
        value: ftd_ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
        mutable: bool,
        expected_kind: Option<&fastn_resolved::KindData>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_resolved::PropertyValue>>
    {
        fastn_resolved::PropertyValue::from_ast_value_with_argument(
            value,
            doc,
            mutable,
            expected_kind,
            &mut None,
            &None,
        )
    }

    fn from_ast_value_with_argument(
        value: ftd_ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
        is_mutable: bool,
        expected_kind: Option<&fastn_resolved::KindData>,
        definition_name_with_arguments: &mut Option<(&str, &mut [fastn_resolved::Argument])>,
        loop_object_name_and_kind: &Option<(String, fastn_resolved::Argument, Option<String>)>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_resolved::PropertyValue>>
    {
        if let Some(reference) =
            try_ok_state!(fastn_resolved::PropertyValue::reference_from_ast_value(
                value.clone(),
                doc,
                is_mutable,
                expected_kind,
                definition_name_with_arguments,
                loop_object_name_and_kind,
            )?)
        {
            Ok(ftd::interpreter::StateWithThing::new_thing(reference))
        } else {
            fastn_resolved::PropertyValue::value_from_ast_value(
                value,
                doc,
                is_mutable,
                expected_kind,
                definition_name_with_arguments,
                loop_object_name_and_kind,
            )
        }
    }

    fn reference_from_ast_value(
        value: ftd_ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
        mutable: bool,
        expected_kind: Option<&fastn_resolved::KindData>,
        definition_name_with_arguments: &mut Option<(&str, &mut [fastn_resolved::Argument])>,
        loop_object_name_and_kind: &Option<(String, fastn_resolved::Argument, Option<String>)>,
    ) -> ftd::interpreter::Result<
        ftd::interpreter::StateWithThing<Option<fastn_resolved::PropertyValue>>,
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
                                return Ok(fastn_resolved::PropertyValue::value_from_ast_value(
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
                            fastn_resolved::PropertyValue::Reference {
                                name: reference,
                                kind: kind.to_owned(),
                                source: fastn_resolved::PropertyValueSource::Global,
                                is_mutable: false,
                                line_number: 0,
                            },
                        )));
                    }
                }
                if let Some(kind) = expected_kind {
                    Ok(ftd::interpreter::StateWithThing::new_thing(Some(
                        fastn_resolved::PropertyValue::Reference {
                            name: expression.trim_start_matches('$').to_string(),
                            kind: kind.to_owned(),
                            source: fastn_resolved::PropertyValueSource::Global,
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
                    fastn_resolved::PropertyValue::Reference {
                        name: "VALUE".to_string(),
                        kind: fastn_resolved::Kind::string()
                            .into_optional()
                            .into_kind_data(),
                        source: fastn_resolved::PropertyValueSource::Global,
                        is_mutable: false,
                        line_number: 0,
                    },
                )))
            }
            Ok(expression) if expression.eq(ftd::interpreter::FTD_SPECIAL_CHECKED) => {
                Ok(ftd::interpreter::StateWithThing::new_thing(Some(
                    fastn_resolved::PropertyValue::Reference {
                        name: "CHECKED".to_string(),
                        kind: fastn_resolved::Kind::boolean()
                            .into_optional()
                            .into_kind_data(),
                        source: fastn_resolved::PropertyValueSource::Global,
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

                let mut function_call = try_ok_state!(fastn_resolved::FunctionCall::from_string(
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
                        return Ok(fastn_resolved::PropertyValue::value_from_ast_value(
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
                    fastn_resolved::PropertyValue::FunctionCall(function_call),
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
                        return Ok(fastn_resolved::PropertyValue::value_from_ast_value(
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
                    fastn_resolved::PropertyValue::Clone {
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

                let reference_full_name = source.get_reference_name(reference.as_str(), doc);

                match expected_kind {
                    _ if found_kind.is_module() => {}
                    _ if expected_kind.map(|k| k.is_kwargs()).unwrap_or(false) => {
                        return Ok(ftd::interpreter::StateWithThing::new_thing(Some(
                            fastn_resolved::PropertyValue::Reference {
                                name: reference_full_name,
                                kind: found_kind,
                                source,
                                is_mutable: mutable,
                                line_number: value.line_number(),
                            },
                        )))
                    }
                    Some(ekind)
                        if !ekind.kind.is_same_as(&found_kind.kind)
                            && (ekind.kind.ref_inner().is_record()
                                || ekind.kind.ref_inner().is_or_type()) =>
                    {
                        return Ok(fastn_resolved::PropertyValue::value_from_ast_value(
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
                    fastn_resolved::PropertyValue::Reference {
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

    fn value_from_ast_value(
        value: ftd_ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
        is_mutable: bool,
        expected_kind: Option<&fastn_resolved::KindData>,
        definition_name_with_arguments: &mut Option<(&str, &mut [fastn_resolved::Argument])>,
        loop_object_name_and_kind: &Option<(String, fastn_resolved::Argument, Option<String>)>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_resolved::PropertyValue>>
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
            value: ftd_ast::VariableValue,
            doc: &mut ftd::interpreter::TDoc,
            is_mutable: bool,
            expected_kind: &fastn_resolved::KindData,
            definition_name_with_arguments: &mut Option<(&str, &mut [fastn_resolved::Argument])>,
            loop_object_name_and_kind: &Option<(String, fastn_resolved::Argument, Option<String>)>,
        ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_resolved::PropertyValue>>
        {
            Ok(match &expected_kind.kind.clone() {
                fastn_resolved::Kind::Optional { kind } => {
                    let kind = kind.clone().into_kind_data();
                    value.is_null();
                    match value {
                        ftd_ast::VariableValue::Optional {
                            value: ref ivalue, ..
                        } => match ivalue.as_ref() {
                            None => ftd::interpreter::StateWithThing::new_thing(
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Optional {
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
                fastn_resolved::Kind::Constant { kind } => {
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
                // Todo: For Template value has variable interpolation. Not just string
                fastn_resolved::Kind::String | fastn_resolved::Kind::Template => {
                    ftd::interpreter::StateWithThing::new_thing(
                        fastn_resolved::PropertyValue::Value {
                            value: fastn_resolved::Value::String {
                                text: value.string(doc.name)?.to_string(),
                            },
                            is_mutable,
                            line_number: value.line_number(),
                        },
                    )
                }
                fastn_resolved::Kind::Integer => ftd::interpreter::StateWithThing::new_thing(
                    fastn_resolved::PropertyValue::Value {
                        value: fastn_resolved::Value::Integer {
                            value: value.string(doc.name)?.parse()?,
                        },
                        is_mutable,
                        line_number: value.line_number(),
                    },
                ),
                fastn_resolved::Kind::Decimal => ftd::interpreter::StateWithThing::new_thing(
                    fastn_resolved::PropertyValue::Value {
                        value: fastn_resolved::Value::Decimal {
                            value: value.string(doc.name)?.parse()?,
                        },
                        is_mutable,
                        line_number: value.line_number(),
                    },
                ),
                fastn_resolved::Kind::Boolean => ftd::interpreter::StateWithThing::new_thing(
                    fastn_resolved::PropertyValue::Value {
                        value: fastn_resolved::Value::Boolean {
                            value: value.string(doc.name)?.parse()?,
                        },
                        is_mutable,
                        line_number: value.line_number(),
                    },
                ),
                fastn_resolved::Kind::List { kind } => {
                    let line_number = value.line_number();
                    let value_list = value.into_list(doc.name, kind.get_name())?;
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
                            try_ok_state!(fastn_resolved::PropertyValue::to_ui_value(
                                &key,
                                value,
                                doc,
                                definition_name_with_arguments,
                                loop_object_name_and_kind
                            )?)
                        } else {
                            try_ok_state!(fastn_resolved::PropertyValue::from_ast_value(
                                value,
                                doc,
                                is_mutable,
                                Some(&fastn_resolved::KindData {
                                    kind: kind.as_ref().clone(),
                                    caption: expected_kind.caption,
                                    body: expected_kind.body,
                                }),
                            )?)
                        });
                    }
                    ftd::interpreter::StateWithThing::new_thing(
                        fastn_resolved::PropertyValue::Value {
                            value: fastn_resolved::Value::List {
                                data: values,
                                kind: expected_kind.clone().inner_list(),
                            },
                            is_mutable,
                            line_number,
                        },
                    )
                }
                fastn_resolved::Kind::Record { name } if value.is_record() || value.is_string() => {
                    let record = try_ok_state!(doc.search_record(name, value.line_number())?);
                    fastn_resolved::PropertyValue::from_record(
                        &record,
                        value,
                        doc,
                        is_mutable,
                        expected_kind,
                        definition_name_with_arguments,
                        loop_object_name_and_kind,
                    )?
                }
                fastn_resolved::Kind::OrType { name, variant, .. } => {
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
                            fastn_resolved::OrTypeVariant::Constant(c) => return ftd::interpreter::utils::e2(format!("Cannot pass constant variant as property, variant: `{}`. Help: Pass variant as value instead", c.name), doc.name, c.line_number),
                            fastn_resolved::OrTypeVariant::AnonymousRecord(record) =>
                                try_ok_state!(fastn_resolved::PropertyValue::from_record(
                        record,
                        value,
                        doc,
                        is_mutable,
                        expected_kind,
                        definition_name_with_arguments,
                        loop_object_name_and_kind,
                    )?),
                            fastn_resolved::OrTypeVariant::Regular(regular) => {
                                let mut variant_name = variant_name.trim_start_matches(format!("{}.", variant.name()).as_str()).trim().to_string();
                                if variant_name.eq(&variant.name()) {
                                    variant_name = "".to_string();
                                }
                                let kind = if regular.kind.kind.ref_inner().is_or_type() && !variant_name.is_empty() {
                                    let (name, variant, _full_variant) = regular.kind.kind.get_or_type().unwrap();
                                    let variant_name = format!("{}.{}", name, variant_name);
                                    fastn_resolved::Kind::or_type_with_variant(name.as_str(), variant.unwrap_or_else(|| variant_name.clone()).as_str(), variant_name.as_str()).into_kind_data()
                                } else {
                                    regular.kind.to_owned()
                                };

                                try_ok_state!(
                            fastn_resolved::PropertyValue::from_ast_value_with_argument(
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
                            fastn_resolved::Value::new_or_type(
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
                            fastn_resolved::Value::new_or_type(
                                name,
                                constant.name.as_str(),
                                constant.name.as_str(),
                                value,
                            )
                            .into_property_value(false, line_number),
                        )
                    }
                }
                fastn_resolved::Kind::Module => ftd::interpreter::StateWithThing::new_thing(
                    fastn_resolved::PropertyValue::Value {
                        value: fastn_resolved::Value::Module {
                            name: doc.resolve_module_name(value.string(doc.name)?),
                            things: Default::default(),
                        },
                        is_mutable,
                        line_number: value.line_number(),
                    },
                ),
                fastn_resolved::Kind::KwArgs => {
                    let line_number = value.line_number();
                    let value_str = value.string(doc.name)?;
                    let value = if let Ok(value) = value_str.parse::<f64>() {
                        fastn_resolved::Value::Decimal { value }
                    } else if let Ok(value) = value_str.parse::<i64>() {
                        fastn_resolved::Value::Integer { value }
                    } else if let Ok(value) = value_str.parse::<bool>() {
                        fastn_resolved::Value::Boolean { value }
                    } else {
                        fastn_resolved::Value::String {
                            text: value_str.to_string(),
                        }
                    };

                    ftd::interpreter::StateWithThing::new_thing(
                        fastn_resolved::PropertyValue::Value {
                            value,
                            is_mutable,
                            line_number,
                        },
                    )
                }
                t => {
                    unimplemented!("t::{:?}  {:?}", t, value)
                }
            })
        }
    }

    fn value(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<&fastn_resolved::Value> {
        match self {
            fastn_resolved::PropertyValue::Value { value, .. } => Ok(value),
            t => ftd::interpreter::utils::e2(
                format!("Expected value found `{:?}`", t).as_str(),
                doc_id,
                line_number,
            ),
        }
    }

    fn from_record(
        record: &fastn_resolved::Record,
        value: ftd_ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
        is_mutable: bool,
        _expected_kind: &fastn_resolved::KindData,
        definition_name_with_arguments: &mut Option<(&str, &mut [fastn_resolved::Argument])>,
        loop_object_name_and_kind: &Option<(String, fastn_resolved::Argument, Option<String>)>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_resolved::PropertyValue>>
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
                ftd_ast::HeaderValues::new(vec![]),
                None,
                value.line_number(),
            )
        };

        // TODO: Check if the record name and the value kind are same
        let mut result_field: fastn_resolved::Map<fastn_resolved::PropertyValue> =
            Default::default();
        for field in record.fields.iter() {
            if field.is_caption() && caption.is_some() {
                let caption = caption.as_ref().unwrap().clone();
                let property_value =
                    try_ok_state!(fastn_resolved::PropertyValue::from_ast_value_with_argument(
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
                let property_value =
                    try_ok_state!(fastn_resolved::PropertyValue::from_ast_value_with_argument(
                        ftd_ast::VariableValue::String {
                            value: body.value.to_string(),
                            line_number: body.line_number,
                            source: ftd_ast::ValueSource::Body,
                            condition: None
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
                    fastn_resolved::PropertyValue::Value {
                        value: fastn_resolved::Value::Optional {
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
                let mut variable = ftd_ast::VariableValue::List {
                    value: vec![],
                    line_number: value.line_number(),
                    condition: None,
                };
                if let Some(header) = headers {
                    variable = header.value.clone();
                    variable.set_line_number(value.line_number())
                }
                let property_value =
                    try_ok_state!(fastn_resolved::PropertyValue::from_ast_value_with_argument(
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
                    fastn_resolved::PropertyValue::Reference {
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

            let property_value =
                try_ok_state!(fastn_resolved::PropertyValue::from_ast_value_with_argument(
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
            fastn_resolved::PropertyValue::Value {
                value: fastn_resolved::Value::Record {
                    name: name.to_string(),
                    fields: result_field,
                },
                is_mutable,
                line_number,
            },
        ))
    }

    fn scan_value_from_ast_value(
        value: ftd_ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
        definition_name_with_arguments: Option<(&str, &[String])>,
        loop_object_name_and_kind: &Option<String>,
    ) -> ftd::interpreter::Result<()> {
        match value {
            ftd_ast::VariableValue::Optional { value, .. } if value.is_some() => {
                fastn_resolved::PropertyValue::scan_ast_value_with_argument(
                    value.unwrap(),
                    doc,
                    definition_name_with_arguments,
                    loop_object_name_and_kind,
                )?;
            }
            ftd_ast::VariableValue::List { value, .. } => {
                for val in value {
                    fastn_resolved::PropertyValue::scan_ast_value_with_argument(
                        val.value,
                        doc,
                        definition_name_with_arguments,
                        loop_object_name_and_kind,
                    )?;
                }
            }
            ftd_ast::VariableValue::Record {
                caption,
                headers,
                body,
                values,
                ..
            } => {
                if let Some(caption) = caption.as_ref() {
                    fastn_resolved::PropertyValue::scan_ast_value_with_argument(
                        caption.to_owned(),
                        doc,
                        definition_name_with_arguments,
                        loop_object_name_and_kind,
                    )?;
                }
                for header in headers.0 {
                    fastn_resolved::PropertyValue::scan_ast_value_with_argument(
                        header.value,
                        doc,
                        definition_name_with_arguments,
                        loop_object_name_and_kind,
                    )?;
                    if let Some(condition) = header.condition {
                        fastn_resolved::Expression::scan_ast_condition(
                            ftd_ast::Condition::new(condition.as_str(), header.line_number),
                            definition_name_with_arguments,
                            loop_object_name_and_kind,
                            doc,
                        )?
                    }
                }
                if let Some(body) = body {
                    fastn_resolved::PropertyValue::scan_string_with_argument(
                        body.value.as_str(),
                        doc,
                        body.line_number,
                        definition_name_with_arguments,
                        loop_object_name_and_kind,
                    )?;
                }

                for val in values {
                    fastn_resolved::PropertyValue::scan_ast_value_with_argument(
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

    fn scan_ast_value_with_argument(
        value: ftd_ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
        definition_name_with_arguments: Option<(&str, &[String])>,
        loop_object_name_and_kind: &Option<String>,
    ) -> ftd::interpreter::Result<()> {
        if fastn_resolved::PropertyValue::scan_reference_from_ast_value(
            value.clone(),
            doc,
            definition_name_with_arguments,
            loop_object_name_and_kind,
        )? {
            Ok(())
        } else {
            fastn_resolved::PropertyValue::scan_value_from_ast_value(
                value,
                doc,
                definition_name_with_arguments,
                loop_object_name_and_kind,
            )
        }
    }

    fn scan_string_with_argument(
        value: &str,
        doc: &mut ftd::interpreter::TDoc,
        line_number: usize,
        definition_name_with_arguments: Option<(&str, &[String])>,
        loop_object_name_and_kind: &Option<String>,
    ) -> ftd::interpreter::Result<()> {
        let value = ftd_ast::VariableValue::String {
            value: value.to_string(),
            line_number,
            source: ftd_ast::ValueSource::Default,
            condition: None,
        };

        fastn_resolved::PropertyValue::scan_ast_value_with_argument(
            value,
            doc,
            definition_name_with_arguments,
            loop_object_name_and_kind,
        )
    }

    fn scan_reference_from_ast_value(
        value: ftd_ast::VariableValue,
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

                fastn_resolved::FunctionCall::scan_string(
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

    fn scan_ast_value(
        value: ftd_ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()> {
        fastn_resolved::PropertyValue::scan_ast_value_with_argument(value, doc, None, &None)
    }

    fn from_string_with_argument(
        value: &str,
        doc: &mut ftd::interpreter::TDoc,
        expected_kind: Option<&fastn_resolved::KindData>,
        mutable: bool,
        line_number: usize,
        definition_name_with_arguments: &mut Option<(&str, &mut [fastn_resolved::Argument])>,
        loop_object_name_and_kind: &Option<(String, fastn_resolved::Argument, Option<String>)>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_resolved::PropertyValue>>
    {
        let value = ftd_ast::VariableValue::String {
            value: value.to_string(),
            line_number,
            source: ftd_ast::ValueSource::Default,
            condition: None,
        };

        fastn_resolved::PropertyValue::from_ast_value_with_argument(
            value,
            doc,
            mutable,
            expected_kind,
            definition_name_with_arguments,
            loop_object_name_and_kind,
        )
    }

    fn is_static(&self, doc: &ftd::interpreter::TDoc) -> bool {
        match self {
            fastn_resolved::PropertyValue::Clone { .. } => true,
            fastn_resolved::PropertyValue::Reference { is_mutable, .. } if *is_mutable => true,
            fastn_resolved::PropertyValue::Reference {
                name, line_number, ..
            } => doc
                .get_variable(name, *line_number)
                .map(|v| v.is_static())
                .unwrap_or(true),
            fastn_resolved::PropertyValue::Value { value, .. } => value.is_static(doc),
            fastn_resolved::PropertyValue::FunctionCall(f) => {
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

    fn value_mut(
        &mut self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<&mut fastn_resolved::Value> {
        match self {
            fastn_resolved::PropertyValue::Value { value, .. } => Ok(value),
            t => ftd::interpreter::utils::e2(
                format!("Expected value found `{:?}`", t).as_str(),
                doc_id,
                line_number,
            ),
        }
    }

    fn to_ui_value(
        key: &str,
        value: ftd_ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
        definition_name_with_arguments: &mut Option<(&str, &mut [fastn_resolved::Argument])>,
        loop_object_name_and_kind: &Option<(String, fastn_resolved::Argument, Option<String>)>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_resolved::PropertyValue>>
    {
        use ftd::interpreter::ComponentExt;

        let line_number = value.line_number();

        if key.eq("ftd.ui") {
            return fastn_resolved::PropertyValue::from_ast_value_with_argument(
                value,
                doc,
                false,
                Some(&fastn_resolved::Kind::ui().into_kind_data()),
                definition_name_with_arguments,
                loop_object_name_and_kind,
            );
        }
        let ast_component =
            ftd_ast::ComponentInvocation::from_variable_value(key, value, doc.name)?;
        let component = try_ok_state!(fastn_resolved::ComponentInvocation::from_ast_component(
            ast_component,
            definition_name_with_arguments,
            doc,
        )?);

        Ok(ftd::interpreter::StateWithThing::new_thing(
            fastn_resolved::PropertyValue::Value {
                value: fastn_resolved::Value::UI {
                    name: component.name.to_string(),
                    kind: fastn_resolved::Kind::ui().into_kind_data(),
                    component,
                },
                is_mutable: false,
                line_number,
            },
        ))
    }
}

pub trait PropertyValueSourceExt {
    fn get_reference_name(&self, name: &str, doc: &ftd::interpreter::TDoc) -> String;
}
impl PropertyValueSourceExt for fastn_resolved::PropertyValueSource {
    fn get_reference_name(&self, name: &str, doc: &ftd::interpreter::TDoc) -> String {
        let name = name
            .strip_prefix(ftd::interpreter::utils::REFERENCE)
            .or_else(|| name.strip_prefix(ftd::interpreter::utils::CLONE))
            .unwrap_or(name);
        match self {
            fastn_resolved::PropertyValueSource::Global => doc.resolve_name(name),
            fastn_resolved::PropertyValueSource::Local(_)
            | fastn_resolved::PropertyValueSource::Loop(_) => {
                if name.contains('#') {
                    name.to_string()
                } else {
                    format!("{}#{}", doc.name.trim_end_matches('/'), name)
                }
            } //TODO: Some different name for loop
        }
    }
}

pub trait ValueExt {
    fn string(&self, doc_id: &str, line_number: usize) -> ftd::interpreter::Result<String>;
    fn decimal(&self, doc_id: &str, line_number: usize) -> ftd::interpreter::Result<f64>;
    fn integer(&self, doc_id: &str, line_number: usize) -> ftd::interpreter::Result<i64>;
    fn bool(&self, doc_id: &str, line_number: usize) -> ftd::interpreter::Result<bool>;
    fn optional_integer(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<Option<i64>>;
    fn string_list(
        &self,
        doc: &ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<Vec<String>>;
    fn get_or_type(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<(&String, &String, &fastn_resolved::PropertyValue)>;
    fn ui(
        &self,
        _doc_id: &str,
        _line_number: usize,
    ) -> ftd::interpreter::Result<fastn_resolved::ComponentInvocation>;
    fn record_fields(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::Map<fastn_resolved::PropertyValue>>;
    fn kwargs(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::Map<fastn_resolved::PropertyValue>>;
    fn into_evalexpr_value(
        self,
        doc: &ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<fastn_resolved::evalexpr::Value>;
    fn to_evalexpr_value(
        &self,
        doc: &ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<fastn_resolved::evalexpr::Value>;
    fn from_evalexpr_value(
        value: fastn_resolved::evalexpr::Value,
        expected_kind: &fastn_resolved::Kind,
        doc_name: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<fastn_resolved::Value>;
    fn is_static(&self, doc: &ftd::interpreter::TDoc) -> bool;
    fn module_thing_optional(&self) -> Option<&ftd::Map<fastn_resolved::ModuleThing>>;
    fn get_kwargs(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<&ftd::Map<fastn_resolved::PropertyValue>>;
    fn optional_string(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<Option<String>>;
    fn into_property_value(
        self,
        is_mutable: bool,
        line_number: usize,
    ) -> fastn_resolved::PropertyValue;
    fn to_serde_value(
        &self,
        doc: &ftd::interpreter::TDoc<'_>,
    ) -> ftd::interpreter::Result<Option<serde_json::Value>>;
    fn to_list(
        &self,
        doc: &ftd::interpreter::TDoc<'_>,
        _use_quotes: bool,
    ) -> ftd::interpreter::Result<Option<Vec<fastn_resolved::Value>>>;
    fn to_json_string(
        &self,
        doc: &ftd::interpreter::TDoc<'_>,
        use_quotes: bool,
    ) -> ftd::interpreter::Result<Option<String>>;
}
impl ValueExt for fastn_resolved::Value {
    fn string(&self, doc_id: &str, line_number: usize) -> ftd::interpreter::Result<String> {
        match self {
            fastn_resolved::Value::String { text } => Ok(text.to_string()),
            t => ftd::interpreter::utils::e2(
                format!("Expected String, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    fn decimal(&self, doc_id: &str, line_number: usize) -> ftd::interpreter::Result<f64> {
        match self {
            fastn_resolved::Value::Decimal { value } => Ok(*value),
            t => ftd::interpreter::utils::e2(
                format!("Expected Decimal, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    fn integer(&self, doc_id: &str, line_number: usize) -> ftd::interpreter::Result<i64> {
        match self {
            fastn_resolved::Value::Integer { value } => Ok(*value),
            t => ftd::interpreter::utils::e2(
                format!("Expected Integer, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    fn bool(&self, doc_id: &str, line_number: usize) -> ftd::interpreter::Result<bool> {
        match self {
            fastn_resolved::Value::Boolean { value } => Ok(*value),
            t => ftd::interpreter::utils::e2(
                format!("Expected Boolean, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    fn optional_integer(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<Option<i64>> {
        match self {
            fastn_resolved::Value::Optional { data, kind } if kind.is_integer() => {
                if let Some(data) = data.as_ref() {
                    data.optional_integer(doc_id, line_number)
                } else {
                    Ok(None)
                }
            }
            fastn_resolved::Value::Integer { value } => Ok(Some(*value)),
            t => ftd::interpreter::utils::e2(
                format!("Expected Optional Integer, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    fn string_list(
        &self,
        doc: &ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<Vec<String>> {
        match self {
            fastn_resolved::Value::List { data, kind } if kind.is_string() => {
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
            fastn_resolved::Value::String { text } => Ok(vec![text.to_string()]),
            t => ftd::interpreter::utils::e2(
                format!("Expected String list, found: `{:?}`", t),
                doc.name,
                line_number,
            ),
        }
    }

    fn get_or_type(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<(&String, &String, &fastn_resolved::PropertyValue)> {
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

    fn ui(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<fastn_resolved::ComponentInvocation> {
        match self {
            fastn_resolved::Value::UI { component, .. } => Ok(component.to_owned()),
            t => ftd::interpreter::utils::e2(
                format!("Expected UI, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    fn record_fields(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::Map<fastn_resolved::PropertyValue>> {
        match self {
            Self::Record { fields, .. } => Ok(fields.to_owned()),
            t => ftd::interpreter::utils::e2(
                format!("Expected record, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    fn kwargs(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::Map<fastn_resolved::PropertyValue>> {
        match self {
            Self::KwArgs { arguments } => Ok(arguments.to_owned()),
            t => ftd::interpreter::utils::e2(
                format!("Expected kwargs, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    fn into_evalexpr_value(
        self,
        doc: &ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<fastn_resolved::evalexpr::Value> {
        match self {
            fastn_resolved::Value::String { text } => {
                Ok(fastn_resolved::evalexpr::Value::String(text))
            }
            fastn_resolved::Value::Integer { value } => {
                Ok(fastn_resolved::evalexpr::Value::Int(value))
            }
            fastn_resolved::Value::Decimal { value } => {
                Ok(fastn_resolved::evalexpr::Value::Float(value))
            }
            fastn_resolved::Value::Boolean { value } => {
                Ok(fastn_resolved::evalexpr::Value::Boolean(value))
            }
            fastn_resolved::Value::Optional { data, .. } => {
                if let Some(data) = data.as_ref() {
                    data.clone().into_evalexpr_value(doc)
                } else {
                    Ok(fastn_resolved::evalexpr::Value::Empty)
                }
            }
            fastn_resolved::Value::OrType { value, .. } => {
                let line_number = value.line_number();
                value.resolve(doc, line_number)?.into_evalexpr_value(doc)
            }
            fastn_resolved::Value::Record { .. } => {
                match ftd::interpreter::utils::get_value(doc, &self) { Ok(Some(value)) => {
                    Ok(fastn_resolved::evalexpr::Value::String(value.to_string()))
                } _ => {
                    unimplemented!("{:?}", self)
                }}
            }
            fastn_resolved::Value::List { data, .. } => {
                let mut values = vec![];
                for item in data {
                    let line_number = item.line_number();
                    values.push(item.resolve(doc, line_number)?.into_evalexpr_value(doc)?);
                }
                Ok(fastn_resolved::evalexpr::Value::Tuple(values))
            }
            t => unimplemented!("{:?}", t),
        }
    }

    fn to_evalexpr_value(
        &self,
        doc: &ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<fastn_resolved::evalexpr::Value> {
        Ok(match self {
            fastn_resolved::Value::String { text } => {
                fastn_resolved::evalexpr::Value::String(text.to_string())
            }
            fastn_resolved::Value::Integer { value } => {
                fastn_resolved::evalexpr::Value::Int(*value)
            }
            fastn_resolved::Value::Decimal { value } => {
                fastn_resolved::evalexpr::Value::Float(*value)
            }
            fastn_resolved::Value::Boolean { value } => {
                fastn_resolved::evalexpr::Value::Boolean(*value)
            }
            fastn_resolved::Value::List { data, .. } => {
                let mut values = vec![];
                for value in data {
                    let v = value
                        .clone()
                        .resolve(doc, line_number)?
                        .to_evalexpr_value(doc, value.line_number())?;
                    values.push(v);
                }
                fastn_resolved::evalexpr::Value::Tuple(values)
            }
            fastn_resolved::Value::Optional { data, .. } => {
                if let Some(data) = data.as_ref() {
                    data.to_evalexpr_value(doc, line_number)?
                } else {
                    fastn_resolved::evalexpr::Value::Empty
                }
            }
            t => unimplemented!("{:?}", t),
        })
    }

    fn from_evalexpr_value(
        value: fastn_resolved::evalexpr::Value,
        expected_kind: &fastn_resolved::Kind,
        doc_name: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<fastn_resolved::Value> {
        Ok(match value {
            fastn_resolved::evalexpr::Value::String(text)
                if expected_kind.is_string() || expected_kind.is_template() =>
            {
                fastn_resolved::Value::String { text }
            }
            fastn_resolved::evalexpr::Value::Float(value) if expected_kind.is_decimal() => {
                fastn_resolved::Value::Decimal { value }
            }
            fastn_resolved::evalexpr::Value::Int(value) if expected_kind.is_integer() => {
                fastn_resolved::Value::Integer { value }
            }
            fastn_resolved::evalexpr::Value::Boolean(value) if expected_kind.is_boolean() => {
                fastn_resolved::Value::Boolean { value }
            }
            fastn_resolved::evalexpr::Value::Tuple(data) if expected_kind.is_list() => {
                let mut values = vec![];
                let val_kind = expected_kind.list_type(doc_name, line_number)?;
                for val in data {
                    values.push(fastn_resolved::PropertyValue::Value {
                        value: fastn_resolved::Value::from_evalexpr_value(
                            val,
                            &val_kind,
                            doc_name,
                            line_number,
                        )?,
                        is_mutable: false,
                        line_number,
                    });
                }
                fastn_resolved::Value::List {
                    data: values,
                    kind: fastn_resolved::KindData::new(val_kind),
                }
            }
            fastn_resolved::evalexpr::Value::Empty if expected_kind.is_optional() => {
                fastn_resolved::Value::Optional {
                    data: Box::new(None),
                    kind: fastn_resolved::KindData::new(expected_kind.clone()),
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

    fn is_static(&self, doc: &ftd::interpreter::TDoc) -> bool {
        match self {
            fastn_resolved::Value::Optional { data, .. } if data.is_some() => {
                data.clone().unwrap().is_static(doc)
            }
            fastn_resolved::Value::List { data, .. } => {
                let mut is_static = true;
                for d in data {
                    if !d.is_static(doc) {
                        is_static = false;
                        break;
                    }
                }
                is_static
            }
            fastn_resolved::Value::Record { fields, .. }
            | fastn_resolved::Value::Object { values: fields, .. }
            | fastn_resolved::Value::KwArgs {
                arguments: fields, ..
            } => {
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

    fn module_thing_optional(&self) -> Option<&ftd::Map<fastn_resolved::ModuleThing>> {
        match self {
            fastn_resolved::Value::Module { things, .. } => Some(things),
            _ => None,
        }
    }
    //////////////////////////

    fn get_kwargs(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<&ftd::Map<fastn_resolved::PropertyValue>> {
        match self {
            Self::KwArgs { arguments } => Ok(arguments),
            t => ftd::interpreter::utils::e2(
                format!("Expected kw-args, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    fn optional_string(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<Option<String>> {
        match self {
            fastn_resolved::Value::Optional { data, kind } if kind.is_string() => {
                if let Some(data) = data.as_ref() {
                    data.optional_string(doc_id, line_number)
                } else {
                    Ok(None)
                }
            }
            fastn_resolved::Value::String { text } => Ok(Some(text.to_string())),
            t => ftd::interpreter::utils::e2(
                format!("Expected Optional String, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    fn into_property_value(
        self,
        is_mutable: bool,
        line_number: usize,
    ) -> fastn_resolved::PropertyValue {
        fastn_resolved::PropertyValue::Value {
            value: self,
            is_mutable,
            line_number,
        }
    }

    fn to_serde_value(
        &self,
        doc: &ftd::interpreter::TDoc<'_>,
    ) -> ftd::interpreter::Result<Option<serde_json::Value>> {
        match self {
            fastn_resolved::Value::String { text, .. } => {
                Ok(Some(serde_json::Value::String(text.to_owned())))
            }
            fastn_resolved::Value::Integer { value } => Ok(Some(serde_json::json!(value))),
            fastn_resolved::Value::Decimal { value } => Ok(Some(serde_json::json!(value))),
            fastn_resolved::Value::Boolean { value } => {
                Ok(Some(serde_json::Value::Bool(value.to_owned())))
            }
            fastn_resolved::Value::Optional { data, .. } => {
                if let Some(data) = data.as_ref() {
                    data.to_serde_value(doc)
                } else {
                    Ok(Some(serde_json::Value::Null))
                }
            }
            fastn_resolved::Value::Object { values } => {
                let mut new_values: ftd::Map<serde_json::Value> = Default::default();
                for (k, v) in values {
                    let resolved_value = v.clone().resolve(doc, 0)?;
                    if let Some(v) = resolved_value.to_serde_value(doc)? {
                        new_values.insert(k.to_owned(), v);
                    }
                }
                Ok(Some(serde_json::to_value(&new_values)?))
            }
            fastn_resolved::Value::KwArgs { arguments } => {
                let mut new_values: ftd::Map<serde_json::Value> = Default::default();
                for (k, v) in arguments {
                    let resolved_value = v.clone().resolve(doc, 0)?;
                    if let Some(v) = resolved_value.to_serde_value(doc)? {
                        new_values.insert(k.to_owned(), v);
                    }
                }
                Ok(Some(serde_json::to_value(&new_values)?))
            }
            fastn_resolved::Value::Record { fields, .. } => {
                let mut new_values: ftd::Map<serde_json::Value> = Default::default();
                for (k, v) in fields {
                    let resolved_value = v.clone().resolve(doc, 0)?;
                    if let Some(v) = resolved_value.to_serde_value(doc)? {
                        new_values.insert(k.to_owned(), v);
                    }
                }
                Ok(Some(serde_json::to_value(&new_values)?))
            }
            fastn_resolved::Value::List { data, .. } => {
                let mut new_values: Vec<serde_json::Value> = Default::default();
                for v in data {
                    let resolved_value = v.clone().resolve(doc, 0)?;
                    if let Some(v) = resolved_value.to_serde_value(doc)? {
                        new_values.push(v);
                    }
                }
                Ok(Some(serde_json::to_value(&new_values)?))
            }
            _ => Ok(None),
        }
    }

    fn to_list(
        &self,
        doc: &ftd::interpreter::TDoc<'_>,
        _use_quotes: bool,
    ) -> ftd::interpreter::Result<Option<Vec<fastn_resolved::Value>>> {
        match self {
            fastn_resolved::Value::List { data, .. } => {
                let mut values = vec![];
                for d in data.iter() {
                    let resolved_value = d.clone().resolve(doc, d.line_number())?;
                    values.push(resolved_value);
                }
                Ok(Some(values))
            }
            _ => Ok(None),
        }
    }

    fn to_json_string(
        &self,
        doc: &ftd::interpreter::TDoc<'_>,
        use_quotes: bool,
    ) -> ftd::interpreter::Result<Option<String>> {
        match self {
            fastn_resolved::Value::String { text } => {
                if use_quotes {
                    Ok(Some(format!("\"{}\"", text)))
                } else {
                    Ok(Some(text.to_string()))
                }
            }
            fastn_resolved::Value::Integer { value } => Ok(Some(value.to_string())),
            fastn_resolved::Value::Decimal { value } => Ok(Some(value.to_string())),
            fastn_resolved::Value::Boolean { value } => Ok(Some(value.to_string())),
            fastn_resolved::Value::Optional { data, .. } => {
                if let Some(data) = data.as_ref() {
                    data.to_json_string(doc, use_quotes)
                } else {
                    Ok(Some("".to_string()))
                }
            }
            fastn_resolved::Value::Object { .. }
            | fastn_resolved::Value::Record { .. }
            | fastn_resolved::Value::List { .. }
            | fastn_resolved::Value::KwArgs { .. } => {
                Ok(Some(serde_json::to_string(&self.to_serde_value(doc)?)?))
            }
            _ => Ok(None),
        }
    }
}

fn get_kind(
    expected_kind: Option<&fastn_resolved::KindData>,
    found_kind: &fastn_resolved::KindData,
) -> fastn_resolved::KindData {
    if let Some(expected_kind) = expected_kind {
        if expected_kind.kind.ref_inner_list().ref_inner().is_ui() {
            expected_kind.clone()
        } else {
            let mut expected_kind = expected_kind.clone();
            if !found_kind.is_module() && !found_kind.is_or_type() {
                expected_kind.kind = found_kind.kind.clone();
            }
            expected_kind
        }
    } else {
        found_kind.clone()
    }
}
