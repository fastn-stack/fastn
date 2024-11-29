impl fastn_core::Config {
    pub async fn read(
        _fastn_ftd: fastn_section::Document,
        _interner: &mut string_interner::DefaultStringInterner,
    ) -> Result<Self, ReadError> {
        Ok(fastn_core::Config {
            sitemap: fastn_core::Sitemap {},
            auto_imports: vec![
                // fastn_unresolved::UR::UnResolved(
                //     fastn_unresolved::Definition {
                //         // symbol requires us to know the module, which we do not know right now, so
                //         // we add empty values here, only the name.
                //         symbol: Some(fastn_unresolved::Symbol::new()),
                //         doc: None,
                //         name: fastn_unresolved::UR::UnResolved(fastn_section::Identifier::new(
                //             "ftd".to_string(),
                //         )),
                //         visibility: fastn_section::Visibility::Public,
                //         inner: fastn_unresolved::InnerDefinition::ModuleAlias(
                //             fastn_unresolved::Module::new("ftd", ""),
                //         ),
                //     },
                // )
            ],
            redirects: vec![],
            dynamic_routes: vec![],
        })
    }
}

#[derive(Debug)]
pub enum ReadError {
    SourceError(fastn_section::Error),
}
