use ftd::interpreter::things::record::FieldExt;
use ftd::interpreter::{PropertyValueExt, PropertyValueSourceExt};

pub trait FunctionExt {
    fn scan_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()>;
    fn from_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_resolved::Function>>;
    fn resolve(
        &self,
        _kind: &fastn_resolved::KindData,
        values: &ftd::Map<fastn_resolved::PropertyValue>,
        doc: &ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<Option<fastn_resolved::Value>>;
    fn convert_to_evalexpr_expression(&self) -> String;
}
impl FunctionExt for fastn_resolved::Function {
    fn scan_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()> {
        use ftd::interpreter::KindDataExt;

        let function = ast.get_function(doc.name)?;
        fastn_resolved::Argument::scan_ast_fields(function.arguments, doc, &Default::default())?;

        fastn_resolved::KindData::scan_ast_kind(
            function.kind,
            &Default::default(),
            doc,
            function.line_number,
        )?;

        Ok(())
    }

    fn from_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_resolved::Function>> {
        use ftd::interpreter::KindDataExt;
        use ftd::interpreter::PropertyValueExt;

        let function = ast.get_function(doc.name)?;
        let name = doc.resolve_name(function.name.as_str());

        let js = if let Some(ref js) = function.js {
            Some(try_ok_state!(
                fastn_resolved::PropertyValue::from_ast_value(
                    ftd_ast::VariableValue::String {
                        value: js.to_string(),
                        line_number: function.line_number(),
                        source: ftd_ast::ValueSource::Default,
                        condition: None
                    },
                    doc,
                    false,
                    Some(&fastn_resolved::Kind::string().into_list().into_kind_data()),
                )?
            ))
        } else {
            None
        };

        let arguments = try_ok_state!(fastn_resolved::Argument::from_ast_fields(
            function.name.as_str(),
            function.arguments,
            doc,
            &Default::default(),
        )?);

        let kind = try_ok_state!(fastn_resolved::KindData::from_ast_kind(
            function.kind,
            &Default::default(),
            doc,
            function.line_number,
        )?);

        let expression = if kind.kind.is_template() {
            parse_template(function.definition.value.as_str())
        } else {
            function.definition.value.to_string()
        };

        let expression = vec![fastn_resolved::FunctionExpression {
            expression,
            line_number: function.definition.line_number,
        }];

        Ok(ftd::interpreter::StateWithThing::new_thing(
            fastn_resolved::Function::new(
                name.as_str(),
                kind,
                arguments,
                expression,
                js,
                function.line_number,
            ),
        ))
    }

