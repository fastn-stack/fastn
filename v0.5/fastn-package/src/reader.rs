#[derive(Debug, Default)]
pub struct State {
    name: fastn_package::UR<(), String>,
    systems: Vec<fastn_package::UR<String, fastn_package::System>>,
    dependencies: Vec<fastn_package::UR<String, fastn_package::Dependency>>,
    pub auto_imports: Vec<fastn_package::AutoImport>,
    apps: Vec<fastn_package::UR<String, fastn_package::App>>,
    packages: std::collections::HashMap<String, fastn_package::Package>,
}

impl fastn_package::Package {
    pub fn reader() -> fastn_continuation::Result<State> {
        fastn_continuation::Result::Stuck(Default::default(), vec!["FASTN.ftd".to_string()])
    }
}

impl fastn_continuation::Continuation for State {
    // we return a package object if we parsed, even a partial package.
    type Output = (
        Option<fastn_package::Package>,
        Vec<fastn_section::Diagnostic>,
    );
    type Needed = Vec<String>; // vec of file names
    type Found = Vec<(
        String, // file name
        Result<Option<(fastn_section::Document, Vec<String>)>, fastn_section::Error>,
    )>;

    fn continue_after(
        mut self,
        n: Vec<(
            String,
            Result<Option<(fastn_section::Document, Vec<String>)>, fastn_section::Error>,
        )>,
    ) -> fastn_continuation::Result<Self> {
        match self.name {
            // if the name is not resolved means this is the first attempt.
            fastn_package::UR::UnResolved(()) => {
                assert_eq!(n.len(), 1);
                assert_eq!(n[0].0, "FASTN.ftd");

                match n.into_iter().next() {
                    Some((_name, Ok(Some((doc, file_list))))) => {
                        let _package = match parse_package(doc, file_list) {
                            Ok(package) => package,
                            Err(_e) => {
                                // we found a "valid" fastn_package::Document, but it is not a valid
                                // FASTN.ftd to the extent
                                // that we could not create even a broken Package object out of it
                                return fastn_continuation::Result::Done((
                                    None,
                                    vec![fastn_section::Diagnostic::Error(
                                        fastn_section::Error::InvalidPackageFile,
                                    )],
                                ));
                            }
                        };
                        self.name = fastn_package::UR::Resolved(Some("foo".to_string()));
                        todo!()
                    }
                    Some((_name, Ok(None))) | Some((_name, Err(_))) => {
                        // Ok(None) means we failed to find a file named FASTN.ftd.
                        // Err(e) means we failed to parse the content of FASTN.ftd.
                        todo!()
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
    }
}

fn parse_package(
    _doc: fastn_section::Document,
    _file_list: Vec<String>,
) -> Result<fastn_package::Package, fastn_section::Error> {
    todo!()
}
