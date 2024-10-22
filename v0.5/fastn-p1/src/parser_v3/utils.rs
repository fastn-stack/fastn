pub fn extend_range(a: &mut fastn_p1::Span, b: fastn_p1::Span) {
    assert_eq!(a.end, b.start);
    a.end = b.end;
}

pub fn spanned<T>(t: T, span: fastn_p1::Span) -> fastn_p1::Spanned<T> {
    fastn_p1::Spanned { value: t, span }
}

// this panics  if start and end do not fall in the span
pub fn subspan_from_end(
    span: fastn_p1::Span,
    begin_from_end: i32,
    end_from_end: i32,
) -> fastn_p1::Span {
    let start = span.end - begin_from_end as usize;
    let end = span.end - end_from_end as usize;
    assert!(start <= span.end);
    assert!(end <= span.end);
    fastn_p1::Span { start, end }
}

impl fastn_p1::ParseOutput {
    pub fn insert_comment(&mut self, span: fastn_p1::Span) {
        self.items.push(fastn_p1::parser_v3::utils::spanned(
            fastn_p1::Item::Comment,
            span,
        ));
    }
}
