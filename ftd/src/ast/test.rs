use pretty_assertions::assert_eq; // macro

#[track_caller]
fn p(s: &str, t: &str, fix: bool, file_location: &std::path::PathBuf) {
    let sections = ftd::p1::parse(s, "foo").unwrap_or_else(|e| panic!("{:?}", e));
    let ast = ftd::ast::AST::from_sections(sections.as_slice(), "foo")
        .unwrap_or_else(|e| panic!("{:?}", e));
    let expected_json = serde_json::to_string_pretty(&ast).unwrap();
    if fix {
        std::fs::write(file_location, expected_json).unwrap();
        return;
    }
    let t: Vec<ftd::ast::AST> = serde_json::from_str(t)
        .unwrap_or_else(|e| panic!("{:?} Expected JSON: {}", e, expected_json));
    assert_eq!(&t, &ast, "Expected JSON: {}", expected_json)
}

/*#[track_caller]
fn f(s: &str, m: &str) {
    let sections = ftd::p1::parse(s, "foo").unwrap_or_else(|e| panic!("{:?}", e));
    let ast = ftd::ast::AST::from_sections(sections.as_slice(), "foo");
    match ast {
        Ok(r) => panic!("expected failure, found: {:?}", r),
        Err(e) => {
            let expected = m.trim();
            let f2 = e.to_string();
            let found = f2.trim();
            if expected != found {
                let patch = diffy::create_patch(expected, found);
                let f = diffy::PatchFormatter::new().with_color();
                print!(
                    "{}",
                    f.fmt_patch(&patch)
                        .to_string()
                        .replace("\\ No newline at end of file", "")
                );
                println!("expected:\n{}\nfound:\n{}\n", expected, f2);
                panic!("test failed")
            }
        }
    }
}*/

#[test]
fn ast_test_all() {
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
        let mut f = ftd::utils::find_all_files_matching_extension_recursively("t/ast", "ftd");
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
