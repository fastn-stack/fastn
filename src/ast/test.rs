use {indoc::indoc, pretty_assertions::assert_eq}; // macro

#[track_caller]
fn p(s: &str, t: &Vec<ftd::ast::Ast>) {
    let sections = ftd::p11::parse(s, "foo").unwrap_or_else(|e| panic!("{:?}", e));
    let ast =
        ftd::ast::Ast::from_p1(sections.as_slice(), "foo").unwrap_or_else(|e| panic!("{:?}", e));
    assert_eq!(t, &ast,)
}

#[track_caller]
fn f(s: &str, m: &str) {
    let sections = ftd::p11::parse(s, "foo").unwrap_or_else(|e| panic!("{:?}", e));
    let ast = ftd::ast::Ast::from_p1(sections.as_slice(), "foo");
    match ast {
        Ok(r) => panic!("expected failure, found: {:?}", r),
        Err(e) => {
            let expected = m.trim();
            let f2 = e.to_string();
            let found = f2.trim();
            if expected != found {
                let patch = diffy::create_patch(expected, found);
                let f = diffy::PatchFormatter::new().with_color();
                print!(
                    "{}",
                    f.fmt_patch(&patch)
                        .to_string()
                        .replace("\\ No newline at end of file", "")
                );
                println!("expected:\n{}\nfound:\n{}\n", expected, f2);
                panic!("test failed")
            }
        }
    }
}

#[test]
fn ast_import() {
    p(
        "-- import: foo",
        &vec![ftd::ast::Ast::Import(ftd::ast::Import {
            module: "foo".to_string(),
            alias: None,
        })],
    );

    p(
        "-- import: foo as f",
        &vec![ftd::ast::Ast::Import(ftd::ast::Import {
            module: "foo".to_string(),
            alias: Some("f".to_string()),
        })],
    );

    f(
        "-- import:",
        "ASTParseError: foo:1 -> Expected value in caption for import statement, found: `None`",
    );

    f(
        indoc!(
            "
            -- import: foo

            -- ftd.text: Hello

            -- end: import
            "
        ),
        "ASTParseError: foo:1 -> Subsection not expected for import statement `Section { name: \
        \"import\", kind: None, caption: Some(KV(KV { line_number: 1, key: \"$caption$\", kind: \
        None, value: Some(\"foo\") })), headers: Headers([]), body: None, sub_sections: [Section { \
        name: \"ftd.text\", kind: None, caption: Some(KV(KV { line_number: 3, key: \"$caption$\", \
        kind: None, value: Some(\"Hello\") })), headers: Headers([]), body: None, sub_sections: [], \
        is_commented: false, line_number: 3, block_body: false }], is_commented: false, \
        line_number: 1, block_body: false }`",
    )
}

#[test]
fn ast_record() {
    p(
        indoc!(
            "
            -- record foo:
            string name:
            integer age: 40
            "
        ),
        &vec![ftd::ast::Ast::Record(
            ftd::ast::Record::new("foo")
                .add_field("name", "string", None)
                .add_field("age", "integer", Some(s("40"))),
        )],
    );

    p(
        indoc!(
            "
            -- record foo:
            integer age:

            -- string foo.details:

            This contains details for record `foo`.
            This is default text for the field details.
            It can be overridden by the variable of this type.
            "
        ),
        &vec![ftd::ast::Ast::Record(
            ftd::ast::Record::new("foo")
                .add_field("age", "integer", None)
                .add_field(
                    "details",
                    "string",
                    Some(s(indoc!(
                        "This contains details for record `foo`.
                        This is default text for the field details.
                        It can be overridden by the variable of this type."
                    ))),
                ),
        )],
    );

    f(
        indoc!(
            "
            -- record foo:
            string name:
            age:
            "
        ),
        "ASTParseError: foo:3 -> Can't find kind for record field: `\"age\"`",
    );
}

fn s(s: &str) -> String {
    s.to_string()
}
