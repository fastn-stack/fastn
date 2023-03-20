#[allow(unreachable_code)]
#[allow(dead_code)]
fn t() {
    // The returned nodes are created in the supplied Arena, and are bound by its lifetime.
    let arena = comrak::Arena::new();

    let root = comrak::parse_document(
        &arena,
        "This is my input.\n\n1. Also my input.\n2. Certainly my input.\n",
        &comrak::ComrakOptions::default(),
    );

    dbg!(root);
    return;

    fn iter_nodes<'a, F>(node: &'a comrak::nodes::AstNode<'a>, f: &F)
    where
        F: Fn(&'a comrak::nodes::AstNode<'a>),
    {
        f(node);
        for c in node.children() {
            iter_nodes(c, f);
        }
    }

    iter_nodes(root, &|node| {
        dbg!(root);
        dbg!(node);
        // match &mut node.data.borrow_mut().value {
        //     &mut NodeValue::Text(ref mut text) => {
        //         let orig = std::mem::replace(text, vec![]);
        //         *text = String::from_utf8(orig).unwrap().replace("my", "your").as_bytes().to_vec();
        //     }
        //     _ => (),
        // }
    });
}

pub fn main() {
    // t();
    // return;

    let id = std::env::args().nth(1);

    if id.is_some() && id.as_ref().unwrap().eq("bm") {
        use std::io::Write;

        let mut log = "".to_string();
        let benchmark_dir = std::path::Path::new("./benchmark-2022/");
        for entry in std::fs::read_dir(benchmark_dir)
            .unwrap_or_else(|_| panic!("{:?} is not a directory", benchmark_dir.to_str()))
        {
            let path = entry.expect("no files inside ./benchmark-2022").path();
            let source = path
                .to_str()
                .unwrap_or_else(|| panic!("Path {:?} cannot be convert to string", path));
            let split: Vec<_> = source.split('/').collect();
            let id = split.last().expect("Filename should be present");
            if id.contains(".ftd") {
                let start = std::time::Instant::now();
                log = format!("{}Processing: {} ... ", log, id);
                let doc = std::fs::read_to_string(source).expect("cant read file");
                ftd_v2_write(id, doc.as_str());
                log = format!("{}Done {:?}\n", log, start.elapsed());
            }
        }
        let mut f =
            std::fs::File::create("./benchmark-2022/.log").expect("failed to create .html file");
        f.write_all(log.as_bytes())
            .expect("failed to write to .log file");
        return;
    }
    let dir = std::path::Path::new("./ftd/examples/");
    let new_ftd_dir = std::path::Path::new("./ftd/t/html/");

    let mut write_doc = indoc::indoc!(
        "
-- ftd.font-size dsize:
line-height: 40
size: 40
letter-spacing: 0

-- ftd.type heading: cursive
weight: 800
style: italic
desktop: $dsize
mobile: $dsize
xl: $dsize

-- ftd.font-size small-dsize:
line-height: 22
size: 20
letter-spacing: 0

-- ftd.type small-heading: cursive
weight: 800
style: italic
desktop: $small-dsize
mobile: $small-dsize
xl: $small-dsize

-- ftd.column:
padding-horizontal: 40
padding-vertical: 20

-- ftd.text: FTD Examples
role: $heading
padding-bottom: 20

"
    )
    .to_string();

    if id.is_none() && new_ftd_dir.is_dir() {
        let mut new_ftd_dir_write_doc = write_doc.clone();
        write_doc = format!(
            "{}\n-- ftd.text: FTD-v3 (EDITION: 2022) Examples \npadding-bottom: 20\nrole: \
            $small-heading\nlink: ftd-v2.html\n\n",
            write_doc,
        );
        for entry in std::fs::read_dir(new_ftd_dir)
            .unwrap_or_else(|_| panic!("{:?} is not a directory", new_ftd_dir.to_str()))
        {
            let path = entry.expect("no files inside ./examples").path();
            let source = path
                .to_str()
                .unwrap_or_else(|| panic!("Path {:?} cannot be convert to string", path));
            let split: Vec<_> = source.split('/').collect();
            let id = split.last().expect("Filename should be present");
            if id.contains(".ftd") {
                let doc = std::fs::read_to_string(source).expect("cant read file");
                ftd_v2_write(id, doc.as_str());
                new_ftd_dir_write_doc = format!(
                    "{}\n-- ftd.text: {} \n link: {}\n\n",
                    new_ftd_dir_write_doc,
                    id.replace(".ftd", ""),
                    id.replace(".ftd", ".html"),
                );
            }
        }
        write("ftd-v2.ftd", new_ftd_dir_write_doc);
    }

    if let Some(id) = id {
        let path = format!("./examples/{}.ftd", id);
        let id = format!("{}.ftd", id);
        let doc = std::fs::read_to_string(path).expect("cant read file");
        write(&id, doc);
        write_doc = format!(
            "{}\n-- ftd.text: {} \n link: {}\n\n",
            write_doc,
            id.replace(".ftd", ""),
            id.replace(".ftd", ".html"),
        );
    } else if dir.is_dir() {
        for entry in std::fs::read_dir(dir)
            .unwrap_or_else(|_| panic!("{:?} is not a directory", dir.to_str()))
        {
            let path = entry.expect("no files inside ./examples").path();
            let source = path
                .to_str()
                .unwrap_or_else(|| panic!("Path {:?} cannot be convert to string", path));
            let split: Vec<_> = source.split('/').collect();
            let id = split.last().expect("Filename should be present");

            if id.contains(".ftd") {
                let doc = std::fs::read_to_string(source).expect("cant read file");
                write(id, doc);
                write_doc = format!(
                    "{}\n-- ftd.text: {} \n link: {}\n\n",
                    write_doc,
                    id.replace(".ftd", ""),
                    id.replace(".ftd", ".html"),
                );
            }
        }
    }
    write("index.ftd", write_doc);
    std::fs::create_dir_all("./docs/ftd/t/").expect("failed to create docs folder");
    std::fs::copy("./ftd/t/test.css", "./docs/ftd/t/test.css").expect("failed to copy test.css");
    std::fs::copy("./ftd/t/test.js", "./docs/ftd/t/test.js").expect("failed to copy test.js");
    std::fs::copy("./ftd/t/web_component.js", "./docs/ftd/t/web_component.js")
        .expect("failed to copy web_component.js");
}

