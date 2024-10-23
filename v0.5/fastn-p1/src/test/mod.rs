#![allow(dead_code)]

mod sorted_json;

#[test]
fn test_all() {
    // we are storing files in folder named `t` and not inside `tests`, because `cargo test`
    // re-compiles the crate, and we don't want to recompile the crate for every test
    let cli_args: Vec<String> = std::env::args().collect();
    let fix = cli_args.iter().any(|v| v.eq("fix=true"));
    match cli_args.iter().find_map(|v| v.strip_prefix("path=")) {
        Some(path) => p1(path, fix),
        None => {
            for file in find_all_files_matching_extension_recursively("t/", "ftd") {
                p1(&file, fix);
            }
        }
    }
}

fn p1(file: impl AsRef<std::path::Path> + std::fmt::Debug, fix: bool) {
    let json = file.as_ref().with_extension("json");
    let s = {
        let mut s = std::fs::read_to_string(&file).unwrap();
        s.push('\n');
        s
    };
    println!("testing {file:?}");
    let output = fastn_p1::ParseOutput::new("foo", &s);
    let expected_json =
        fastn_p1::test::sorted_json::to_json(&serde_json::to_value(&output).unwrap());
    if fix {
        println!("fixing {file:?}");
        std::fs::write(json, expected_json).unwrap();
        return;
    }
    println!("testing {file:?}");
    let t = std::fs::read_to_string(&json).unwrap();
    assert_eq!(&t, &expected_json, "Expected JSON: {expected_json}")
}

pub fn find_all_files_matching_extension_recursively(
    dir: impl AsRef<std::path::Path> + std::fmt::Debug,
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
