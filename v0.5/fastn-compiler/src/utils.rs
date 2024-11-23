impl fastn_compiler::Compiler {
    pub(crate) fn resolved_content(&self) -> Vec<&fastn_resolved::ComponentInvocation> {
        // self.content should be all UR::R now
        // every symbol in self.symbol_used in the bag must be UR::R now
        self.document
            .content
            .iter()
            .map(|ur| ur.resolved().unwrap())
            .collect()
    }

    pub(crate) fn used_definitions(
        &self,
    ) -> indexmap::IndexMap<String, &fastn_resolved::Definition> {
        // go through self.symbols_used and get the resolved definitions
        let definitions = indexmap::IndexMap::new();
        for definition in self.definitions_used.iter() {
            if let Some(_definition) = self.definitions.get(definition) {
                // definitions.insert(symbol.clone(), definition);
                todo!()
            } else if let Some(_definition) =
                fastn_builtins::builtins().get(definition.symbol(&self.interner))
            {
                // definitions.insert(symbol.clone(), definition);
                todo!()
            }
        }
        definitions
    }

    pub(crate) fn external_js_files(
        &self,
        used_definitions: &indexmap::IndexMap<String, &fastn_resolved::Definition>,
    ) -> Vec<String> {
        used_definitions
            .values()
            .filter_map(|definition| match definition {
                fastn_resolved::Definition::WebComponent(web_component) => web_component.js(),
                fastn_resolved::Definition::Function(f) => f.js(),
                _ => None,
            })
            .map(ToOwned::to_owned)
            .collect()
    }

    pub(crate) fn external_css_files(
        &self,
        _needed_symbols: &indexmap::IndexMap<String, &fastn_resolved::Definition>,
    ) -> Vec<String> {
        // go through needed_symbols and get the external css files
        todo!()
    }
}
