use ftd::interpreter::FieldExt;

pub trait WebComponentDefinitionExt {
    fn scan_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()>;
    fn from_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<
        ftd::interpreter::StateWithThing<fastn_resolved::WebComponentDefinition>,
    >;
}

impl WebComponentDefinitionExt for fastn_resolved::WebComponentDefinition {
    fn scan_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()> {
        let web_component_definition = ast.get_web_component_definition(doc.name)?;

        fastn_resolved::Argument::scan_ast_fields(
            web_component_definition.arguments,
            doc,
            &Default::default(),
        )?;

        Ok(())
    }

    fn from_ast(
        ast: ftd_ast::Ast,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<
        ftd::interpreter::StateWithThing<fastn_resolved::WebComponentDefinition>,
    > {
        use ftd::interpreter::PropertyValueExt;

        let web_component_definition = ast.get_web_component_definition(doc.name)?;
        let name = doc.resolve_name(web_component_definition.name.as_str());

        let js = try_ok_state!(fastn_resolved::PropertyValue::from_ast_value(
            ftd_ast::VariableValue::String {
                line_number: web_component_definition.line_number(),
                value: web_component_definition.js,
                source: ftd_ast::ValueSource::Default,
                condition: None
            },
            doc,
            false,
            Some(&fastn_resolved::Kind::string().into_kind_data()),
        )?);

        let arguments = try_ok_state!(fastn_resolved::Argument::from_ast_fields(
            web_component_definition.name.as_str(),
            web_component_definition.arguments,
            doc,
            &Default::default(),
        )?);

        Ok(ftd::interpreter::StateWithThing::new_thing(
            fastn_resolved::WebComponentDefinition::new(
                name.as_str(),
                arguments,
                js,
                web_component_definition.line_number,
            ),
        ))
    }
}
