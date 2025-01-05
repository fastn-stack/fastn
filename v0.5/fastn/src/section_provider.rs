#[derive(Default)]
pub struct SectionProvider {
    cache: std::collections::HashMap<Option<String>, fastn_utils::section_provider::NResult>,
}

impl SectionProvider {
    pub async fn read<T, C>(&mut self, reader: fastn_continuation::Result<C>) -> T
    where
        C: fastn_continuation::Continuation<
                Output = fastn_utils::section_provider::PResult<T>,
                Needed = Vec<String>,
                Found = fastn_utils::section_provider::Found,
            > + Default,
    {
        match reader.mut_consume_async(self).await {
            Ok((value, warnings)) => {
                for warning in warnings {
                    eprintln!("{warning:?}");
                }
                value
            }
            Err(diagnostics) => {
                eprintln!("failed to parse package: ");
                for diagnostic in diagnostics {
                    eprintln!("{diagnostic:?}");
                }
                std::process::exit(1);
            }
        }
    }
}

#[async_trait::async_trait]
impl fastn_continuation::AsyncMutProvider for &mut SectionProvider {
    type Needed = Vec<String>;
    type Found = fastn_utils::section_provider::Found;

    async fn provide(&mut self, needed: Vec<String>) -> Self::Found {
        // file name will be FASTN.ftd for current package. for dependencies the file name will be
        // <name-of-package>/FASTN.ftd.
        let mut r: Self::Found = vec![];
        for f in needed {
            let (package, package_dir) = fastn_utils::section_provider::name_to_package(&f);

            if let Some(doc) = self.cache.get(&package) {
                r.push((package, doc.clone()));
                continue;
            }

            let file_list = get_file_list(&package_dir);

            match tokio::fs::read_to_string(&format!("{package_dir}FASTN.ftd")).await {
                Ok(v) => {
                    let d = fastn_section::Document::parse(&arcstr::ArcStr::from(v));
                    self.cache
                        .insert(package.clone(), Ok((d.clone(), file_list.clone())));
                    r.push((package, Ok((d, file_list))));
                }
                Err(e) => {
                    eprintln!("failed to read file: {e:?}");
                    let e = std::sync::Arc::new(e);
                    self.cache.insert(package.clone(), Err(e.clone()));
                    r.push((package, Err(e)));
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
