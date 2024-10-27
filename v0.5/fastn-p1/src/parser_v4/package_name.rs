/// package names for fastn as domain names.
///
/// domain names usually do not allow Unicode, and you have to use punycode.
/// but we allow Unicode in package names.
///
/// TODO: domain name can contain hyphens.
/// TODO: domain name can’t begin or end with a hyphen.
/// underscore is not permitted in domain names.
///
/// `.` is allowed in domain names.
/// TODO: domain name can't begin or end with a `.`.
/// TODO: `.` can't be repeated.
pub fn package_name(scanner: &mut fastn_p1::parser_v4::Scanner) -> Option<fastn_p1::PackageName> {
    let first = scanner.peek()?;
    if !first.is_alphabetic() {
        return None;
    }

    let start = scanner.index();
    scanner.pop();

    while let Some(c) = scanner.peek() {
        if !c.is_alphanumeric() && c != '.' {
            break;
        }
        scanner.pop();
    }

    Some(fastn_p1::PackageName {
        name: fastn_p1::Span {
            start,
            end: scanner.index(),
        },
    })
}

#[cfg(test)]
mod test {
    macro_rules! t {
        ($source:expr, $debug:tt, $remaining:expr) => {
            fastn_p1::parser_v4::p(
                $source,
                super::package_name,
                serde_json::json!($debug),
                $remaining,
            );
        };
    }

    #[test]
    fn package_name() {
        t!(" foo.com", null, " foo.com");
        t!("foo.com", "foo.com", "");
        t!("foo.com ", "foo.com", " ");
    }
}
