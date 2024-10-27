/// public | private | public<package> | public<module>
pub fn visibility(scanner: &mut fastn_p1::parser::Scanner) -> Option<fastn_p1::Visibility> {
    println!("started: {}", scanner.remaining());

    match scanner.one_of(&["public", "private"]) {
        Some("public") => (),
        Some("private") => return Some(fastn_p1::Visibility::Private),
        _ => return None,
    }

    let index = scanner.index();

    // we are here means we have `public`
    println!("here 0 {}", scanner.remaining());
    scanner.skip_spaces();
    println!("here 1 {}", scanner.remaining());

    if !scanner.take('<') {
        println!("here 2 {}", scanner.remaining());
        scanner.reset(index);
        return Some(fastn_p1::Visibility::Public);
    }
    scanner.skip_spaces();

    println!("here 3 {}", scanner.remaining());
    match scanner.one_of(&["package", "module"]) {
        Some("package") => {
            scanner.skip_spaces();
            if !scanner.take('>') {
                return None;
            }
            Some(fastn_p1::Visibility::Package)
        }
        Some("module") => {
            scanner.skip_spaces();
            if !scanner.take('>') {
                return None;
            }
            Some(fastn_p1::Visibility::Module)
        }
        _ => None,
    }
}

#[cfg(test)]
mod test {
    macro_rules! t {
        ($source:expr, $debug:tt, $remaining:expr) => {
            println!("source: {}", $source);
            fastn_p1::parser::p(
                $source,
                super::visibility,
                serde_json::json!($debug),
                $remaining,
            );
        };
    }

    #[test]
    fn visibility() {
        t!("public", "Public", "");
        t!("public ", "Public", " ");
        t!("private", "Private", "");
        t!("private ", "Private", " ");
        t!("public<package>", "Package", "");
        t!("public <package>", "Package", "");
        t!("public < package>", "Package", "");
        t!("public< package >", "Package", "");
        t!("public<package >", "Package", "");
        t!("public  <module>", "Module", "");
        t!("public  <    module>", "Module", "");
        t!("public\t<  \t  module\t> ", "Module", " ");
    }
}