pub fn ftd_v2_interpret_helper(
    name: &str,
    source: &str,
) -> ftd::interpreter2::Result<ftd::interpreter2::Document> {
    let mut s = ftd::interpreter2::interpret(name, source)?;
    let document;
    loop {
        match s {
            ftd::interpreter2::Interpreter::Done { document: doc } => {
                document = doc;
                break;
            }
            ftd::interpreter2::Interpreter::StuckOnImport {
                module, state: st, ..
            } => {
                let mut source = "".to_string();
                let mut foreign_variable = vec![];
                let mut foreign_function = vec![];
                if module.eq("test") {
                    foreign_variable.push("var".to_string());
                    foreign_function.push("fn".to_string());
                }
                if let Ok(value) = std::fs::read_to_string(format!("./ftd/t/html/{}.ftd", module)) {
                    source = value;
                }
                let document = ftd::interpreter2::ParsedDocument::parse_with_line_number(
                    module.as_str(),
                    source.as_str(),
                    0,
                )?;

                s = st.continue_after_import(
                    module.as_str(),
                    document,
                    foreign_variable,
                    foreign_function,
                    0,
                )?;
            }
            ftd::interpreter2::Interpreter::StuckOnProcessor {
                state, ast, module, ..
            } => {
                let variable_definition = ast.clone().get_variable_definition(module.as_str())?;
                let processor = variable_definition.processor.unwrap();
                let value = ftd::interpreter2::Value::String {
                    text: variable_definition
                        .value
                        .caption()
                        .unwrap_or(processor)
                        .to_uppercase()
                        .to_string(),
                };
                s = state.continue_after_processor(value, ast)?;
            }
            ftd::interpreter2::Interpreter::StuckOnForeignVariable {
                state,
                module,
                variable,
                ..
            } => {
                if module.eq("test") {
                    let value = ftd::interpreter2::Value::String {
                        text: variable.to_uppercase().to_string(),
                    };
                    s = state.continue_after_variable(module.as_str(), variable.as_str(), value)?;
                } else {
                    return ftd::interpreter2::utils::e2(
                        format!("Unknown module {}", module),
                        module.as_str(),
                        0,
                    );
                }
            }
        }
    }
    Ok(document)
}

