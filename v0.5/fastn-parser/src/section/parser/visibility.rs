pub fn visibility(
    scanner: &mut fastn_parser::section::Scanner,
) -> Option<fastn_parser::Visibility> {
    match scanner.one_of(&["public", "private"]) {
        Some("public") => (),
        Some("private") => return Some(fastn_parser::Visibility::Private),
        _ => return None,
    }

    let index = scanner.index();

    // we are here means we have `public`
    scanner.skip_spaces();

    if !scanner.take('<') {
        scanner.reset(index);
        return Some(fastn_parser::Visibility::Public);
    }
    scanner.skip_spaces();

    match scanner.one_of(&["package", "module"]) {
        Some("package") => {
            scanner.skip_spaces();
            if !scanner.take('>') {
                return None;
            }
            Some(fastn_parser::Visibility::Package)
        }
        Some("module") => {
            scanner.skip_spaces();
            if !scanner.take('>') {
                return None;
            }
            Some(fastn_parser::Visibility::Module)
        }
        _ => None,
    }
}

#[cfg(test)]
mod test {
    fastn_parser::tt!(super::visibility);

    #[test]
    fn visibility() {
        t!("public", "Public");
        t!("public ", "Public", " ");
        t!("private", "Private");
        t!("private ", "Private", " ");
        t!("public<package>", "Package");
        t!("public <package> ", "Package", " ");
        t!("public < package>", "Package");
        t!("public< package > ", "Package", " ");
        t!("public<package >   \t", "Package", "   \t");
        t!("public  <module>", "Module");
        t!("public  <    module>", "Module");
        t!("public\t<  \t  module\t> ", "Module", " ");
    }
}