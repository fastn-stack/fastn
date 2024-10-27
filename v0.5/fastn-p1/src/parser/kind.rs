/// example: `public list<string>`. ends with a space.
/// `foo<a, b>` | `foo<bar<k>>` | `foo<a, b<asd>, c, d>` | `foo<a, b, c, d, e>`
pub fn kind(_scanner: &mut fastn_p1::parser::Scanner) -> Option<fastn_p1::Kind> {
    todo!()
}