fn ftd_v2_write(id: &str, s: &str) {
    use std::io::Write;
    let start = std::time::Instant::now();
    print!("Processing: {} ... ", id);
    let doc = ftd_v2_interpret_helper("foo", s).unwrap_or_else(|e| panic!("{:?}", e));
    let executor =
        ftd::executor::ExecuteDoc::from_interpreter(doc).unwrap_or_else(|e| panic!("{:?}", e));
    let node = ftd::node::NodeData::from_rt(executor);
    let html_ui = ftd::html1::HtmlUI::from_node_data(node, "main", false)
        .unwrap_or_else(|e| panic!("{:?}", e));
    let ftd_js = std::fs::read_to_string("./ftd/build.js").expect("build.js not found");
    let html_str = ftd::html1::utils::trim_all_lines(
        std::fs::read_to_string("./ftd/build.html")
            .expect("cant read ftd.html")
            .replace("__ftd_doc_title__", "")
            .replace("__ftd_data__", html_ui.variables.as_str())
            .replace("__ftd_external_children__", "{}")
            .replace("__ftd__", html_ui.html.as_str())
            .replace("__ftd_js__", ftd_js.as_str())
            .replace("__extra_js__", html_ui.js.as_str())
            .replace("__base_url__", "/")
            .replace("__extra_css__", html_ui.css.as_str())
            .replace(
                "__ftd_functions__",
                format!(
                    "{}\n{}\n{}\n{}\n{}\n{}\n{}",
                    html_ui.functions.as_str(),
                    html_ui.dependencies.as_str(),
                    html_ui.variable_dependencies.as_str(),
                    html_ui.dummy_html.as_str(),
                    html_ui.raw_html.as_str(),
                    html_ui.mutable_variable,
                    html_ui.immutable_variable
                )
                .as_str(),
            )
            .replace("__ftd_body_events__", html_ui.outer_events.as_str())
            .replace("__ftd_css__", ftd::css())
            .replace("__ftd_element_css__", "")
            .as_str(),
    );
    std::fs::create_dir_all("./docs").expect("failed to create docs folder");
    let mut f = std::fs::File::create(format!("./docs/{}", id.replace(".ftd", ".html")))
        .expect("failed to create .html file");
    f.write_all(html_str.as_bytes())
        .expect("failed to write to .html file");
    let duration = start.elapsed();
    println!("Done {:?}", duration);
}

