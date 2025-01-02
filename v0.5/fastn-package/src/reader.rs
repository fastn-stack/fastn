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
            fastn_package::UR::UnResolved(()) => {
                assert_eq!(n.len(), 1);

                match n.get(0) {
                    Some((name, Ok(Some((_doc, _file_list))))) => {
                        assert_eq!(name, "FASTN.ftd");
                        todo!()
                    }
                    _ => todo!(),
                }
            }
            _ => todo!(),
        }
    }
}
