#[derive(Debug, Default)]
pub struct Reader {
    name: String,
    file_list: std::collections::HashMap<String, Vec<String>>,
    waiting_for: Vec<String>,
}

pub fn reader() -> fastn_continuation::Result<Reader> {
    fastn_continuation::Result::Stuck(Box::new(Reader::default()), vec!["FASTN.ftd".to_string()])
}

impl Reader {
    fn finalize(self) -> fastn_continuation::Result<Self> {
        let mut needed = vec![];
        for name in self.waiting_for.iter() {
            if !self.file_list.contains_key(name) {
                needed.push(fastn_utils::section_provider::package_file(name));
            }
        }

        if needed.is_empty() {
            return fastn_continuation::Result::Done(Ok((
                fastn_router::Router {
                    name: self.name,
                    file_list: self.file_list,
                    ..Default::default()
                },
                vec![],
            )));
        }

        fastn_continuation::Result::Stuck(Box::new(self), needed)
    }

    fn process_doc(&mut self, doc: fastn_section::Document, file_list: Vec<String>) {
        let (name, deps) = match get_dependencies(doc) {
            Some(v) => v,
            None => return,
        };

        if self.name.is_empty() {
            self.name = name.clone();
        }

        self.file_list.insert(name, file_list);
        self.waiting_for.extend(deps);
    }
}

impl fastn_continuation::Continuation for Reader {
    type Output = fastn_utils::section_provider::PResult<fastn_router::Router>;
    type Needed = Vec<String>; // vec of file names
    type Found = fastn_utils::section_provider::Found;

    fn continue_after(
        mut self,
        n: fastn_utils::section_provider::Found,
    ) -> fastn_continuation::Result<Self> {
        for (_name, result) in n.into_iter() {
            if let Ok((doc, file_list)) = result {
                self.process_doc(doc, file_list);
            }
        }

        self.finalize()
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
        let mut section_provider = fastn_utils::section_provider::test::SectionProvider::new(
            main,
            rest,
            fastn_section::Arena::default(),
        );
        let (package, warnings) = fastn_router::reader()
            .mut_consume(&mut section_provider)
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