    fn resolve(
        &self,
        _kind: &fastn_resolved::KindData,
        values: &ftd::Map<fastn_resolved::PropertyValue>,
        doc: &ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<Option<fastn_resolved::Value>> {
        use fastn_resolved::evalexpr::*;
        use ftd::interpreter::{PropertyValueExt, ValueExt};

        struct VariableContext {
            value: fastn_resolved::evalexpr::Value,
            reference: Option<String>,
            mutable: bool,
            kind: fastn_resolved::Kind,
        }

        let mut context: ftd::Map<VariableContext> = Default::default();
        for argument in self.arguments.iter() {
            let function_value =
                values
                    .get(argument.name.as_str())
                    .ok_or(ftd::interpreter::Error::ParseError {
                        message: format!(
                            "{} argument not found for function call `{}`",
                            argument.name, self.name
                        ),
                        doc_id: doc.name.to_string(),
                        line_number,
                    })?;
            if !argument.mutable.eq(&function_value.is_mutable()) {
                return ftd::interpreter::utils::e2(
                    format!(
                        "Mutability conflict for argument `{}` in function `{}`",
                        argument.name, self.name
                    ),
                    doc.name,
                    line_number,
                );
            }
            if !argument.kind.kind.is_same_as(&function_value.kind()) {
                return ftd::interpreter::utils::e2(
                    format!(
                        "Expected kind: `{:?}` found: `{:?}`",
                        argument.kind.kind,
                        function_value.kind()
                    ),
                    doc.name,
                    line_number,
                );
            }

            let value = function_value.clone().resolve(doc, line_number)?;
            context.insert(
                argument.name.to_string(),
                VariableContext {
                    value: value.to_evalexpr_value(doc, line_number)?,
                    reference: function_value.reference_name().map(ToOwned::to_owned),
                    mutable: argument.mutable,
                    kind: argument.kind.kind.clone(),
                },
            );
        }

        let mut evalexpr_context = ftd::interpreter::default::default_context()?;
        for (key, context) in context.iter() {
            evalexpr_context.set_value(key.to_string(), context.value.to_owned())?;
        }

        let expression = self.convert_to_evalexpr_expression();

        let eval = fastn_resolved::evalexpr::eval_with_context_mut(
            expression.as_str(),
            &mut evalexpr_context,
        )?;

        for (key, context) in context {
            match context.reference {
                Some(reference) if context.mutable => {
                    let value = fastn_resolved::Value::from_evalexpr_value(
                        evalexpr_context.get_value(key.as_str()).unwrap().clone(),
                        &context.kind,
                        doc.name,
                        line_number,
                    )?;
                    // TODO: insert new value in doc.bag
                    let _variable = doc.set_value(
                        reference.as_str(),
                        fastn_resolved::PropertyValue::Value {
                            value,
                            is_mutable: true,
                            line_number,
                        },
                        line_number,
                    )?;
                }
                _ => {}
            }
        }

        if !self.return_kind.is_void() {
            return Ok(Some(fastn_resolved::Value::from_evalexpr_value(
                eval,
                &self.return_kind.kind,
                doc.name,
                line_number,
            )?));
        }
        Ok(None)
    }

    fn convert_to_evalexpr_expression(&self) -> String {
        use itertools::Itertools;

        self.expression
            .iter()
            .map(|v| v.expression.to_string())
            .collect_vec()
            .join("\n")
    }
}

fn parse_template(value: &str) -> String {
    let mut result = String::from("\"");
    let mut var_mode = false;
    let mut var_name = String::new();

    for c in value.chars() {
        if var_mode {
            if c.is_alphanumeric() || c == '_' {
                var_name.push(c);
            } else {
                result.push_str(&format!(r#""+{var_name}+""#));
                var_mode = false;
                var_name.clear();
                if c == '$' {
                    var_mode = true;
                } else {
                    result.push(c);
                }
            }
        } else {
            if c == '$' {
                var_mode = true;
            } else {
                if c == '\\' {
                    // Escape sequences
                    result.push_str("\\\\");
                } else if c == '\n' {
                    // Escape sequences
                    result.push_str("\\\\n");
                } else if c == '"' {
                    result.push_str("\\\"");
                } else {
                    result.push(c);
                }
            }
        }
    }

    if var_mode && !var_name.is_empty() {
        result.push_str(&format!(r#""+{var_name}"#));
    } else {
        result.push_str("\"");
    }
    result
}

/*
Todo: Convert Expression into
    #[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
    pub enum Expression {
        Value(fastn_resolved::PropertyValue),
        Operation(Operation),
    }
    #[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct Operation(pub String);
*/

pub(crate) trait FunctionCallExt {
    fn from_string(
        value: &str,
        doc: &mut ftd::interpreter::TDoc,
        mutable: bool,
        definition_name_with_arguments: &mut Option<(&str, &mut [fastn_resolved::Argument])>,
        loop_object_name_and_kind: &Option<(String, fastn_resolved::Argument, Option<String>)>,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_resolved::FunctionCall>>;

    fn scan_string(
        value: &str,
        doc: &mut ftd::interpreter::TDoc,
        definition_name_with_arguments: Option<(&str, &[String])>,
        loop_object_name_and_kind: &Option<String>,
        line_number: usize,
    ) -> ftd::interpreter::Result<()>;
}

impl FunctionCallExt for fastn_resolved::FunctionCall {
    fn from_string(
        value: &str,
        doc: &mut ftd::interpreter::TDoc,
        mutable: bool,
        definition_name_with_arguments: &mut Option<(&str, &mut [fastn_resolved::Argument])>,
        loop_object_name_and_kind: &Option<(String, fastn_resolved::Argument, Option<String>)>,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_resolved::FunctionCall>>
    {
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
        let mut source = fastn_resolved::PropertyValueSource::Global;
        if let Some((ref argument, ref function, source_)) = initial_kind_with_remaining_and_source
        {
            source = source_;
            if argument.kind.is_module() {
                if let Some(fastn_resolved::PropertyValue::Value {
                    value: fastn_resolved::Value::Module { ref name, .. },
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
        let mut values: ftd::Map<fastn_resolved::PropertyValue> = Default::default();
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
                try_ok_state!(fastn_resolved::PropertyValue::from_ast_value_with_argument(
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
                    None if argument.kind.is_optional() => fastn_resolved::PropertyValue::new_none(
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
            fastn_resolved::FunctionCall::new(
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
            fastn_resolved::PropertyValue::scan_string_with_argument(
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
