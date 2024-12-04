pub struct CompiledDocument {
    pub name: String,
    pub definitions: indexmap::IndexMap<String, fastn_resolved::Definition>,
}

impl CompiledDocument {
    fn get(&self, name: &str) -> Option<&fastn_resolved::Definition> {
        if let Some(definition) = self.definitions.get(name) {
            return Some(definition);
        }

        if let Some(definition) = fastn_builtins::builtins().get(name) {
            return Some(definition);
        }

        None
    }
}

impl fastn_resolved::tdoc::TDoc for CompiledDocument {
    #[cfg(not(feature = "owned-tdoc"))]
    fn get_opt_function(&self, name: &str) -> Option<&fastn_resolved::Function> {
        match self.get(name) {
            Some(fastn_resolved::Definition::Function(f)) => Some(f),
            _ => None,
        }
    }

    #[cfg(feature = "owned-tdoc")]
    fn get_opt_function(&self, name: &str) -> Option<fastn_resolved::Function> {
        match self.get(name) {
            Some(fastn_resolved::Definition::Function(f)) => Some(f.clone()),
            _ => None,
        }
    }

    #[cfg(not(feature = "owned-tdoc"))]
    fn get_opt_record(&self, name: &str) -> Option<&fastn_resolved::Record> {
        match self.get(name) {
            Some(fastn_resolved::Definition::Record(f)) => Some(f),
            _ => None,
        }
    }

    #[cfg(feature = "owned-tdoc")]
    fn get_opt_record(&self, name: &str) -> Option<fastn_resolved::Record> {
        match self.get(name) {
            Some(fastn_resolved::Definition::Record(f)) => Some(f.clone()),
            _ => None,
        }
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }

    #[cfg(not(feature = "owned-tdoc"))]
    fn get_opt_component(&self, name: &str) -> Option<&fastn_resolved::ComponentDefinition> {
        match self.get(name) {
            Some(fastn_resolved::Definition::Component(f)) => Some(f),
            _ => None,
        }
    }

    #[cfg(feature = "owned-tdoc")]
    fn get_opt_component(&self, name: &str) -> Option<fastn_resolved::ComponentDefinition> {
        match self.get(name) {
            Some(fastn_resolved::Definition::Component(f)) => Some(f.clone()),
            _ => None,
        }
    }

    #[cfg(not(feature = "owned-tdoc"))]
    fn get_opt_web_component(&self, name: &str) -> Option<&fastn_resolved::WebComponentDefinition> {
        match self.get(name) {
            Some(fastn_resolved::Definition::WebComponent(f)) => Some(f),
            _ => None,
        }
    }

    #[cfg(feature = "owned-tdoc")]
    fn get_opt_web_component(&self, name: &str) -> Option<fastn_resolved::WebComponentDefinition> {
        match self.get(name) {
            Some(fastn_resolved::Definition::WebComponent(f)) => Some(f.clone()),
            _ => None,
        }
    }

    fn definitions(&self) -> &indexmap::IndexMap<String, fastn_resolved::Definition> {
        &self.definitions
    }
}
