#[test]
fn test_all() {
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
            p1(&s, &t, fix, &json);
        }
    }
}

#[track_caller]
fn p1(s: &str, t: &str, fix: bool, file_location: &std::path::PathBuf) {
    let data = super::parse("foo", s).unwrap_or_else(|e| panic!("{:?}", e));
    let expected_json = fastn_p1::sorted_json::to_json(&serde_json::to_value(&data).unwrap());
    if fix {
        std::fs::write(file_location, expected_json).unwrap();
        return;
    }
    assert_eq!(&t, &expected_json, "Expected JSON: {t}")
}

fn find_file_groups() -> Vec<(Vec<std::path::PathBuf>, std::path::PathBuf)> {
    let files = {
        let mut f = find_all_files_matching_extension_recursively("t/", "ftd");
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
