impl fastn_compiler::Compiler {
    pub(crate) fn js(&self) -> fastn_resolved_to_js::HtmlInput {
        use fastn_resolved::tdoc::TDoc;
        use fastn_resolved_to_js::extensions::*;

        // this function should look a bit like ftd::js::document_into_js_ast(), we do not need
        // to construct the Document object there, but will fetch all the fields as variables

        // self.content should be all UR::R now
        let resolved_content = self.resolved_content();
        // every symbol in self.symbol_used in the bag must be UR::R now
        let used_definitions = self.used_definitions();
        let doc = fastn_compiler::TDoc {
            name: "", // Todo: Package name
            definitions: &used_definitions,
            builtins: fastn_builtins::builtins(),
        };

        let css_files = self.external_css_files(&used_definitions);
        let js_files = self.external_js_files(&used_definitions);
        let output = fastn_resolved_to_js::get_all_asts(
            &doc,
            resolved_content.as_slice(),
            used_definitions.into_iter().map(|(_, v)| v),
        );
        let js = fastn_js::to_js(output.ast.as_slice(), "");

        fastn_resolved_to_js::HtmlInput {
            package: Default::default(), // Todo
            js,
            css_files,
            js_files,
            doc: Box::new(doc),
            has_rive_component: output.has_rive_components,
        }
    }
}
