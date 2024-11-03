use fastn_lang::unresolved::{Definition, Document};

pub struct DS {}

#[async_trait::async_trait]
impl fastn_lang::DS for DS {
    async fn source(&mut self, document: &str) -> fastn_lang::Result<String> {
        todo!()
    }

    async fn unresolved(&mut self, qualified_identifier: &str) -> fastn_lang::Result<Definition> {
        todo!()
    }

    async fn resolved(&mut self, qualified_identifier: &str) -> fastn_lang::Result<fastn_lang::resolved::Definition> {
        todo!()
    }

    async fn add_resolved(&mut self, qualified_identifier: &str, resolved: fastn_lang::resolved::Definition) -> fastn_lang::Result<()> {
        todo!()
    }

    async fn unresolved_document(&mut self, document: &str) -> fastn_lang::Result<Vec<Document>> {
        todo!()
    }

    async fn resolved_document(&mut self, document: &str) -> fastn_lang::Result<Vec<fastn_lang::resolved::Document>> {
        todo!()
    }
}
