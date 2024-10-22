impl fastn_p1::Section<'_> {
    pub fn with_name<'a>(from: usize, to: usize) -> fastn_p1::Section<'a> {
        fastn_p1::Section {
            name: fastn_p1::KindedName {
                kind: None,
                name: fastn_p1::Sourced {
                    from,
                    to,
                    value: "foo",
                },
            },
            ..Default::default()
        }
    }
}
