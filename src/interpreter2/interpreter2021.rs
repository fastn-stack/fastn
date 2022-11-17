impl ftd::interpreter2::InterpreterState {}

pub fn interpret_2021(id: &str, source: &str) -> ftd::interpreter2::Result<Interpreter> {
    let mut s = ftd::interpreter2::InterpreterState::new(id.to_string());
    s.document_stack.push(ParsedDocument::parse(id, source)?);
    s.continue_()
}

#[derive(Debug)]
pub enum Interpreter {
    StuckOnImport {
        module: String,
        state: ftd::interpreter2::InterpreterState,
    },
    Done {
        document: ftd::interpreter2::Document,
    },
    StuckOnImport2021 {
        module: String,
        state: ftd::InterpreterState,
    },
    StuckOnProcessor2021 {
        state: ftd::InterpreterState,
        section: ftd::p1::Section,
    },
    StuckOnForeignVariable2021 {
        variable: String,
        state: ftd::InterpreterState,
    },
    CheckID2021 {
        replace_blocks: Vec<ftd::ReplaceLinkBlock<std::collections::HashSet<String>>>,
        state: ftd::InterpreterState,
    },
}
