use pretty_assertions::assert_eq; // macro

pub fn interpret_helper(
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

#[track_caller]
fn p(s: &str, t: &str, fix: bool, file_location: &std::path::PathBuf) {
    let doc = interpret_helper("foo", s).unwrap_or_else(|e| panic!("{:?}", e));
    let executor =
        ftd::executor::ExecuteDoc::from_interpreter(doc).unwrap_or_else(|e| panic!("{:?}", e));
    let html = executor.main.to_node("foo").to_html("foo");
    let html_str = ftd::html1::utils::trim_all_lines(
        std::fs::read_to_string("ftd.html")
            .expect("cant read ftd.html")
            .replace("__ftd_doc_title__", "")
            .replace("__ftd_data__", "")
            .replace("__ftd_external_children__", "")
            .replace("__ftd__", html.as_str())
            .replace("__ftd_js__", "")
            .replace("__ftd_body_events__", "")
            .replace("__ftd_css__", "")
            .replace("__ftd_element_css__", "")
            .as_str(),
    );
    if fix {
        std::fs::write(file_location, html_str).unwrap();
        return;
    }
    assert_eq!(&t, &html_str, "Expected JSON: {}", html_str)
}

#[test]
fn html_test_all() {
    // we are storing files in folder named `t` and not inside `tests`, because `cargo test`
    // re-compiles the crate and we don't want to recompile the crate for every test
    let cli_args: Vec<String> = std::env::args().collect();
    let fix = cli_args.iter().any(|v| v.eq("fix"));
    for (files, json) in find_file_groups() {
        let t = std::fs::read_to_string(&json).unwrap();
        for f in files {
            let s = std::fs::read_to_string(&f).unwrap();
            println!("{} {}", if fix { "fixing" } else { "testing" }, f.display());
            p(&s, &t, fix, &json);
        }
    }
}

fn find_all_files_matching_extension_recursively(
    dir: impl AsRef<std::path::Path>,
    extension: &str,
) -> Vec<std::path::PathBuf> {
    let mut files = vec![];
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            files.extend(find_all_files_matching_extension_recursively(
                &path, extension,
            ));
        } else {
            match path.extension() {
                Some(ext) if ext == extension => files.push(path),
                _ => continue,
            }
        }
    }
    files
}

fn find_file_groups() -> Vec<(Vec<std::path::PathBuf>, std::path::PathBuf)> {
    let files = {
        let mut f = find_all_files_matching_extension_recursively("t/html", "ftd");
        f.sort();
        f
    };

    let mut o: Vec<(Vec<std::path::PathBuf>, std::path::PathBuf)> = vec![];

    for f in files {
        let json = filename_with_second_last_extension_replaced_with_json(&f);
        match o.last_mut() {
            Some((v, j)) if j == &json => v.push(f),
            _ => o.push((vec![f], json)),
        }
    }

    o
}

fn filename_with_second_last_extension_replaced_with_json(
    path: &std::path::Path,
) -> std::path::PathBuf {
    let stem = path.file_stem().unwrap().to_str().unwrap();

    path.with_file_name(format!(
        "{}.html",
        match stem.split_once('.') {
            Some((b, _)) => b,
            None => stem,
        }
    ))
}
