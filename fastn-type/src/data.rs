#[derive(Debug)]
pub enum Data {
    Data(fastn_type::Value),
    Reference(ReferenceData),
    ConditionalFormula(Vec<fastn_type::Property>),
    FunctionCall(fastn_type::FunctionCall),
    Clone(String),
}

#[derive(Debug)]
pub struct ReferenceData {
    pub name: String,
    pub value: Option<fastn_type::PropertyValue>,
}

impl fastn_type::Argument {
    pub(crate) fn get_default_value(&self) -> Option<fastn_type::Data> {
        if let Some(ref value) = self.value {
            Some(value.to_data())
        } else if self.kind.is_list() {
            Some(fastn_type::Data::Data(fastn_type::Value::List {
                data: vec![],
                kind: self.kind.clone(),
            }))
        } else if self.kind.is_optional() {
            Some(fastn_type::Data::Data(fastn_type::Value::Optional {
                data: Box::new(None),
                kind: self.kind.clone(),
            }))
        } else {
            None
        }
    }
}

impl fastn_type::Data {
    pub(crate) fn to_set_property_value(
        &self,
        doc: &fastn_type::TDoc,
        rdata: &fastn_type::ResolverData,
    ) -> fastn_js::SetPropertyValue {
        self.to_set_property_value_with_ui(doc, rdata, &mut false, false)
    }

    pub(crate) fn to_set_property_value_with_ui(
        &self,
        doc: &fastn_type::TDoc,
        rdata: &fastn_type::ResolverData,
        has_rive_components: &mut bool,
        should_return: bool,
    ) -> fastn_js::SetPropertyValue {
        match self {
            fastn_type::Data::Data(value) => {
                value.to_fastn_js_value(doc, rdata, has_rive_components, should_return)
            }
            fastn_type::Data::Reference(data) => {
                if let Some(value) = &data.value {
                    if let fastn_type::Kind::OrType {
                        name,
                        variant: Some(variant),
                        full_variant: Some(full_variant),
                    } = value.kind().inner()
                    {
                        let (js_variant, has_value) = fastn_type::ftd_to_js_variant(
                            name.as_str(),
                            variant.as_str(),
                            full_variant.as_str(),
                            value,
                            doc.name,
                            value.line_number(),
                        );

                        // return or-type value with reference
                        if has_value {
                            return fastn_js::SetPropertyValue::Value(fastn_js::Value::OrType {
                                variant: js_variant,
                                value: Some(Box::new(fastn_js::SetPropertyValue::Reference(
                                    fastn_type::utils::update_reference(data.name.as_str(), rdata),
                                ))),
                            });
                        }

                        // return or-type value
                        return fastn_js::SetPropertyValue::Value(fastn_js::Value::OrType {
                            variant: js_variant,
                            value: None,
                        });
                    }
                }

                // for other datatypes, simply return a reference
                fastn_js::SetPropertyValue::Reference(fastn_type::utils::update_reference(
                    data.name.as_str(),
                    rdata,
                ))
            }
            fastn_type::Data::ConditionalFormula(formulas) => fastn_js::SetPropertyValue::Formula(
                properties_to_js_conditional_formula(doc, formulas, rdata),
            ),
            fastn_type::Data::FunctionCall(function_call) => fastn_js::SetPropertyValue::Formula(
                fastn_type::utils::function_call_to_js_formula(function_call, doc, rdata),
            ),
            fastn_type::Data::Clone(name) => {
                fastn_js::SetPropertyValue::Clone(fastn_type::utils::update_reference(name, rdata))
            }
        }
    }
}

impl fastn_type::PropertyValue {
    pub(crate) fn to_fastn_js_value_with_none(
        &self,
        doc: &fastn_type::TDoc,
        has_rive_components: &mut bool,
    ) -> fastn_js::SetPropertyValue {
        self.to_fastn_js_value_with_ui(
            doc,
            &fastn_type::ResolverData::none(),
            has_rive_components,
            false,
        )
    }

    pub(crate) fn to_fastn_js_value(
        &self,
        doc: &fastn_type::TDoc,
        rdata: &fastn_type::ResolverData,
        should_return: bool,
    ) -> fastn_js::SetPropertyValue {
        self.to_fastn_js_value_with_ui(doc, rdata, &mut false, should_return)
    }

    pub(crate) fn to_fastn_js_value_with_ui(
        &self,
        doc: &fastn_type::TDoc,
        rdata: &fastn_type::ResolverData,
        has_rive_components: &mut bool,
        should_return: bool,
    ) -> fastn_js::SetPropertyValue {
        self.to_data()
            .to_set_property_value_with_ui(doc, rdata, has_rive_components, should_return)
    }

    pub(crate) fn to_data(&self) -> fastn_type::Data {
        match self {
            fastn_type::PropertyValue::Value { ref value, .. } => {
                fastn_type::Data::Data(value.to_owned())
            }
            fastn_type::PropertyValue::Reference { ref name, .. } => {
                fastn_type::Data::Reference(ReferenceData {
                    name: name.clone().to_string(),
                    value: Some(self.clone()),
                })
            }
            fastn_type::PropertyValue::FunctionCall(ref function_call) => {
                fastn_type::Data::FunctionCall(function_call.to_owned())
            }
            fastn_type::PropertyValue::Clone { ref name, .. } => {
                fastn_type::Data::Clone(name.to_owned())
            }
        }
    }
}

impl fastn_type::Value {
    pub(crate) fn to_fastn_js_value(
        &self,
        doc: &fastn_type::TDoc,
        rdata: &fastn_type::ResolverData,
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
                let (js_variant, has_value) = fastn_type::ftd_to_js_variant(
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

fn properties_to_js_conditional_formula(
    doc: &fastn_type::TDoc,
    properties: &[fastn_type::Property],
    rdata: &fastn_type::ResolverData,
) -> fastn_js::Formula {
    let mut deps = vec![];
    let mut conditional_values = vec![];
    for property in properties {
        deps.extend(property.value.get_deps(rdata));
        if let Some(ref condition) = property.condition {
            deps.extend(condition.get_deps(rdata));
        }

        conditional_values.push(fastn_js::ConditionalValue {
            condition: property
                .condition
                .as_ref()
                .map(|condition| condition.update_node_with_variable_reference_js(rdata)),
            expression: property.value.to_fastn_js_value(doc, rdata, false),
        });
    }

    fastn_js::Formula {
        deps,
        type_: fastn_js::FormulaType::Conditional(conditional_values),
    }
}

impl fastn_type::PropertyValue {
    pub(crate) fn get_deps(&self, rdata: &fastn_type::ResolverData) -> Vec<String> {
        let mut deps = vec![];
        if let Some(reference) = self.get_reference_or_clone() {
            deps.push(fastn_type::utils::update_reference(reference, rdata));
        } else if let Some(function) = self.get_function() {
            for value in function.values.values() {
                deps.extend(value.get_deps(rdata));
            }
        }
        deps
    }
}
