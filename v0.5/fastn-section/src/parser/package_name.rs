pub fn package_name(
    scanner: &mut fastn_section::Scanner<fastn_section::token::Document>,
) -> Option<fastn_section::token::PackageName> {
    let first = scanner.peek()?;
    if !first.is_alphabetic() {
        return None;
    }

    let span = scanner.take_while(|c| c.is_alphanumeric() || c == '.')?;

    let o_name = scanner.source(&span);
    let name = o_name.split_once('.').unwrap_or((o_name, "")).0;

    Some(fastn_section::token::PackageName {
        alias: fastn_section::Span {
            start: span.start,
            end: span.start + name.len(),
        },
        name: span,
    })
}

#[cfg(test)]
mod test {
    fastn_section::tt!(super::package_name);

    #[test]
    fn package_name() {
        t!(" foo.com", null, " foo.com");
        t!("foo.com", "foo.com as foo");
        t!("foo.com ", "foo.com as foo", " ");
    }
}
