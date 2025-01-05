// # note on error handling.
//
// we can handle error in this parser in such a way that our rest of parsers that come after,
// like router parser, and the actual compiler,
// can run even if there are some errors encountered in this phase.
//
// but for simplicity’s sake, we are going to not do that now, and return either a package object
// and warning if there are no errors or an error if there are any errors.

#[derive(Debug, Default)]
pub struct Reader {
    name: fastn_package::UR<(), String>,
    systems: Vec<fastn_package::UR<String, fastn_package::System>>,
    dependencies: Vec<fastn_package::UR<String, fastn_package::Dependency>>,
    auto_imports: Vec<fastn_package::AutoImport>,
    apps: Vec<fastn_package::UR<String, fastn_package::App>>,
    packages: std::collections::HashMap<String, fastn_package::Package>,
    diagnostics: Vec<fastn_section::Spanned<fastn_section::Diagnostic>>,
    // if both a/FASTN.ftd and b/FASTN.ftd need x/FASTN.ftd, this will contain x => [a, b].
    // this will reset on every "continue after".
    waiting_for: std::collections::HashMap<String, Vec<String>>,
}

fn collect_dependencies(
    waiting_for: &mut std::collections::HashMap<String, Vec<String>>,
    p: &fastn_package::Package,
) {
    if p.name.is_empty() {
        return;
    }
    for dependency in p.dependencies.iter() {
        waiting_for
            .entry(dependency.name.clone())
            .or_default()
            .push(p.name.clone());
    }
}

impl Reader {
    fn process_package(
        &mut self,
        doc: fastn_section::Document,
        new_dependencies: &mut std::collections::HashMap<String, Vec<String>>,
        expected_name: Option<&str>,
    ) -> Result<String, ()> {
        self.collect_diagnostics(&doc);
        match parse_package(doc) {
            Ok((package, warnings)) => {
                if let Some(expected_name) = expected_name {
                    assert_eq!(package.name, expected_name);
                }
                self.diagnostics.extend(
                    warnings
                        .into_iter()
                        .map(|v| v.map(fastn_section::Diagnostic::Warning)),
                );
                collect_dependencies(new_dependencies, &package);
                let package_name = package.name.clone();
                if !package_name.is_empty() {
                    self.packages.insert(package.name.clone(), package);
                }
                Ok(package_name)
            }
            Err(diagnostics) => {
                self.diagnostics.extend(diagnostics);
                Err(())
            }
        }
    }

    fn finalize(self) -> fastn_continuation::Result<Self> {
        if self
            .diagnostics
            .iter()
            .any(|d| matches!(d.value, fastn_section::Diagnostic::Error(_)))
        {
            return fastn_continuation::Result::Done(Err(self.diagnostics));
        }

        if self.waiting_for.is_empty() {
            return fastn_continuation::Result::Done(Ok((
                fastn_package::MainPackage {
                    name: self.name.into_resolved(),
                    systems: vec![],
                    apps: vec![],
                    packages: self.packages,
                },
                self.diagnostics
                    .into_iter()
                    .map(|v| v.map(|v| v.into_warning()))
                    .collect(),
            )));
        }

        let needed = self
            .waiting_for
            .keys()
            .map(|p| fastn_utils::section_provider::package_file(p))
            .collect();

        fastn_continuation::Result::Stuck(Box::new(self), needed)
    }

    fn collect_diagnostics(&mut self, doc: &fastn_section::Document) {
        for diagnostic in doc.diagnostics_cloned() {
            self.diagnostics.push(diagnostic);
        }
    }
}

impl fastn_continuation::Continuation for Reader {
    // we return a package object if we parsed, even a partial package.
    type Output = fastn_utils::section_provider::PResult<fastn_package::MainPackage>;
    type Needed = Vec<String>; // vec of file names
    type Found = fastn_utils::section_provider::Found;

    fn continue_after(
        mut self,
        n: fastn_utils::section_provider::Found,
    ) -> fastn_continuation::Result<Self> {
        let mut new_dependencies: std::collections::HashMap<String, Vec<String>> =
            Default::default();

        match self.name {
            // if the name is not resolved means this is the first attempt.
            fastn_package::UR::UnResolved(()) => {
                assert_eq!(n.len(), 1);
                assert_eq!(n[0].0, None);
                assert!(self.waiting_for.is_empty());

                match n.into_iter().next() {
                    Some((_name, Ok((doc, _file_list)))) => {
                        match self.process_package(doc, &mut new_dependencies, None) {
                            Ok(package_name) => {
                                if !package_name.is_empty() {
                                    self.name = fastn_package::UR::Resolved(Some(package_name));
                                }
                            }
                            Err(()) => return self.finalize(),
                        }
                    }
                    Some((_name, Err(_))) => {
                        self.diagnostics.push(fastn_section::Span::default().wrap(
                            fastn_section::Diagnostic::Error(
                                fastn_section::Error::PackageFileNotFound,
                            ),
                        ));
                        return self.finalize();
                    }
                    None => unreachable!("we did a check for this already, list has 1 element"),
                }
            }
            // even if we failed to find name, we still continue to process as many dependencies,
            // etc. as possible.
            // so this case handles both name found and name error cases.
            _ => {
                assert_eq!(n.len(), self.waiting_for.len());

                for d in n.into_iter() {
                    match d {
                        (Some(p), Ok((doc, _file_list))) => {
                            if let Err(()) =
                                self.process_package(doc, &mut new_dependencies, Some(&p))
                            {
                                return self.finalize();
                            }
                        }
                        (Some(_p), Err(_e)) => {
                            todo!()
                        }
                        (None, _) => panic!(
                            "only main package can be None, and we have already processed it"
                        ),
                    }
                }
            }
        }

        self.waiting_for.clear();
        self.waiting_for.extend(new_dependencies);
        self.finalize()
    }
}

