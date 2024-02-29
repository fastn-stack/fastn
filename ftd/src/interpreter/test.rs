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
fn p(s: &str, t: &str, fix: bool, file_location: &std::path::PathBuf) {
    let mut i = interpret_helper("foo", s).unwrap_or_else(|e| panic!("{:?}", e));
    for thing in ftd::interpreter::default::get_default_bag().keys() {
        i.data.swap_remove(thing);
    }
    let expected_json = serde_json::to_string_pretty(&i).unwrap();
    if fix {
        std::fs::write(file_location, expected_json).unwrap();
        return;
    }
    let t: ftd::interpreter::Document = serde_json::from_str(t)
        .unwrap_or_else(|e| panic!("{:?} Expected JSON: {}", e, expected_json));
    assert_eq!(&t, &i, "Expected JSON: {}", expected_json)
}

#[test]
fn interpreter_test_all() {
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
        let mut f =
            ftd::utils::find_all_files_matching_extension_recursively("t/interpreter", "ftd");
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
        "{}.json",
        match stem.split_once('.') {
            Some((b, _)) => b,
            None => stem,
        }
    ))
}

#[test]
fn evalexpr_test() {
    use fastn_grammar::evalexpr::*;
    let mut context = ftd::interpreter::default::default_context().unwrap();
    dbg!(fastn_grammar::evalexpr::build_operator_tree("$a >= $b").unwrap());
    dbg!(fastn_grammar::evalexpr::build_operator_tree(
        "(e = \"\"; ftd.is_empty(e)) && (d = \
        4; d > 7) && (6 > 7)"
    )
    .unwrap());
    dbg!(fastn_grammar::evalexpr::build_operator_tree("(6 > 7) && (true)").unwrap());
    assert_eq!(
        eval_with_context_mut(
            "(e = \"\"; ftd.is_empty(e)) && (d = 4; d > 7)",
            &mut context
        ),
        Ok(Value::from(false))
    );

    /*

        ExprNode {
        operator: RootNode,
        children: [
            ExprNode {
                operator: And,
                children: [
                    ExprNode {
                        operator: RootNode,
                        children: [
                            ExprNode {
                                operator: Gt,
                                children: [
                                    ExprNode {
                                        operator: Const {
                                            value: Int(
                                                6,
                                            ),
                                        },
                                        children: [],
                                    },
                                    ExprNode {
                                        operator: Const {
                                            value: Int(
                                                7,
                                            ),
                                        },
                                        children: [],
                                    },
                                ],
                            },
                        ],
                    },
                    ExprNode {
                        operator: Const {
                            value: Boolean(
                                true,
                            ),
                        },
                        children: [],
                    },
                ],
            },
        ],
    }

            ExprNode {
            operator: Add,
            children: [
                ExprNode {
                    operator: RootNode,
                    children: [
                        ExprNode {
                            operator: Gt,
                            children: [
                                ExprNode {
                                    operator: Const {
                                        value: Int(
                                            0,
                                        ),
                                    },
                                    children: [],
                                },
                                ExprNode {
                                    operator: Const {
                                        value: Int(
                                            2,
                                        ),
                                    },
                                    children: [],
                                },
                            ],
                        },
                    ],
                },
                ExprNode {
                    operator: RootNode,
                    children: [
                        ExprNode {
                            operator: Const {
                                value: Boolean(
                                    true,
                                ),
                            },
                            children: [],
                        },
                    ],
                },
            ],
        }
            */
}

#[test]
fn test_extract_kwargs() {
    let doc = interpret_helper(
        "foo",
        r#"-- component foo:
    kw-args data:

    -- ftd.text: Hello world

    -- end: foo

    -- foo:
    bar: Hello
    baz: World
"#,
    )
    .unwrap_or_else(|e| panic!("{:?}", e));

    let tdoc = doc.tdoc();

    let instructions = doc.get_instructions("foo#foo");
    let instruction = instructions.first().unwrap();
    let kwargs = instruction.get_kwargs("data", &tdoc).unwrap().unwrap();
    // TODO: resolve value based on kwarg kind
    let data: std::collections::BTreeMap<String, String> = kwargs
        .iter()
        .map(|(name, property_value)| {
            (
                name.to_string(),
                property_value.to_value().get_string_data().unwrap(),
            )
        })
        .collect();

    assert_eq!(data.get("bar"), Some(&String::from("Hello")));
    assert_eq!(data.get("baz"), Some(&String::from("World")));
}
