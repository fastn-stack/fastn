#[derive(Debug, Default)]
pub struct State {
    name: String,
    file_list: std::collections::HashMap<String, Vec<String>>,
}

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
        mut self,
        n: fastn_utils::section_provider::Found,
    ) -> fastn_continuation::Result<Self> {
        assert_eq!(n.len(), 1);
        assert_eq!(n[0].0, None);
        match n.into_iter().next() {
            Some((_name, Ok((doc, _file_list)))) => {
                if let Some((name, deps)) = get_dependencies(doc) {
                    self.name = name.clone();
                    self.file_list.insert(name, deps);
                }
                todo!()
            }
            _ => todo!(),
        }
    }
}

fn get_dependencies(doc: fastn_section::Document) -> Option<(String, Vec<String>)> {
    let mut name: Option<String> = None;
    let mut deps = vec![];

    for section in doc.sections.iter() {
        if let Some("package") = section.simple_name() {
            if let Some(n) = section.simple_caption() {
                name = Some(n.to_string());
            }
        }

        if let Some("dependency") = section.simple_name() {
            if let Some(name) = section.simple_caption() {
                deps.push(name.to_string());
            }
        }
    }

    name.map(|v| (v, deps))
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
