mod body;
mod condition;
mod doc_comment;
mod header_value;
mod headers;
mod identifier;
mod identifier_reference;
mod kind;
mod kinded_name;
mod kinded_reference;
mod section;
mod section_init;
mod tes;
mod visibility;

#[cfg(test)]
pub mod test;

pub use body::body;
pub use condition::condition;
pub use doc_comment::{module_doc_comment, regular_doc_comment};
pub use header_value::header_value;
pub use headers::headers;
pub use identifier::identifier;
pub use identifier_reference::identifier_reference;
pub use kind::kind;
pub use kinded_name::kinded_name;
pub use kinded_reference::kinded_reference;
pub use section::section;
pub use section_init::section_init;
pub use tes::{tes_till, tes_till_newline};
pub use visibility::visibility;

impl fastn_section::Document {
    pub fn parse(
        source: &arcstr::ArcStr,
        module: fastn_section::Module,
    ) -> fastn_section::Document {
        let mut scanner = fastn_section::Scanner::new(
            source,
            Default::default(),
            module,
            fastn_section::Document {
                module,
                module_doc: None,
                sections: vec![],
                errors: vec![],
                warnings: vec![],
                comments: vec![],
                line_starts: vec![],
            },
        );
        document(&mut scanner);
        scanner.output
    }
}

pub fn document(scanner: &mut fastn_section::Scanner<fastn_section::Document>) {
    // Parse module-level documentation at the start of the file
    scanner.skip_spaces();
    scanner.output.module_doc = fastn_section::parser::module_doc_comment(scanner);
    scanner.skip_spaces();
    scanner.skip_new_lines();
    scanner.skip_spaces();

    loop {
        // Try to parse a section (which will handle its own doc comments)
        if let Some(section) = fastn_section::parser::section(scanner) {
            scanner.output.sections.push(section);
            scanner.skip_spaces();
            scanner.skip_new_lines();
            scanner.skip_spaces();
        } else {
            // No section found - check if there's an orphaned doc comment
            if let Some(doc_span) = fastn_section::parser::regular_doc_comment(scanner) {
                // Orphaned doc comment - report error
                scanner.add_error(doc_span, fastn_section::Error::UnexpectedDocComment);
                scanner.skip_spaces();
                scanner.skip_new_lines();
                scanner.skip_spaces();
                // Continue to try parsing more content
                continue;
            }
            // No more content to parse
            break;
        }
    }
}

#[cfg(test)]
mod document_tests {
    fn doc(
        scanner: &mut fastn_section::Scanner<fastn_section::Document>,
    ) -> fastn_section::Document {
        fastn_section::parser::document(scanner);
        scanner.output.clone()
    }

    fastn_section::tt!(doc);
    #[test]
    fn document() {
        // Document with module doc
        t!(
            ";-; This is a module doc
            ;-; It describes the entire file

            -- foo: Hello World",
            {
                "module-doc": ";-; This is a module doc\n;-; It describes the entire file\n",
                "sections": [{
                    "init": {"name": "foo"},
                    "caption": "Hello World"
                }]
            }
        );

        // Document without module doc
        t!(
            "-- foo: Hello World",
            {
                "sections": [{
                    "init": {"name": "foo"},
                    "caption": "Hello World"
                }]
            }
        );

        t!(
            "-- foo: Hello World from foo\n-- bar: Hello World from bar",
            {
                "sections": [
                    {
                        "init": {"name": "foo"},
                        "caption": "Hello World from foo"
                    },
                    {
                        "init": {"name": "bar"},
                        "caption": "Hello World from bar"
                    }
                ]
            }
        );

        // Section with doc comment
        t!(
            "
            ;;; Documentation
            -- foo: Hello",
            {
                "sections": [{
                    "init": {
                        "name": "foo",
                        "doc": ";;; Documentation\n"
                    },
                    "caption": "Hello"
                }]
            }
        );

        // Multiple sections with doc comments
        t!(
            "
            ;;; First section docs
            -- foo: First
            
            ;;; Second section docs
            -- bar: Second",
            {
                "sections": [
                    {
                        "init": {
                            "name": "foo",
                            "doc": ";;; First section docs\n"
                        },
                        "caption": "First"
                    },
                    {
                        "init": {
                            "name": "bar",
                            "doc": ";;; Second section docs\n"
                        },
                        "caption": "Second"
                    }
                ]
            }
        );

        // Orphaned doc comment at beginning
        t_err!(
            "
            ;;; This is orphaned
            
            -- foo: Section",
            {
                "sections": [{
                    "init": {"name": "foo"},
                    "caption": "Section"
                }]
            },
            "unexpected_doc_comment"
        );

        // Multiple orphaned doc comments
        t_err!(
            "
            ;;; First orphan
            
            ;;; Second orphan
            
            -- foo: Section",
            {
                "sections": [{
                    "init": {"name": "foo"},
                    "caption": "Section"
                }]
            },
            ["unexpected_doc_comment", "unexpected_doc_comment"]
        );

        // Orphaned doc comment at end of file
        t_err!(
            "
            -- foo: Section
            
            ;;; This doc comment has no section after it",
            {
                "sections": [{
                    "init": {"name": "foo"},
                    "caption": "Section"
                }]
            },
            "unexpected_doc_comment"
        );

        // Orphaned doc comment between sections (with blank line)
        t_err!(
            "
            -- foo: First
            
            ;;; Orphaned comment
            
            -- bar: Second",
            {
                "sections": [
                    {
                        "init": {"name": "foo"},
                        "caption": "First"
                    },
                    {
                        "init": {"name": "bar"},
                        "caption": "Second"
                    }
                ]
            },
            "unexpected_doc_comment"
        );
    }
}
