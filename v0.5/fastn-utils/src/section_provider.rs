pub type PResult<T> = std::result::Result<
    (T, Vec<fastn_section::Spanned<fastn_section::Warning>>),
    Vec<fastn_section::Spanned<fastn_section::Diagnostic>>,
>;
pub type NResult = Result<(fastn_section::Document, Vec<String>), std::sync::Arc<std::io::Error>>;
pub type Found = Vec<(Option<String>, NResult)>;

pub fn name_to_package(name: &str) -> (Option<String>, String) {
    match name.rsplit_once('/') {
        Some((package, rest)) => {
            assert_eq!("FASTN.ftd", rest);
            (
                Some(package.to_string()),
                format!(".fastn/packages/{package}/"),
            )
        }
        None => {
            assert_eq!("FASTN.ftd", name);
            (None, "./".to_string())
        }
    }
}

pub fn package_file(package_name: &str) -> String {
    if package_name.ends_with('/') {
        format!("{package_name}FASTN.ftd")
    } else {
        format!("{package_name}/FASTN.ftd")
    }
}

#[cfg(feature = "test-utils")]
pub mod test {

    #[derive(Debug)]
    pub struct SectionProvider {
        pub data: std::collections::HashMap<String, (String, Vec<String>)>,
        pub arena: fastn_section::Arena,
    }

    impl SectionProvider {
        pub fn new(
            main: &'static str,
            mut rest: std::collections::HashMap<&'static str, &'static str>,
            arena: fastn_section::Arena,
        ) -> Self {
            let mut data = std::collections::HashMap::from([(
                "FASTN.ftd".to_string(),
                (main.to_string(), vec![]),
            )]);
            for (k, v) in rest.drain() {
                data.insert(
                    fastn_utils::section_provider::package_file(k),
                    (v.to_string(), vec![]),
                );
            }

            fastn_utils::section_provider::test::SectionProvider { data, arena }
        }
    }

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("file not found")]
        NotFound,
    }

    impl fastn_continuation::MutProvider for &mut SectionProvider {
        type Needed = Vec<String>;
        type Found = super::Found;

        fn provide(&mut self, needed: Vec<String>) -> Self::Found {
            let mut r = vec![];
            for f in needed {
                let package = super::name_to_package(&f).0;

                let module = match package {
                    Some(ref v) => fastn_section::Module::new(v, None, &mut self.arena),
                    None => fastn_section::Module::new("main", None, &mut self.arena),
                };

                match self.data.get(&f) {
                    Some((content, file_list)) => {
                        let d =
                            fastn_section::Document::parse(&arcstr::ArcStr::from(content), module);
                        r.push((package, Ok((d, file_list.to_owned()))));
                    }
                    None => {
                        r.push((
                            package,
                            Err(std::sync::Arc::new(std::io::Error::new(
                                std::io::ErrorKind::NotFound,
                                Error::NotFound,
                            ))),
                        ));
                    }
                };
            }

            r
        }
    }
}
