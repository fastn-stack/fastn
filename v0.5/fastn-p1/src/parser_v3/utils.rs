pub fn extend_range(a: &mut fastn_p1::Span, b: fastn_p1::Span) {
    assert_eq!(a.end, b.start);
    a.end = b.end;
}
