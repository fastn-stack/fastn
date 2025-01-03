// # note on error handling.
//
// we can handle error in this parser in such a way that our rest of parsers that come after,
// like router parser, and the actual compiler, can run even if there are some issues (error, not
// warning) encountered in this phase.
//
// but for simplicity’s sake, we are going to not do that now, and return either a package object
// if there are no errors (maybe warnings).
//
// even within this parser, we bail early if any one FASTN.ftd is found with errors in it, this is
// kind of acceptable as all FASTN.ftd files, other the one in the current package, must not have
// errors because they are published dependencies.
//
// though this is not strictly true, say if one of the dependencies needed some system, and main
// package has not provided it, then dependency can also have errors.

#[derive(Debug, Default)]
pub struct State {
    name: fastn_package::UR<(), String>,
    systems: Vec<fastn_package::UR<String, fastn_package::System>>,
    dependencies: Vec<fastn_package::UR<String, fastn_package::Dependency>>,
    pub auto_imports: Vec<fastn_package::AutoImport>,
    apps: Vec<fastn_package::UR<String, fastn_package::App>>,
    packages: std::collections::HashMap<String, fastn_package::Package>,
    pub diagnostics: Vec<fastn_section::Spanned<fastn_section::Diagnostic>>,
    // if both a/FASTN.ftd and b/FASTN.ftd need x/FASTN.ftd, this will contain x => [a, b].
    // this will reset on every "continue after".
    pub waiting_for: std::collections::HashMap<String, Vec<String>>,
}

type PResult<T> = std::result::Result<
    (T, Vec<fastn_section::Spanned<fastn_section::Warning>>),
    Vec<fastn_section::Spanned<fastn_section::Diagnostic>>,
>;
type NResult = Result<(fastn_section::Document, Vec<String>), std::sync::Arc<std::io::Error>>;

impl fastn_package::Package {
    pub fn reader() -> fastn_continuation::Result<State> {
        fastn_continuation::Result::Stuck(Default::default(), vec!["FASTN.ftd".to_string()])
    }
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

impl State {
    fn finalize(self) -> fastn_continuation::Result<Self> {
        if self
            .diagnostics
            .iter()
            .any(|d| matches!(d.value, fastn_section::Diagnostic::Error(_)))
        {
            return fastn_continuation::Result::Done(Err(self.diagnostics));
        }
        if !self.waiting_for.is_empty() {
            let needed = self
                .waiting_for
                .keys()
                .map(|p| {
                    if p.ends_with('/') {
                        format!("{p}FASTN.ftd")
                    } else {
                        format!("{p}/FASTN.ftd")
                    }
                })
                .collect();
            return fastn_continuation::Result::Stuck(Box::new(self), needed);
        }
        fastn_continuation::Result::Done(Ok((
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
        )))
    }

    fn collect_diagnostics(&mut self, doc: &fastn_section::Document) {
        for diagnostic in doc.diagnostics_cloned() {
            self.diagnostics.push(diagnostic);
        }
    }
}

impl fastn_continuation::Continuation for State {
    // we return a package object if we parsed, even a partial package.
    type Output = PResult<fastn_package::MainPackage>;
    type Needed = Vec<String>; // vec of file names
    type Found = Vec<(Option<String>, NResult)>;

    fn continue_after(
        mut self,
        n: Vec<(Option<String>, NResult)>,
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
                    Some((_name, Ok((doc, file_list)))) => {
                        self.collect_diagnostics(&doc);
                        let package = match parse_package(doc, file_list) {
                            Ok((package, warnings)) => {
                                self.diagnostics.extend(
                                    warnings
                                        .into_iter()
                                        .map(|v| v.map(fastn_section::Diagnostic::Warning)),
                                );
                                collect_dependencies(&mut new_dependencies, &package);
                                package
                            }
                            Err(diagnostics) => {
                                self.diagnostics.extend(diagnostics);
                                return self.finalize();
                            }
                        };
                        if !package.name.is_empty() {
                            self.name = fastn_package::UR::Resolved(Some(package.name.clone()));
                            self.packages.insert(package.name.clone(), package);
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
                todo!()
            }
        }

        self.waiting_for.clear();
        self.waiting_for.extend(new_dependencies);
        self.finalize()
    }
}

fn parse_package(
    doc: fastn_section::Document,
    file_list: Vec<String>,
) -> PResult<fastn_package::Package> {
    let warnings = vec![];
    let mut errors = vec![];

    let mut package = fastn_package::Package {
        name: "".to_string(),
        dependencies: vec![],
        auto_imports: vec![],
        favicon: None,
        urls: vec![],
        file_list,
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
    pub struct TestProvider {
        data: std::collections::HashMap<String, (String, Vec<String>)>,
    }

    impl fastn_continuation::Provider for &TestProvider {
        type Needed = Vec<String>;
        type Found = Vec<(Option<String>, super::NResult)>;

        fn provide(&self, _needed: Vec<String>) -> Self::Found {
            todo!()
        }
    }

    #[test]
    fn basic() {
        let section_provider = TestProvider {
            data: Default::default(),
        };
        let (_package, warnings) = fastn_package::Package::reader()
            .consume(&section_provider)
            .unwrap();

        assert!(warnings.is_empty())
    }
}
