#[derive(Debug, Default)]
pub struct State {
    name: String,
    systems: Vec<fastn_package::UR<String, fastn_package::System>>,
    dependencies: Vec<fastn_package::UR<String, fastn_package::Dependency>>,
    pub auto_imports: Vec<fastn_package::AutoImport>,
    apps: Vec<fastn_package::UR<String, fastn_package::App>>,
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
        dbg!(n);
        todo!()
    }
}
