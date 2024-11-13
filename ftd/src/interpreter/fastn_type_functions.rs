pub(crate) trait KindExt {
    fn list_type(
        &self,
        doc_name: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<fastn_type::Kind>;
}
impl KindExt for fastn_type::Kind {
    fn list_type(
        &self,
        doc_name: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<fastn_type::Kind> {
        match &self {
            fastn_type::Kind::List { kind } => Ok(kind.as_ref().clone()),
            t => ftd::interpreter::utils::e2(
                format!("Expected List, found: `{:?}`", t),
                doc_name,
                line_number,
            ),
        }
    }
}

pub trait KindDataExt {
    fn from_ast_kind(
        var_kind: ftd_ast::VariableKind,
        known_kinds: &ftd::Map<fastn_type::Kind>,
        doc: &mut ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_type::KindData>>;

    fn into_by_ast_modifier(self, modifier: &ftd_ast::VariableModifier) -> Self;
    fn scan_ast_kind(
        var_kind: ftd_ast::VariableKind,
        known_kinds: &ftd::Map<fastn_type::Kind>,
        doc: &mut ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<()>;
}
impl KindDataExt for fastn_type::KindData {
    fn scan_ast_kind(
        var_kind: ftd_ast::VariableKind,
        known_kinds: &ftd::Map<fastn_type::Kind>,
        doc: &mut ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<()> {
        let ast_kind = var_kind.kind;
        match ast_kind.as_ref() {
            "string" | "object" | "integer" | "decimal" | "boolean" | "void" | "ftd.ui"
            | "children" => Ok(()),
            k if known_kinds.contains_key(k) => Ok(()),
            k => doc.scan_thing(k, line_number),
        }
    }

    fn from_ast_kind(
        var_kind: ftd_ast::VariableKind,
        known_kinds: &ftd::Map<fastn_type::Kind>,
        doc: &mut ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_type::KindData>> {
        let mut ast_kind = ftd_p1::AccessModifier::remove_modifiers(var_kind.kind.as_str());
        // let mut ast_kind = var_kind.kind.clone();
        let (caption, body) = check_for_caption_and_body(&mut ast_kind);
        if ast_kind.is_empty() {
            if !(caption || body) {
                return Err(ftd::interpreter::utils::invalid_kind_error(
                    ast_kind,
                    doc.name,
                    line_number,
                ));
            }

            let mut kind_data = fastn_type::KindData {
                kind: fastn_type::Kind::String,
                caption,
                body,
            };

            if let Some(ref modifier) = var_kind.modifier {
                kind_data = kind_data.into_by_ast_modifier(modifier);
            }

            return Ok(ftd::interpreter::StateWithThing::new_thing(kind_data));
        }
        let kind = match ast_kind.as_ref() {
            "string" => fastn_type::Kind::string(),
            "object" => fastn_type::Kind::object(),
            "integer" => fastn_type::Kind::integer(),
            "decimal" => fastn_type::Kind::decimal(),
            "boolean" => fastn_type::Kind::boolean(),
            "void" => fastn_type::Kind::void(),
            "ftd.ui" => fastn_type::Kind::ui(),
            "module" => fastn_type::Kind::module(),
            "kw-args" => fastn_type::Kind::kwargs(),
            "children" => {
                if let Some(modifier) = var_kind.modifier {
                    return ftd::interpreter::utils::e2(
                        format!("Can't add modifier `{:?}`", modifier),
                        doc.name,
                        line_number,
                    );
                }
                fastn_type::Kind::List {
                    kind: Box::new(fastn_type::Kind::subsection_ui()),
                }
            }
            k if known_kinds.contains_key(k) => known_kinds.get(k).unwrap().to_owned(),
            k => match try_ok_state!(doc.search_thing(k, line_number)?) {
                ftd::interpreter::Thing::Record(r) => fastn_type::Kind::record(r.name.as_str()),
                ftd::interpreter::Thing::Component(_) => fastn_type::Kind::ui(),
                ftd::interpreter::Thing::OrType(o) => fastn_type::Kind::or_type(o.name.as_str()),
                ftd::interpreter::Thing::OrTypeWithVariant { or_type, variant } => {
                    fastn_type::Kind::or_type_with_variant(
                        or_type.as_str(),
                        variant.name().as_str(),
                        variant.name().as_str(),
                    )
                }
                ftd::interpreter::Thing::Variable(v) => v.kind.kind,
                t => {
                    return ftd::interpreter::utils::e2(
                        format!("Can't get find for `{:?}`", t),
                        doc.name,
                        line_number,
                    )
                }
            },
        };

        let mut kind_data = fastn_type::KindData {
            kind,
            caption,
            body,
        };

        if let Some(ref modifier) = var_kind.modifier {
            kind_data = kind_data.into_by_ast_modifier(modifier);
        }

        Ok(ftd::interpreter::StateWithThing::new_thing(kind_data))
    }

    fn into_by_ast_modifier(self, modifier: &ftd_ast::VariableModifier) -> Self {
        match modifier {
            ftd_ast::VariableModifier::Optional => self.optional(),
            ftd_ast::VariableModifier::List => self.list(),
            ftd_ast::VariableModifier::Constant => self.constant(),
        }
    }
}

pub(crate) trait PropertyValueExt {
    fn resolve(
        self,
        doc: &ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<fastn_type::Value>;

    fn resolve_with_inherited(
        self,
        doc: &ftd::interpreter::TDoc,
        line_number: usize,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::interpreter::Result<fastn_type::Value>;

    fn from_ast_value(
        value: ftd_ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
        mutable: bool,
        expected_kind: Option<&fastn_type::KindData>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_type::PropertyValue>>;

    fn from_ast_value_with_argument(
        value: ftd_ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
        is_mutable: bool,
        expected_kind: Option<&fastn_type::KindData>,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_type::PropertyValue>>;

    fn reference_from_ast_value(
        value: ftd_ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
        mutable: bool,
        expected_kind: Option<&fastn_type::KindData>,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<Option<fastn_type::PropertyValue>>>;

    fn value_from_ast_value(
        value: ftd_ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
        is_mutable: bool,
        expected_kind: Option<&fastn_type::KindData>,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_type::PropertyValue>>;

    fn value(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<&fastn_type::Value>;

    fn from_record(
        record: &ftd::interpreter::Record,
        value: ftd_ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
        is_mutable: bool,
        _expected_kind: &fastn_type::KindData,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_type::PropertyValue>>;

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
        expected_kind: Option<&fastn_type::KindData>,
        mutable: bool,
        line_number: usize,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_type::PropertyValue>>;

    fn is_static(&self, doc: &ftd::interpreter::TDoc) -> bool;

    fn value_mut(
        &mut self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<&mut fastn_type::Value>;

    fn value_optional(&self) -> Option<&fastn_type::Value>;
    fn to_ui_value(
        key: &str,
        value: ftd_ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_type::PropertyValue>>;
}
impl PropertyValueExt for fastn_type::PropertyValue {
    fn resolve(
        self,
        doc: &ftd::interpreter::TDoc,
        line_number: usize, // Todo: Remove this line number instead use self.line_number()
    ) -> ftd::interpreter::Result<fastn_type::Value> {
        self.resolve_with_inherited(doc, line_number, &Default::default())
    }

    fn resolve_with_inherited(
        self,
        doc: &ftd::interpreter::TDoc,
        line_number: usize,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::interpreter::Result<fastn_type::Value> {
        match self {
            fastn_type::PropertyValue::Value { value, .. } => Ok(value),
            fastn_type::PropertyValue::Reference { name, kind, .. }
            | fastn_type::PropertyValue::Clone { name, kind, .. } => {
                doc.resolve_with_inherited(name.as_str(), &kind, line_number, inherited_variables)
            }
            fastn_type::PropertyValue::FunctionCall(fastn_type::FunctionCall {
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
        expected_kind: Option<&fastn_type::KindData>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_type::PropertyValue>> {
        fastn_type::PropertyValue::from_ast_value_with_argument(
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
        expected_kind: Option<&fastn_type::KindData>,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_type::PropertyValue>> {
        if let Some(reference) = try_ok_state!(fastn_type::PropertyValue::reference_from_ast_value(
            value.clone(),
            doc,
            is_mutable,
            expected_kind,
            definition_name_with_arguments,
            loop_object_name_and_kind,
        )?) {
            Ok(ftd::interpreter::StateWithThing::new_thing(reference))
        } else {
            fastn_type::PropertyValue::value_from_ast_value(
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
        expected_kind: Option<&fastn_type::KindData>,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<Option<fastn_type::PropertyValue>>>
    {
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
                                return Ok(fastn_type::PropertyValue::value_from_ast_value(
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
                            fastn_type::PropertyValue::Reference {
                                name: reference,
                                kind: kind.to_owned(),
                                source: fastn_type::PropertyValueSource::Global,
                                is_mutable: false,
                                line_number: 0,
                            },
                        )));
                    }
                }
                if let Some(kind) = expected_kind {
                    Ok(ftd::interpreter::StateWithThing::new_thing(Some(
                        fastn_type::PropertyValue::Reference {
                            name: expression.trim_start_matches('$').to_string(),
                            kind: kind.to_owned(),
                            source: fastn_type::PropertyValueSource::Global,
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
                    fastn_type::PropertyValue::Reference {
                        name: "VALUE".to_string(),
                        kind: fastn_type::Kind::string().into_optional().into_kind_data(),
                        source: fastn_type::PropertyValueSource::Global,
                        is_mutable: false,
                        line_number: 0,
                    },
                )))
            }
            Ok(expression) if expression.eq(ftd::interpreter::FTD_SPECIAL_CHECKED) => {
                Ok(ftd::interpreter::StateWithThing::new_thing(Some(
                    fastn_type::PropertyValue::Reference {
                        name: "CHECKED".to_string(),
                        kind: fastn_type::Kind::boolean().into_optional().into_kind_data(),
                        source: fastn_type::PropertyValueSource::Global,
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

                let mut function_call = try_ok_state!(fastn_type::FunctionCall::from_string(
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
                        return Ok(fastn_type::PropertyValue::value_from_ast_value(
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
                    fastn_type::PropertyValue::FunctionCall(function_call),
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
                        return Ok(fastn_type::PropertyValue::value_from_ast_value(
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
                    fastn_type::PropertyValue::Clone {
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
                        return Ok(fastn_type::PropertyValue::value_from_ast_value(
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
                    fastn_type::PropertyValue::Reference {
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
        expected_kind: Option<&fastn_type::KindData>,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_type::PropertyValue>> {
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
            expected_kind: &fastn_type::KindData,
            definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
            loop_object_name_and_kind: &Option<(
                String,
                ftd::interpreter::Argument,
                Option<String>,
            )>,
        ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_type::PropertyValue>>
        {
            Ok(match &expected_kind.kind.clone() {
                fastn_type::Kind::Optional { kind } => {
                    let kind = kind.clone().into_kind_data();
                    value.is_null();
                    match value {
                        ftd_ast::VariableValue::Optional {
                            value: ref ivalue, ..
                        } => match ivalue.as_ref() {
                            None => ftd::interpreter::StateWithThing::new_thing(
                                fastn_type::PropertyValue::Value {
                                    value: fastn_type::Value::Optional {
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
                fastn_type::Kind::Constant { kind } => {
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
                fastn_type::Kind::String => {
                    ftd::interpreter::StateWithThing::new_thing(fastn_type::PropertyValue::Value {
                        value: fastn_type::Value::String {
                            text: value.string(doc.name)?,
                        },
                        is_mutable,
                        line_number: value.line_number(),
                    })
                }
                fastn_type::Kind::Integer => {
                    ftd::interpreter::StateWithThing::new_thing(fastn_type::PropertyValue::Value {
                        value: fastn_type::Value::Integer {
                            value: value.string(doc.name)?.parse()?,
                        },
                        is_mutable,
                        line_number: value.line_number(),
                    })
                }
                fastn_type::Kind::Decimal => {
                    ftd::interpreter::StateWithThing::new_thing(fastn_type::PropertyValue::Value {
                        value: fastn_type::Value::Decimal {
                            value: value.string(doc.name)?.parse()?,
                        },
                        is_mutable,
                        line_number: value.line_number(),
                    })
                }
                fastn_type::Kind::Boolean => {
                    ftd::interpreter::StateWithThing::new_thing(fastn_type::PropertyValue::Value {
                        value: fastn_type::Value::Boolean {
                            value: value.string(doc.name)?.parse()?,
                        },
                        is_mutable,
                        line_number: value.line_number(),
                    })
                }
                fastn_type::Kind::List { kind } => {
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
                            try_ok_state!(fastn_type::PropertyValue::to_ui_value(
                                &key,
                                value,
                                doc,
                                definition_name_with_arguments,
                                loop_object_name_and_kind
                            )?)
                        } else {
                            try_ok_state!(fastn_type::PropertyValue::from_ast_value(
                                value,
                                doc,
                                is_mutable,
                                Some(&fastn_type::KindData {
                                    kind: kind.as_ref().clone(),
                                    caption: expected_kind.caption,
                                    body: expected_kind.body,
                                }),
                            )?)
                        });
                    }
                    ftd::interpreter::StateWithThing::new_thing(fastn_type::PropertyValue::Value {
                        value: fastn_type::Value::List {
                            data: values,
                            kind: expected_kind.clone().inner_list(),
                        },
                        is_mutable,
                        line_number,
                    })
                }
                fastn_type::Kind::Record { name } if value.is_record() || value.is_string() => {
                    let record = try_ok_state!(doc.search_record(name, value.line_number())?);
                    fastn_type::PropertyValue::from_record(
                        &record,
                        value,
                        doc,
                        is_mutable,
                        expected_kind,
                        definition_name_with_arguments,
                        loop_object_name_and_kind,
                    )?
                }
                fastn_type::Kind::OrType { name, variant, .. } => {
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
                            ftd::interpreter::OrTypeVariant::AnonymousRecord(record) =>
                                try_ok_state!(fastn_type::PropertyValue::from_record(
                        record,
                        value,
                        doc,
                        is_mutable,
                        expected_kind,
                        definition_name_with_arguments,
                        loop_object_name_and_kind,
                    )?),
                            ftd::interpreter::OrTypeVariant::Regular(regular) => {
                                let mut variant_name = variant_name.trim_start_matches(format!("{}.", variant.name()).as_str()).trim().to_string();
                                if variant_name.eq(&variant.name()) {
                                    variant_name = "".to_string();
                                }
                                let kind = if regular.kind.kind.ref_inner().is_or_type() && !variant_name.is_empty() {
                                    let (name, variant, _full_variant) = regular.kind.kind.get_or_type().unwrap();
                                    let variant_name = format!("{}.{}", name, variant_name);
                                    fastn_type::Kind::or_type_with_variant(name.as_str(), variant.unwrap_or_else(|| variant_name.clone()).as_str(), variant_name.as_str()).into_kind_data()
                                } else {
                                    regular.kind.to_owned()
                                };

                                try_ok_state!(
                            fastn_type::PropertyValue::from_ast_value_with_argument(
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
                            fastn_type::Value::new_or_type(
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
                            fastn_type::Value::new_or_type(
                                name,
                                constant.name.as_str(),
                                constant.name.as_str(),
                                value,
                            )
                            .into_property_value(false, line_number),
                        )
                    }
                }
                fastn_type::Kind::Module => {
                    ftd::interpreter::StateWithThing::new_thing(fastn_type::PropertyValue::Value {
                        value: fastn_type::Value::Module {
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

    fn value(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<&fastn_type::Value> {
        match self {
            fastn_type::PropertyValue::Value { value, .. } => Ok(value),
            t => ftd::interpreter::utils::e2(
                format!("Expected value found `{:?}`", t).as_str(),
                doc_id,
                line_number,
            ),
        }
    }

    fn from_record(
        record: &ftd::interpreter::Record,
        value: ftd_ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
        is_mutable: bool,
        _expected_kind: &fastn_type::KindData,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_type::PropertyValue>> {
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
        let mut result_field: fastn_type::Map<fastn_type::PropertyValue> = Default::default();
        for field in record.fields.iter() {
            if field.is_caption() && caption.is_some() {
                let caption = caption.as_ref().unwrap().clone();
                let property_value =
                    try_ok_state!(fastn_type::PropertyValue::from_ast_value_with_argument(
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
                    try_ok_state!(fastn_type::PropertyValue::from_ast_value_with_argument(
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
                    fastn_type::PropertyValue::Value {
                        value: fastn_type::Value::Optional {
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
                    try_ok_state!(fastn_type::PropertyValue::from_ast_value_with_argument(
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
                    fastn_type::PropertyValue::Reference {
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
                try_ok_state!(fastn_type::PropertyValue::from_ast_value_with_argument(
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
            fastn_type::PropertyValue::Value {
                value: fastn_type::Value::Record {
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
                fastn_type::PropertyValue::scan_ast_value_with_argument(
                    value.unwrap(),
                    doc,
                    definition_name_with_arguments,
                    loop_object_name_and_kind,
                )?;
            }
            ftd_ast::VariableValue::List { value, .. } => {
                for val in value {
                    fastn_type::PropertyValue::scan_ast_value_with_argument(
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
                    fastn_type::PropertyValue::scan_ast_value_with_argument(
                        caption.to_owned(),
                        doc,
                        definition_name_with_arguments,
                        loop_object_name_and_kind,
                    )?;
                }
                for header in headers.0 {
                    fastn_type::PropertyValue::scan_ast_value_with_argument(
                        header.value,
                        doc,
                        definition_name_with_arguments,
                        loop_object_name_and_kind,
                    )?;
                    if let Some(condition) = header.condition {
                        ftd::interpreter::Expression::scan_ast_condition(
                            ftd_ast::Condition::new(condition.as_str(), header.line_number),
                            definition_name_with_arguments,
                            loop_object_name_and_kind,
                            doc,
                        )?
                    }
                }
                if let Some(body) = body {
                    fastn_type::PropertyValue::scan_string_with_argument(
                        body.value.as_str(),
                        doc,
                        body.line_number,
                        definition_name_with_arguments,
                        loop_object_name_and_kind,
                    )?;
                }

                for val in values {
                    fastn_type::PropertyValue::scan_ast_value_with_argument(
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
        if fastn_type::PropertyValue::scan_reference_from_ast_value(
            value.clone(),
            doc,
            definition_name_with_arguments,
            loop_object_name_and_kind,
        )? {
            Ok(())
        } else {
            fastn_type::PropertyValue::scan_value_from_ast_value(
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

        fastn_type::PropertyValue::scan_ast_value_with_argument(
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

    fn scan_ast_value(
        value: ftd_ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()> {
        fastn_type::PropertyValue::scan_ast_value_with_argument(value, doc, None, &None)
    }

    fn from_string_with_argument(
        value: &str,
        doc: &mut ftd::interpreter::TDoc,
        expected_kind: Option<&fastn_type::KindData>,
        mutable: bool,
        line_number: usize,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_type::PropertyValue>> {
        let value = ftd_ast::VariableValue::String {
            value: value.to_string(),
            line_number,
            source: ftd_ast::ValueSource::Default,
            condition: None,
        };

        fastn_type::PropertyValue::from_ast_value_with_argument(
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
            fastn_type::PropertyValue::Clone { .. } => true,
            fastn_type::PropertyValue::Reference { is_mutable, .. } if *is_mutable => true,
            fastn_type::PropertyValue::Reference {
                name, line_number, ..
            } => doc
                .get_variable(name, *line_number)
                .map(|v| v.is_static())
                .unwrap_or(true),
            fastn_type::PropertyValue::Value { value, .. } => value.is_static(doc),
            fastn_type::PropertyValue::FunctionCall(f) => {
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
    ) -> ftd::interpreter::Result<&mut fastn_type::Value> {
        match self {
            fastn_type::PropertyValue::Value { value, .. } => Ok(value),
            t => ftd::interpreter::utils::e2(
                format!("Expected value found `{:?}`", t).as_str(),
                doc_id,
                line_number,
            ),
        }
    }

    fn value_optional(&self) -> Option<&fastn_type::Value> {
        self.value("", 0).ok()
    }

    fn to_ui_value(
        key: &str,
        value: ftd_ast::VariableValue,
        doc: &mut ftd::interpreter::TDoc,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_type::PropertyValue>> {
        use ftd::interpreter::ComponentExt;

        let line_number = value.line_number();

        if key.eq("ftd.ui") {
            return fastn_type::PropertyValue::from_ast_value_with_argument(
                value,
                doc,
                false,
                Some(&fastn_type::Kind::ui().into_kind_data()),
                definition_name_with_arguments,
                loop_object_name_and_kind,
            );
        }
        let ast_component =
            ftd_ast::ComponentInvocation::from_variable_value(key, value, doc.name)?;
        let component = try_ok_state!(fastn_type::Component::from_ast_component(
            ast_component,
            definition_name_with_arguments,
            doc,
        )?);

        Ok(ftd::interpreter::StateWithThing::new_thing(
            fastn_type::PropertyValue::Value {
                value: fastn_type::Value::UI {
                    name: component.name.to_string(),
                    kind: fastn_type::Kind::ui().into_kind_data(),
                    component,
                },
                is_mutable: false,
                line_number,
            },
        ))
    }
}

pub(crate) trait ValueExt {
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
    ) -> ftd::interpreter::Result<(&String, &String, &fastn_type::PropertyValue)>;
    fn ui(
        &self,
        _doc_id: &str,
        _line_number: usize,
    ) -> ftd::interpreter::Result<fastn_type::Component>;
    fn record_fields(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::Map<fastn_type::PropertyValue>>;
    fn kwargs(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::Map<fastn_type::PropertyValue>>;
    fn into_evalexpr_value(
        self,
        doc: &ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<fastn_grammar::evalexpr::Value>;
    fn to_evalexpr_value(
        &self,
        doc: &ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<fastn_grammar::evalexpr::Value>;
    fn from_evalexpr_value(
        value: fastn_grammar::evalexpr::Value,
        expected_kind: &fastn_type::Kind,
        doc_name: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<fastn_type::Value>;
    fn is_static(&self, doc: &ftd::interpreter::TDoc) -> bool;
    fn module_thing_optional(&self) -> Option<&ftd::Map<ftd::interpreter::ModuleThing>>;
}
impl ValueExt for fastn_type::Value {
    fn string(&self, doc_id: &str, line_number: usize) -> ftd::interpreter::Result<String> {
        match self {
            fastn_type::Value::String { text } => Ok(text.to_string()),
            t => ftd::interpreter::utils::e2(
                format!("Expected String, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    fn decimal(&self, doc_id: &str, line_number: usize) -> ftd::interpreter::Result<f64> {
        match self {
            fastn_type::Value::Decimal { value } => Ok(*value),
            t => ftd::interpreter::utils::e2(
                format!("Expected Decimal, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    fn integer(&self, doc_id: &str, line_number: usize) -> ftd::interpreter::Result<i64> {
        match self {
            fastn_type::Value::Integer { value } => Ok(*value),
            t => ftd::interpreter::utils::e2(
                format!("Expected Integer, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    fn bool(&self, doc_id: &str, line_number: usize) -> ftd::interpreter::Result<bool> {
        match self {
            fastn_type::Value::Boolean { value } => Ok(*value),
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
            fastn_type::Value::Optional { data, kind } if kind.is_integer() => {
                if let Some(data) = data.as_ref() {
                    data.optional_integer(doc_id, line_number)
                } else {
                    Ok(None)
                }
            }
            fastn_type::Value::Integer { value } => Ok(Some(*value)),
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
            fastn_type::Value::List { data, kind } if kind.is_string() => {
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
            fastn_type::Value::String { text } => Ok(vec![text.to_string()]),
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
    ) -> ftd::interpreter::Result<(&String, &String, &fastn_type::PropertyValue)> {
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
        _doc_id: &str,
        _line_number: usize,
    ) -> ftd::interpreter::Result<fastn_type::Component> {
        todo!()
        // match self {
        //     fastn_type::Value::UI { component, .. } => Ok(component.to_owned()),
        //     t => ftd::interpreter::utils::e2(
        //         format!("Expected UI, found: `{:?}`", t),
        //         doc_id,
        //         line_number,
        //     ),
        // }
    }

    fn record_fields(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::Map<fastn_type::PropertyValue>> {
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
    ) -> ftd::interpreter::Result<ftd::Map<fastn_type::PropertyValue>> {
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
    ) -> ftd::interpreter::Result<fastn_grammar::evalexpr::Value> {
        match self {
            fastn_type::Value::String { text } => Ok(fastn_grammar::evalexpr::Value::String(text)),
            fastn_type::Value::Integer { value } => Ok(fastn_grammar::evalexpr::Value::Int(value)),
            fastn_type::Value::Decimal { value } => {
                Ok(fastn_grammar::evalexpr::Value::Float(value))
            }
            fastn_type::Value::Boolean { value } => {
                Ok(fastn_grammar::evalexpr::Value::Boolean(value))
            }
            fastn_type::Value::Optional { data, .. } => {
                if let Some(data) = data.as_ref() {
                    data.clone().into_evalexpr_value(doc)
                } else {
                    Ok(fastn_grammar::evalexpr::Value::Empty)
                }
            }
            fastn_type::Value::OrType { value, .. } => {
                let line_number = value.line_number();
                value.resolve(doc, line_number)?.into_evalexpr_value(doc)
            }
            fastn_type::Value::Record { .. } => {
                if let Ok(Some(value)) = ftd::interpreter::utils::get_value(doc, &self) {
                    Ok(fastn_grammar::evalexpr::Value::String(value.to_string()))
                } else {
                    unimplemented!("{:?}", self)
                }
            }
            fastn_type::Value::List { data, .. } => {
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

    fn to_evalexpr_value(
        &self,
        doc: &ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<fastn_grammar::evalexpr::Value> {
        Ok(match self {
            fastn_type::Value::String { text } => {
                fastn_grammar::evalexpr::Value::String(text.to_string())
            }
            fastn_type::Value::Integer { value } => fastn_grammar::evalexpr::Value::Int(*value),
            fastn_type::Value::Decimal { value } => fastn_grammar::evalexpr::Value::Float(*value),
            fastn_type::Value::Boolean { value } => fastn_grammar::evalexpr::Value::Boolean(*value),
            fastn_type::Value::List { data, .. } => {
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
            fastn_type::Value::Optional { data, .. } => {
                if let Some(data) = data.as_ref() {
                    data.to_evalexpr_value(doc, line_number)?
                } else {
                    fastn_grammar::evalexpr::Value::Empty
                }
            }
            t => unimplemented!("{:?}", t),
        })
    }

    fn from_evalexpr_value(
        value: fastn_grammar::evalexpr::Value,
        expected_kind: &fastn_type::Kind,
        doc_name: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<fastn_type::Value> {
        use ftd::interpreter::fastn_type_functions::KindExt;

        Ok(match value {
            fastn_grammar::evalexpr::Value::String(text) if expected_kind.is_string() => {
                fastn_type::Value::String { text }
            }
            fastn_grammar::evalexpr::Value::Float(value) if expected_kind.is_decimal() => {
                fastn_type::Value::Decimal { value }
            }
            fastn_grammar::evalexpr::Value::Int(value) if expected_kind.is_integer() => {
                fastn_type::Value::Integer { value }
            }
            fastn_grammar::evalexpr::Value::Boolean(value) if expected_kind.is_boolean() => {
                fastn_type::Value::Boolean { value }
            }
            fastn_grammar::evalexpr::Value::Tuple(data) if expected_kind.is_list() => {
                let mut values = vec![];
                let val_kind = expected_kind.list_type(doc_name, line_number)?;
                for val in data {
                    values.push(fastn_type::PropertyValue::Value {
                        value: fastn_type::Value::from_evalexpr_value(
                            val,
                            &val_kind,
                            doc_name,
                            line_number,
                        )?,
                        is_mutable: false,
                        line_number,
                    });
                }
                fastn_type::Value::List {
                    data: values,
                    kind: fastn_type::KindData::new(val_kind),
                }
            }
            fastn_grammar::evalexpr::Value::Empty if expected_kind.is_optional() => {
                fastn_type::Value::Optional {
                    data: Box::new(None),
                    kind: fastn_type::KindData::new(expected_kind.clone()),
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
            fastn_type::Value::Optional { data, .. } if data.is_some() => {
                data.clone().unwrap().is_static(doc)
            }
            fastn_type::Value::List { data, .. } => {
                let mut is_static = true;
                for d in data {
                    if !d.is_static(doc) {
                        is_static = false;
                        break;
                    }
                }
                is_static
            }
            fastn_type::Value::Record { fields, .. }
            | fastn_type::Value::Object { values: fields, .. }
            | fastn_type::Value::KwArgs {
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

    fn module_thing_optional(&self) -> Option<&ftd::Map<ftd::interpreter::ModuleThing>> {
        match self {
            fastn_type::Value::Module { things, .. } => Some(things),
            _ => None,
        }
    }
}

pub(crate) trait PropertyValueSourceExt {
    fn get_reference_name(&self, name: &str, doc: &ftd::interpreter::TDoc) -> String;
}
impl PropertyValueSourceExt for fastn_type::PropertyValueSource {
    fn get_reference_name(&self, name: &str, doc: &ftd::interpreter::TDoc) -> String {
        let name = name
            .strip_prefix(ftd::interpreter::utils::REFERENCE)
            .or_else(|| name.strip_prefix(ftd::interpreter::utils::CLONE))
            .unwrap_or(name);
        match self {
            fastn_type::PropertyValueSource::Global => doc.resolve_name(name),
            fastn_type::PropertyValueSource::Local(_)
            | fastn_type::PropertyValueSource::Loop(_) => {
                if name.contains('#') {
                    name.to_string()
                } else {
                    format!("{}#{}", doc.name.trim_end_matches('/'), name)
                }
            } //TODO: Some different name for loop
        }
    }
}

pub(crate) trait FunctionCallExt {
    fn from_string(
        value: &str,
        doc: &mut ftd::interpreter::TDoc,
        mutable: bool,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_type::FunctionCall>>;

    fn scan_string(
        value: &str,
        doc: &mut ftd::interpreter::TDoc,
        definition_name_with_arguments: Option<(&str, &[String])>,
        loop_object_name_and_kind: &Option<String>,
        line_number: usize,
    ) -> ftd::interpreter::Result<()>;
}

impl FunctionCallExt for fastn_type::FunctionCall {
    fn from_string(
        value: &str,
        doc: &mut ftd::interpreter::TDoc,
        mutable: bool,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_type::FunctionCall>> {
        let expression = value
            .trim_start_matches(ftd::interpreter::utils::REFERENCE)
            .to_string();

        let (function_name, properties) =
            ftd::interpreter::utils::get_function_name_and_properties(
                expression.as_str(),
                doc.name,
                line_number,
            )?;

        let mut resolved_function_name = function_name.clone();
        let initial_kind_with_remaining_and_source =
            ftd::interpreter::utils::get_argument_for_reference_and_remaining(
                resolved_function_name.as_str(),
                doc,
                definition_name_with_arguments,
                loop_object_name_and_kind,
                line_number,
            )?;

        let mut module_name = None;
        let mut source = fastn_type::PropertyValueSource::Global;
        if let Some((ref argument, ref function, source_)) = initial_kind_with_remaining_and_source
        {
            source = source_;
            if argument.kind.is_module() {
                if let Some(fastn_type::PropertyValue::Value {
                    value: fastn_type::Value::Module { ref name, .. },
                    ..
                }) = argument.value
                {
                    if let Some(function) = function {
                        module_name = Some((
                            name.to_string(),
                            source
                                .get_name()
                                .map(|v| {
                                    source.get_reference_name(
                                        format!("{v}.{}", argument.name).as_str(),
                                        doc,
                                    )
                                })
                                .unwrap_or(argument.name.to_string()),
                        ));
                        resolved_function_name = format!("{name}#{function}");
                    } else {
                        return ftd::interpreter::utils::e2(
                            format!("No function found: {}", expression),
                            doc.name,
                            argument.line_number,
                        );
                    }
                } else {
                    return ftd::interpreter::utils::e2(
                        format!("Default value not found for module {}", argument.name),
                        doc.name,
                        argument.line_number,
                    );
                }
            }
        }

        let function =
            try_ok_state!(doc.search_function(resolved_function_name.as_str(), line_number)?);
        let mut values: ftd::Map<fastn_type::PropertyValue> = Default::default();
        let mut order = vec![];

        for argument in function.arguments.iter() {
            let property_value = if let Some((property, property_key, mutable)) =
                properties.iter().find_map(|(key, property)| {
                    let (property_key, mutable) =
                        if let Some(key) = key.strip_prefix(ftd::interpreter::utils::REFERENCE) {
                            (key.to_string(), true)
                        } else {
                            (key.to_string(), false)
                        };
                    if argument.name.eq(property_key.as_str()) {
                        Some((property.to_string(), property_key, mutable))
                    } else {
                        None
                    }
                }) {
                if !(mutable.eq(&argument.mutable)) {
                    return ftd::interpreter::utils::e2(
                        format!(
                            "Mutability conflict in argument `{}` for function `{}`",
                            property_key, resolved_function_name
                        ),
                        doc.name,
                        line_number,
                    );
                }
                try_ok_state!(fastn_type::PropertyValue::from_ast_value_with_argument(
                    ftd_ast::VariableValue::String {
                        value: property,
                        line_number,
                        source: ftd_ast::ValueSource::Default,
                        condition: None
                    },
                    doc,
                    mutable,
                    Some(&argument.kind),
                    definition_name_with_arguments,
                    loop_object_name_and_kind,
                )?)
            } else {
                match argument.value {
                    Some(ref value) => value.clone(),
                    None if argument.kind.is_optional() => fastn_type::PropertyValue::new_none(
                        argument.kind.clone(),
                        argument.line_number,
                    ),
                    _ => {
                        return ftd::interpreter::utils::e2(
                            format!(
                                "Cannot find argument `{}` in function `{}`",
                                argument.name, function_name
                            ),
                            doc.name,
                            line_number,
                        )
                    }
                }
            };
            values.insert(argument.name.to_string(), property_value);
            order.push(argument.name.to_string());
        }

        let reference_full_name = source.get_reference_name(function_name.as_str(), doc);

        Ok(ftd::interpreter::StateWithThing::new_thing(
            fastn_type::FunctionCall::new(
                reference_full_name.as_str(),
                function.return_kind.clone(),
                mutable,
                line_number,
                values,
                order,
                module_name,
            ),
        ))
    }

    fn scan_string(
        value: &str,
        doc: &mut ftd::interpreter::TDoc,
        definition_name_with_arguments: Option<(&str, &[String])>,
        loop_object_name_and_kind: &Option<String>,
        line_number: usize,
    ) -> ftd::interpreter::Result<()> {
        let expression = value
            .trim_start_matches(ftd::interpreter::utils::REFERENCE)
            .to_string();

        let (function_name, properties) =
            ftd::interpreter::utils::get_function_name_and_properties(
                expression.as_str(),
                doc.name,
                line_number,
            )?;

        let initial_kind_with_remaining_and_source =
            ftd::interpreter::utils::is_argument_in_component_or_loop(
                function_name.as_str(),
                doc,
                definition_name_with_arguments,
                loop_object_name_and_kind,
            );

        if !initial_kind_with_remaining_and_source {
            doc.scan_initial_thing(function_name.as_str(), line_number)?;
        }

        for (_, value) in properties.iter() {
            fastn_type::PropertyValue::scan_string_with_argument(
                value,
                doc,
                line_number,
                definition_name_with_arguments,
                loop_object_name_and_kind,
            )?;
        }

        Ok(())
    }
}

pub(crate) trait PropertyExt {
    fn resolve(
        &self,
        doc: &ftd::interpreter::TDoc,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::interpreter::Result<Option<fastn_type::Value>>;
    fn from_ast_properties_and_children(
        ast_properties: Vec<ftd_ast::Property>,
        ast_children: Vec<ftd_ast::ComponentInvocation>,
        component_name: &str,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
        doc: &mut ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<Vec<fastn_type::Property>>>;
    fn get_argument_for_children(
        component_arguments: &[ftd::interpreter::Argument],
    ) -> Option<&ftd::interpreter::Argument>;
    fn from_ast_children(
        ast_children: Vec<ftd_ast::ComponentInvocation>,
        component_name: &str,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<Option<fastn_type::Property>>>;
    fn scan_ast_children(
        ast_children: Vec<ftd_ast::ComponentInvocation>,
        definition_name_with_arguments: Option<(&str, &[String])>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()>;
    fn scan_ast_properties(
        ast_properties: Vec<ftd_ast::Property>,
        definition_name_with_arguments: Option<(&str, &[String])>,
        loop_object_name_and_kind: &Option<String>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()>;
    fn scan_ast_property(
        ast_property: ftd_ast::Property,
        definition_name_with_arguments: Option<(&str, &[String])>,
        loop_object_name_and_kind: &Option<String>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()>;
    fn from_ast_properties(
        ast_properties: Vec<ftd_ast::Property>,
        component_name: &str,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
        doc: &mut ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<Vec<fastn_type::Property>>>;
    fn from_ast_property(
        ast_property: ftd_ast::Property,
        component_name: &str,
        component_arguments: &[ftd::interpreter::Argument],
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_type::Property>>;
    fn get_argument_for_property(
        ast_property: &ftd_ast::Property,
        component_name: &str,
        component_argument: &[ftd::interpreter::Argument],
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<ftd::interpreter::Argument>>;
    fn get_local_argument(&self, component_name: &str) -> Option<String>;
}

impl PropertyExt for fastn_type::Property {
    fn resolve(
        &self,
        doc: &ftd::interpreter::TDoc,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::interpreter::Result<Option<fastn_type::Value>> {
        use ftd::interpreter::PropertyValueExt;

        Ok(match self.condition {
            Some(ref condition) if !condition.eval(doc)? => None,
            _ => Some(self.value.clone().resolve_with_inherited(
                doc,
                self.line_number,
                inherited_variables,
            )?),
        })
    }

    fn from_ast_properties_and_children(
        ast_properties: Vec<ftd_ast::Property>,
        ast_children: Vec<ftd_ast::ComponentInvocation>,
        component_name: &str,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
        doc: &mut ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<Vec<fastn_type::Property>>> {
        let mut properties = try_ok_state!(fastn_type::Property::from_ast_properties(
            ast_properties,
            component_name,
            definition_name_with_arguments,
            loop_object_name_and_kind,
            doc,
            line_number,
        )?);

        // todo: validate_duplicate_properties() a property cannot be repeat if it's not list

        validate_children_kind_property_against_children(
            properties.as_slice(),
            ast_children.as_slice(),
            doc.name,
        )?;

        if let Some(property) = try_ok_state!(fastn_type::Property::from_ast_children(
            ast_children,
            component_name,
            definition_name_with_arguments,
            doc,
        )?) {
            properties.push(property);
        }

        return Ok(ftd::interpreter::StateWithThing::new_thing(properties));

        fn validate_children_kind_property_against_children(
            properties: &[fastn_type::Property],
            ast_children: &[ftd_ast::ComponentInvocation],
            doc_id: &str,
        ) -> ftd::interpreter::Result<()> {
            use itertools::Itertools;

            let properties = properties
                .iter()
                .filter(|v| v.value.kind().inner_list().is_subsection_ui())
                .collect_vec();

            if properties.is_empty() {
                return Ok(());
            }

            let first_property = properties.first().unwrap();

            if properties.len() > 1 {
                return ftd::interpreter::utils::e2(
                    "Can't pass multiple children",
                    doc_id,
                    first_property.line_number,
                );
            }

            if !ast_children.is_empty() {
                return ftd::interpreter::utils::e2(
                    "Can't have children passed in both subsection and header",
                    doc_id,
                    first_property.line_number,
                );
            }

            if first_property.condition.is_some() {
                return ftd::interpreter::utils::e2(
                    "Not supporting condition for children",
                    doc_id,
                    first_property.line_number,
                );
            }

            Ok(())
        }
    }

    fn get_argument_for_children(
        component_arguments: &[ftd::interpreter::Argument],
    ) -> Option<&ftd::interpreter::Argument> {
        component_arguments
            .iter()
            .find(|v| v.kind.kind.clone().inner_list().is_subsection_ui())
    }

    fn from_ast_children(
        ast_children: Vec<ftd_ast::ComponentInvocation>,
        component_name: &str,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<Option<fastn_type::Property>>>
    {
        if ast_children.is_empty() {
            return Ok(ftd::interpreter::StateWithThing::new_thing(None));
        }

        let line_number = ast_children.first().unwrap().line_number;
        let component_arguments = try_ok_state!(ftd::interpreter::Argument::for_component(
            component_name,
            definition_name_with_arguments,
            doc,
            line_number,
        )?);

        let _argument = fastn_type::Property::get_argument_for_children(&component_arguments)
            .ok_or(ftd::interpreter::Error::ParseError {
                message: "SubSection is unexpected".to_string(),
                doc_id: doc.name.to_string(),
                line_number,
            })?;

        let children = {
            let mut children = vec![];
            for child in ast_children {
                children.push(try_ok_state!(fastn_type::Component::from_ast_component(
                    child,
                    definition_name_with_arguments,
                    doc
                )?));
            }
            children
        };

        let value = fastn_type::PropertyValue::Value {
            value: fastn_type::Value::List {
                data: children
                    .into_iter()
                    .map(|v| fastn_type::PropertyValue::Value {
                        line_number: v.line_number,
                        value: fastn_type::Value::UI {
                            name: v.name.to_string(),
                            kind: fastn_type::Kind::subsection_ui().into_kind_data(),
                            component: v,
                        },
                        is_mutable: false,
                    })
                    .collect(),
                kind: fastn_type::Kind::subsection_ui().into_kind_data(),
            },
            is_mutable: false,
            line_number,
        };

        Ok(ftd::interpreter::StateWithThing::new_thing(Some(
            fastn_type::Property {
                value,
                source: fastn_type::PropertySource::Subsection,
                condition: None,
                line_number,
            },
        )))
    }

    fn scan_ast_children(
        ast_children: Vec<ftd_ast::ComponentInvocation>,
        definition_name_with_arguments: Option<(&str, &[String])>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()> {
        if ast_children.is_empty() {
            return Ok(());
        }

        for child in ast_children {
            fastn_type::Component::scan_ast_component(child, definition_name_with_arguments, doc)?;
        }

        Ok(())
    }

    fn scan_ast_properties(
        ast_properties: Vec<ftd_ast::Property>,
        definition_name_with_arguments: Option<(&str, &[String])>,
        loop_object_name_and_kind: &Option<String>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()> {
        for property in ast_properties {
            fastn_type::Property::scan_ast_property(
                property,
                definition_name_with_arguments,
                loop_object_name_and_kind,
                doc,
            )?;
        }
        Ok(())
    }

    fn scan_ast_property(
        ast_property: ftd_ast::Property,
        definition_name_with_arguments: Option<(&str, &[String])>,
        loop_object_name_and_kind: &Option<String>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()> {
        use ftd::interpreter::PropertyValueExt;

        fastn_type::PropertyValue::scan_ast_value_with_argument(
            ast_property.value.to_owned(),
            doc,
            definition_name_with_arguments,
            loop_object_name_and_kind,
        )?;

        if let Some(ref v) = ast_property.condition {
            ftd::interpreter::Expression::scan_ast_condition(
                ftd_ast::Condition::new(v, ast_property.line_number),
                definition_name_with_arguments,
                loop_object_name_and_kind,
                doc,
            )?;
        }

        Ok(())
    }

    fn from_ast_properties(
        ast_properties: Vec<ftd_ast::Property>,
        component_name: &str,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
        doc: &mut ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<Vec<fastn_type::Property>>> {
        let mut properties = vec![];
        let component_arguments =
            try_ok_state!(ftd::interpreter::Argument::for_component_or_web_component(
                component_name,
                definition_name_with_arguments,
                doc,
                line_number,
            )?);

        let kw_args = component_arguments.iter().find(|a| a.kind.is_kwargs());

        let mut extra_arguments = vec![];

        for property in ast_properties {
            match fastn_type::Property::from_ast_property(
                property.clone(),
                component_name,
                component_arguments.as_slice(),
                definition_name_with_arguments,
                loop_object_name_and_kind,
                doc,
            ) {
                Ok(property) => {
                    properties.push(try_ok_state!(property));
                }
                Err(e) => {
                    if kw_args.is_some() {
                        if let Some((name, value)) =
                            ftd::interpreter::things::component::get_extra_argument_property_value(
                                property,
                                doc.name.to_string(),
                            )?
                        {
                            extra_arguments.push((name, value));
                            continue;
                        };
                    }

                    return Err(e);
                }
            };
        }

        if let Some(kw_args) = kw_args {
            properties.push(fastn_type::Property {
                value: fastn_type::PropertyValue::Value {
                    value: fastn_type::Value::KwArgs {
                        arguments: std::collections::BTreeMap::from_iter(extra_arguments),
                    },
                    is_mutable: false,
                    line_number: kw_args.line_number,
                },
                source: fastn_type::PropertySource::Header {
                    name: kw_args.name.clone(),
                    mutable: false,
                },
                condition: None,
                line_number: kw_args.line_number,
            });
        }

        try_ok_state!(
            ftd::interpreter::things::component::search_things_for_module(
                component_name,
                properties.as_slice(),
                doc,
                component_arguments.as_slice(),
                definition_name_with_arguments,
                line_number,
            )?
        );

        crate::interpreter::things::component::check_if_property_is_provided_for_required_argument(
            &component_arguments,
            &properties,
            component_name,
            line_number,
            doc.name,
        )?;

        Ok(ftd::interpreter::StateWithThing::new_thing(properties))
    }

    fn from_ast_property(
        ast_property: ftd_ast::Property,
        component_name: &str,
        component_arguments: &[ftd::interpreter::Argument],
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_type::Property>> {
        use ftd::interpreter::PropertyValueExt;

        let argument = try_ok_state!(fastn_type::Property::get_argument_for_property(
            &ast_property,
            component_name,
            component_arguments,
            doc,
        )?);

        let value = try_ok_state!(fastn_type::PropertyValue::from_ast_value_with_argument(
            ast_property.value.to_owned(),
            doc,
            argument.mutable,
            Some(&argument.kind),
            definition_name_with_arguments,
            loop_object_name_and_kind,
        )?);

        let condition = if let Some(ref v) = ast_property.condition {
            Some(try_ok_state!(
                ftd::interpreter::Expression::from_ast_condition(
                    ftd_ast::Condition::new(v, ast_property.line_number),
                    definition_name_with_arguments,
                    loop_object_name_and_kind,
                    doc,
                )?
            ))
        } else {
            None
        };

        if ast_property.value.is_null() && !argument.kind.is_optional() {
            return ftd::interpreter::utils::e2(
                format!(
                    "Excepted Value for argument {} in component {}",
                    argument.name, component_name
                ),
                doc.name,
                ast_property.line_number,
            );
        }

        let source = {
            let mut source = fastn_type::PropertySource::from_ast(ast_property.source);
            if let fastn_type::PropertySource::Header { name, .. } = &mut source {
                *name = argument.name;
            }
            source
        };

        Ok(ftd::interpreter::StateWithThing::new_thing(
            fastn_type::Property {
                value,
                source,
                condition,
                line_number: ast_property.line_number,
            },
        ))
    }

    fn get_argument_for_property(
        ast_property: &ftd_ast::Property,
        component_name: &str,
        component_argument: &[ftd::interpreter::Argument],
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<ftd::interpreter::Argument>>
    {
        match &ast_property.source {
            ftd_ast::PropertySource::Caption => Ok(ftd::interpreter::StateWithThing::new_thing(
                component_argument
                    .iter()
                    .find(|v| v.is_caption())
                    .ok_or(ftd::interpreter::Error::ParseError {
                        message: format!(
                            "Caption type argument not found for component `{}`",
                            component_name
                        ),
                        doc_id: doc.name.to_string(),
                        line_number: ast_property.line_number,
                    })
                    .map(ToOwned::to_owned)?,
            )),
            ftd_ast::PropertySource::Body => Ok(ftd::interpreter::StateWithThing::new_thing(
                component_argument
                    .iter()
                    .find(|v| v.is_body())
                    .ok_or(ftd::interpreter::Error::ParseError {
                        message: format!(
                            "Body type argument not found for component `{}`",
                            component_name
                        ),
                        doc_id: doc.name.to_string(),
                        line_number: ast_property.line_number,
                    })
                    .map(ToOwned::to_owned)?,
            )),
            ftd_ast::PropertySource::Header { name, mutable } => {
                let (name, remaining) = ftd::interpreter::utils::split_at(name, ".");
                let mut argument = component_argument
                    .iter()
                    .find(|v| v.name.eq(name.as_str()))
                    .ok_or(ftd::interpreter::Error::ParseError {
                        message: format!(
                            "Header type `{}` mutable: `{}` argument not found for component `{}`",
                            name, mutable, component_name
                        ),
                        doc_id: doc.name.to_string(),
                        line_number: ast_property.line_number,
                    })?
                    .to_owned();
                if !argument.mutable.eq(mutable) {
                    let mutable = if argument.mutable {
                        "mutable"
                    } else {
                        "immutable"
                    };
                    return ftd::interpreter::utils::e2(
                        format!("Expected `{}` for {}", mutable, argument.name),
                        doc.name,
                        ast_property.line_number,
                    );
                }

                if let Some(variant) = remaining {
                    try_ok_state!(argument.update_with_or_type_variant(
                        doc,
                        variant.as_str(),
                        ast_property.line_number
                    )?);
                }

                Ok(ftd::interpreter::StateWithThing::new_thing(argument))
            }
        }
    }

    fn get_local_argument(&self, component_name: &str) -> Option<String> {
        if let Some(reference) = self.value.get_reference_or_clone() {
            if let Some(reference) = reference.strip_prefix(format!("{}.", component_name).as_str())
            {
                return Some(reference.to_string());
            }
        }
        None
    }
}

pub(crate) trait ComponentExt {
    fn scan_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()>;

    fn from_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_type::Component>>;

    fn from_ast_component(
        ast_component: ftd_ast::ComponentInvocation,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_type::Component>>;

    fn scan_ast_component(
        ast_component: ftd_ast::ComponentInvocation,
        definition_name_with_arguments: Option<(&str, &[String])>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()>;

    fn assert_no_private_properties_while_invocation(
        properties: &[fastn_type::Property],
        arguments: &[ftd::interpreter::Argument],
    ) -> ftd::interpreter::Result<()>;
    fn get_interpreter_value_of_argument(
        &self,
        argument_name: &str,
        doc: &ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<Option<fastn_type::Value>>;
    fn get_interpreter_property_value_of_all_arguments(
        &self,
        doc: &ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::Map<fastn_type::PropertyValue>>;
    // Todo: Remove this function after removing 0.3
    fn get_children_property(&self) -> Option<fastn_type::Property>;
    fn get_children_properties(&self) -> Vec<fastn_type::Property>;
    fn get_children(
        &self,
        doc: &ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<Vec<fastn_type::Component>>;
    fn get_kwargs(
        &self,
        doc: &ftd::interpreter::Document,
        kwargs_name: &str,
    ) -> ftd::interpreter::Result<ftd::Map<String>>;
    /// Component which is a variable
    /// -- s:
    /// where `s` is a variable of `ftd.ui` type
    #[allow(clippy::too_many_arguments)]
    fn variable_component_from_ast(
        name: &str,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        doc: &mut ftd::interpreter::TDoc,
        iteration: &Option<fastn_type::Loop>,
        condition: &Option<ftd::interpreter::Expression>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
        events: &[fastn_type::Event],
        ast_properties: &Vec<ftd_ast::Property>,
        ast_children: &Vec<ftd_ast::ComponentInvocation>,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<Option<fastn_type::Component>>>;
}
impl ComponentExt for fastn_type::Component {
    fn scan_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()> {
        let component_invocation = ast.get_component_invocation(doc.name)?;
        fastn_type::Component::scan_ast_component(component_invocation, None, doc)
    }

    fn from_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_type::Component>> {
        let component_invocation = ast.get_component_invocation(doc.name)?;
        fastn_type::Component::from_ast_component(component_invocation, &mut None, doc)
    }

    fn from_ast_component(
        ast_component: ftd_ast::ComponentInvocation,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_type::Component>> {
        let name = doc.resolve_name(ast_component.name.as_str());

        // If the component is from `module` type argument
        ftd::interpreter::utils::insert_module_thing(
            &fastn_type::Kind::ui().into_kind_data(),
            ast_component.name.as_str(),
            name.as_str(),
            definition_name_with_arguments,
            ast_component.line_number(),
            doc,
        )
        .ok();

        let mut loop_object_name_and_kind = None;
        let iteration = if let Some(v) = ast_component.iteration {
            let iteration = try_ok_state!(fastn_type::Loop::from_ast_loop(
                v,
                definition_name_with_arguments,
                doc
            )?);
            loop_object_name_and_kind = Some((
                iteration.alias.to_string(),
                iteration.loop_object_as_argument(doc)?,
                iteration.loop_counter_alias.to_owned(),
            ));
            Some(iteration)
        } else {
            None
        };

        let condition = if let Some(v) = ast_component.condition {
            Some(try_ok_state!(
                ftd::interpreter::Expression::from_ast_condition(
                    v,
                    definition_name_with_arguments,
                    &loop_object_name_and_kind,
                    doc,
                )?
            ))
        } else {
            None
        };

        let events = try_ok_state!(fastn_type::Event::from_ast_events(
            ast_component.events,
            definition_name_with_arguments,
            &loop_object_name_and_kind,
            doc,
        )?);

        if let Some(component) = try_ok_state!(fastn_type::Component::variable_component_from_ast(
            ast_component.name.as_str(),
            definition_name_with_arguments,
            doc,
            &iteration,
            &condition,
            &loop_object_name_and_kind,
            events.as_slice(),
            &ast_component.properties,
            &ast_component.children,
            ast_component.line_number
        )?) {
            return Ok(ftd::interpreter::StateWithThing::new_thing(component));
        }

        let properties = try_ok_state!(fastn_type::Property::from_ast_properties_and_children(
            ast_component.properties,
            ast_component.children,
            ast_component.name.as_str(),
            definition_name_with_arguments,
            &loop_object_name_and_kind,
            doc,
            ast_component.line_number,
        )?);
        if let Some((_name, arguments)) = definition_name_with_arguments {
            fastn_type::Component::assert_no_private_properties_while_invocation(
                &properties,
                arguments,
            )?;
        } else if let ftd::interpreter::Thing::Component(c) =
            doc.get_thing(name.as_str(), ast_component.line_number)?
        {
            Self::assert_no_private_properties_while_invocation(&properties, &c.arguments)?;
        }

        let id = ast_component.id;

        Ok(ftd::interpreter::StateWithThing::new_thing(
            fastn_type::Component {
                id,
                name,
                properties,
                iteration: Box::new(iteration),
                condition: Box::new(condition),
                events,
                children: vec![],
                source: Default::default(),
                line_number: ast_component.line_number,
            },
        ))
    }

    fn scan_ast_component(
        ast_component: ftd_ast::ComponentInvocation,
        definition_name_with_arguments: Option<(&str, &[String])>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()> {
        fastn_type::Property::scan_ast_children(
            ast_component.children,
            definition_name_with_arguments,
            doc,
        )?;
        match definition_name_with_arguments {
            Some((definition, _))
                if ast_component.name.eq(definition)
                    || ast_component
                        .name
                        .starts_with(format!("{definition}.").as_str()) => {}
            _ => doc.scan_thing(ast_component.name.as_str(), ast_component.line_number)?,
        }

        let mut loop_object_name_and_kind = None;
        if let Some(v) = ast_component.iteration {
            loop_object_name_and_kind = Some(doc.resolve_name(v.alias.as_str()));
            fastn_type::Loop::scan_ast_loop(v, definition_name_with_arguments, doc)?;
        };

        if let Some(v) = ast_component.condition {
            ftd::interpreter::Expression::scan_ast_condition(
                v,
                definition_name_with_arguments,
                &loop_object_name_and_kind,
                doc,
            )?;
        }

        fastn_type::Event::scan_ast_events(
            ast_component.events,
            definition_name_with_arguments,
            &loop_object_name_and_kind,
            doc,
        )?;

        fastn_type::Property::scan_ast_properties(
            ast_component.properties,
            definition_name_with_arguments,
            &loop_object_name_and_kind,
            doc,
        )?;

        Ok(())
    }

    fn assert_no_private_properties_while_invocation(
        properties: &[fastn_type::Property],
        arguments: &[ftd::interpreter::Argument],
    ) -> ftd::interpreter::Result<()> {
        let mut private_arguments: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        for arg in arguments.iter() {
            if !arg.access_modifier.is_public() {
                private_arguments.insert(arg.name.clone());
            }
        }

        for property in properties.iter() {
            if let fastn_type::PropertySource::Header { name, .. } = &property.source {
                if private_arguments.contains(name.as_str()) {
                    return Err(ftd::interpreter::Error::InvalidAccessError {
                        message: format!(
                            "{} argument is private and can't be accessed on \
                        invocation",
                            name
                        ),
                        line_number: property.line_number,
                    });
                }
            }
        }

        Ok(())
    }

    fn get_interpreter_value_of_argument(
        &self,
        argument_name: &str,
        doc: &ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<Option<fastn_type::Value>> {
        let component_definition = doc.get_component(self.name.as_str(), 0).unwrap();
        let argument = component_definition
            .arguments
            .iter()
            .find(|v| v.name.eq(argument_name))
            .unwrap();
        argument.get_default_interpreter_value(doc, self.properties.as_slice())
    }

    fn get_interpreter_property_value_of_all_arguments(
        &self,
        doc: &ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::Map<fastn_type::PropertyValue>> {
        let component_definition = doc.get_component(self.name.as_str(), 0).unwrap();
        let mut property_values: ftd::Map<fastn_type::PropertyValue> = Default::default();
        for argument in component_definition.arguments.iter() {
            if let Some(property_value) =
                argument.get_default_interpreter_property_value(self.properties.as_slice())?
            {
                property_values.insert(argument.name.to_string(), property_value);
            }
        }
        Ok(property_values)
    }

    // Todo: Remove this function after removing 0.3
    fn get_children_property(&self) -> Option<fastn_type::Property> {
        self.get_children_properties().first().map(|v| v.to_owned())
    }

    fn get_children_properties(&self) -> Vec<fastn_type::Property> {
        ftd::interpreter::utils::get_children_properties_from_properties(&self.properties)
    }

    fn get_children(
        &self,
        doc: &ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<Vec<fastn_type::Component>> {
        use ftd::interpreter::PropertyValueExt;

        let property = if let Some(property) = self.get_children_property() {
            property
        } else {
            return Ok(vec![]);
        };

        let value = property.value.clone().resolve(doc, property.line_number)?;
        if let fastn_type::Value::UI { component, .. } = value {
            return Ok(vec![component]);
        }
        if let fastn_type::Value::List { data, kind } = value {
            if kind.is_ui() {
                let mut children = vec![];
                for value in data {
                    let value = value.resolve(doc, property.line_number)?;
                    if let fastn_type::Value::UI { component, .. } = value {
                        children.push(component);
                    }
                }
                return Ok(children);
            }
        }

        Ok(vec![])
    }

    fn get_kwargs(
        &self,
        doc: &ftd::interpreter::Document,
        kwargs_name: &str,
    ) -> ftd::interpreter::Result<ftd::Map<String>> {
        use ftd::interpreter::ValueExt;
        use ftd::js::fastn_type_functions::PropertyValueExt;

        let property = match self.get_interpreter_value_of_argument(kwargs_name, &doc.tdoc())? {
            Some(property) => property,
            None => {
                return Err(ftd::interpreter::Error::OtherError(format!(
                    "kw-args '{}' does not exists on component.",
                    kwargs_name
                )));
            }
        };

        let kwargs = property
            .kwargs(doc.name.as_str(), self.line_number)?
            .iter()
            .map(|(name, value)| {
                let value = match value.to_value().get_string_data() {
                    Some(v) => v,
                    None => {
                        return Err(ftd::interpreter::Error::ParseError {
                            message: "Could not parse keyword argument value as string."
                                .to_string(),
                            doc_id: doc.name.clone(),
                            line_number: value.line_number(),
                        });
                    }
                };

                Ok((name.to_string(), value))
            })
            .collect::<Result<ftd::Map<String>, _>>()?;

        Ok(kwargs)
    }

    /// Component which is a variable
    /// -- s:
    /// where `s` is a variable of `ftd.ui` type
    #[allow(clippy::too_many_arguments)]
    fn variable_component_from_ast(
        name: &str,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        doc: &mut ftd::interpreter::TDoc,
        iteration: &Option<fastn_type::Loop>,
        condition: &Option<ftd::interpreter::Expression>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
        events: &[fastn_type::Event],
        ast_properties: &Vec<ftd_ast::Property>,
        ast_children: &Vec<ftd_ast::ComponentInvocation>,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<Option<fastn_type::Component>>>
    {
        use ftd::interpreter::{PropertyValueExt, PropertyValueSourceExt};

        let name = doc.resolve_name(name);

        if definition_name_with_arguments.is_none()
            || doc
                .resolve_name(definition_name_with_arguments.as_ref().unwrap().0)
                .ne(&name)
        {
            let mut var_name = if let Some(value) =
                ftd::interpreter::utils::get_argument_for_reference_and_remaining(
                    name.as_str(),
                    doc,
                    definition_name_with_arguments,
                    loop_object_name_and_kind,
                    line_number,
                )? {
                Some((
                    value.2.get_reference_name(name.as_str(), doc),
                    Some(value.0),
                ))
            } else {
                None
            };

            if var_name.is_none() {
                if let Ok(variable) = doc.search_variable(name.as_str(), line_number) {
                    try_ok_state!(variable);
                    var_name = Some((name.to_string(), None));
                }
            }

            if let Some((name, arg)) = var_name {
                let mut properties = vec![];
                if let Some(arg) = arg {
                    if arg.kind.is_module() {
                        let component_name = {
                            let (m_name, _) = match arg
                                .value
                                .as_ref()
                                .unwrap()
                                .clone()
                                .resolve(doc, line_number)?
                            {
                                fastn_type::Value::Module { name, things } => (name, things),
                                t => {
                                    return ftd::interpreter::utils::e2(
                                        format!("Expected module, found: {:?}", t),
                                        doc.name,
                                        line_number,
                                    );
                                }
                            };
                            let component_name = definition_name_with_arguments.as_ref().unwrap().0;
                            format!(
                                "{}#{}",
                                m_name,
                                name.trim_start_matches(
                                    format!("{}#{}.{}.", doc.name, component_name, arg.name)
                                        .as_str()
                                )
                            )
                        };

                        properties =
                            try_ok_state!(fastn_type::Property::from_ast_properties_and_children(
                                ast_properties.to_owned(),
                                ast_children.to_owned(),
                                component_name.as_str(),
                                definition_name_with_arguments,
                                loop_object_name_and_kind,
                                doc,
                                line_number,
                            )?);
                    }
                }

                return Ok(ftd::interpreter::StateWithThing::new_thing(Some(
                    fastn_type::Component {
                        id: None,
                        name,
                        properties,
                        iteration: Box::new(iteration.to_owned()),
                        condition: Box::new(condition.to_owned()),
                        events: events.to_vec(),
                        children: vec![],
                        source: fastn_type::ComponentSource::Variable,
                        line_number,
                    },
                )));
            }
        }

        Ok(ftd::interpreter::StateWithThing::new_thing(None))
    }
}

pub(crate) trait LoopExt {
    fn new(
        on: fastn_type::PropertyValue,
        alias: &str,
        loop_counter_alias: Option<String>,
        line_number: usize,
    ) -> fastn_type::Loop;
    fn from_ast_loop(
        ast_loop: ftd_ast::Loop,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_type::Loop>>;
    fn loop_object_as_argument(
        &self,
        doc: &ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::Argument>;
    fn loop_object_kind(&self, doc_id: &str) -> ftd::interpreter::Result<fastn_type::Kind>;
    fn scan_ast_loop(
        ast_loop: ftd_ast::Loop,
        definition_name_with_arguments: Option<(&str, &[String])>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()>;
    fn children(
        &self,
        doc: &ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<(Vec<fastn_type::PropertyValue>, fastn_type::KindData)>;
}

impl LoopExt for fastn_type::Loop {
    fn new(
        on: fastn_type::PropertyValue,
        alias: &str,
        loop_counter_alias: Option<String>,
        line_number: usize,
    ) -> fastn_type::Loop {
        fastn_type::Loop {
            on,
            alias: alias.to_string(),
            line_number,
            loop_counter_alias,
        }
    }
    fn from_ast_loop(
        ast_loop: ftd_ast::Loop,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_type::Loop>> {
        use ftd::interpreter::PropertyValueExt;

        let mut on = try_ok_state!(fastn_type::PropertyValue::from_string_with_argument(
            ast_loop.on.as_str(),
            doc,
            None,
            false,
            ast_loop.line_number,
            definition_name_with_arguments,
            &None,
        )?);

        if let Some(reference) = ast_loop.on.strip_prefix(ftd::interpreter::utils::REFERENCE) {
            if let Ok(ftd::interpreter::StateWithThing::Thing(t)) = doc.get_kind_with_argument(
                reference,
                ast_loop.line_number,
                definition_name_with_arguments,
                &None,
            ) {
                on.set_mutable(t.2);
            }
        }

        if ast_loop.on.starts_with(ftd::interpreter::utils::CLONE) {
            on.set_mutable(true);
        }

        Ok(ftd::interpreter::StateWithThing::new_thing(
            fastn_type::Loop::new(
                on,
                doc.resolve_name(ast_loop.alias.as_str()).as_str(),
                ast_loop
                    .loop_counter_alias
                    .map(|loop_counter_alias| doc.resolve_name(loop_counter_alias.as_str())),
                ast_loop.line_number,
            ),
        ))
    }
    fn loop_object_as_argument(
        &self,
        doc: &ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::Argument> {
        let kind = self.loop_object_kind(doc.name)?;
        Ok(ftd::interpreter::Argument {
            name: self.alias.to_string(),
            kind: fastn_type::KindData::new(kind),
            mutable: self.on.is_mutable(),
            value: Some(self.on.to_owned()),
            line_number: self.on.line_number(),
            access_modifier: Default::default(),
        })
    }

    fn loop_object_kind(&self, doc_id: &str) -> ftd::interpreter::Result<fastn_type::Kind> {
        let kind = self.on.kind();
        match kind {
            fastn_type::Kind::List { kind } => Ok(kind.as_ref().to_owned()),
            t => ftd::interpreter::utils::e2(
                format!("Expected list kind, found: {:?}", t),
                doc_id,
                self.line_number,
            ),
        }
    }

    fn scan_ast_loop(
        ast_loop: ftd_ast::Loop,
        definition_name_with_arguments: Option<(&str, &[String])>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()> {
        use ftd::interpreter::PropertyValueExt;

        fastn_type::PropertyValue::scan_string_with_argument(
            ast_loop.on.as_str(),
            doc,
            ast_loop.line_number,
            definition_name_with_arguments,
            &None,
        )?;

        Ok(())
    }
    fn children(
        &self,
        doc: &ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<(Vec<fastn_type::PropertyValue>, fastn_type::KindData)> {
        use ftd::interpreter::PropertyValueExt;

        let value = self.on.clone().resolve(doc, self.line_number)?;
        if let fastn_type::Value::List { data, kind } = value {
            Ok((data, kind))
        } else {
            ftd::interpreter::utils::e2(
                format!("Expected list type data, found: {:?}", self.on),
                doc.name,
                self.line_number,
            )
        }
    }
}

pub(crate) trait EventExt {
    fn from_ast_event(
        ast_event: ftd_ast::Event,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_type::Event>>;
    fn from_ast_events(
        ast_events: Vec<ftd_ast::Event>,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<Vec<fastn_type::Event>>>;
    fn scan_ast_events(
        ast_events: Vec<ftd_ast::Event>,
        definition_name_with_arguments: Option<(&str, &[String])>,
        loop_object_name_and_kind: &Option<String>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()>;
    fn scan_ast_event(
        ast_event: ftd_ast::Event,
        definition_name_with_arguments: Option<(&str, &[String])>,
        loop_object_name_and_kind: &Option<String>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()>;
}

impl EventExt for fastn_type::Event {
    fn from_ast_event(
        ast_event: ftd_ast::Event,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_type::Event>> {
        use ftd::interpreter::{EventNameExt, FunctionCallExt};

        let action = try_ok_state!(fastn_type::FunctionCall::from_string(
            ast_event.action.as_str(),
            doc,
            false,
            definition_name_with_arguments,
            loop_object_name_and_kind,
            ast_event.line_number,
        )?);

        if action.module_name.is_some() {
            let (function_name, _) = ftd::interpreter::utils::get_function_name_and_properties(
                ast_event.action.as_str(),
                doc.name,
                ast_event.line_number,
            )?;

            let reference = function_name.as_str().trim_start_matches('$');
            let reference_full_name = action.name.as_str();

            ftd::interpreter::utils::insert_module_thing(
                &action.kind,
                reference,
                reference_full_name,
                definition_name_with_arguments,
                ast_event.line_number,
                doc,
            )?;
        }

        let event_name = fastn_type::EventName::from_string(
            ast_event.name.as_str(),
            doc.name,
            ast_event.line_number,
        )?;

        Ok(ftd::interpreter::StateWithThing::new_thing(
            fastn_type::Event {
                name: event_name,
                action,
                line_number: ast_event.line_number,
            },
        ))
    }

    fn from_ast_events(
        ast_events: Vec<ftd_ast::Event>,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument, Option<String>)>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<Vec<fastn_type::Event>>> {
        let mut events = vec![];
        for event in ast_events {
            events.push(try_ok_state!(fastn_type::Event::from_ast_event(
                event,
                definition_name_with_arguments,
                loop_object_name_and_kind,
                doc,
            )?));
        }
        Ok(ftd::interpreter::StateWithThing::new_thing(events))
    }

    fn scan_ast_events(
        ast_events: Vec<ftd_ast::Event>,
        definition_name_with_arguments: Option<(&str, &[String])>,
        loop_object_name_and_kind: &Option<String>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()> {
        for event in ast_events {
            fastn_type::Event::scan_ast_event(
                event,
                definition_name_with_arguments,
                loop_object_name_and_kind,
                doc,
            )?;
        }
        Ok(())
    }

    fn scan_ast_event(
        ast_event: ftd_ast::Event,
        definition_name_with_arguments: Option<(&str, &[String])>,
        loop_object_name_and_kind: &Option<String>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()> {
        use ftd::interpreter::FunctionCallExt;

        fastn_type::FunctionCall::scan_string(
            ast_event.action.as_str(),
            doc,
            definition_name_with_arguments,
            loop_object_name_and_kind,
            ast_event.line_number,
        )?;

        Ok(())
    }
}

pub(crate) trait EventNameExt {
    fn from_string(
        e: &str,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<fastn_type::EventName>;
}

impl EventNameExt for fastn_type::EventName {
    fn from_string(
        e: &str,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<fastn_type::EventName> {
        use itertools::Itertools;

        match e {
            "click" => Ok(fastn_type::EventName::Click),
            "mouse-enter" => Ok(fastn_type::EventName::MouseEnter),
            "mouse-leave" => Ok(fastn_type::EventName::MouseLeave),
            "click-outside" => Ok(fastn_type::EventName::ClickOutside),
            "input" => Ok(fastn_type::EventName::Input),
            "change" => Ok(fastn_type::EventName::Change),
            "blur" => Ok(fastn_type::EventName::Blur),
            "focus" => Ok(fastn_type::EventName::Focus),
            t if t.starts_with("global-key[") && t.ends_with(']') => {
                let keys = t
                    .trim_start_matches("global-key[")
                    .trim_end_matches(']')
                    .split('-')
                    .map(|v| v.to_string())
                    .collect_vec();
                Ok(fastn_type::EventName::GlobalKey(keys))
            }
            t if t.starts_with("global-key-seq[") && t.ends_with(']') => {
                let keys = t
                    .trim_start_matches("global-key-seq[")
                    .trim_end_matches(']')
                    .split('-')
                    .map(|v| v.to_string())
                    .collect_vec();
                Ok(fastn_type::EventName::GlobalKeySeq(keys))
            }
            t if t.starts_with("rive-play[") && t.ends_with(']') => {
                let timeline = t
                    .trim_start_matches("rive-play[")
                    .trim_end_matches(']')
                    .to_string();
                Ok(fastn_type::EventName::RivePlay(timeline))
            }
            t if t.starts_with("rive-state-change[") && t.ends_with(']') => {
                let state = t
                    .trim_start_matches("rive-state-change[")
                    .trim_end_matches(']')
                    .to_string();
                Ok(fastn_type::EventName::RiveStateChange(state))
            }
            t if t.starts_with("rive-pause[") && t.ends_with(']') => {
                let pause = t
                    .trim_start_matches("rive-pause[")
                    .trim_end_matches(']')
                    .to_string();
                Ok(fastn_type::EventName::RivePause(pause))
            }
            t => {
                ftd::interpreter::utils::e2(format!("`{}` event not found", t), doc_id, line_number)
            }
        }
    }
}

pub(crate) trait PropertySourceExt {
    fn from_ast(item: ftd_ast::PropertySource) -> Self;
}

impl PropertySourceExt for fastn_type::PropertySource {
    fn from_ast(item: ftd_ast::PropertySource) -> Self {
        match item {
            ftd_ast::PropertySource::Caption => fastn_type::PropertySource::Caption,
            ftd_ast::PropertySource::Body => fastn_type::PropertySource::Body,
            ftd_ast::PropertySource::Header { name, mutable } => {
                fastn_type::PropertySource::Header { name, mutable }
            }
        }
    }
}

pub fn check_for_caption_and_body(s: &mut String) -> (bool, bool) {
    use itertools::Itertools;

    let mut caption = false;
    let mut body = false;

    let mut expr = s.split_whitespace().collect_vec();

    if expr.is_empty() {
        return (caption, body);
    }

    if is_caption_or_body(expr.as_slice()) {
        caption = true;
        body = true;
        expr = expr[3..].to_vec();
    } else if is_caption(expr[0]) {
        caption = true;
        expr = expr[1..].to_vec();
    } else if is_body(expr[0]) {
        body = true;
        expr = expr[1..].to_vec();
    }

    *s = expr.join(" ");

    (caption, body)
}

pub(crate) fn is_caption_or_body(expr: &[&str]) -> bool {
    if expr.len() < 3 {
        return false;
    }
    if is_caption(expr[0]) && expr[1].eq("or") && is_body(expr[2]) {
        return true;
    }

    if is_body(expr[0]) && expr[1].eq("or") && is_caption(expr[2]) {
        return true;
    }

    false
}

pub(crate) fn is_caption(s: &str) -> bool {
    s.eq("caption")
}

pub fn is_body(s: &str) -> bool {
    s.eq("body")
}

fn get_kind(
    expected_kind: Option<&fastn_type::KindData>,
    found_kind: &fastn_type::KindData,
) -> fastn_type::KindData {
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
