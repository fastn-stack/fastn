impl fastn_type::Function {
    pub fn to_ast(&self, doc: &fastn_type::TDoc) -> fastn_js::Ast {
        use itertools::Itertools;

        fastn_js::udf_with_arguments(
            self.name.as_str(),
            self.expression
                .iter()
                .map(|e| {
                    fastn_grammar::evalexpr::build_operator_tree(e.expression.as_str()).unwrap()
                })
                .collect_vec(),
            self.arguments
                .iter()
                .map(|v| {
                    v.get_default_value()
                        .map(|val| {
                            (
                                v.name.to_string(),
                                val.to_set_property_value(
                                    doc,
                                    &fastn_type::ResolverData::new_with_component_definition_name(
                                        &Some(self.name.to_string()),
                                    ),
                                ),
                            )
                        })
                        .unwrap_or_else(|| {
                            (v.name.to_string(), fastn_js::SetPropertyValue::undefined())
                        })
                })
                .collect_vec(),
            self.js.is_some(),
        )
    }
}
