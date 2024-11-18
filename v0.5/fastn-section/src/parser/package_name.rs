pub fn package_name(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Option<fastn_section::PackageName> {
    let first = scanner.peek()?;
    if !first.is_alphabetic() {
        return None;
    }

    let span = scanner.take_while(|c| c.is_alphanumeric() || c == '.')?;

    let name = span.str().split_once('.').unwrap_or((span.str(), "")).0;

    Some(fastn_section::PackageName {
        alias: scanner.span_range(span.start(), span.start() + name.len()),
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
