fn main() {
    let _c = fastn_resolved::ComponentInvocation {
        id: None,
        name: "ftd#text".to_string(),
        properties: vec![], // add hello-world caption etc.
        iteration: Box::new(None),
        condition: Box::new(None),
        events: vec![],
        children: vec![],
        source: Default::default(),
        line_number: 0,
    };

    // this main should create a HTML file, and store it in current folder as index.html etc.
}
