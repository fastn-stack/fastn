#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct WebComponentDefinition {
    pub name: String,
    pub arguments: Vec<ftd::interpreter::Argument>,
    pub js: ftd::interpreter::PropertyValue,
    pub line_number: usize,
}

impl WebComponentDefinition {
    pub(crate) fn new(
        name: &str,
        arguments: Vec<ftd::interpreter::Argument>,
        js: ftd::interpreter::PropertyValue,
        line_number: usize,
    ) -> WebComponentDefinition {
        WebComponentDefinition {
            name: name.to_string(),
            arguments,
            js,
            line_number,
        }
    }

    pub(crate) fn scan_ast(
        ast: ftd::ast::AST,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<()> {
        let web_component_definition = ast.get_web_component_definition(doc.name)?;

        ftd::interpreter::Argument::scan_ast_fields(
            web_component_definition.arguments,
            doc,
            &Default::default(),
        )?;

        Ok(())
    }

    pub(crate) fn from_ast(
        ast: ftd::ast::AST,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::interpreter::Result<ftd::interpreter::StateWithThing<WebComponentDefinition>> {
        let web_component_definition = ast.get_web_component_definition(doc.name)?;
        let name = doc.resolve_name(web_component_definition.name.as_str());

        let js = try_ok_state!(ftd::interpreter::PropertyValue::from_ast_value(
            ftd::ast::VariableValue::String {
                line_number: web_component_definition.line_number(),
                value: web_component_definition.js,
                source: ftd::ast::ValueSource::Default,
                condition: None
            },
            doc,
            false,
            Some(&ftd::interpreter::Kind::string().into_kind_data()),
        )?);

        let arguments = try_ok_state!(ftd::interpreter::Argument::from_ast_fields(
            web_component_definition.name.as_str(),
            web_component_definition.arguments,
            doc,
            &Default::default(),
        )?);

        Ok(ftd::interpreter::StateWithThing::new_thing(
            WebComponentDefinition::new(
                name.as_str(),
                arguments,
                js,
                web_component_definition.line_number,
            ),
        ))
    }
}
