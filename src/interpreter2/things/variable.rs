#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Variable {
    pub name: String,
    pub kind: ftd::interpreter2::KindData,
    pub mutable: bool,
    pub value: ftd::interpreter2::PropertyValue,
    pub conditional_value: Vec<ConditionalValue>,
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
        let value = ftd::interpreter2::PropertyValue::from_ast_value(
            variable_definition.value,
            doc,
            variable_definition.mutable,
            Some(&kind),
        )?;
        Ok(Variable {
            name,
            kind,
            mutable: variable_definition.mutable,
            value,
            conditional_value: vec![],
            line_number: variable_definition.line_number,
        })
    }

    pub(crate) fn update_from_ast(
        ast: ftd::ast::AST,
        doc: &ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::Variable> {
        let variable_definition = ast.get_variable_invocation(doc.name)?;
        let kind = doc.get_kind(
            variable_definition.name.as_str(),
            variable_definition.line_number,
        )?;

        let value = ftd::interpreter2::PropertyValue::from_ast_value(
            variable_definition.value,
            doc,
            true,
            Some(&kind),
        )?;
        let variable = doc.set_value(
            variable_definition.name.as_str(),
            value,
            variable_definition.line_number,
        )?;
        Ok(variable)
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ConditionalValue {
    pub condition: ftd::interpreter2::Boolean,
    pub value: ftd::interpreter2::PropertyValue,
    pub line_number: usize,
}

impl ConditionalValue {
    pub fn new(
        condition: ftd::interpreter2::Boolean,
        value: ftd::interpreter2::PropertyValue,
        line_number: usize,
    ) -> ConditionalValue {
        ConditionalValue {
            condition,
            value,
            line_number,
        }
    }
}
