use ftd::interpreter::ComponentExt;
use pretty_assertions::assert_eq;

#[track_caller]
fn p(
    s: &str,
    t: &Option<String>,
    e: &Option<String>,
    fix: bool,
    file_location: &std::path::PathBuf,
    error_file_location: &std::path::PathBuf,
) {
    let mut i = match ftd::parse_doc("foo", s) {
        Ok(i) => i,
        Err(expected_error) => {
            if fix {
                let expected_error = expected_error.to_string();
                std::fs::write(error_file_location, expected_error).unwrap();
                if file_location.exists() {
                    std::fs::remove_file(file_location).unwrap();
                }
                return;
            } else {
                if t.is_some() {
                    panic!("{file_location:?} file not expected. found: {expected_error:?}");
                }
                match e.as_ref() {
                    Some(found_error) => {
                        let expected_error = expected_error.to_string();
                        assert_eq!(
                            found_error, &expected_error,
                            "Expected Error: {}",
                            expected_error
                        );
                        return;
                    }
                    None => {
                        panic!("{expected_error:?}");
                    }
                }
            }
        }
    };
    for thing in ftd::interpreter::default::builtins().keys() {
        i.data.swap_remove(thing);
    }
    let expected_json = serde_json::to_string_pretty(&i).unwrap();
    if fix {
        std::fs::write(file_location, expected_json).unwrap();
        return;
    }
    let t: ftd::interpreter::Document = serde_json::from_str(&t.clone().unwrap_or_default())
        .unwrap_or_else(|e| panic!("{e:?} Expected JSON: {expected_json}"));
    assert_eq!(&t, &i, "Expected JSON: {}", expected_json)
}

#[test]
fn interpreter_test_all() {
    // we are storing files in folder named `t` and not inside `tests`, because `cargo test`
    // re-compiles the crate and we don't want to recompile the crate for every test
    let cli_args: Vec<String> = std::env::args().collect();
    let fix = cli_args.iter().any(|v| v.eq("fix=true"));
    let path = cli_args.iter().find_map(|v| v.strip_prefix("path="));
    for (files, json, error) in find_file_groups() {
        let t = if fix {
            None
        } else {
            std::fs::read_to_string(&json).ok()
        };

        let e = if fix {
            None
        } else {
            std::fs::read_to_string(&error).ok()
        };

        for f in files {
            match path {
                Some(path) if !f.to_str().unwrap().contains(path) => continue,
                _ => {}
            }
            let s = std::fs::read_to_string(&f).unwrap();
            println!("{} {}", if fix { "fixing" } else { "testing" }, f.display());
            p(&s, &t, &e, fix, &json, &error);
        }
    }
}

fn find_file_groups() -> Vec<(
    Vec<std::path::PathBuf>,
    std::path::PathBuf,
    std::path::PathBuf,
)> {
    let files = {
        let mut f =
            ftd_p1::utils::find_all_files_matching_extension_recursively("t/interpreter", "ftd");
        f.sort();
        f
    };

    let mut o: Vec<(
        Vec<std::path::PathBuf>,
        std::path::PathBuf,
        std::path::PathBuf,
    )> = vec![];

    for f in files {
        let (json, error) = filename_with_second_last_extension_replaced_with_json(&f);
        match o.last_mut() {
            Some((v, j, _)) if j == &json => v.push(f),
            _ => o.push((vec![f], json, error)),
        }
    }

    o
}

fn filename_with_second_last_extension_replaced_with_json(
    path: &std::path::Path,
) -> (std::path::PathBuf, std::path::PathBuf) {
    let stem = path.file_stem().unwrap().to_str().unwrap();

    (
        path.with_file_name(format!(
            "{}.json",
            match stem.split_once('.') {
                Some((b, _)) => b,
                None => stem,
            }
        )),
        path.with_file_name(format!("{stem}.error")),
    )
}

#[test]
fn evalexpr_test() {
    use fastn_resolved::evalexpr::*;
    let mut context = ftd::interpreter::default::default_context().unwrap();
    dbg!(fastn_resolved::evalexpr::build_operator_tree("$a >= $b").unwrap());
    dbg!(
        fastn_resolved::evalexpr::build_operator_tree(
            "(e = \"\"; ftd.is_empty(e)) && (d = \
        4; d > 7) && (6 > 7)"
        )
        .unwrap()
    );
    dbg!(fastn_resolved::evalexpr::build_operator_tree("(6 > 7) && (true)").unwrap());
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
    let doc = ftd::parse_doc(
        "foo",
        r#"
            -- component fizz:
            kw-args data:

            -- ftd.text: Hello world

            -- end: fizz

            -- fizz:
            id: test
            bar: Hello
            baz: World
        "#,
    )
    .unwrap();

    let component = doc.get_component_by_id("test").unwrap();
    let data = component.get_kwargs(&doc, "data").unwrap();

    assert_eq!(data.get("bar"), Some(&String::from("Hello")));
    assert_eq!(data.get("baz"), Some(&String::from("World")));
}
