#[async_trait::async_trait]
pub trait DocumentStore {
    // /// if a document is not yet processed, the compiler needs to get the source of the document
    // /// to process it.
    // ///
    // /// the `document` parameter is the path that was found in the `import` statement, mapping the
    // /// document to a file in the file system is the job of the DS implementation.
    // ///
    // /// we are actually not going to have this method because any call to `unresolved` will
    // /// internally call the source, parse the sections, update the unresolved in db, and return
    // /// the resolved symbol, or an error.
    // ///
    // /// this is being done so the logic lives with implementor and compiler code is simplified.
    // ///
    // /// at times, we will still need entire document structure, eg all sections of a document,
    // /// for which we have the `unresolved_document` method
    // async fn source(&mut self, document: &str) -> Result<String>;

    /// this takes the qualified name, and returns the unresolved definition
    async fn unresolved(
        &mut self,
        qualified_identifier: &str,
    ) -> fastn_section::Result<fastn_unresolved::Definition>;
    /// this takes the qualified name, and returns the resolved definition.
    ///
    /// we cannot always get symbols by id because the source contains symbol names.
    /// TODO: fix type
    async fn resolved(
        &mut self,
        qualified_identifier: &str,
    ) -> fastn_section::Result<fastn_unresolved::Definition>;
    /// each symbol also has an id, and when referring to symbols we use their ids
    /// TODO: fix return type
    async fn resolved_by_id(
        &mut self,
        qualified_identifier: usize,
    ) -> fastn_section::Result<fastn_unresolved::Definition>;
    // we are removing save_resolved_definitions too, because our compiler can simply return the
    // newly resolved definitions, there by making the trait even simpler.
    //
    // /// persist is called multiple times, at each "resolve-symbols" cycle and at the end of the
    // /// compilation process, with the generated JavaScript for the main document.
    // ///
    // /// we do not send the name of JS to save, as that corresponds to the document being rendered.
    // ///
    // /// in one cycle, we could have resolved one or more symbols (as much progress we can make based
    // /// on existing resolved symbols, or unresolved symbols that do not depend on each other.
    // ///
    // /// the backend should also store the `fastn_lang::resolved::Definition::deps()` in the
    // /// dependency table.
    // async fn save_resolved_definitions(
    //     &mut self,
    //     definitions: Vec<fastn_lang::resolved::Definition>,
    //     // /// after more thought, we can leave the job of persisting JS to the implementor, so we
    //     // /// can further reduce the work compiler has to do.
    //     // js: Option<String>,
    // ) -> fastn_section::Result<()>;
    // /// this returns the unresolved document; actually we kind of never have to do this for regular
    // /// js generation, as the fastn_lang::unresolved::Document should be passed to compiler to begin
    // /// with.
    // async fn unresolved_document(
    //     &mut self,
    //     document: &str,
    // ) -> Result<fastn_lang::unresolved::Document>;
}
