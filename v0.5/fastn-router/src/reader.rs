#[derive(Default)]
pub struct State {}

impl fastn_router::Router {
    pub fn reader() -> fastn_continuation::Result<State> {
        fastn_continuation::Result::Stuck(Default::default(), vec!["FASTN.ftd".to_string()])
    }
}

impl fastn_continuation::Continuation for State {
    type Output = fastn_router::Router;
    type Needed = Vec<String>; // vec of file names
    type Found = Vec<(Option<String>, fastn_utils::section_provider::NResult)>;

    fn continue_after(
        self,
        _n: Vec<(Option<String>, fastn_utils::section_provider::NResult)>,
    ) -> fastn_continuation::Result<Self> {
        todo!()
    }
}
