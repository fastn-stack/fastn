#[derive(Default)]
pub struct State {}

impl fastn_router::Router {
    pub fn reader() -> fastn_continuation::Result<State> {
        fastn_continuation::Result::Stuck(Default::default(), vec!["FASTN.ftd".to_string()])
    }
}

impl fastn_continuation::Continuation for State {
    type Output = fastn_utils::section_provider::PResult<fastn_router::Router>;
    type Needed = Vec<String>; // vec of file names
    type Found = fastn_utils::section_provider::Found;

    fn continue_after(
        self,
        _n: fastn_utils::section_provider::Found,
    ) -> fastn_continuation::Result<Self> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    #[track_caller]
    fn ok<F>(main: &'static str, rest: std::collections::HashMap<&'static str, &'static str>, f: F)
    where
        F: FnOnce(fastn_router::Router, Vec<fastn_section::Spanned<fastn_section::Warning>>),
    {
        let section_provider =
            fastn_utils::section_provider::test::SectionProvider::new(main, rest);
        let (package, warnings) = fastn_router::Router::reader()
            .consume(&section_provider)
            .unwrap();

        f(package, warnings)
    }

    #[test]
    fn basic() {
        ok(
            indoc! {"
                -- package: foo
            "},
            Default::default(),
            |package, warnings| {
                assert_eq!(package.name, "foo");
                assert!(warnings.is_empty());
            },
        );
    }
}
