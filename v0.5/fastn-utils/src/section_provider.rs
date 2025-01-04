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

fastn_utils::section_provider_ok! {
    "-- package: test",
    "foo.com/asdf" => "-- package: foo.com/asdf",
    |(package, warnings)| {
        assert_eq!(package.name, "test");
        assert!(warnings.is_empty());
    }
}

macro_rules! section_provider_ok {
    ($main:expr, $($file:expr => $content:expr),?, $block:expr) => {
        let main = indoc::indoc!($main);
    };
}

pub mod test {
    pub struct SectionProvider {
        pub data: std::collections::HashMap<String, (String, Vec<String>)>,
    }

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("file not found")]
        NotFound,
    }

    impl fastn_continuation::Provider for &SectionProvider {
        type Needed = Vec<String>;
        type Found = super::Found;

        fn provide(&self, needed: Vec<String>) -> Self::Found {
            let mut r = vec![];
            for f in needed {
                let package = super::name_to_package(&f).0;

                match self.data.get(&f) {
                    Some((content, file_list)) => {
                        let d = fastn_section::Document::parse(&arcstr::ArcStr::from(content));
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
