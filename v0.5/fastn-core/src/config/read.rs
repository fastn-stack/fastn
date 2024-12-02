impl fastn_core::Config {
    pub async fn read(
        _fastn_ftd: fastn_section::Document,
        arena: &mut fastn_unresolved::Arena,
    ) -> Result<Self, ReadError> {
        Ok(fastn_core::Config {
            sitemap: fastn_core::Sitemap {},
            auto_imports: desugar_auto_imports(arena, &[]),
            redirects: vec![],
            dynamic_routes: vec![],
        })
    }
}

#[derive(Debug)]
pub enum ReadError {
    SourceError(fastn_section::Error),
}

fn desugar_auto_imports(
    arena: &mut fastn_unresolved::Arena,
    _auto_imports: &[fastn_core::config::AutoImport],
) -> fastn_unresolved::AliasesID {
    let id = arena.new_aliases();
    let ftd = fastn_unresolved::SoM::Module(fastn_unresolved::Module::new("ftd", None, arena));
    let aliases = arena.aliases.get_mut(id).unwrap();
    aliases.insert("ftd".to_string(), ftd);
    id
}
