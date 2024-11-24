fn main() {
    let c = fastn_resolved::ComponentInvocation {
        id: None,
        name: "ftd#text".to_string(),
        properties: vec![fastn_resolved::Property {
            value: fastn_resolved::Value::new_string("Hello World!").into_property_value(false, 0),
            source: Default::default(),
            condition: None,
            line_number: 0,
        }], // add hello-world caption etc.
        iteration: Box::new(None),
        condition: Box::new(None),
        events: vec![],
        children: vec![],
        source: Default::default(),
        line_number: 0,
    };

    let doc = fastn_resolved_to_js::TDoc {
        name: "foo", // Todo: Package name
        definitions: Default::default(),
        builtins: fastn_builtins::builtins(),
    };

    let output = fastn_resolved_to_js::get_all_asts(
        &doc,
        &[&c],
        std::iter::IntoIterator::into_iter([fastn_builtins::builtins().get("ftd#text").unwrap()]),
    );

    let js_document_script = fastn_js::to_js(output.ast.as_slice(), "foo");
    let js_ftd_script = fastn_js::to_js(
        fastn_resolved_to_js::default_bag_into_js_ast(&doc).as_slice(),
        "foo",
    );
    let js = format!("{js_ftd_script}\n{js_document_script}");
    let html = fastn_resolved_to_js::HtmlInput {
        package: fastn_resolved_to_js::Package::new_name("foo"), // Todo
        js,
        css_files: vec![],
        js_files: vec![],
        doc: Box::new(doc),
        has_rive_component: output.has_rive_components,
    };

    let html_str = html.to_test_html();

    std::fs::write(
        std::path::PathBuf::from("fastn-resolved-to-js/output.html"),
        html_str,
    )
    .unwrap();

    // this main should create a HTML file, and store it in current folder as index.html etc.
}
