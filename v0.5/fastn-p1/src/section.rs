impl fastn_p1::Section<'_> {
    pub fn with_name<'a>(_name: fastn_p1::Token) -> fastn_p1::Section<'a> {
        fastn_p1::Section {
            name: fastn_p1::KindedName {
                kind: None,
                name: fastn_p1::Sourced {
                    from: 0,
                    to: 0,
                    value: "foo",
                },
            },
            ..Default::default()
        }
    }
}