fn parse_package(
    doc: fastn_section::Document,
) -> fastn_utils::section_provider::PResult<fastn_package::Package> {
    let warnings = vec![];
    let mut errors = vec![];

    let mut package = fastn_package::Package {
        name: "".to_string(),
        dependencies: vec![],
        auto_imports: vec![],
        favicon: None,
    };

    for section in doc.sections.iter() {
        let span = section.span();

        match section.simple_name() {
            Some("package") => {
                match section.simple_caption() {
                    Some(name) => package.name = name.to_string(),
                    None => {
                        // we do not bail at this point,
                        // missing package name is just a warning for now
                        errors.push(span.wrap(fastn_section::Error::PackageNameNotInCaption));
                    }
                };

                for header in section.headers.iter() {
                    match header.name() {
                        "favicon" => match header.simple_value() {
                            Some(v) => package.favicon = Some(v.to_string()),
                            None => errors.push(
                                header
                                    .name_span()
                                    .wrap(fastn_section::Error::ArgumentValueRequired),
                            ),
                        },
                        // TODO: default-language, language-<code> etc
                        _ => errors.push(
                            header
                                .name_span()
                                .wrap(fastn_section::Error::ExtraArgumentFound),
                        ),
                    }
                }
            }
            Some("dependency") => {
                match section.simple_caption() {
                    Some(name) => {
                        package.dependencies.push(fastn_package::Dependency {
                            name: name.to_string(),
                            capabilities: vec![],
                            dependencies: vec![],
                            auto_imports: vec![],
                        });
                    }
                    None => {
                        // we do not bail at this point,
                        // missing package name is just a warning for now
                        errors.push(span.wrap(fastn_section::Error::PackageNameNotInCaption));
                    }
                };

                for header in section.headers.iter() {
                    header
                        .name_span()
                        .wrap(fastn_section::Error::ExtraArgumentFound);
                }
            }
            Some("auto-import") => {
                todo!()
            }
            Some("app") => {
                todo!()
            }
            Some("urls") => {
                todo!()
            }
            Some(_t) => {
                errors.push(
                    section
                        .span()
                        .wrap(fastn_section::Error::UnexpectedSectionInPackageFile),
                );
            }
            None => {
                // we found a section without name.
                // this is an error that Document must already have collected it, nothing to do.
                return Err(doc.diagnostics());
            }
        }
    }

    if package.name.is_empty()
        && !errors
            .iter()
            .any(|v| v.value == fastn_section::Error::PackageDeclarationMissing)
    {
        // we do not bail at this point, missing package name is just a warning for now
        errors.push(
            fastn_section::Span::default().wrap(fastn_section::Error::PackageDeclarationMissing),
        );
    }

    Ok((package, warnings))
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    #[track_caller]
    fn ok<F>(main: &'static str, rest: std::collections::HashMap<&'static str, &'static str>, f: F)
    where
        F: FnOnce(fastn_package::MainPackage, Vec<fastn_section::Spanned<fastn_section::Warning>>),
    {
        let section_provider =
            fastn_utils::section_provider::test::SectionProvider::new(main, rest);
        let (package, warnings) = fastn_package::Package::reader()
            .consume(&section_provider)
            .unwrap();

        f(package, warnings)
    }

    #[track_caller]
    fn ok0<F>(main: &'static str, f: F)
    where
        F: FnOnce(fastn_package::MainPackage, Vec<fastn_section::Spanned<fastn_section::Warning>>),
    {
        ok(main, Default::default(), f)
    }

    #[track_caller]
    fn err<F>(main: &'static str, rest: std::collections::HashMap<&'static str, &'static str>, f: F)
    where
        F: FnOnce(Vec<fastn_section::Spanned<fastn_section::Diagnostic>>),
    {
        let section_provider =
            fastn_utils::section_provider::test::SectionProvider::new(main, rest);
        let diagnostics = fastn_package::Package::reader()
            .consume(&section_provider)
            .unwrap_err();

        f(diagnostics)
    }

    #[test]
    fn basic() {
        ok0(
            indoc! {"
                -- package: foo
            "},
            |package, warnings| {
                assert_eq!(package.name, "foo");
                assert!(warnings.is_empty());
            },
        );

        ok(
            indoc! {"
                -- package: foo

                -- dependency: bar
            "},
            std::collections::HashMap::from([("bar", "-- package: bar")]),
            |package, warnings| {
                // TODO: use fastn_section::Debug to make these terser more exhaustive
                assert_eq!(package.name, "foo");
                assert_eq!(package.packages.len(), 2);
                assert!(warnings.is_empty());
                let bar = package.packages.get("bar").unwrap();
                assert_eq!(bar.name, "bar");
                assert_eq!(bar.dependencies.len(), 0);
            },
        );
    }
}
