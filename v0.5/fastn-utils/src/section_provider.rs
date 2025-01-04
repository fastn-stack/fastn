pub type PResult<T> = std::result::Result<
    (T, Vec<fastn_section::Spanned<fastn_section::Warning>>),
    Vec<fastn_section::Spanned<fastn_section::Diagnostic>>,
>;
pub type NResult = Result<(fastn_section::Document, Vec<String>), std::sync::Arc<std::io::Error>>;

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
        type Found = Vec<(Option<String>, super::NResult)>;

        fn provide(&self, needed: Vec<String>) -> Self::Found {
            let mut r = vec![];
            for f in needed {
                let package = match f.rsplit_once('/') {
                    Some((package, rest)) => {
                        assert_eq!("FASTN.ftd", rest);
                        Some(package.to_string())
                    }
                    None => {
                        assert_eq!("FASTN.ftd", &f);
                        None
                    }
                };

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
