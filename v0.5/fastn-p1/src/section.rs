impl fastn_p1::Section {
    pub fn with_name((start, end): (usize, usize), is_function: bool) -> fastn_p1::Section {
        fastn_p1::Section {
            name: fastn_p1::KindedName {
                kind: None,
                name: std::ops::Range { start, end },
            },
            is_function,
            ..Default::default()
        }
    }
}
