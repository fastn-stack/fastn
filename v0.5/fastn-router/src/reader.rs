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
    type Found = Vec<(
        String, // file name
        Result<fastn_section::Document, fastn_section::Error>,
    )>;

    fn continue_after(
        self,
        _n: Vec<(
            String,
            Result<fastn_section::Document, fastn_section::Error>,
        )>,
    ) -> fastn_continuation::Result<Self> {
        todo!()
    }
}
