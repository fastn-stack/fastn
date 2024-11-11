use crate::js::value::ReferenceData;

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
                ftd::js::Value::Reference(ReferenceData {
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
