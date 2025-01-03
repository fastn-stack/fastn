#[derive(Default)]
pub struct State {}

impl fastn_router::Router {
    pub fn reader() -> fastn_continuation::Result<State> {
        fastn_continuation::Result::Stuck(Default::default(), vec!["FASTN.ftd".to_string()])
    }
}

type NResult = Result<(fastn_section::Document, Vec<String>), std::sync::Arc<std::io::Error>>;

impl fastn_continuation::Continuation for State {
    type Output = fastn_router::Router;
    type Needed = Vec<String>; // vec of file names
    type Found = Vec<(Option<String>, NResult)>;

    fn continue_after(
        self,
        _n: Vec<(Option<String>, NResult)>,
    ) -> fastn_continuation::Result<Self> {
        todo!()
    }
}
