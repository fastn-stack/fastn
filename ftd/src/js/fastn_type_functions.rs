use ftd::js::value::{ArgumentExt, ExpressionExt};

pub(crate) trait FunctionCallExt {
    fn to_js_function(
        &self,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
    ) -> fastn_js::Function;
}

impl FunctionCallExt for fastn_type::FunctionCall {
    fn to_js_function(
        &self,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
    ) -> fastn_js::Function {
        let mut parameters = vec![];
        let mut name = self.name.to_string();
        let mut function_name = fastn_js::FunctionData::Name(self.name.to_string());
        if let Some((default_module, module_variable_name)) = &self.module_name {
            function_name =
                fastn_js::FunctionData::Definition(fastn_js::SetPropertyValue::Reference(
                    ftd::js::utils::update_reference(name.as_str(), rdata),
                ));
            name = name.replace(
                format!("{module_variable_name}.").as_str(),
                format!("{default_module}#").as_str(),
            );
        }
        let function = doc.get_function(name.as_str(), self.line_number).unwrap();
        for argument in function.arguments {
            if let Some(value) = self.values.get(argument.name.as_str()) {
                parameters.push((
                    argument.name.to_string(),
                    value.to_value().to_set_property_value(doc, rdata),
                ));
            } else if argument.get_default_value().is_none() {
                panic!("Argument value not found {:?}", argument)
            }
        }
        fastn_js::Function {
            name: Box::from(function_name),
            parameters,
        }
    }
}

pub(crate) trait PropertyValueExt {
    fn get_deps(&self, rdata: &ftd::js::ResolverData) -> Vec<String>;

    fn to_fastn_js_value_with_none(
        &self,
        doc: &ftd::interpreter::TDoc,
        has_rive_components: &mut bool,
    ) -> fastn_js::SetPropertyValue;

    fn to_fastn_js_value(
        &self,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
        should_return: bool,
    ) -> fastn_js::SetPropertyValue;

    fn to_fastn_js_value_with_ui(
        &self,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
        has_rive_components: &mut bool,
        is_ui_component: bool,
    ) -> fastn_js::SetPropertyValue;

    fn to_value(&self) -> ftd::js::Value;
}

impl PropertyValueExt for fastn_type::PropertyValue {
    fn get_deps(&self, rdata: &ftd::js::ResolverData) -> Vec<String> {
        let mut deps = vec![];
        if let Some(reference) = self.get_reference_or_clone() {
            deps.push(ftd::js::utils::update_reference(reference, rdata));
        } else if let Some(function) = self.get_function() {
            for value in function.values.values() {
                deps.extend(value.get_deps(rdata));
            }
        }
        deps
    }

    fn to_fastn_js_value_with_none(
        &self,
        doc: &ftd::interpreter::TDoc,
        has_rive_components: &mut bool,
    ) -> fastn_js::SetPropertyValue {
        self.to_fastn_js_value_with_ui(
            doc,
            &ftd::js::ResolverData::none(),
            has_rive_components,
            false,
        )
    }

    fn to_fastn_js_value(
        &self,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
        should_return: bool,
    ) -> fastn_js::SetPropertyValue {
        self.to_fastn_js_value_with_ui(doc, rdata, &mut false, should_return)
    }

    fn to_fastn_js_value_with_ui(
        &self,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
        has_rive_components: &mut bool,
        should_return: bool,
    ) -> fastn_js::SetPropertyValue {
        self.to_value().to_set_property_value_with_ui(
            doc,
            rdata,
            has_rive_components,
            should_return,
        )
    }

    fn to_value(&self) -> ftd::js::Value {
        match self {
            fastn_type::PropertyValue::Value { ref value, .. } => {
                ftd::js::Value::Data(value.to_owned())
            }
            fastn_type::PropertyValue::Reference { ref name, .. } => {
                ftd::js::Value::Reference(ftd::js::value::ReferenceData {
                    name: name.clone().to_string(),
                    value: Some(self.clone()),
                })
            }
            fastn_type::PropertyValue::FunctionCall(ref function_call) => {
                ftd::js::Value::FunctionCall(function_call.to_owned())
            }
            fastn_type::PropertyValue::Clone { ref name, .. } => {
                ftd::js::Value::Clone(name.to_owned())
            }
        }
    }
}

