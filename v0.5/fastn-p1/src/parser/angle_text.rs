/// angle_text parses the rest of the angle text.
///
/// `foo>` | `foo<bar>>` | `foo.bar>` | `foo.bar.baz>` | `foo-bar>` | `foo-bar-baz>` |
/// `foo-bar.baz>` | `foo<bar.baz>>` | `foo<bar-baz>>`
///
/// returns
///
/// when the angle text is called, the scanner is expected to be at the start of the angle text.
///
/// if it finds AngleText it returns it, if not it returns an error message, and advances the
/// scanner. e.g., `foo<>` is error, but at the end the cursor would be right after the closing `>`.
/// other errors: `foo.<bar>` (incomplete identifier), or `foo<bar.>` or `foo..bar<asd>` etc.
/// `foo<asd bar`, the cursor would be at the space.
fn angle_text(_scanner: &mut fastn_p1::parser::scanner::Scanner) -> bool {
    println!("angle_text");
    true
}
