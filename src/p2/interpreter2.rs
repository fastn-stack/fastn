pub(crate) struct InterpreterState {
    pub bag: std::collections::BTreeMap<String, ftd::p2::Thing>,
    pub document_stack: Vec<ParsedDocument>,
    pub aliases: std::collections::BTreeMap<String, String>,
    pub parsed_libs: Vec<String>,
}

pub struct ParsedDocument {
    name: String,
    sections: Vec<ftd::p1::Section>,
    start_from: usize,
}

impl ParsedDocument {
    fn parse(id: &str, source: &str) -> ftd::p1::Result<ParsedDocument> {
        Ok(
            ParsedDocument {
                name: id.to_string(),
                sections: ftd::p1::parse(source, id)?,
                start_from: 0,
            })
    }
}

pub(crate) enum Interpreter {
    Parsed(InterpreterState),
    StuckOnImport{module: String, state: InterpreterState},
    StuckOnProcessor{state: InterpreterState, section: ftd::p1::Section},
    Done{state: InterpreterState, instructions: Vec<ftd::Instruction>},
}

impl Interpreter {
    fn new(id: &str, source: &str) -> ftd::p1::Result<Interpreter> {
        Ok(Interpreter::Parsed(InterpreterState {
            document_stack: vec![ParsedDocument::new(id, source)?],
            ..Default::default()
        }))
    }
}