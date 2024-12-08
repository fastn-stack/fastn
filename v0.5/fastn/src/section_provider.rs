#[derive(Default)]
pub struct SectionProvider {}

#[async_trait::async_trait]
impl fastn_continuation::AsyncMutProvider for &SectionProvider {
    type Needed = Vec<String>;
    type Found = Vec<(String, Option<fastn_section::Document>)>;

    async fn provide(&mut self, _needed: Vec<String>) -> Self::Found {
        todo!()
    }
}
