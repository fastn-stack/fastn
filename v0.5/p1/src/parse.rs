#[derive(Default)]
pub struct Engine {
    pub doc_name: String,
    pub edits: Vec<Edit>,
}

pub struct Edit {
    pub from: usize,
    pub to: usize,
    pub text: String,
}

pub fn parse_edit<'a>(_old: &mut fastn_p1::ParseOutput<'a>, _e: &'a Edit) {
    todo!()
}
