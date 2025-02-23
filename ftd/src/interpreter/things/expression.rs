pub(crate) trait ExpressionExt {
    fn scan_ast_condition(
        condition: ftd_ast::Condition,
        definition_name_with_arguments: Option<(&str, &[String])>,
        loop_object_name_and_kind: &Option<String>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()>;
    fn from_ast_condition(
        condition: ftd_ast::Condition,
        definition_name_with_arguments: &mut Option<(&str, &mut [fastn_resolved::Argument])>,
        loop_object_name_and_kind: &Option<(String, fastn_resolved::Argument, Option<String>)>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_resolved::Expression>>;
    fn scan_references(
        node: &mut fastn_resolved::evalexpr::ExprNode,
        definition_name_with_arguments: Option<(&str, &[String])>,
        loop_object_name_and_kind: &Option<String>,
        doc: &mut ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<()>;
    fn get_references(
        node: &mut fastn_resolved::evalexpr::ExprNode,
        definition_name_with_arguments: &mut Option<(&str, &mut [fastn_resolved::Argument])>,
        loop_object_name_and_kind: &Option<(String, fastn_resolved::Argument, Option<String>)>,
        doc: &mut ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<
        ftd::interpreter::StateWithThing<ftd::Map<fastn_resolved::PropertyValue>>,
    >;
    fn eval(&self, doc: &ftd::interpreter::TDoc) -> ftd::interpreter::Result<bool>;
    fn is_static(&self, doc: &ftd::interpreter::TDoc) -> bool;
    fn update_node_with_variable_reference(&self) -> fastn_resolved::evalexpr::ExprNode;
}

impl ExpressionExt for fastn_resolved::Expression {
    fn scan_ast_condition(
        condition: ftd_ast::Condition,
        definition_name_with_arguments: Option<(&str, &[String])>,
        loop_object_name_and_kind: &Option<String>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()> {
        if let Some(expression_mode) = get_expression_mode(condition.expression.as_str()) {
            let mut node = fastn_resolved::evalexpr::build_operator_tree(expression_mode.as_str())?;
            fastn_resolved::Expression::scan_references(
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

    fn from_ast_condition(
        condition: ftd_ast::Condition,
        definition_name_with_arguments: &mut Option<(&str, &mut [fastn_resolved::Argument])>,
        loop_object_name_and_kind: &Option<(String, fastn_resolved::Argument, Option<String>)>,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_resolved::Expression>>
    {
        if let Some(expression_mode) = get_expression_mode(condition.expression.as_str()) {
            let mut node = fastn_resolved::evalexpr::build_operator_tree(expression_mode.as_str())?;
            let references = try_ok_state!(fastn_resolved::Expression::get_references(
                &mut node,
                definition_name_with_arguments,
                loop_object_name_and_kind,
                doc,
                condition.line_number,
            )?);

            return Ok(ftd::interpreter::StateWithThing::new_thing(
                fastn_resolved::Expression::new(node, references, condition.line_number),
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

    fn scan_references(
        node: &mut fastn_resolved::evalexpr::ExprNode,
        definition_name_with_arguments: Option<(&str, &[String])>,
        loop_object_name_and_kind: &Option<String>,
        doc: &mut ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<()> {
        use ftd::interpreter::PropertyValueExt;

        let variable_identifier_reads = get_variable_identifier_read(node);
        for variable in variable_identifier_reads {
            let full_variable_name =
                doc.resolve_reference_name(format!("${}", variable.value).as_str(), line_number)?;
            fastn_resolved::PropertyValue::scan_string_with_argument(
                full_variable_name.as_str(),
                doc,
                line_number,
                definition_name_with_arguments,
                loop_object_name_and_kind,
            )?;
        }
        Ok(())
    }

    fn get_references(
        node: &mut fastn_resolved::evalexpr::ExprNode,
        definition_name_with_arguments: &mut Option<(&str, &mut [fastn_resolved::Argument])>,
        loop_object_name_and_kind: &Option<(String, fastn_resolved::Argument, Option<String>)>,
        doc: &mut ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::interpreter::Result<
        ftd::interpreter::StateWithThing<ftd::Map<fastn_resolved::PropertyValue>>,
    > {
        use ftd::interpreter::PropertyValueExt;

        let variable_identifier_reads = get_variable_identifier_read(node);
        let mut result: ftd::Map<fastn_resolved::PropertyValue> = Default::default();
        for variable in variable_identifier_reads {
            let full_variable_name =
                doc.resolve_reference_name(format!("${}", variable.value).as_str(), line_number)?;

            let value = try_ok_state!(match variable
                .infer_from
                .map(|infer_from| result.get(&infer_from.value).unwrap())
            {
                Some(infer_from_value) => {
                    match fastn_resolved::PropertyValue::from_string_with_argument(
                        full_variable_name.as_str(),
                        doc,
                        None,
                        false,
                        line_number,
                        definition_name_with_arguments,
                        loop_object_name_and_kind,
                    ) {
                        Ok(v) => match v {
                            ftd::interpreter::StateWithThing::Thing(thing)
                                if infer_from_value.kind().inner().is_or_type() =>
                            {
                                if thing.kind().inner().eq(&infer_from_value.kind().inner()) {
                                    ftd::interpreter::StateWithThing::new_thing(thing)
                                } else {
                                    return ftd::interpreter::utils::e2(
                                        format!(
                                            "Invalid value on the right-hand side. Expected \"{}\" but found \"{}\".",
                                            infer_from_value.kind().inner().get_name(),
                                            thing.kind().inner().get_name()
                                        ),
                                        doc.name,
                                        line_number,
                                    );
                                }
                            }
                            t => t,
                        },
                        Err(e) => match infer_from_value.kind().get_or_type_name() {
                            Some(name) => {
                                let name = format!("${}.{}", name, variable.value);
                                let full_variable_name =
                                    doc.resolve_reference_name(name.as_str(), line_number)?;

                                fastn_resolved::PropertyValue::from_string_with_argument(
                                    full_variable_name.as_str(),
                                    doc,
                                    None,
                                    false,
                                    line_number,
                                    definition_name_with_arguments,
                                    loop_object_name_and_kind,
                                )
                            }
                            None => Err(e),
                        }?,
                    }
                }
                None => fastn_resolved::PropertyValue::from_string_with_argument(
                    full_variable_name.as_str(),
                    doc,
                    None,
                    false,
                    line_number,
                    definition_name_with_arguments,
                    loop_object_name_and_kind,
                )?,
            });

            ftd::interpreter::utils::insert_module_thing(
                &value.kind().into_kind_data(),
                variable.value.as_str(),
                full_variable_name.as_str(),
                definition_name_with_arguments,
                line_number,
                doc,
            )
            .ok();
            result.insert(variable.value, value);
        }
        Ok(ftd::interpreter::StateWithThing::new_thing(result))
    }

    fn eval(&self, doc: &ftd::interpreter::TDoc) -> ftd::interpreter::Result<bool> {
        use ftd::interpreter::{PropertyValueExt, ValueExt};

        let mut values: ftd::Map<fastn_resolved::evalexpr::Value> = Default::default();
        for (key, property_value) in self.references.iter() {
            values.insert(
                key.to_string(),
                property_value
                    .clone()
                    .resolve(doc, self.line_number)?
                    .into_evalexpr_value(doc)?,
            );
        }
        let node = update_node_with_value(&self.expression, &values);
        let mut context = ftd::interpreter::default::default_context()?;
        Ok(node.eval_boolean_with_context_mut(&mut context)?)
    }

    fn is_static(&self, doc: &ftd::interpreter::TDoc) -> bool {
        use ftd::interpreter::PropertyValueExt;

        for val in self.references.values() {
            if !val.is_static(doc) {
                return false;
            }
        }
        true
    }

    fn update_node_with_variable_reference(&self) -> fastn_resolved::evalexpr::ExprNode {
        return update_node_with_variable_reference_(&self.expression, &self.references);

        fn update_node_with_variable_reference_(
            expr: &fastn_resolved::evalexpr::ExprNode,
            references: &ftd::Map<fastn_resolved::PropertyValue>,
        ) -> fastn_resolved::evalexpr::ExprNode {
            let mut operator = expr.operator().clone();
            if let fastn_resolved::evalexpr::Operator::VariableIdentifierRead { ref identifier } =
                operator
            {
                if format!("${}", ftd::interpreter::FTD_LOOP_COUNTER).eq(identifier) {
                    if let Some(fastn_resolved::PropertyValue::Value {
                        value: fastn_resolved::Value::Integer { value },
                        ..
                    }) = references.get(identifier)
                    {
                        operator = fastn_resolved::evalexpr::Operator::VariableIdentifierRead {
                            identifier: value.to_string(),
                        }
                    }
                } else if let Some(fastn_resolved::PropertyValue::Reference { name, .. }) =
                    references.get(identifier)
                {
                    operator = fastn_resolved::evalexpr::Operator::VariableIdentifierRead {
                        identifier: format!(
                            "resolve_reference(\"{}\", data)",
                            ftd::interpreter::utils::js_reference_name(name)
                        ),
                    }
                }
            }
            let mut children = vec![];
            for child in expr.children() {
                children.push(update_node_with_variable_reference_(child, references));
            }
            fastn_resolved::evalexpr::ExprNode::new(operator).add_children(children)
        }
    }
}

fn get_expression_mode(exp: &str) -> Option<String> {
    exp.strip_prefix('{')
        .and_then(|exp| exp.strip_suffix('}'))
        .map(ToString::to_string)
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct VariableIdentifierReadNode {
    value: String,
    infer_from: Option<Box<VariableIdentifierReadNode>>,
}

fn get_variable_identifier_read(
    node: &mut fastn_resolved::evalexpr::ExprNode,
) -> Vec<VariableIdentifierReadNode> {
    return get_variable_identifier_read_(node, &mut vec![], false, None);

    fn get_variable_identifier_read_(
        node: &mut fastn_resolved::evalexpr::ExprNode,
        write_variable: &mut Vec<String>,
        add_infer_type: bool,
        last_variable_identifier_read: Option<Box<VariableIdentifierReadNode>>,
    ) -> Vec<VariableIdentifierReadNode> {
        let mut values: Vec<VariableIdentifierReadNode> = vec![];
        if let Some(operator) = node.operator().get_variable_identifier_write() {
            write_variable.push(operator);
            // TODO: if operator.eq(ftd_ast::NULL) throw error
        } else if let Some(operator) = node.operator().get_variable_identifier_read() {
            if operator.eq(ftd_ast::NULL) {
                *node.operator_mut() = fastn_resolved::evalexpr::Operator::Const {
                    value: fastn_resolved::evalexpr::Value::Empty,
                };
            } else if !write_variable.contains(&operator) {
                values.push(VariableIdentifierReadNode {
                    value: operator,
                    infer_from: if add_infer_type {
                        last_variable_identifier_read
                    } else {
                        None
                    },
                });
            }
        }
        let operator = node.operator().clone();
        for child in node.mut_children().iter_mut() {
            values.extend(get_variable_identifier_read_(
                child,
                write_variable,
                matches!(
                    operator,
                    fastn_resolved::evalexpr::Operator::Eq
                        | fastn_resolved::evalexpr::Operator::Neq
                ),
                values.last().map(|last| Box::new(last.clone())),
            ));
        }
        values
    }
}

pub(crate) fn update_node_with_value(
    expr: &fastn_resolved::evalexpr::ExprNode,
    values: &ftd::Map<fastn_resolved::evalexpr::Value>,
) -> fastn_resolved::evalexpr::ExprNode {
    let mut operator = expr.operator().clone();
    if let fastn_resolved::evalexpr::Operator::VariableIdentifierRead { ref identifier } = operator
    {
        if let Some(value) = values.get(identifier) {
            operator = fastn_resolved::evalexpr::Operator::Const {
                value: value.to_owned(),
            }
        }
    }
    let mut children = vec![];
    for child in expr.children() {
        children.push(update_node_with_value(child, values));
    }
    fastn_resolved::evalexpr::ExprNode::new(operator).add_children(children)
}
