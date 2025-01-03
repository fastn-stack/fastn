#[derive(Default)]
pub struct State {}

impl fastn_router::Router {
    pub fn reader() -> fastn_continuation::Result<State> {
        fastn_continuation::Result::Stuck(Default::default(), vec!["FASTN.ftd".to_string()])
    }
}

type NResult = std::result::Result<
    Option<(fastn_section::Document, Vec<String>)>,
    fastn_section::Spanned<fastn_section::Error>,
>;

impl fastn_continuation::Continuation for State {
    type Output = fastn_router::Router;
    type Needed = Vec<String>; // vec of file names
    type Found = Vec<(String, NResult)>;

    fn continue_after(self, _n: Vec<(String, NResult)>) -> fastn_continuation::Result<Self> {
        todo!()
    }
}
