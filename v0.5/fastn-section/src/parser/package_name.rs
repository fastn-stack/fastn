/// Parses a package name from the scanner.
///
/// Package names follow domain/hostname rules and can be:
/// - Single words: `fastn`, `mypackage`, `test42`
/// - Domain-like: `foo.com`, `example.co.uk`, `foo.bar.com`
/// - With hyphens: `my-package`, `foo-bar.com`
/// - Starting with numbers: `123foo.com`, `42` (valid hostnames)
/// - IP addresses: `192.168.1.1`
///
/// The alias is extracted from the first segment before the first dot.
/// For example:
/// - `foo.com` → name: "foo.com", alias: "foo"
/// - `example.co.uk` → name: "example.co.uk", alias: "example"
/// - `fastn` → name: "fastn", alias: "fastn"
///
/// Valid characters:
/// - Can start with: alphanumeric
/// - Can contain: alphanumeric, dots (.), hyphens (-)
/// - Cannot contain: underscores, slashes, or other special characters
///
/// This follows standard DNS hostname conventions where underscores are not
/// allowed in hostnames (though they may appear in some DNS records).
pub fn package_name(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Option<fastn_section::PackageName> {
    let first = scanner.peek()?;
    // Package names can start with alphanumeric (valid for domains/hostnames)
    if !first.is_alphanumeric() {
        return None;
    }

    // Allow alphanumeric, dots, and hyphens (valid in domains)
    let span = scanner.take_while(|c| c.is_alphanumeric() || c == '.' || c == '-')?;

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
        // Basic cases
        t!(" foo.com", null, " foo.com"); // Can't start with space
        t!("foo.com", "foo.com as foo");
        t!("foo.com ", "foo.com as foo", " ");

        // Single word packages
        t!("fastn", "fastn");
        t!("mypackage", "mypackage");

        // Multiple dots
        t!("foo.bar.com", "foo.bar.com as foo");
        t!("example.co.uk", "example.co.uk as example");

        // Numbers in package names (valid for domains)
        t!("foo123.com", "foo123.com as foo123");
        t!("test42", "test42");
        t!("123foo.com", "123foo.com as 123foo"); // Valid - domains can start with numbers
        t!("42", "42"); // Valid - single number is valid hostname

        // Hyphens in package names (valid in domains)
        t!("foo-bar.com", "foo-bar.com as foo-bar");
        t!("my-package", "my-package");
        t!("test-123", "test-123");

        // Package name stops at invalid domain characters
        t!("foo.com/bar", "foo.com as foo", "/bar");
        t!("foo.com#bar", "foo.com as foo", "#bar");

        // Underscores are not valid in domains - parser stops at them
        t!("foo.com_bar", "foo.com as foo", "_bar");
        t!("test_123", "test", "_123");

        // Can't start with underscore
        f!("_test");

        // Edge cases
        t!("a.b", "a.b as a");
        t!("x", "x");
        t!("192.168.1.1", "192.168.1.1 as 192"); // IP addresses work too
    }
}
