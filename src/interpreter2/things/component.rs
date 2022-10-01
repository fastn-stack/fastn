#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ComponentDefinition {
    pub name: String,
    pub arguments: Vec<Argument>,
    // pub definition: Component,
    pub line_number: usize,
}

impl ComponentDefinition {
    pub(crate) fn new(
        name: &str,
        arguments: Vec<Argument>,
        line_number: usize,
    ) -> ComponentDefinition {
        ComponentDefinition {
            name: name.to_string(),
            arguments,
            line_number,
        }
    }
    pub(crate) fn from_ast(
        ast: ftd::ast::AST,
        doc: &ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<ComponentDefinition> {
        let component_definition = ast.get_component_definition(doc.name)?;
        let name = doc.resolve_name(component_definition.name.as_str());
        let arguments = Argument::from_ast_fields(component_definition.arguments, doc)?;
        Ok(ComponentDefinition::new(
            name.as_str(),
            arguments,
            component_definition.line_number,
        ))
    }
}

type Argument = ftd::interpreter2::Field;
