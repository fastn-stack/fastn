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
                let mut source = "".to_string();
                let mut foreign_variable = vec![];
                let mut foreign_function = vec![];
                if module.eq("test") {
                    foreign_variable.push("var".to_string());
                    foreign_function.push("fn".to_string());
                }
                if let Ok(value) = std::fs::read_to_string(format!("./t/html/{}.ftd", module)) {
                    source = value;
                }
                let document =
                    ftd::interpreter::ParsedDocument::parse(module.as_str(), source.as_str())?;

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
fn p(s: &str, t: &str, fix: bool, file_location: &std::path::PathBuf) {
    let doc = interpret_helper("foo", s).unwrap_or_else(|e| panic!("{:?}", e));
    let executor =
        ftd::executor::ExecuteDoc::from_interpreter(doc).unwrap_or_else(|e| panic!("{:?}", e));
    let node = ftd::node::NodeData::from_rt(executor);
    let html_ui = ftd::html::HtmlUI::from_node_data(node, "main", false)
        .unwrap_or_else(|e| panic!("{:?}", e));
    let ftd_js = std::fs::read_to_string("build.js").expect("build.js not found");
    let html_str = ftd::html::utils::trim_all_lines(
        std::fs::read_to_string("build.html")
            .expect("cant read ftd.html")
            .replace(
                "__ftd_meta_data__",
                ftd::html::utils::get_meta_data(&html_ui.html_data).as_str(),
            )
            .replace(
                "__ftd_doc_title__",
                html_ui.html_data.title.unwrap_or_default().as_str(),
            )
            .replace("__ftd_data__", html_ui.variables.as_str())
            .replace("__ftd_external_children__", "{}")
            .replace("__ftd__", html_ui.html.as_str())
            .replace("__ftd_js__", ftd_js.as_str())
            .replace(
                "__extra_js__",
                format!("{}{}", html_ui.js.as_str(), html_ui.rive_data.as_str()).as_str(),
            )
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
    let fix = cli_args.iter().any(|v| v.eq("fix=true"));
    let path = cli_args.iter().find_map(|v| v.strip_prefix("path="));
    for (files, json) in find_file_groups() {
        let t = if fix {
            "".to_string()
        } else {
            std::fs::read_to_string(&json).unwrap()
        };
        for f in files {
            match path {
                Some(path) if !f.to_str().unwrap().contains(path) => continue,
                _ => {}
            }
            let s = std::fs::read_to_string(&f).unwrap();
            println!("{} {}", if fix { "fixing" } else { "testing" }, f.display());
            p(&s, &t, fix, &json);
        }
    }
}

fn find_file_groups() -> Vec<(Vec<std::path::PathBuf>, std::path::PathBuf)> {
    let files = {
        let mut f = ftd0::utils::find_all_files_matching_extension_recursively("t/html", "ftd");
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
