pub struct State {}

impl fastn_package::Package {
    pub fn reader() -> fastn_continuation::Result<State> {
        // TODO: lets make as much progress as we can
        fastn_continuation::Result::Stuck(Box::new(State {}), Default::default())
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
