#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Variable {
    pub name: String,
    pub kind: ftd::interpreter2::KindData,
    pub mutable: bool,
    pub value: ftd::interpreter2::PropertyValue,
    pub line_number: usize,
}

impl Variable {
    pub(crate) fn from_ast(
        ast: ftd::ast::AST,
        doc: &ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::Variable> {
        let variable_definition = ast.get_variable_definition(doc.name)?;
        let name = doc.resolve_name(variable_definition.name.as_str());
        let kind = ftd::interpreter2::KindData::from_ast_kind(
            variable_definition.kind,
            doc,
            variable_definition.line_number,
        )?;
        let value = ftd::interpreter2::PropertyValue::from_ast_value_with_kind(
            variable_definition.value,
            doc,
            &kind,
        )?;
        Ok(Variable {
            name,
            kind,
            mutable: variable_definition.mutable,
            value,
            line_number: variable_definition.line_number,
        })
    }

    pub(crate) fn update_from_ast(
        ast: ftd::ast::AST,
        doc: &ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::Variable> {
        let variable_definition = ast.get_variable_invocation(doc.name)?;
        let mut variable_thing = doc.get_variable(
            variable_definition.line_number,
            variable_definition.name.as_str(),
        )?;
        let value = ftd::interpreter2::PropertyValue::from_ast_value_with_kind(
            variable_definition.value,
            doc,
            &variable_thing.kind,
        )?;
        variable_thing.value = value;
        Ok(variable_thing)
    }
}