fn write(id: &str, doc: String) {
    use std::io::Write;
    let start = std::time::Instant::now();
    print!("Processing: {} ... ", id);
    let lib = ftd::ExampleLibrary {};

    let b = match interpret_helper(id, &doc, &lib) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to parse {}: {:?}", id, &e);
            return;
        }
    };
    std::fs::create_dir_all("./docs").expect("failed to create docs folder");
    let mut f = std::fs::File::create(format!("./docs/{}", id.replace(".ftd", ".html")))
        .expect("failed to create .html file");

    let doc = b.to_rt("main", id);

    let ftd_js = std::fs::read_to_string("./ftd/ftd.js").expect("ftd.js not found");
    let test_css = std::fs::read_to_string("./ftd/t/test.css").expect("t/test.css not found");

    let doc_title = match &b.title() {
        Some(x) => x.original.clone(),
        _ => id.to_string(),
    };

    f.write_all(
        std::fs::read_to_string("./ftd/ftd.html")
            .expect("cant read ftd.html")
            .replace("__ftd_doc_title__", doc_title.as_str())
            .replace(
                "__ftd_data__",
                serde_json::to_string_pretty(&doc.data)
                    .expect("failed to convert document to json")
                    .as_str(),
            )
            .replace(
                "__ftd_external_children__",
                serde_json::to_string_pretty(&doc.external_children)
                    .expect("failed to convert document to json")
                    .as_str(),
            )
            .replace(
                "__extra_css__",
                format!("<style>{}</style>", test_css).as_str(),
            )
            .replace("__ftd__", doc.html.as_str())
            .replace("__ftd_js__", ftd_js.as_str())
            .replace("__ftd_body_events__", doc.body_events.as_str())
            .replace("__ftd_css__", ftd::css())
            .replace("__ftd_element_css__", doc.css_collector.as_str())
            .as_bytes(),
    )
    .expect("failed to write to .html file");
    let duration = start.elapsed();
    println!("Done {:?}", duration);
}

pub fn interpret_helper(
    name: &str,
    source: &str,
    lib: &ftd::ExampleLibrary,
) -> ftd::p1::Result<ftd::p2::Document> {
    let mut s = ftd::interpret(name, source, &None)?;
    let document;
    loop {
        match s {
            ftd::Interpreter::Done { document: doc } => {
                document = doc;
                break;
            }
            ftd::Interpreter::StuckOnProcessor { state, section } => {
                if ftd::ExampleLibrary::is_lazy_processor(
                    &section,
                    &state.tdoc(&mut Default::default(), &mut Default::default()),
                )? {
                    s = state.continue_after_storing_section(&section)?;
                } else {
                    let value = lib.process(
                        &section,
                        &state.tdoc(&mut Default::default(), &mut Default::default()),
                    )?;
                    s = state.continue_after_processor(&section, value)?;
                }
            }
            ftd::Interpreter::StuckOnImport { module, state: st } => {
                let source = lib.get_with_result(
                    module.as_str(),
                    &st.tdoc(&mut Default::default(), &mut Default::default()),
                )?;
                s = st.continue_after_import(module.as_str(), source.as_str())?;
            }
            ftd::Interpreter::StuckOnForeignVariable {
                state: st,
                variable,
            } => {
                s = st.continue_after_variable(
                    variable.as_str(),
                    ftd::Value::None {
                        kind: ftd::p2::Kind::Object {
                            default: None,
                            is_reference: false,
                        },
                    },
                )?;
            }
            ftd::Interpreter::CheckID {
                replace_blocks,
                state: st,
            } => {
                // No config in ftd::ExampleLibrary using dummy global_ids map for debugging
                let mut mapped_replace_blocks: Vec<
                    ftd::ReplaceLinkBlock<std::collections::HashMap<String, String>>,
                > = vec![];

                for (captured_id_set, source, ln) in replace_blocks.iter() {
                    let mut id_map: std::collections::HashMap<String, String> =
                        std::collections::HashMap::new();
                    for id in captured_id_set {
                        let link = lib
                            .dummy_global_ids_map()
                            .get(id)
                            .ok_or_else(|| ftd::p1::Error::ForbiddenUsage {
                                message: format!("id: {} not found while linking", id),
                                doc_id: st.id.clone(),
                                line_number: *ln,
                            })?
                            .to_string();
                        id_map.insert(id.to_string(), link);
                    }
                    mapped_replace_blocks.push((id_map, source.to_owned(), ln.to_owned()));
                }

                s = st.continue_after_checking_id(mapped_replace_blocks)?;
            }
        }
    }
    Ok(document)
}
