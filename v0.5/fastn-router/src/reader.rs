pub struct State {}

impl fastn_router::Router {
    pub fn reader() -> fastn_continuation::Result<State> {
        // TODO: lets make as much progress as we can
        fastn_continuation::Result::Stuck(Box::new(State {}), Default::default())
    }
}

impl fastn_continuation::Continuation for State {
    type Output = fastn_router::Router;
    type Needed = Vec<String>;
    // File name
    type Found = Vec<(String, Option<String>)>;

    fn continue_after(self, _n: Self::Found) -> fastn_continuation::Result<Self> {
        todo!()
    }
}
