#[derive(Debug, PartialEq)]
pub struct DummyInstruction {
    pub parent_container: Vec<usize>,
    pub start_index: usize,
    pub instruction: ftd::interpreter2::Component,
}
