pub fn main() {
    let id = std::env::args().nth(1);

    let dir = std::path::Path::new("./examples/");

    let mut write_doc = indoc::indoc!(
        "
-- ftd.font-size dsize:
line-height: 40
size: 40
tracking: 0.0

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
        for entry in std::fs::read_dir(dir).expect("./examples is not a directory") {
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
    let b = match ftd::p2::Document::from(id, &*doc, &lib) {
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
            .as_bytes(),
    )
    .expect("failed to write to .html file");
    let duration = start.elapsed();
    println!("Done {:?}", duration);
}
