use pretty_assertions::assert_eq; // macro

pub fn interpret_helper(
    name: &str,
    source: &str,
) -> ftd::interpreter::Result<ftd::interpreter::Document> {
    let mut s = ftd::interpreter::interpret(name, source)?;
    let document;
    loop {
        match s {
            ftd::interpreter::Interpreter::Done { document: doc } => {
                document = doc;
                break;
            }
            ftd::interpreter::Interpreter::StuckOnImport {
                module, state: st, ..
            } => {
                let source = "";
                let mut foreign_variable = vec![];
                let mut foreign_function = vec![];
                if module.eq("test") {
                    foreign_variable.push("var".to_string());
                    foreign_function.push("fn".to_string());
                }
                let document = ftd::interpreter::ParsedDocument::parse(module.as_str(), source)?;
                s = st.continue_after_import(
                    module.as_str(),
                    document,
                    foreign_variable,
                    foreign_function,
                    0,
                )?;
            }
            ftd::interpreter::Interpreter::StuckOnProcessor {
                state, ast, module, ..
            } => {
                let variable_definition = ast.clone().get_variable_definition(module.as_str())?;
                let processor = variable_definition.processor.unwrap();
                let value = ftd::interpreter::Value::String {
                    text: variable_definition
                        .value
                        .caption()
                        .unwrap_or(processor)
                        .to_uppercase()
                        .to_string(),
                };
                s = state.continue_after_processor(value, ast)?;
            }
            ftd::interpreter::Interpreter::StuckOnForeignVariable {
                state,
                module,
                variable,
                ..
            } => {
                if module.eq("test") {
                    let value = ftd::interpreter::Value::String {
                        text: variable.to_uppercase().to_string(),
                    };
                    s = state.continue_after_variable(module.as_str(), variable.as_str(), value)?;
                } else {
                    return ftd::interpreter::utils::e2(
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

#[track_caller]
fn p(s: &str, t: &str, fix: bool, manual: bool, file_location: &std::path::PathBuf) {
    let i = interpret_helper("foo", s).unwrap_or_else(|e| panic!("{:?}", e));
    let js_ast_data = ftd::js::document_into_js_ast(i);
    let js_document_script = fastn_js::to_js(js_ast_data.asts.as_slice(), true);
    let js_ftd_script = fastn_js::to_js(ftd::js::default_bag_into_js_ast().as_slice(), false);
    let ssr_body =
        fastn_js::ssr_with_js_string(format!("{js_ftd_script}\n{js_document_script}").as_str());

    let html_str = std::fs::read_to_string("ftd-js.html")
        .expect("can't read ftd-js.html")
        .replace("__js_script__", js_document_script.as_str())
        .replace("__html_body__", ssr_body.as_str())
        .replace(
            "__script_file__",
            format!(
                "{}{}",
                js_ast_data.scripts.join(""),
                if manual {
                    format!("<script>\n{}\n</script>", ftd::js::all_js_with_test())
                } else {
                    "<script src=\"fastn-js.js\"></script>".to_string()
                }
            )
            .as_str(),
        );
    if fix || manual {
        std::fs::write(file_location, html_str).unwrap();
        return;
    }
    assert_eq!(&t, &html_str, "Expected HTML: {}", html_str)
}

#[test]
fn fastn_js_test_all() {
    // we are storing files in folder named `t` and not inside `tests`, because `cargo test`
    // re-compiles the crate and we don't want to recompile the crate for every test
    let cli_args: Vec<String> = std::env::args().collect();
    let fix = cli_args.iter().any(|v| v.eq("fix=true"));
    let manual = cli_args.iter().any(|v| v.eq("manual=true"));
    let path = cli_args.iter().find_map(|v| v.strip_prefix("path="));
    for (files, html_file_location) in find_file_groups(manual) {
        let t = if fix || manual {
            "".to_string()
        } else {
            std::fs::read_to_string(&html_file_location).unwrap()
        };
        for f in files {
            match path {
                Some(path) if !f.to_str().unwrap().contains(path) => continue,
                _ => {}
            }
            let s = std::fs::read_to_string(&f).unwrap();
            println!(
                "{} {}",
                if fix {
                    "fixing"
                } else if manual {
                    "Running manual test"
                } else {
                    "testing"
                },
                f.display()
            );
            p(&s, &t, fix, manual, &html_file_location);
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

fn find_file_groups(manual: bool) -> Vec<(Vec<std::path::PathBuf>, std::path::PathBuf)> {
    let files = {
        let mut f = find_all_files_matching_extension_recursively("t/js", "ftd");
        f.sort();
        f
    };

    let mut o: Vec<(Vec<std::path::PathBuf>, std::path::PathBuf)> = vec![];

    for f in files {
        let json = filename_with_second_last_extension_replaced_with_json(&f, manual);
        match o.last_mut() {
            Some((v, j)) if j == &json => v.push(f),
            _ => o.push((vec![f], json)),
        }
    }

    o
}

fn filename_with_second_last_extension_replaced_with_json(
    path: &std::path::Path,
    manual: bool,
) -> std::path::PathBuf {
    let stem = path.file_stem().unwrap().to_str().unwrap();

    path.with_file_name(format!(
        "{}{}.html",
        match stem.split_once('.') {
            Some((b, _)) => b,
            None => stem,
        },
        if manual { ".manual" } else { "" }
    ))
}
