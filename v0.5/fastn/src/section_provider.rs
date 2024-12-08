#[derive(Default)]
pub struct SectionProvider {
    cache: std::collections::HashMap<String, Option<fastn_section::Document>>,
}

#[async_trait::async_trait]
impl<'a> fastn_continuation::AsyncMutProvider for &'a SectionProvider {
    type Needed = Vec<String>;
    type Found = Vec<(String, &'a Option<fastn_section::Document>)>;

    async fn provide(&mut self, needed: Vec<String>) -> Self::Found {
        let mut r = vec![];
        for f in needed {
            if let Some(doc) = self.cache.get(&f) {
                r.push((f, doc));
            }
        }
        r
    }
}