pub(crate) trait ValueExt {
    fn to_fastn_js_value(
        &self,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
        has_rive_components: &mut bool,
        should_return: bool,
    ) -> fastn_js::SetPropertyValue;
}

impl ValueExt for fastn_type::Value {
    fn to_fastn_js_value(
        &self,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
        has_rive_components: &mut bool,
        should_return: bool,
    ) -> fastn_js::SetPropertyValue {
        use itertools::Itertools;

        match self {
            fastn_type::Value::Boolean { value } => {
                fastn_js::SetPropertyValue::Value(fastn_js::Value::Boolean(*value))
            }
            fastn_type::Value::Optional { data, .. } => {
                if let Some(data) = data.as_ref() {
                    data.to_fastn_js_value(doc, rdata, has_rive_components, should_return)
                } else {
                    fastn_js::SetPropertyValue::Value(fastn_js::Value::Null)
                }
            }
            fastn_type::Value::String { text } => {
                fastn_js::SetPropertyValue::Value(fastn_js::Value::String(text.to_string()))
            }
            fastn_type::Value::Integer { value } => {
                fastn_js::SetPropertyValue::Value(fastn_js::Value::Integer(*value))
            }
            fastn_type::Value::Decimal { value } => {
                fastn_js::SetPropertyValue::Value(fastn_js::Value::Decimal(*value))
            }
            fastn_type::Value::OrType {
                name,
                value,
                full_variant,
                variant,
            } => {
                let (js_variant, has_value) = ftd::js::value::ftd_to_js_variant(
                    name,
                    variant,
                    full_variant,
                    value,
                    doc.name,
                    value.line_number(),
                );
                if has_value {
                    return fastn_js::SetPropertyValue::Value(fastn_js::Value::OrType {
                        variant: js_variant,
                        value: Some(Box::new(value.to_fastn_js_value(doc, rdata, should_return))),
                    });
                }
                fastn_js::SetPropertyValue::Value(fastn_js::Value::OrType {
                    variant: js_variant,
                    value: None,
                })
            }
            fastn_type::Value::List { data, .. } => {
                fastn_js::SetPropertyValue::Value(fastn_js::Value::List {
                    value: data
                        .iter()
                        .map(|v| {
                            v.to_fastn_js_value_with_ui(
                                doc,
                                rdata,
                                has_rive_components,
                                should_return,
                            )
                        })
                        .collect_vec(),
                })
            }
            fastn_type::Value::Record {
                fields: record_fields,
                name,
            } => {
                let record = doc.get_record(name, 0).unwrap();
                let mut fields = vec![];
                for field in record.fields {
                    if let Some(value) = record_fields.get(field.name.as_str()) {
                        fields.push((
                            field.name.to_string(),
                            value.to_fastn_js_value_with_ui(
                                doc,
                                &rdata
                                    .clone_with_new_record_definition_name(&Some(name.to_string())),
                                has_rive_components,
                                false,
                            ),
                        ));
                    } else {
                        fields.push((
                            field.name.to_string(),
                            field
                                .get_default_value()
                                .unwrap()
                                .to_set_property_value_with_ui(
                                    doc,
                                    &rdata.clone_with_new_record_definition_name(&Some(
                                        name.to_string(),
                                    )),
                                    has_rive_components,
                                    false,
                                ),
                        ));
                    }
                }
                fastn_js::SetPropertyValue::Value(fastn_js::Value::Record {
                    fields,
                    other_references: vec![],
                })
            }
            fastn_type::Value::UI { component, .. } => {
                fastn_js::SetPropertyValue::Value(fastn_js::Value::UI {
                    value: component.to_component_statements(
                        fastn_js::FUNCTION_PARENT,
                        0,
                        doc,
                        &rdata.clone_with_default_inherited_variable(),
                        should_return,
                        has_rive_components,
                    ),
                })
            }
            fastn_type::Value::Module { name, .. } => {
                fastn_js::SetPropertyValue::Value(fastn_js::Value::Module {
                    name: name.to_string(),
                })
            }
            t => todo!("{:?}", t),
        }
    }
}

pub(crate) trait EventExt {
    fn to_event_handler_js(
        &self,
        element_name: &str,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
    ) -> Option<fastn_js::EventHandler>;
}

