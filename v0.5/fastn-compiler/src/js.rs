impl fastn_compiler::Compiler {
    pub(crate) fn js(&self) -> String {
        // this function should look a bit like ftd::js::document_into_js_ast(), we do not need
        // to construct the Document object there, but will fetch all the fields as variables

        // self.content should be all UR::R now
        let _resolved_content = self.resolved_content();
        // every symbol in self.symbol_used in the bag must be UR::R now
        let needed_symbols = self.needed_symbols();
        let _js_files = self.external_js_files(&needed_symbols);
        let _css_files = self.external_css_files(&needed_symbols);

        todo!()
    }
}
