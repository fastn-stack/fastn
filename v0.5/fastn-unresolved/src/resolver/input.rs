pub struct Input<'a> {
    pub definitions: &'a std::collections::HashMap<fastn_unresolved::Symbol, fastn_unresolved::URD>,
    pub interner: &'a string_interner::DefaultStringInterner,
}

impl Input<'_> {
    pub fn get_component(
        &self,
        symbol: &fastn_unresolved::Symbol,
    ) -> Option<&fastn_resolved::ComponentDefinition> {
        if let Some(fastn_unresolved::UR::Resolved(fastn_resolved::Definition::Component(v))) =
            self.definitions.get(symbol)
        {
            return Some(v);
        }
        if let Some(fastn_resolved::Definition::Component(v)) =
            fastn_builtins::builtins().get(symbol.str(self.interner))
        {
            return Some(v);
        }
        None
    }
}