impl EventExt for fastn_type::Event {
    fn to_event_handler_js(
        &self,
        element_name: &str,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
    ) -> Option<fastn_js::EventHandler> {
        use ftd::js::fastn_type_functions::FunctionCallExt;

        self.name
            .to_js_event_name()
            .map(|event| fastn_js::EventHandler {
                event,
                action: self.action.to_js_function(doc, rdata),
                element_name: element_name.to_string(),
            })
    }
}

pub(crate) trait EventNameExt {
    fn to_js_event_name(&self) -> Option<fastn_js::Event>;
}

impl EventNameExt for fastn_type::EventName {
    fn to_js_event_name(&self) -> Option<fastn_js::Event> {
        use itertools::Itertools;

        match self {
            fastn_type::EventName::Click => Some(fastn_js::Event::Click),
            fastn_type::EventName::MouseEnter => Some(fastn_js::Event::MouseEnter),
            fastn_type::EventName::MouseLeave => Some(fastn_js::Event::MouseLeave),
            fastn_type::EventName::ClickOutside => Some(fastn_js::Event::ClickOutside),
            fastn_type::EventName::GlobalKey(gk) => Some(fastn_js::Event::GlobalKey(
                gk.iter().map(|v| ftd::js::utils::to_key(v)).collect_vec(),
            )),
            fastn_type::EventName::GlobalKeySeq(gk) => Some(fastn_js::Event::GlobalKeySeq(
                gk.iter().map(|v| ftd::js::utils::to_key(v)).collect_vec(),
            )),
            fastn_type::EventName::Input => Some(fastn_js::Event::Input),
            fastn_type::EventName::Change => Some(fastn_js::Event::Change),
            fastn_type::EventName::Blur => Some(fastn_js::Event::Blur),
            fastn_type::EventName::Focus => Some(fastn_js::Event::Focus),
            fastn_type::EventName::RivePlay(_)
            | fastn_type::EventName::RivePause(_)
            | fastn_type::EventName::RiveStateChange(_) => None,
        }
    }
}

pub(crate) trait ComponentExt {
    fn to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
        should_return: bool,
        has_rive_components: &mut bool,
    ) -> Vec<fastn_js::ComponentStatement>;
    fn to_component_statements_(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
        should_return: bool,
        has_rive_components: &mut bool,
    ) -> Vec<fastn_js::ComponentStatement>;
    fn kernel_to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
        should_return: bool,
        has_rive_components: &mut bool,
    ) -> Option<Vec<fastn_js::ComponentStatement>>;
    fn defined_component_to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
        should_return: bool,
        has_rive_components: &mut bool,
    ) -> Option<Vec<fastn_js::ComponentStatement>>;
    fn header_defined_component_to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
        should_return: bool,
        has_rive_components: &mut bool,
    ) -> Option<Vec<fastn_js::ComponentStatement>>;
    fn variable_defined_component_to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
        should_return: bool,
        has_rive_components: &mut bool,
    ) -> Option<Vec<fastn_js::ComponentStatement>>;
    fn is_loop(&self) -> bool;
}

impl ComponentExt for fastn_type::Component {
    fn to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
        should_return: bool,
        has_rive_components: &mut bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        use ftd::js::fastn_type_functions::PropertyValueExt;
        use itertools::Itertools;

        let loop_alias = self.iteration.clone().map(|v| v.alias);
        let loop_counter_alias = self.iteration.clone().and_then(|v| {
            if let Some(ref loop_counter_alias) = v.loop_counter_alias {
                let (_, loop_counter_alias, _remaining) =
                    ftd::interpreter::utils::get_doc_name_and_thing_name_and_remaining(
                        loop_counter_alias.as_str(),
                        doc.name,
                        v.line_number,
                    );
                return Some(loop_counter_alias);
            }
            None
        });
        let mut component_statements = if self.is_loop() || self.condition.is_some() {
            self.to_component_statements_(
                fastn_js::FUNCTION_PARENT,
                0,
                doc,
                &rdata.clone_with_new_loop_alias(
                    &loop_alias,
                    &loop_counter_alias,
                    doc.name.to_string(),
                ),
                true,
                has_rive_components,
            )
        } else {
            self.to_component_statements_(
                parent,
                index,
                doc,
                &rdata.clone_with_new_loop_alias(&None, &None, doc.name.to_string()),
                should_return,
                has_rive_components,
            )
        };

