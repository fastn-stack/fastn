pub fn main() {
    let id = std::env::args().nth(1);

    let dir = std::path::Path::new("./examples/");

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

-- ftd.column:
padding-horizontal: 40
padding-vertical: 20

-- ftd.text: FTD Examples
role: $heading
padding-bottom: 20

"
    )
    .to_string();

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

fn write(id: &str, doc: String) {
    use std::io::Write;
    let start = std::time::Instant::now();
    print!("Processing: {} ... ", id);
    let lib = ftd::ExampleLibrary {};

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
    lib: &ftd::ExampleLibrary,
) -> ftd::p1::Result<ftd::p2::Document> {
    let mut s = ftd::interpret(name, source)?;
    let document;
    loop {
        match s {
            ftd::Interpreter::Done { document: doc } => {
                document = doc;
                break;
            }
            ftd::Interpreter::StuckOnProcessor { state, section } => {
                let value = lib.process(&section, &state.tdoc(&mut Default::default()))?;
                s = state.continue_after_processor(&section, value)?;
            }
            ftd::Interpreter::StuckOnImport { module, state: st } => {
                let source =
                    lib.get_with_result(module.as_str(), &st.tdoc(&mut Default::default()))?;
                s = st.continue_after_import(module.as_str(), source.as_str())?;
            }
            ftd::Interpreter::StuckOnForeignVariable {
                state: st,
                variable,
            } => {
                s = st.continue_after_variable(
                    variable.as_str(),
                    ftd::Value::None {
                        kind: ftd::p2::Kind::Object { default: None },
                    },
                )?;
            }
        }
    }
    Ok(document)
}
