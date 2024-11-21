pub struct TDoc<'a> {
    pub name: &'a str,
    pub definitions: &'a indexmap::IndexMap<String, &'a fastn_resolved::Definition>,
    pub builtins: &'static indexmap::IndexMap<String, fastn_resolved::Definition>,
}

impl TDoc<'_> {
    fn get(&self, name: &str) -> Option<&fastn_resolved::Definition> {
        if let Some(definition) = self.definitions.get(name) {
            return Some(definition);
        }

        if let Some(definition) = self.builtins.get(name) {
            return Some(definition);
        }

        None
    }
}

impl<'a> fastn_resolved::tdoc::TDoc for TDoc<'a> {
    fn get_opt_function(&self, name: &str) -> Option<&fastn_resolved::Function> {
        match self.get(name) {
            Some(fastn_resolved::Definition::Function(f)) => Some(f),
            _ => None,
        }
    }

    fn get_opt_record(&self, name: &str) -> Option<&fastn_resolved::Record> {
        match self.get(name) {
            Some(fastn_resolved::Definition::Record(f)) => Some(f),
            _ => None,
        }
    }

    fn name(&self) -> &str {
        self.name
    }

    fn get_opt_component(&self, name: &str) -> Option<&fastn_resolved::ComponentDefinition> {
        match self.get(name) {
            Some(fastn_resolved::Definition::Component(f)) => Some(f),
            _ => None,
        }
    }

    fn get_opt_web_component(&self, name: &str) -> Option<&fastn_resolved::WebComponentDefinition> {
        match self.get(name) {
            Some(fastn_resolved::Definition::WebComponent(f)) => Some(f),
            _ => None,
        }
    }
}