        if let Some(condition) = self.condition.as_ref() {
            component_statements = vec![fastn_js::ComponentStatement::ConditionalComponent(
                fastn_js::ConditionalComponent {
                    deps: condition
                        .references
                        .values()
                        .flat_map(|v| {
                            v.get_deps(&rdata.clone_with_new_loop_alias(
                                &loop_alias,
                                &loop_counter_alias,
                                doc.name.to_string(),
                            ))
                        })
                        .collect_vec(),
                    condition: condition.update_node_with_variable_reference_js(
                        &rdata.clone_with_new_loop_alias(
                            &loop_alias,
                            &loop_counter_alias,
                            doc.name.to_string(),
                        ),
                    ),
                    statements: component_statements,
                    parent: parent.to_string(),
                    should_return: self.is_loop() || should_return,
                },
            )];
        }

        if let Some(iteration) = self.iteration.as_ref() {
            component_statements = vec![fastn_js::ComponentStatement::ForLoop(fastn_js::ForLoop {
                list_variable: iteration.on.to_fastn_js_value(
                    doc,
                    &rdata.clone_with_new_loop_alias(
                        &loop_alias,
                        &loop_counter_alias,
                        doc.name.to_string(),
                    ),
                    false,
                ),
                statements: component_statements,
                parent: parent.to_string(),
                should_return,
            })];
        }

