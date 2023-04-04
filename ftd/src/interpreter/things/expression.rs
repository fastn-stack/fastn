#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Expression {
    pub expression: ftd::evalexpr::ExprNode,
    pub references: ftd::Map<ftd::interpreter::PropertyValue>,
    pub line_number: usize,
}

impl Expression {
    pub fn new(
        expression: ftd::evalexpr::ExprNode,
        references: ftd::Map<ftd::interpreter::PropertyValue>,
        line_number: usize,
    ) -> Expression {
        Expression {
            expression,
            references,
            line_number,
        }
    }

    pub(crate) fn scan_ast_condition(
        condition: ftd::ast::Condition,
        definition_name_with_arguments: Option<(&str, &[String])>,
        loop_object_name_and_kind: &Option<String>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()> {
        if let Some(expression_mode) = get_expression_mode(condition.expression.as_str()) {
            let mut node = ftd::evalexpr::build_operator_tree(expression_mode.as_str())?;
            Expression::scan_references(
                &mut node,
                definition_name_with_arguments,
                loop_object_name_and_kind,
                doc,
                condition.line_number,
            )?;

            return Ok(());
        }
        ftd::interpreter::utils::e2(
            format!(
                "Expected condition in expression mode, found: {}",
                condition.expression
            ),
            doc.name,
            condition.line_number,
        )
    }

    pub(crate) fn from_ast_condition(
        condition: ftd::ast::Condition,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument)>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<Expression>> {
        if let Some(expression_mode) = get_expression_mode(condition.expression.as_str()) {
            let mut node = ftd::evalexpr::build_operator_tree(expression_mode.as_str())?;
            let references = try_ok_state!(Expression::get_references(
                &mut node,
                definition_name_with_arguments,
                loop_object_name_and_kind,
                doc,
                condition.line_number,
            )?);

            return Ok(ftd::interpreter::StateWithThing::new_thing(
                Expression::new(node, references, condition.line_number),
            ));
        }
        ftd::interpreter::utils::e2(
            format!(
                "Expected condition in expression mode, found: {}",
                condition.expression
            ),
            doc.name,
            condition.line_number,
        )
    }

    pub(crate) fn scan_references(
        node: &mut ftd::evalexpr::ExprNode,
        definition_name_with_arguments: Option<(&str, &[String])>,
        loop_object_name_and_kind: &Option<String>,
        doc: &mut ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<()> {
        let variable_identifier_reads = get_variable_identifier_read(node);
        for variable in variable_identifier_reads {
            let full_variable_name =
                doc.resolve_reference_name(format!("${}", variable).as_str(), line_number)?;
            ftd::interpreter::PropertyValue::scan_string_with_argument(
                full_variable_name.as_str(),
                doc,
                line_number,
                definition_name_with_arguments,
                loop_object_name_and_kind,
            )?;
        }
        Ok(())
    }

    pub(crate) fn get_references(
        node: &mut ftd::evalexpr::ExprNode,
        definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter::Argument)>,
        doc: &mut ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<
        ftd::interpreter::StateWithThing<ftd::Map<ftd::interpreter::PropertyValue>>,
    > {
        let variable_identifier_reads = get_variable_identifier_read(node);
        let mut result: ftd::Map<ftd::interpreter::PropertyValue> = Default::default();
        for variable in variable_identifier_reads {
            let full_variable_name =
                doc.resolve_reference_name(format!("${}", variable).as_str(), line_number)?;
            let value = try_ok_state!(ftd::interpreter::PropertyValue::from_string_with_argument(
                full_variable_name.as_str(),
                doc,
                None,
                false,
                line_number,
                definition_name_with_arguments,
                loop_object_name_and_kind,
            )?);
            result.insert(variable, value);
        }
        Ok(ftd::interpreter::StateWithThing::new_thing(result))
    }

    pub fn eval(&self, doc: &ftd::interpreter::TDoc) -> ftd::interpreter::Result<bool> {
        let mut values: ftd::Map<ftd::evalexpr::Value> = Default::default();
        for (key, property_value) in self.references.iter() {
            values.insert(
                key.to_string(),
                property_value
                    .clone()
                    .resolve(doc, self.line_number)?
                    .into_evalexpr_value(doc)?,
            );
        }
        let node = self.expression.update_node_with_value(&values);
        let mut context = ftd::interpreter::default::default_context()?;
        Ok(node.eval_boolean_with_context_mut(&mut context)?)
    }

    pub fn is_static(&self, doc: &ftd::interpreter::TDoc) -> bool {
        for val in self.references.values() {
            if !val.is_static(doc) {
                return false;
            }
        }
        true
    }
}

fn get_expression_mode(exp: &str) -> Option<String> {
    exp.strip_prefix('{')
        .and_then(|exp| exp.strip_suffix('}'))
        .map(ToString::to_string)
}

fn get_variable_identifier_read(node: &mut ftd::evalexpr::ExprNode) -> Vec<String> {
    return get_variable_identifier_read_(node, &mut vec![]);

    fn get_variable_identifier_read_(
        node: &mut ftd::evalexpr::ExprNode,
        write_variable: &mut Vec<String>,
    ) -> Vec<String> {
        let mut values = vec![];
        if let Some(operator) = node.operator().get_variable_identifier_write() {
            write_variable.push(operator);
            // TODO: if operator.eq(ftd::ast::NULL) throw error
        } else if let Some(operator) = node.operator().get_variable_identifier_read() {
            if operator.eq(ftd::ast::NULL) {
                *node.operator_mut() = ftd::evalexpr::Operator::Const {
                    value: ftd::evalexpr::Value::Empty,
                };
            } else if !write_variable.contains(&operator) {
                values.push(operator);
            }
        }
        for child in node.mut_children().iter_mut() {
            values.extend(get_variable_identifier_read_(child, write_variable));
        }
        values
    }
}

impl ftd::evalexpr::ExprNode {
    pub fn update_node_with_value(
        &self,
        values: &ftd::Map<ftd::evalexpr::Value>,
    ) -> ftd::evalexpr::ExprNode {
        let mut operator = self.operator().clone();
        if let ftd::evalexpr::Operator::VariableIdentifierRead { ref identifier } = operator {
            if let Some(value) = values.get(identifier) {
                operator = ftd::evalexpr::Operator::Const {
                    value: value.to_owned(),
                }
            }
        }
        let mut children = vec![];
        for child in self.children() {
            children.push(child.update_node_with_value(values));
        }
        ftd::evalexpr::ExprNode::new(operator).add_children(children)
    }

    pub fn update_node_with_variable_reference(
        &self,
        references: &ftd::Map<ftd::interpreter::PropertyValue>,
    ) -> ftd::evalexpr::ExprNode {
        let mut operator = self.operator().clone();
        if let ftd::evalexpr::Operator::VariableIdentifierRead { ref identifier } = operator {
            if format!("${}", ftd::interpreter::FTD_LOOP_COUNTER).eq(identifier) {
                if let Some(ftd::interpreter::PropertyValue::Value {
                    value: ftd::interpreter::Value::Integer { value },
                    ..
                }) = references.get(identifier)
                {
                    operator = ftd::evalexpr::Operator::VariableIdentifierRead {
                        identifier: value.to_string(),
                    }
                }
            } else if let Some(ftd::interpreter::PropertyValue::Reference { name, .. }) =
                references.get(identifier)
            {
                operator = ftd::evalexpr::Operator::VariableIdentifierRead {
                    identifier: format!(
                        "resolve_reference(\"{}\", data)",
                        ftd::interpreter::utils::js_reference_name(name)
                    ),
                }
            }
        }
        let mut children = vec![];
        for child in self.children() {
            children.push(child.update_node_with_variable_reference(references));
        }
        ftd::evalexpr::ExprNode::new(operator).add_children(children)
    }
}
