pub fn package_name(
    scanner: &mut fastn_lang::Scanner<fastn_lang::section::Document>,
) -> Option<fastn_lang::section::PackageName> {
    let first = scanner.peek()?;
    if !first.is_alphabetic() {
        return None;
    }

    let span = scanner.take_while(|c| c.is_alphanumeric() || c == '.')?;

    let o_name = scanner.source(&span);
    let name = o_name.split_once('.').unwrap_or((o_name, "")).0;

    Some(fastn_lang::section::PackageName {
        alias: fastn_lang::Span {
            start: span.start,
            end: span.start + name.len(),
        },
        name: span,
    })
}

#[cfg(test)]
mod test {
    fastn_lang::tt!(super::package_name);

    #[test]
    fn package_name() {
        t!(" foo.com", null, " foo.com");
        t!("foo.com", "foo.com as foo");
        t!("foo.com ", "foo.com as foo", " ");
    }
}
