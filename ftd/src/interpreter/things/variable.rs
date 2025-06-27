pub trait VariableExt {
    fn scan_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()>;
    fn from_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
        number_of_scan: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_resolved::Variable>>;
    fn scan_update_from_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()>;
    fn update_from_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_resolved::Variable>>;
    fn set_static(self, doc: &ftd::interpreter::TDoc) -> Self;
}
impl VariableExt for fastn_resolved::Variable {
    fn scan_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()> {
        use ftd::interpreter::{KindDataExt, PropertyValueExt};

        let variable_definition = ast.clone().get_variable_definition(doc.name)?;
        fastn_resolved::KindData::scan_ast_kind(
            variable_definition.kind,
            &Default::default(),
            doc,
            variable_definition.line_number,
        )?;

        fastn_resolved::PropertyValue::scan_ast_value(variable_definition.value, doc)?;

        if let Some(processor) = variable_definition.processor {
            let name = doc.resolve_name(processor.as_str());
            let state = if let Some(state) = {
                match &mut doc.bag {
                    ftd::interpreter::tdoc::BagOrState::Bag(_) => None,
                    ftd::interpreter::tdoc::BagOrState::State(s) => Some(s),
                }
            } {
                state
            } else {
                return ftd::interpreter::utils::e2(
                    format!("Processor: `{processor}` not found"),
                    doc.name,
                    variable_definition.line_number,
                );
            };
            let (doc_name, thing_name, _remaining) =
                ftd::interpreter::utils::get_doc_name_and_thing_name_and_remaining(
                    name.as_str(),
                    doc.name,
                    variable_definition.line_number,
                );

            if !state.parsed_libs.contains_key(doc_name.as_str()) {
                state
                    .pending_imports
                    .stack
                    .push(ftd::interpreter::PendingImportItem {
                        module: doc_name.to_string(),
                        thing_name: name,
                        line_number: ast.line_number(),
                        caller: doc.name.to_string(),
                        exports: vec![],
                    });
                state
                    .pending_imports
                    .contains
                    .insert((doc_name.to_string(), format!("{doc_name}#{thing_name}")));
            }

            return Ok(());
        }

        Ok(())
    }

    fn from_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
        number_of_scan: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_resolved::Variable>> {
        use ftd::interpreter::{KindDataExt, PropertyValueExt};

        let variable_definition = ast.clone().get_variable_definition(doc.name)?;
        let name = doc.resolve_name(variable_definition.name.as_str());
        let kind = try_ok_state!(fastn_resolved::KindData::from_ast_kind(
            variable_definition.kind,
            &Default::default(),
            doc,
            variable_definition.line_number,
        )?);

        if let Some(processor) = variable_definition.processor {
            let state = if let Some(state) = {
                match &mut doc.bag {
                    ftd::interpreter::tdoc::BagOrState::Bag(_) => None,
                    ftd::interpreter::tdoc::BagOrState::State(s) => Some(s),
                }
            } {
                (*state).clone()
            } else {
                return ftd::interpreter::utils::e2(
                    format!("Processor: `{processor}` not found"),
                    doc.name,
                    variable_definition.line_number,
                );
            };
            let (doc_name, thing_name, remaining) =
                ftd::interpreter::utils::get_doc_name_and_thing_name_and_remaining(
                    doc.resolve_name(processor.as_str()).as_str(),
                    doc.name,
                    variable_definition.line_number,
                );

            let parsed_document = match state.parsed_libs.get(doc_name.as_str()) {
                Some(p) => p,
                None => {
                    return Ok(ftd::interpreter::StateWithThing::new_state(
                        ftd::interpreter::InterpreterWithoutState::StuckOnImport {
                            module: doc_name,
                            caller_module: doc.name.to_string(),
                        },
                    ));
                }
            };

            return if parsed_document
                .foreign_function
                .iter()
                .any(|v| thing_name.eq(v))
            {
                if number_of_scan.lt(&1) {
                    fastn_resolved::PropertyValue::scan_ast_value(variable_definition.value, doc)?;
                    return Ok(ftd::interpreter::StateWithThing::new_continue());
                }
                let result = ftd::interpreter::StateWithThing::new_state(
                    ftd::interpreter::InterpreterWithoutState::StuckOnProcessor {
                        ast,
                        module: doc_name,
                        processor: if let Some(remaining) = remaining {
                            format!("{thing_name}.{remaining}")
                        } else {
                            thing_name
                        },
                        caller_module: doc.name.to_string(),
                    },
                );
                let initial_length = if let Some(state) = doc.state() {
                    state.pending_imports.stack.len()
                } else {
                    return Ok(result);
                };
                fastn_resolved::PropertyValue::scan_ast_value(variable_definition.value, doc)?;
                if initial_length < doc.state().unwrap().pending_imports.stack.len() {
                    return Ok(ftd::interpreter::StateWithThing::new_continue());
                }
                Ok(result)
            } else {
                doc.err(
                    "not found",
                    processor,
                    "Variable::from_ast",
                    variable_definition.line_number,
                )
            };
        }

        let value = try_ok_state!(fastn_resolved::PropertyValue::from_ast_value(
            variable_definition.value,
            doc,
            variable_definition.mutable,
            Some(&kind),
        )?);

        let variable = fastn_resolved::Variable {
            name,
            kind,
            mutable: variable_definition.mutable,
            value,
            conditional_value: vec![],
            line_number: variable_definition.line_number,
            is_static: true,
        }
        .set_static(doc);

        ftd::interpreter::utils::validate_variable(&variable, doc)?;

        Ok(ftd::interpreter::StateWithThing::new_thing(variable))
    }

    fn scan_update_from_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()> {
        use ftd::interpreter::PropertyValueExt;

        let variable_definition = ast.get_variable_invocation(doc.name)?;
        fastn_resolved::PropertyValue::scan_ast_value(variable_definition.value, doc)
    }

    fn update_from_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<fastn_resolved::Variable>> {
        use ftd::interpreter::PropertyValueExt;

        let variable_definition = ast.get_variable_invocation(doc.name)?;
        let kind = try_ok_state!(doc.get_kind(
            variable_definition.name.as_str(),
            variable_definition.line_number,
        )?);

        let value = try_ok_state!(fastn_resolved::PropertyValue::from_ast_value(
            variable_definition.value,
            doc,
            true,
            Some(&kind),
        )?);

        let variable = doc.set_value(
            variable_definition.name.as_str(),
            value,
            variable_definition.line_number,
        )?;
        Ok(ftd::interpreter::StateWithThing::new_thing(variable))
    }

    fn set_static(self, doc: &ftd::interpreter::TDoc) -> Self {
        use ftd::interpreter::PropertyValueExt;

        let mut variable = self;
        if !variable.is_static {
            return variable;
        }
        if variable.mutable || !variable.value.is_static(doc) {
            variable.is_static = false;
            return variable;
        }

        for cv in variable.conditional_value.iter() {
            if !cv.value.is_static(doc) {
                variable.is_static = false;
                return variable;
            }
            for b in cv.condition.references.values() {
                if !b.is_static(doc) {
                    variable.is_static = false;
                    return variable;
                }
            }
        }

        variable
    }
}
