#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Variable {
    pub name: String,
    pub kind: ftd::interpreter2::KindData,
    pub mutable: bool,
    pub value: ftd::interpreter2::PropertyValue,
    pub conditional_value: Vec<ConditionalValue>,
    pub line_number: usize,
    pub is_static: bool,
}

impl Variable {
    pub(crate) fn from_ast(
        ast: ftd::ast::AST,
        doc: &ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::StateWithThing<ftd::interpreter2::Variable>>
    {
        let variable_definition = ast.get_variable_definition(doc.name)?;
        let name = doc.resolve_name(variable_definition.name.as_str());
        let kind = try_ready!(ftd::interpreter2::KindData::from_ast_kind(
            variable_definition.kind,
            &Default::default(),
            doc,
            variable_definition.line_number,
        )?);

        let value = try_ready!(ftd::interpreter2::PropertyValue::from_ast_value(
            variable_definition.value,
            doc,
            variable_definition.mutable,
            Some(&kind),
        )?);

        let variable = Variable {
            name,
            kind,
            mutable: variable_definition.mutable,
            value,
            conditional_value: vec![],
            line_number: variable_definition.line_number,
            is_static: true,
        }
        .set_static(doc);

        ftd::interpreter2::utils::validate_variable(&variable, doc)?;

        Ok(ftd::interpreter2::StateWithThing::new_thing(variable))
    }

    pub(crate) fn update_from_ast(
        ast: ftd::ast::AST,
        doc: &ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::StateWithThing<ftd::interpreter2::Variable>>
    {
        let variable_definition = ast.get_variable_invocation(doc.name)?;
        let kind = try_ready!(doc.get_kind(
            variable_definition.name.as_str(),
            variable_definition.line_number,
        )?);

        let value = try_ready!(ftd::interpreter2::PropertyValue::from_ast_value(
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
        Ok(ftd::interpreter2::StateWithThing::new_thing(variable))
    }

    pub fn set_static(self, doc: &ftd::interpreter2::TDoc) -> Self {
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

    pub fn is_static(&self) -> bool {
        !self.mutable && self.is_static
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ConditionalValue {
    pub condition: ftd::interpreter2::Expression,
    pub value: ftd::interpreter2::PropertyValue,
    pub line_number: usize,
}

impl ConditionalValue {
    pub fn new(
        condition: ftd::interpreter2::Expression,
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
