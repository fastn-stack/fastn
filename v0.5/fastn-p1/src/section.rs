impl fastn_p1::Section {
    pub fn with_name(
        name: fastn_p1::Span,
        function_marker: Option<fastn_p1::Span>,
    ) -> fastn_p1::Section {
        fastn_p1::Section {
            name: fastn_p1::KindedName { kind: None, name },
            function_marker,
            ..Default::default()
        }
    }
}