        component_statements
    }

    fn to_component_statements_(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
        should_return: bool,
        has_rive_components: &mut bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        if let Some(kernel_component_statements) = self.kernel_to_component_statements(
            parent,
            index,
            doc,
            rdata,
            should_return,
            has_rive_components,
        ) {
            kernel_component_statements
        } else if let Some(defined_component_statements) = self
            .defined_component_to_component_statements(
                parent,
                index,
                doc,
                rdata,
                should_return,
                has_rive_components,
            )
        {
            defined_component_statements
        } else if let Some(header_defined_component_statements) = self
            .header_defined_component_to_component_statements(
                parent,
                index,
                doc,
                rdata,
                should_return,
                has_rive_components,
            )
        {
            header_defined_component_statements
        } else if let Some(variable_defined_component_to_component_statements) = self
            .variable_defined_component_to_component_statements(
                parent,
                index,
                doc,
                rdata,
                should_return,
                has_rive_components,
            )
        {
            variable_defined_component_to_component_statements
        } else {
            panic!("Can't find, {}", self.name)
        }
    }

    fn kernel_to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
        should_return: bool,
        has_rive_components: &mut bool,
    ) -> Option<Vec<fastn_js::ComponentStatement>> {
        if ftd::js::element::is_kernel(self.name.as_str()) {
            if !*has_rive_components {
                *has_rive_components = ftd::js::element::is_rive_component(self.name.as_str());
            }
            Some(
                ftd::js::Element::from_interpreter_component(self, doc).to_component_statements(
                    parent,
                    index,
                    doc,
                    rdata,
                    should_return,
                    has_rive_components,
                ),
            )
        } else {
            None
        }
    }

    fn defined_component_to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
        should_return: bool,
        has_rive_components: &mut bool,
    ) -> Option<Vec<fastn_js::ComponentStatement>> {
        if let Some(arguments) =
            ftd::js::utils::get_set_property_values_for_provided_component_properties(
                doc,
                rdata,
                self.name.as_str(),
                self.properties.as_slice(),
                self.line_number,
                has_rive_components,
            )
        {
            let mut component_statements = vec![];
            let instantiate_component = fastn_js::InstantiateComponent::new(
                self.name.as_str(),
                arguments,
                parent,
                rdata.inherited_variable_name,
                index,
                false,
            );

            let instantiate_component_var_name = instantiate_component.var_name.clone();

            component_statements.push(fastn_js::ComponentStatement::InstantiateComponent(
                instantiate_component,
            ));

            component_statements.extend(self.events.iter().filter_map(|event| {
                event
                    .to_event_handler_js(instantiate_component_var_name.as_str(), doc, rdata)
                    .map(|event_handler| {
                        fastn_js::ComponentStatement::AddEventHandler(event_handler)
                    })
            }));

            if should_return {
                component_statements.push(fastn_js::ComponentStatement::Return {
                    component_name: instantiate_component_var_name.to_string(),
                });
            }

            Some(component_statements)
        } else {
            None
        }
    }

    // ftd.ui type header
    fn header_defined_component_to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
        should_return: bool,
        has_rive_components: &mut bool,
    ) -> Option<Vec<fastn_js::ComponentStatement>> {
        let (component_name, remaining) = ftd::interpreter::utils::get_doc_name_and_remaining(
            self.name.as_str(),
            doc.name,
            self.line_number,
        );

        let remaining = remaining?;

        match rdata.component_definition_name {
            Some(ref component_definition_name) if component_name.eq(component_definition_name) => {
            }
            _ => return None,
        }

        let component = doc
            .get_component(component_name.as_str(), self.line_number)
            .ok()?;

        let mut arguments = vec![];

        if let Some(component_name) =
            ftd::js::utils::is_module_argument(component.arguments.as_slice(), remaining.as_str())
        {
            arguments = ftd::js::utils::get_set_property_values_for_provided_component_properties(
                doc,
                rdata,
                component_name.as_str(),
                self.properties.as_slice(),
                self.line_number,
                has_rive_components,
            )?;
        } else if !ftd::js::utils::is_ui_argument(
            component.arguments.as_slice(),
            remaining.as_str(),
        ) {
            return None;
        }

        let value = ftd::js::Value::Reference(ftd::js::value::ReferenceData {
            name: self.name.to_owned(),
            value: None,
        })
        .to_set_property_value_with_ui(doc, rdata, has_rive_components, should_return);
        let instantiate_component = fastn_js::InstantiateComponent::new_with_definition(
            value,
            arguments,
            parent,
            rdata.inherited_variable_name,
            index,
            true,
        );

        let mut component_statements = vec![];
        let instantiate_component_var_name = instantiate_component.var_name.clone();

        component_statements.push(fastn_js::ComponentStatement::InstantiateComponent(
            instantiate_component,
        ));

        component_statements.extend(self.events.iter().filter_map(|event| {
            event
                .to_event_handler_js(&instantiate_component_var_name, doc, rdata)
                .map(fastn_js::ComponentStatement::AddEventHandler)
        }));

        if should_return {
            component_statements.push(fastn_js::ComponentStatement::Return {
                component_name: instantiate_component_var_name.to_string(),
            });
        }

        Some(component_statements)
    }

    fn variable_defined_component_to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
        should_return: bool,
        has_rive_components: &mut bool,
    ) -> Option<Vec<fastn_js::ComponentStatement>> {
        /*
        Todo: Check if the `self.name` is a loop-alias of `ftd.ui list` variable and then
         uncomment the bellow code which checks for `self.name` as variable of `ftd.ui` type
        if !doc
            .get_variable(self.name.as_str(), self.line_number)
            .ok()?
            .kind
            .is_ui()
        {
            return None;
        }*/

        // The reference `self.name` is either the ftd.ui type variable or the loop-alias
        let value = ftd::js::Value::Reference(ftd::js::value::ReferenceData {
            name: self.name.to_owned(),
            value: None,
        })
        .to_set_property_value_with_ui(doc, rdata, has_rive_components, should_return);

        let instantiate_component = fastn_js::InstantiateComponent::new_with_definition(
            value,
            vec![],
            parent,
            rdata.inherited_variable_name,
            index,
            true,
        );

        let mut component_statements = vec![];
        let instantiate_component_var_name = instantiate_component.var_name.clone();

        component_statements.push(fastn_js::ComponentStatement::InstantiateComponent(
            instantiate_component,
        ));

        component_statements.extend(self.events.iter().filter_map(|event| {
            event
                .to_event_handler_js(&instantiate_component_var_name, doc, rdata)
                .map(fastn_js::ComponentStatement::AddEventHandler)
        }));

        if should_return {
            component_statements.push(fastn_js::ComponentStatement::Return {
                component_name: instantiate_component_var_name.to_string(),
            });
        }

        Some(component_statements)
    }

    fn is_loop(&self) -> bool {
        self.iteration.is_some()
    }
}
