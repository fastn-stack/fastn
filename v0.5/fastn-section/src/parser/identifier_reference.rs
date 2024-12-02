pub fn identifier_reference(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Option<fastn_section::IdentifierReference> {
    let first = scanner.peek()?;
    // the first character should be is_alphabetic or `_`
    if !first.is_alphabetic() && first != '_' {
        return None;
    }

    // later characters should be is_alphanumeric or `_` or `-`
    let span = scanner.take_while(|c| {
        // we allow foo-bar.com/bar#yo as valid identifier reference
        c.is_alphabetic() || c == '_' || c == '-' || c == '.' || c == '#' || c == '/'
    })?;

    match from_span(span.clone()) {
        Ok(v) => Some(v),
        Err(e) => {
            scanner.add_error(span, e);
            None
        }
    }
}

fn from_span(
    span: fastn_section::Span,
) -> Result<fastn_section::IdentifierReference, fastn_section::Error> {
    if let Some((module, name)) = span.str().split_once("#") {
        validate_name(name)?;
        let (package, module) = module.split_once("/").unwrap_or((module, ""));
        return Ok(fastn_section::IdentifierReference::Absolute {
            package: span.inner_str(package),
            module: if module.is_empty() {
                None
            } else {
                Some(span.inner_str(module))
            },
            name: span.inner_str(name),
        });
    }

    if let Some((module, name)) = span.str().split_once(".") {
        validate_name(name)?;

        return Ok(fastn_section::IdentifierReference::Imported {
            module: span.inner_str(module),
            name: span.inner_str(name),
        });
    }

    Ok(fastn_section::IdentifierReference::Local(span))
}

fn validate_name(name: &str) -> Result<(), fastn_section::Error> {
    if name.contains('.') || name.contains('#') || name.contains('/') {
        // TODO: make this a more fine grained error
        return Err(fastn_section::Error::InvalidIdentifier);
    }

    Ok(())
}

#[cfg(test)]
mod test {
    fastn_section::tt!(super::identifier_reference);

    #[test]
    fn identifier_reference() {
        // identifiers can't start with a space
        t!(" foo", null, " foo");
        t!("foo", "foo");
        t!("foo bar", "foo", " bar");
        t!("_foo bar", "_foo", " bar");
        t!("_foo-bar", "_foo-bar");
        t!("नम", "नम");
        t!("_नम-जन ", "_नम-जन", " ");
        t!("_नाम-जाने", "_नाम-जाने");
        t!("_नाम-जाने ", "_नाम-जाने", " ");
        // emoji is not a valid identifier
        t!("नम😦", "नम", "😦");
        t!("नम 😦", "नम", " 😦");
        t!("😦नम ", null, "😦नम ");
    }
}
