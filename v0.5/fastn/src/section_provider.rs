#[derive(Default)]
pub struct SectionProvider {
    #[expect(clippy::type_complexity)]
    cache: std::collections::HashMap<
        String,
        Result<Option<(fastn_section::Document, Vec<String>)>, fastn_section::Error>,
    >,
}

type NResult = std::result::Result<
    Option<(fastn_section::Document, Vec<String>)>,
    fastn_section::Spanned<fastn_section::Error>,
>;

#[async_trait::async_trait]
impl fastn_continuation::AsyncMutProvider for &mut SectionProvider {
    type Needed = Vec<String>;
    type Found = Vec<(String, NResult)>;

    async fn provide(&mut self, needed: Vec<String>) -> Self::Found {
        // file name will be FASTN.ftd for current package. for dependencies the file name will be
        // <name-of-package>/FASTN.ftd.
        let mut r = vec![];
        for f in needed {
            if let Some(doc) = self.cache.get(&f) {
                r.push((f, doc.clone()));
                continue;
            }

            let (file_to_read, file_list) = match f.split_once('/') {
                Some((package, rest)) => {
                    assert_eq!("FASTN.ftd", rest);
                    let package_dir = format!(".fastn/packages/{package}/");
                    (
                        format!("{package_dir}FASTN.ftd"),
                        get_file_list(package_dir.as_str()),
                    )
                }
                None => {
                    assert_eq!("FASTN.ftd", &f);
                    (f.to_string(), get_file_list("."))
                }
            };

            match tokio::fs::read_to_string(&file_to_read).await {
                Ok(v) => {
                    let d = fastn_section::Document::parse(&arcstr::ArcStr::from(v));
                    self.cache
                        .insert(f.clone(), Ok(Some((d.clone(), file_list.clone()))));
                    r.push((f, Ok(Some((d, file_list)))));
                }
                Err(e) => {
                    eprintln!("failed to read file: {e:?}");
                    self.cache.insert(f.clone(), Ok(None));
                    r.push((f, Ok(None)));
                }
            }
        }
        r
    }
}

fn get_file_list(package_dir: &str) -> Vec<String> {
    let file_walker = ignore::WalkBuilder::new(package_dir)
        .hidden(false)
        .git_ignore(true)
        .git_exclude(true)
        .git_global(true)
        .ignore(true)
        .parents(true)
        .build();

    let mut files = vec![];
    for path in file_walker.flatten() {
        if path.path().is_dir() {
            continue;
        }

        let file_name = match path.path().to_str() {
            Some(v) => v.to_string(),
            None => {
                eprintln!("file path is not valid: {:?}", path.path());
                continue;
            }
        };

        if file_name.starts_with(".git/")
            || file_name.starts_with(".github/")
            || file_name.eq(".gitignore")
        {
            continue;
        }

        files.push(file_name);
    }

    files
}
