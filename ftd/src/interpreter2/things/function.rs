#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Function {
    pub name: String,
    pub return_kind: ftd::interpreter2::KindData,
    pub arguments: Vec<ftd::interpreter2::Argument>,
    pub expression: Vec<Expression>,
    pub js: Option<ftd::interpreter2::PropertyValue>,
    pub line_number: usize,
}

impl Function {
    fn new(
        name: &str,
        return_kind: ftd::interpreter2::KindData,
        arguments: Vec<ftd::interpreter2::Argument>,
        expression: Vec<Expression>,
        js: Option<ftd::interpreter2::PropertyValue>,
        line_number: usize,
    ) -> Function {
        Function {
            name: name.to_string(),
            return_kind,
            arguments,
            expression,
            js,
            line_number,
        }
    }

    pub(crate) fn scan_ast(
        ast: ftd::ast::AST,
        doc: &mut ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<()> {
        let function = ast.get_function(doc.name)?;
        ftd::interpreter2::Argument::scan_ast_fields(function.arguments, doc, &Default::default())?;

        ftd::interpreter2::KindData::scan_ast_kind(
            function.kind,
            &Default::default(),
            doc,
            function.line_number,
        )?;

        Ok(())
    }

    pub(crate) fn from_ast(
        ast: ftd::ast::AST,
        doc: &mut ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::StateWithThing<ftd::interpreter2::Function>>
    {
        let function = ast.get_function(doc.name)?;
        let name = doc.resolve_name(function.name.as_str());

        let js = if let Some(ref js) = function.js {
            Some(try_ok_state!(
                ftd::interpreter2::PropertyValue::from_ast_value(
                    ftd::ast::VariableValue::String {
                        value: js.to_string(),
                        line_number: function.line_number(),
                        source: ftd::ast::ValueSource::Default
                    },
                    doc,
                    false,
                    Some(&ftd::interpreter2::Kind::string().into_kind_data()),
                )?
            ))
        } else {
            None
        };

        let arguments = try_ok_state!(ftd::interpreter2::Argument::from_ast_fields(
            function.name.as_str(),
            function.arguments,
            doc,
            &Default::default(),
        )?);

        let kind = try_ok_state!(ftd::interpreter2::KindData::from_ast_kind(
            function.kind,
            &Default::default(),
            doc,
            function.line_number,
        )?);

        let expression = vec![Expression {
            expression: function.definition.value.to_string(),
            line_number: function.definition.line_number,
        }];

        Ok(ftd::interpreter2::StateWithThing::new_thing(Function::new(
            name.as_str(),
            kind,
            arguments,
            expression,
            js,
            function.line_number,
        )))
    }

    pub(crate) fn resolve(
        &self,
        _kind: &ftd::interpreter2::KindData,
        values: &ftd::Map<ftd::interpreter2::PropertyValue>,
        doc: &ftd::interpreter2::TDoc,
        line_number: usize,
    ) -> ftd::interpreter2::Result<Option<ftd::interpreter2::Value>> {
        use ftd::evalexpr::*;

        struct VariableContext {
            value: ftd::evalexpr::Value,
            reference: Option<String>,
            mutable: bool,
            kind: ftd::interpreter2::Kind,
        }

        let mut context: ftd::Map<VariableContext> = Default::default();
        for argument in self.arguments.iter() {
            let function_value =
                values
                    .get(argument.name.as_str())
                    .ok_or(ftd::interpreter2::Error::ParseError {
                        message: format!(
                            "{} argument not found for function call `{}`",
                            argument.name, self.name
                        ),
                        doc_id: doc.name.to_string(),
                        line_number,
                    })?;
            if !argument.mutable.eq(&function_value.is_mutable()) {
                return ftd::interpreter2::utils::e2(
                    format!(
                        "Mutability conflict for argument `{}` in function `{}`",
                        argument.name, self.name
                    ),
                    doc.name,
                    line_number,
                );
            }
            if !argument.kind.kind.is_same_as(&function_value.kind()) {
                return ftd::interpreter2::utils::e2(
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

        let mut evalexpr_context = ftd::interpreter2::default::default_context()?;
        for (key, context) in context.iter() {
            evalexpr_context.set_value(key.to_string(), context.value.to_owned())?;
        }

        let expression = self.convert_to_evalexpr_expression();

        let eval =
            ftd::evalexpr::eval_with_context_mut(expression.as_str(), &mut evalexpr_context)?;

        for (key, context) in context {
            match context.reference {
                Some(reference) if context.mutable => {
                    let value = ftd::interpreter2::Value::from_evalexpr_value(
                        evalexpr_context.get_value(key.as_str()).unwrap().clone(),
                        &context.kind,
                        doc.name,
                        line_number,
                    )?;
                    // TODO: insert new value in doc.bag
                    let _variable = doc.set_value(
                        reference.as_str(),
                        ftd::interpreter2::PropertyValue::Value {
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
            return Ok(Some(ftd::interpreter2::Value::from_evalexpr_value(
                eval,
                &self.return_kind.kind,
                doc.name,
                line_number,
            )?));
        }
        Ok(None)
    }

    pub(crate) fn convert_to_evalexpr_expression(&self) -> String {
        use itertools::Itertools;

        self.expression
            .iter()
            .map(|v| v.expression.to_string())
            .collect_vec()
            .join("\n")
    }
}

/*
Todo: Convert Expression into
    #[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
    pub enum Expression {
        Value(ftd::interpreter2::PropertyValue),
        Operation(Operation),
    }
    #[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct Operation(pub String);
*/

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Expression {
    pub expression: String,
    pub line_number: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct FunctionCall {
    pub name: String,
    pub kind: ftd::interpreter2::KindData,
    pub is_mutable: bool,
    pub line_number: usize,
    pub values: ftd::Map<ftd::interpreter2::PropertyValue>,
    pub order: Vec<String>,
}

impl FunctionCall {
    pub fn new(
        name: &str,
        kind: ftd::interpreter2::KindData,
        is_mutable: bool,
        line_number: usize,
        values: ftd::Map<ftd::interpreter2::PropertyValue>,
        order: Vec<String>,
    ) -> FunctionCall {
        FunctionCall {
            name: name.to_string(),
            kind,
            is_mutable,
            line_number,
            values,
            order,
        }
    }

    pub(crate) fn scan_string(
        value: &str,
        doc: &mut ftd::interpreter2::TDoc,
        definition_name_with_arguments: Option<(&str, &[String])>,
        loop_object_name_and_kind: &Option<String>,
        line_number: usize,
    ) -> ftd::interpreter2::Result<()> {
        let expression = value
            .trim_start_matches(ftd::interpreter2::utils::REFERENCE)
            .to_string();

        let (function_name, properties) =
            ftd::interpreter2::utils::get_function_name_and_properties(
                expression.as_str(),
                doc.name,
                line_number,
            )?;

        doc.scan_initial_thing(function_name.as_str(), line_number)?;

        for (_, value) in properties.iter() {
            ftd::interpreter2::PropertyValue::scan_string_with_argument(
                value,
                doc,
                line_number,
                definition_name_with_arguments,
                loop_object_name_and_kind,
            )?;
        }

        Ok(())
    }

    pub(crate) fn from_string(
        value: &str,
        doc: &mut ftd::interpreter2::TDoc,
        mutable: bool,
        definition_name_with_arguments: Option<(&str, &[ftd::interpreter2::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter2::Argument)>,
        line_number: usize,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::StateWithThing<ftd::interpreter2::FunctionCall>>
    {
        let expression = value
            .trim_start_matches(ftd::interpreter2::utils::REFERENCE)
            .to_string();

        let (function_name, properties) =
            ftd::interpreter2::utils::get_function_name_and_properties(
                expression.as_str(),
                doc.name,
                line_number,
            )?;
        let function = try_ok_state!(doc.search_function(function_name.as_str(), line_number)?);
        let mut values: ftd::Map<ftd::interpreter2::PropertyValue> = Default::default();
        let mut order = vec![];

        for argument in function.arguments.iter() {
            let property_value = if let Some((property, property_key, mutable)) =
                properties.iter().find_map(|(key, property)| {
                    let (property_key, mutable) =
                        if let Some(key) = key.strip_prefix(ftd::interpreter2::utils::REFERENCE) {
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
                    return ftd::interpreter2::utils::e2(
                        format!(
                            "Mutability conflict in argument `{}` for function `{}`",
                            property_key, function_name
                        ),
                        doc.name,
                        line_number,
                    );
                }
                try_ok_state!(
                    ftd::interpreter2::PropertyValue::from_ast_value_with_argument(
                        ftd::ast::VariableValue::String {
                            value: property,
                            line_number,
                            source: ftd::ast::ValueSource::Default
                        },
                        doc,
                        mutable,
                        Some(&argument.kind),
                        definition_name_with_arguments,
                        loop_object_name_and_kind,
                    )?
                )
            } else {
                match argument.value {
                    Some(ref value) => value.clone(),
                    None if argument.kind.is_optional() => {
                        ftd::interpreter2::PropertyValue::new_none(
                            argument.kind.clone(),
                            argument.line_number,
                        )
                    }
                    _ => {
                        return ftd::interpreter2::utils::e2(
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

        let reference_full_name = ftd::interpreter2::PropertyValueSource::Global
            .get_reference_name(function_name.as_str(), doc);

        Ok(ftd::interpreter2::StateWithThing::new_thing(
            ftd::interpreter2::FunctionCall::new(
                reference_full_name.as_str(),
                function.return_kind,
                mutable,
                line_number,
                values,
                order,
            ),
        ))
    }
}
