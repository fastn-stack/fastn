#[derive(Default)]
pub struct Engine {
    pub doc_name: String,
    pub edits: Vec<String>,
}

pub struct Edit<'a> {
    pub from: usize,
    pub to: usize,
    pub text: &'a str,
}

pub fn parse_edit<'a>(_old: &mut fastn_p1::ParseOutput<'a>, _e: Edit<'a>) {
    todo!()
}
