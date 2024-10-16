#[derive(Default)]
pub struct Engine {
    pub doc_name: String,
    pub edits: Vec<Edit>,
}

impl Engine {
    pub fn new(doc_name: String) -> Self {
        Self {
            doc_name,
            edits: vec![],
        }
    }

    pub fn add_edit(&mut self, from: usize, to: usize, text: String) -> &Edit {
        self.edits.push(Edit { from, to, text });
        self.edits.last().unwrap()
    }
}

pub struct Edit {
    pub from: usize,
    pub to: usize,
    pub text: String,
}

pub fn parse_edit(_old: &mut fastn_p1::ParseOutput, _e: &Edit) {
    todo!()
}
