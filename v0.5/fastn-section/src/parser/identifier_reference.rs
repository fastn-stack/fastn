/// Parses an identifier reference from the scanner.
///
/// An identifier reference can be:
/// - A simple local identifier: `foo`, `bar_baz`, `test-123`
/// - A qualified reference with dots: `module.name`, `ftd.text`
/// - An absolute reference with hash: `package#item`, `foo.com#bar`
/// - A path reference with slashes: `foo.com/bar#item`
///
/// The identifier must start with an alphabetic character or underscore.
/// Subsequent characters can be alphanumeric, underscore, hyphen, dot, hash, or slash.
/// Numbers are allowed after the first character (e.g., `foo123`, `test_42`).
///
/// This differs from a plain identifier which only allows alphanumeric, underscore, and hyphen.
/// The additional characters (`.`, `#`, `/`) enable parsing of qualified module references
/// and package paths used throughout the fastn system.
///
/// Examples:
/// - `foo` - simple local identifier
/// - `ftd.text` - qualified reference to text in ftd module
/// - `foo.com#bar` - absolute reference to bar in foo.com package
/// - `foo.com/bar#item` - reference to item in bar module of foo.com package
pub fn identifier_reference(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Option<fastn_section::IdentifierReference> {
    let first = scanner.peek()?;
    // the first character should be is_alphabetic or `_`
    if !first.is_alphabetic() && first != '_' {
        return None;
    }

    // later characters should be is_alphanumeric or `_` or `-` or special qualifiers
    let span = scanner.take_while(|c| {
        // we allow foo-bar.com/bar#yo as valid identifier reference
        c.is_alphanumeric() || c == '_' || c == '-' || c == '.' || c == '#' || c == '/'
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

        // Numbers in identifiers (after first character)
        t!("foo123", "foo123");
        t!("test_42", "test_42");
        t!("var-2-name", "var-2-name");
        t!("_9lives", "_9lives");
        t!("item0", "item0");
        t!("v2", "v2");

        // Can't start with a number
        t!("123foo", null, "123foo");
        t!("42", null, "42");
        t!("9_lives", null, "9_lives");

        // Mixed with unicode and numbers
        t!("नाम123", "नाम123");
        t!("test123 bar", "test123", " bar");

        // Valid qualified identifier references with . # /
        t!("module123.name456", "module123.name456");
        t!("pkg99#item2", "pkg99#item2");
        t!("foo123.com#bar456", "foo123.com#bar456");

        // With spaces, the identifier stops at the space
        t!("module123 . name456", "module123", " . name456");
        t!("pkg99 # item2", "pkg99", " # item2");
        t!("foo123 / bar", "foo123", " / bar");
        t!("test_42  .  field", "test_42", "  .  field");
        t!("var-2-name\t#\titem", "var-2-name", "\t#\titem");
        t!("_9lives   /   path", "_9lives", "   /   path");
        t!("item0 #", "item0", " #");
        t!("v2 .", "v2", " .");

        // Test complex patterns that module_name/qualified_identifier would handle
        t!("foo.com#bar", "foo.com#bar");
        t!("foo.com/module#item", "foo.com/module#item");
        t!("foo.com/path/to/module#item", "foo.com/path/to/module#item");
        t!("package#simple", "package#simple");
        t!("ftd.text", "ftd.text");
    }
}
