#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Boolean {
    pub expression: ftd::evalexpr::Node,
    pub references: ftd::Map<ftd::interpreter2::PropertyValue>,
    pub line_number: usize,
}

impl Boolean {
    pub fn new(
        expression: ftd::evalexpr::Node,
        references: ftd::Map<ftd::interpreter2::PropertyValue>,
        line_number: usize,
    ) -> Boolean {
        Boolean {
            expression,
            references,
            line_number,
        }
    }

    pub(crate) fn from_ast_condition(
        condition: ftd::ast::Condition,
        definition_name_with_arguments: Option<(&str, &[ftd::interpreter2::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter2::Argument)>,
        doc: &ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<Boolean> {
        if let Some(expression_mode) = get_expression_mode(condition.expression.as_str()) {
            let node = ftd::evalexpr::build_operator_tree(expression_mode.as_str())?;
            let references = Boolean::get_references(
                &node,
                definition_name_with_arguments,
                loop_object_name_and_kind,
                doc,
                condition.line_number,
            )?;
            return Ok(Boolean::new(node, references, condition.line_number));
        }
        ftd::interpreter2::utils::e2(
            format!(
                "Expected condition in expression mode, found: {}",
                condition.expression
            ),
            doc.name,
            condition.line_number,
        )
    }

    pub(crate) fn get_references(
        node: &ftd::evalexpr::Node,
        definition_name_with_arguments: Option<(&str, &[ftd::interpreter2::Argument])>,
        loop_object_name_and_kind: &Option<(String, ftd::interpreter2::Argument)>,
        doc: &ftd::interpreter2::TDoc,
        line_number: usize,
    ) -> ftd::interpreter2::Result<ftd::Map<ftd::interpreter2::PropertyValue>> {
        let variable_identifier_reads = get_variable_identifier_read(node);
        let mut result: ftd::Map<ftd::interpreter2::PropertyValue> = Default::default();
        for variable in variable_identifier_reads {
            let full_variable_name =
                doc.resolve_reference_name(format!("${}", variable).as_str(), line_number)?;
            let value = ftd::interpreter2::PropertyValue::from_string_with_argument(
                full_variable_name.as_str(),
                doc,
                None,
                false,
                line_number,
                definition_name_with_arguments,
                loop_object_name_and_kind,
            )?;
            result.insert(variable, value);
        }
        Ok(result)
    }

    pub fn eval(&self, doc: &ftd::interpreter2::TDoc) -> ftd::interpreter2::Result<bool> {
        let mut values: ftd::Map<ftd::evalexpr::Value> = Default::default();
        for (key, property_value) in self.references.iter() {
            values.insert(
                key.to_string(),
                property_value
                    .clone()
                    .resolve(doc, self.line_number)?
                    .into_evalexpr_value(),
            );
        }
        let node = self.expression.update_node_with_value(&values);
        let mut context = ftd::interpreter2::default::default_context()?;
        Ok(node.eval_boolean_with_context_mut(&mut context)?)
    }
}

fn get_expression_mode(exp: &str) -> Option<String> {
    exp.strip_prefix('{')
        .and_then(|exp| exp.strip_suffix('}'))
        .map(ToString::to_string)
}

fn get_variable_identifier_read(node: &ftd::evalexpr::Node) -> Vec<String> {
    return get_variable_identifier_read_(node, &mut vec![]);

    fn get_variable_identifier_read_(
        node: &ftd::evalexpr::Node,
        write_variable: &mut Vec<String>,
    ) -> Vec<String> {
        let mut values = vec![];
        if let Some(operator) = node.operator().get_variable_identifier_write() {
            write_variable.push(operator);
        } else if let Some(operator) = node.operator().get_variable_identifier_read() {
            if !write_variable.contains(&operator) {
                values.push(operator);
            }
        }
        for child in node.children().iter() {
            values.extend(get_variable_identifier_read_(child, write_variable));
        }
        values
    }
}

impl ftd::evalexpr::Node {
    pub fn update_node_with_value(
        &self,
        values: &ftd::Map<ftd::evalexpr::Value>,
    ) -> ftd::evalexpr::Node {
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
        ftd::evalexpr::Node::new(operator).add_children(children)
    }

    pub fn update_node_with_variable_reference(
        &self,
        references: &ftd::Map<ftd::interpreter2::PropertyValue>,
    ) -> ftd::evalexpr::Node {
        let mut operator = self.operator().clone();
        if let ftd::evalexpr::Operator::VariableIdentifierRead { ref identifier } = operator {
            if let Some(ftd::interpreter2::PropertyValue::Reference { name, .. }) =
                references.get(identifier)
            {
                operator = ftd::evalexpr::Operator::VariableIdentifierRead {
                    identifier: format!("data[\"{}\"]", name),
                }
            }
        }
        let mut children = vec![];
        for child in self.children() {
            children.push(child.update_node_with_variable_reference(references));
        }
        ftd::evalexpr::Node::new(operator).add_children(children)
    }
}
