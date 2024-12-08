#[derive(Default)]
pub struct State {}

impl fastn_router::Router {
    pub fn reader() -> fastn_continuation::Result<State> {
        fastn_continuation::Result::Stuck(Default::default(), vec!["FASTN.ftd".to_string()])
    }
}

impl fastn_continuation::Continuation for State {
    type Output = fastn_router::Router;
    type Needed = Vec<String>;
    type Found = Vec<(String, Option<fastn_section::Document>)>;

    fn continue_after(
        self,
        _n: Vec<(String, Option<fastn_section::Document>)>,
    ) -> fastn_continuation::Result<Self> {
        todo!()
    }
}
