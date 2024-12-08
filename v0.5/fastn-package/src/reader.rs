#[derive(Debug)]
pub struct State {
    name: String,
    systems: Vec<fastn_package::UR<fastn_package::System, ()>>,
    dependencies: Vec<fastn_package::UR<fastn_package::Dependency, ()>>,
    pub auto_imports: Vec<fastn_package::AutoImport>,
    apps: Vec<fastn_package::UR<fastn_package::App, ()>>,
}

impl fastn_package::Package {
    pub fn reader() -> fastn_continuation::Result<State> {
        // TODO: lets make as much progress as we can
        fastn_continuation::Result::Stuck(
            Box::new(State {
                name: "".to_string(),
                systems: vec![],
                dependencies: vec![],
                auto_imports: vec![],
                apps: vec![],
            }),
            Default::default(),
        )
    }
}

impl fastn_continuation::Continuation for State {
    type Output = fastn_package::Package;
    type Needed = Vec<String>;
    // File name
    type Found = Vec<(String, Option<String>)>;

    fn continue_after(self, _n: Vec<(String, Option<String>)>) -> fastn_continuation::Result<Self> {
        todo!()
    }
}
