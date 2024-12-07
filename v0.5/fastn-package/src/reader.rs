pub struct State {}

impl fastn_package::Package {
    pub fn reader() -> fastn_continuation::Result<State> {
        // TODO: lets make as much progress as we can
        fastn_continuation::Result::Stuck(Box::new(State {}), Default::default())
    }
}

impl fastn_continuation::Continuation for State {
    type Output = fastn_package::Package;
    type NeededInput = Vec<String>;
    // File name
    type NeededOutput = Vec<(String, Option<String>)>;

    fn continue_after(self, _n: Self::NeededOutput) -> fastn_continuation::Result<Self> {
        todo!()
    }
}
