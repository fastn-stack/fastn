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
    type Output = fastn_package::Package;
    type Needed = Vec<String>; // vec of file names
    type Found = Vec<(
        String, // file name
        Result<Option<(fastn_section::Document, Vec<String>)>, fastn_section::Error>,
    )>;

    fn continue_after(
        self,
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

                match n.get(0) {
                    Some((_name, Ok(Some((_doc, _file_list))))) => {
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
