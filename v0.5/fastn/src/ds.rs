use fastn_lang::unresolved::Definition;

pub struct DS {}

#[expect(clippy::new_without_default)]
impl DS {
    pub fn new() -> DS {
        DS {}
    }
}

#[async_trait::async_trait]
impl fastn_lang::DS for DS {
    async fn unresolved(&mut self, _qualified_identifier: &str) -> fastn_lang::Result<Definition> {
        todo!()
    }

    async fn resolved(
        &mut self,
        _qualified_identifier: &str,
    ) -> fastn_lang::Result<fastn_lang::resolved::Definition> {
        todo!()
    }

    async fn resolved_by_id(
        &mut self,
        _qualified_identifier: usize,
    ) -> fastn_lang::Result<fastn_lang::resolved::Definition> {
        todo!()
    }

    async fn save_resolved_definitions(
        &mut self,
        _definitions: Vec<fastn_lang::resolved::Definition>,
    ) -> fastn_lang::Result<()> {
        todo!()
    }
}
