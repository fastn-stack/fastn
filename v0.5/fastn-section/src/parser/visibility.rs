/// Parses visibility modifiers for fastn declarations.
///
/// Visibility controls the accessibility scope of declarations like sections,
/// headers, and other elements. fastn supports several visibility levels
/// that determine where an item can be accessed from.
///
/// # Grammar
/// ```text
/// visibility = "private" | "public" | "public" spaces_or_tabs "<" ws scope ws ">"
/// scope = "package" | "module"
/// spaces_or_tabs = (space | tab)*
/// ws = (space | tab | newline | comment)*
/// comment = ";;" <any text until end of line>
/// ```
///
/// # Visibility Levels
/// - `private`: Only accessible within the current scope
/// - `public`: Accessible from anywhere  
/// - `public<package>`: Accessible within the same package
/// - `public<module>`: Accessible within the same module
///
/// # Parsing Rules
/// - The parser first checks for "public" or "private" keywords
/// - If "public" is found, it optionally looks for angle brackets with scope modifiers
/// - Between "public" and "<", only spaces and tabs are allowed (no newlines/comments)
/// - Inside angle brackets, whitespace, newlines, and comments are all allowed
/// - Multiple consecutive newlines and comments are allowed inside brackets
/// - If angle brackets are opened but not properly closed or contain invalid scope, returns `None`
///
/// # Examples
/// ```text
/// private              -> Visibility::Private
/// public               -> Visibility::Public
/// public<package>      -> Visibility::Package
/// public <module>      -> Visibility::Module (space before <)
/// public<
///   module
/// >                    -> Visibility::Module (newlines inside <>)
/// public<
///   ;; Accessible within module
///   module
/// >                    -> Visibility::Module (comments inside <>)
/// ```
///
/// # Returns
/// Returns `Some(Visibility)` if a valid visibility modifier is found, `None` otherwise.
#[allow(dead_code)]
pub fn visibility(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Option<fastn_section::Visibility> {
    match scanner.one_of(&["public", "private"]) {
        Some("public") => (),
        Some("private") => return Some(fastn_section::Visibility::Private),
        _ => return None,
    }

    let index = scanner.index();

    // we are here means we have `public`
    scanner.skip_spaces(); // Only spaces/tabs, not newlines or comments

    if !scanner.take('<') {
        scanner.reset(&index);
        return Some(fastn_section::Visibility::Public);
    }
    scanner.skip_all_whitespace();

    match scanner.one_of(&["package", "module"]) {
        Some("package") => {
            scanner.skip_all_whitespace();
            if !scanner.take('>') {
                return None;
            }
            Some(fastn_section::Visibility::Package)
        }
        Some("module") => {
            scanner.skip_all_whitespace();
            if !scanner.take('>') {
                return None;
            }
            Some(fastn_section::Visibility::Module)
        }
        _ => None,
    }
}

#[cfg(test)]
mod test {
    fastn_section::tt!(super::visibility);

    #[test]
    fn visibility() {
        // Basic cases
        t!("public", "Public");
        t!("public ", "Public", " ");
        t!("private", "Private");
        t!("private ", "Private", " ");

        // Package visibility - simple
        t!("public<package>", "Package");
        t!("public <package> ", "Package", " ");
        t!("public < package>", "Package");
        t!("public< package > ", "Package", " ");
        t!("public<package >   \t", "Package", "   \t");

        // Module visibility - simple
        t!("public  <module>", "Module");
        t!("public  <    module>", "Module");
        t!("public\t<  \t  module\t> ", "Module", " ");

        // Newlines inside angle brackets
        t!(
            "
            public<
            package>",
            "Package"
        );

        t!(
            "
            public<
              package
            >",
            "Package"
        );

        t!(
            "
            public<

              module

            >",
            "Module"
        );

        // Comments inside angle brackets
        t!(
            "
            public<;; comment
            package>",
            "Package"
        );

        t!(
            "
            public<
              ;; This is package scoped
              package
            >",
            "Package"
        );

        t!(
            "
            public<
              ;; Module visibility
              ;; Another comment
              module
            >",
            "Module"
        );

        // Mixed whitespace and comments
        t!(
            "
            public<
              	;; comment
              	module  
            	>",
            "Module"
        );
    }
}
