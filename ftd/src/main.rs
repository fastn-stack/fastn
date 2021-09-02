pub fn main() {
    let dir = std::path::Path::new("./examples/");

    let mut write_doc =
        "-- ftd.text: Examples Index\nsize: 50\npadding-bottom: 20\nstyle: bold\n".to_string();

    if dir.is_dir() {
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
                    "{}\n-- ftd.text: {} \n link: /{}\n\n \n-- ftd.text: {}-rt \n link: /{}\n",
                    write_doc,
                    id.replace(".ftd", ""),
                    id.replace(".ftd", ".html"),
                    id.replace(".ftd", ""),
                    id.replace(".ftd", "-rt.html")
                );
            }
        }
    }
    write("index.ftd", write_doc);

    std::fs::copy("../ftd-rt/pkg/ftd_rt.js", "./build/ftd_rt.js")
        .expect("cant copy ftd_rt.js file");

    std::fs::copy("../ftd-rt/pkg/ftd_rt_bg.wasm", "./build/ftd_rt_bg.wasm")
        .expect("cant copy ftd_rt.js file");
}

fn write(id: &str, doc: String) {
    use std::io::Write;

    let lib = ftd::p2::TestLibrary {};
    let b = match ftd::p2::Document::from(id, &*doc, &lib) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to parse {}: {:?}", id, &e);
            return;
        }
    };
    let data = {
        let mut d: ftd_rt::Map = Default::default();
        for (k, v) in b.data.iter() {
            if let ftd::p2::Thing::Variable(ftd::Variable {
                value: ftd::Value::Boolean { value },
                ..
            }) = v
            {
                d.insert(k.to_string(), value.to_string());
            }
        }
        d
    };
    let doc = ftd_rt::Document {
        data,
        tree: b.main.to_node(),
    };
    let _dir = std::fs::create_dir_all("./build").expect("failed to create build folder");
    let mut f = std::fs::File::create(format!("./build/{}", id.replace(".ftd", ".html")))
        .expect("failed to create .html file");

    // TODO: indent things properly
    f.write_all(
        std::fs::read_to_string("ftd.html")
            .expect("cant read ftd.html")
            .replace(
                "___ftd___",
                b.main
                    .to_node()
                    .to_html(&Default::default(), &Default::default())
                    .as_str(),
            )
            .as_bytes(),
    )
    .expect("failed to write to .html file");

    let mut rt = std::fs::File::create(format!("./build/{}", id.replace(".ftd", "-rt.html")))
        .expect("failed to create .html file");
    rt.write_all(
        std::fs::read_to_string("rt.html")
            .expect("cant read rt.html")
            .replace(
                "___ftd_json___",
                serde_json::to_string(&doc)
                    .expect("failed to convert document to json")
                    .as_str(),
            )
            .as_bytes(),
    )
    .expect("failed to write to .html file");
}
