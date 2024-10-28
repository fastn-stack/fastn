impl fastn_p1::Section {
    pub fn with_name(
        name: fastn_p1::Span,
        function_marker: Option<fastn_p1::Span>,
    ) -> Box<fastn_p1::Section> {
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
