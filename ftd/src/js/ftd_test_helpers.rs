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
                if let Ok(value) = std::fs::read_to_string(format!("./t/js/{}.ftd", module)) {
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

fn test_available_code_themes() -> String {
    let themes = ftd::theme_css();
    let mut result = vec![];
    for theme in themes.keys() {
        result.push(format!(
            "fastn_dom.codeData.availableThemes[\"{theme}\"] = \"../../theme_css/{theme}.css\";"
        ))
    }
    result.join("\n")
}

fn get_dummy_package_data() -> String {
    return indoc::indoc! {
        "
        let __fastn_package_name__ = \"foo\";
        "
    }
    .trim()
    .to_string();
}

#[track_caller]
fn p(s: &str, t: &str, fix: bool, manual: bool, script: bool, file_location: &std::path::PathBuf) {
    let i = interpret_helper("foo", s).unwrap_or_else(|e| panic!("{:?}", e));
    let js_ast_data = ftd::js::document_into_js_ast(i);
    let js_document_script = fastn_js::to_js(js_ast_data.asts.as_slice(), "foo");
    let js_ftd_script = fastn_js::to_js(ftd::js::default_bag_into_js_ast().as_slice(), "foo");
    let dummy_package_data = get_dummy_package_data();

    let html_str = {
        if script {
            format!(
                indoc::indoc! {"
                        <html>
                        <script>
                        {dummy_package_data}
                        {all_js}
                        {js_ftd_script}
                        {js_document_script}
                        fastn_virtual.ssr(main);
                        </script>
                        </html>
                    "},
                dummy_package_data = dummy_package_data,
                all_js = fastn_js::all_js_with_test(),
                js_ftd_script = js_ftd_script,
                js_document_script = js_document_script
            )
        } else {
            let ssr_body = fastn_js::ssr_with_js_string(
                "foo",
                format!("{js_ftd_script}\n{js_document_script}").as_str(),
            );

            ftd::ftd_js_html()
                .replace("__extra_js__", "")
                .replace("__fastn_package__", dummy_package_data.as_str())
                .replace(
                    "__js_script__",
                    format!("{js_document_script}{}", test_available_code_themes()).as_str(),
                )
                .replace("__favicon_html_tag__", "")
                .replace("__html_body__", ssr_body.as_str())
                .replace("<base href=\"__base_url__\">", "")
                .replace(
                    "__script_file__",
                    format!(
                        "{}{}",
                        js_ast_data.scripts.join(""),
                        if manual {
                            format!(
                                r#"
                            <script src="../../prism/prism.js"></script>
                            <script src="../../prism/prism-line-highlight.js"></script>
                            <script src="../../prism/prism-line-numbers.js"></script>
                            <script src="../../prism/prism-rust.js"></script>
                            <script src="../../prism/prism-json.js"></script>
                            <script src="../../prism/prism-python.js"></script>
                            <script src="../../prism/prism-markdown.js"></script>
                            <script src="../../prism/prism-bash.js"></script>
                            <script src="../../prism/prism-sql.js"></script>
                            <script src="../../prism/prism-javascript.js"></script>
                            <link rel="stylesheet" href="../../prism/prism-line-highlight.css">
                            <link rel="stylesheet" href="../../prism/prism-line-numbers.css">
                            <script>{}</script>
                        "#,
                                ftd::js::all_js_without_test("foo")
                            )
                        } else {
                            "<script src=\"fastn-js.js\"></script>".to_string()
                        }
                    )
                    .as_str(),
                )
                .replace(
                    "__default_css__",
                    (if manual { ftd::ftd_js_css() } else { "" })
                        .to_string()
                        .as_str(),
                )
        }
    };
    if fix || manual || script {
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
    let script = cli_args.iter().any(|v| v.eq("script=true"));
    let clear = cli_args.iter().any(|v| v.eq("clear"));
    let path = cli_args.iter().find_map(|v| v.strip_prefix("path="));
    for (files, html_file_location) in find_file_groups(manual, script) {
        if clear {
            for f in &files {
                match path {
                    Some(path) if !f.to_str().unwrap().contains(path) => continue,
                    _ => {}
                }
                let script = filename_with_second_last_extension_replaced_with_json(f, false, true);

                if std::fs::remove_file(&script).is_ok() {
                    println!("Removed {}", script.display());
                }
                let manual = filename_with_second_last_extension_replaced_with_json(f, true, false);
                if std::fs::remove_file(&manual).is_ok() {
                    println!("Removed {}", manual.display());
                }
            }
            continue;
        }

        let t = if fix || manual || script {
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
                } else if script {
                    "Creating script file"
                } else {
                    "testing"
                },
                f.display()
            );
            p(&s, &t, fix, manual, script, &html_file_location);
        }
    }
}

fn find_file_groups(
    manual: bool,
    script: bool,
) -> Vec<(Vec<std::path::PathBuf>, std::path::PathBuf)> {
    let files = {
        let mut f = ftd::utils::find_all_files_matching_extension_recursively("t/js", "ftd");
        f.sort();
        f
    };

    let mut o: Vec<(Vec<std::path::PathBuf>, std::path::PathBuf)> = vec![];

    for f in files {
        let json = filename_with_second_last_extension_replaced_with_json(&f, manual, script);
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
    script: bool,
) -> std::path::PathBuf {
    let stem = path.file_stem().unwrap().to_str().unwrap();

    path.with_file_name(format!(
        "{}{}.html",
        match stem.split_once('.') {
            Some((b, _)) => b,
            None => stem,
        },
        if manual {
            ".manual"
        } else if script {
            ".script"
        } else {
            ""
        }
    ))
}
