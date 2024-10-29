impl fastn_p1::Section<'_> {
    pub fn with_name<'input>(
        name: fastn_p1::Span<'input>,
        function_marker: Option<fastn_p1::Span<'input>>,
    ) -> Box<fastn_p1::Section<'input>> {
        Box::new(fastn_p1::Section {
            init: fastn_p1::SectionInit {
                name: fastn_p1::KindedName {
                    kind: None,
                    name: name.into(),
                },
                ..Default::default()
            },
            function_marker,
            ..Default::default()
        })
    }
}
