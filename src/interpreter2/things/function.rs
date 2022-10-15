#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Function {
    pub name: String,
    pub return_kind: Option<ftd::interpreter2::Kind>,
    pub arguments: Vec<ftd::interpreter2::Argument>,
    pub expression: Vec<Expression>,
    pub line_number: usize,
}

impl Function {
    pub(crate) fn from_ast(
        ast: ftd::ast::AST,
        doc: &ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::Function> {
    }

    pub(crate) fn resolve(
        &self,
        _kind: &ftd::interpreter2::KindData,
        values: &ftd::Map<ftd::interpreter2::PropertyValue>,
        doc: &ftd::interpreter2::TDoc,
        line_number: usize,
    ) -> ftd::interpreter2::Result<Option<ftd::interpreter2::Value>> {
        use evalexpr::*;

        struct VariableContext {
            value: evalexpr::Value,
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
            if !(argument.mutable && function_value.is_mutable()) {
                return ftd::interpreter2::utils::e2("mutability conflict", doc.name, line_number);
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

        let mut evalexpr_context = evalexpr::HashMapContext::new();
        for (key, context) in context.iter() {
            evalexpr_context.set_value(key.to_string(), context.value.to_owned());
        }

        let expression = self.convert_to_evalexpr_expression();

        let eval = evalexpr::eval_with_context_mut(expression.as_str(), &mut evalexpr_context)?;

        for (key, context) in context {
            if context.mutable {
                let value = ftd::interpreter2::Value::from_evalexpr_value(
                    evalexpr_context.get_value(key.as_str()).unwrap().clone(),
                    &context.kind,
                    doc.name,
                    line_number,
                )?;
                // TODO: insert new value in doc.bag
                let _variable = doc.set_value(
                    key.as_str(),
                    ftd::interpreter2::PropertyValue::Value {
                        value,
                        is_mutable: true,
                        line_number,
                    },
                    line_number,
                )?;
            }
        }

        if let Some(ref return_kind) = self.return_kind {
            return Ok(Some(ftd::interpreter2::Value::from_evalexpr_value(
                eval,
                return_kind,
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
            .map(|v| v.0.to_string())
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
pub struct Expression(pub String);
