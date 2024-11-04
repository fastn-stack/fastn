pub struct DS {}

#[expect(clippy::new_without_default)]
impl DS {
    pub fn new() -> DS {
        DS {}
    }
}

#[async_trait::async_trait]
impl fastn_lang::DS for DS {
    async fn source(&mut self, _document: &str) -> fastn_lang::Result<String> {
        todo!()
    }

    async fn unresolved(
        &mut self,
        _qualified_identifier: &str,
    ) -> fastn_lang::Result<fastn_lang::unresolved::Definition> {
        todo!()
    }

    async fn resolved(
        &mut self,
        _qualified_identifier: &str,
    ) -> fastn_lang::Result<fastn_lang::resolved::Definition> {
        todo!()
    }

    async fn add_resolved(
        &mut self,
        _qualified_identifier: &str,
        _resolved: fastn_lang::resolved::Definition,
    ) -> fastn_lang::Result<()> {
        todo!()
    }

    async fn unresolved_document(
        &mut self,
        _document: &str,
    ) -> fastn_lang::Result<Vec<fastn_lang::unresolved::Document>> {
        todo!()
    }

    async fn resolved_document(
        &mut self,
        _document: &str,
    ) -> fastn_lang::Result<Vec<fastn_lang::resolved::Document>> {
        todo!()
    }

    async fn purge(&mut self, _document: &str) -> fastn_lang::Result<()> {
        todo!()
    }

    async fn store_js(&mut self, _qualified_identifier: &str, _js: &str) -> fastn_lang::Result<()> {
        todo!()
    }

    async fn document_js(&mut self, _document: &str) -> Option<String> {
        todo!()
    }
}
