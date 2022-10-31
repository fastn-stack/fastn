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

    let dir = std::path::Path::new("./examples/");
    let new_ftd_dir = std::path::Path::new("./t/html/");

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
            "{}\n-- ftd.text: FTD-v2 Examples \npadding-bottom: 20\nrole: $small-heading\nlink: ftd-v2.html\n\n",
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
            ftd::interpreter2::Interpreter::StuckOnImport { module, state: st } => {
                let source = "";
                s = st.continue_after_import(module.as_str(), source)?;
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
    let html_ui =
        ftd::html1::HtmlUI::from_node_data(node, "main").unwrap_or_else(|e| panic!("{:?}", e));
    let ftd_js = std::fs::read_to_string("build.js").expect("build.js not found");
    let html_str = ftd::html1::utils::trim_all_lines(
        std::fs::read_to_string("build.html")
            .expect("cant read ftd.html")
            .replace("__ftd_doc_title__", "")
            .replace("__ftd_data__", html_ui.variables.as_str())
            .replace("__ftd_external_children__", "{}")
            .replace("__ftd__", html_ui.html.as_str())
            .replace("__ftd_js__", ftd_js.as_str())
            .replace(
                "__ftd_functions__",
                format!(
                    "{}\n{}",
                    html_ui.functions.as_str(),
                    html_ui.dependencies.as_str()
                )
                .as_str(),
            )
            .replace("__ftd_body_events__", "")
            .replace("__ftd_css__", "")
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
    let lib = ExampleLibrary {};

    let b = match interpret_helper(id, &*doc, &lib) {
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

    let ftd_js = std::fs::read_to_string("ftd.js").expect("ftd.js not found");

    let doc_title = match &b.title() {
        Some(x) => x.original.clone(),
        _ => id.to_string(),
    };

    f.write_all(
        std::fs::read_to_string("ftd.html")
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
    lib: &ExampleLibrary,
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

pub struct ExampleLibrary {}

impl ExampleLibrary {
    pub fn dummy_global_ids_map(&self) -> std::collections::HashMap<String, String> {
        let mut global_ids: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        global_ids.insert("foo".to_string(), "/foo/bar/#foo".to_string());
        global_ids.insert("hello".to_string(), "/hello/there/#hello".to_string());
        global_ids.insert("some id".to_string(), "/some/id/#some-id".to_string());

        // To debug for section
        global_ids.insert("scp".to_string(), "/foo/bar/#scp".to_string());
        global_ids.insert("sh".to_string(), "/hello/there/#sh".to_string());
        global_ids.insert("sb".to_string(), "/some/id/#sb".to_string());

        // To debug for subsection
        global_ids.insert("sscp".to_string(), "/foo/bar/#sscp".to_string());
        global_ids.insert("ssh".to_string(), "/hello/there/#ssh".to_string());
        global_ids.insert("ssb".to_string(), "/some/id/#ssb".to_string());

        // More dummy instances for debugging purposes
        global_ids.insert("a".to_string(), "/some/#a".to_string());
        global_ids.insert("b".to_string(), "/some/#b".to_string());
        global_ids.insert("c".to_string(), "/some/#c".to_string());
        global_ids.insert("d".to_string(), "/some/#d".to_string());

        // to debug in case of checkboxes
        global_ids.insert("x".to_string(), "/some/#x".to_string());
        global_ids.insert("X".to_string(), "/some/#X".to_string());

        global_ids
    }

    pub fn get(&self, name: &str, _doc: &ftd::p2::TDoc) -> Option<String> {
        std::fs::read_to_string(format!("./examples/{}.ftd", name)).ok()
    }

    /// checks if the current processor is a lazy processor
    /// or not
    ///
    /// for more details
    /// visit www.fpm.dev/glossary/#lazy-processor
    pub fn is_lazy_processor(
        section: &ftd::p1::Section,
        doc: &ftd::p2::TDoc,
    ) -> ftd::p1::Result<bool> {
        Ok(section
            .header
            .str(doc.name, section.line_number, "$processor$")?
            .eq("page-headings"))
    }

    pub fn process(
        &self,
        section: &ftd::p1::Section,
        doc: &ftd::p2::TDoc,
    ) -> ftd::p1::Result<ftd::Value> {
        ftd::p2::utils::unknown_processor_error(
            format!("unimplemented for section {:?} and doc {:?}", section, doc),
            doc.name.to_string(),
            section.line_number,
        )
    }

    pub fn get_with_result(&self, name: &str, doc: &ftd::p2::TDoc) -> ftd::p1::Result<String> {
        match self.get(name, doc) {
            Some(v) => Ok(v),
            None => ftd::p2::utils::e2(format!("library not found: {}", name), "", 0),
        }
    }
}
