#[derive(Default)]
pub(crate) struct InterpreterState {
    pub bag: std::collections::BTreeMap<String, ftd::p2::Thing>,
    pub document_stack: Vec<ParsedDocument>,
    pub aliases: std::collections::BTreeMap<String, String>,
    pub current_doc_aliases: std::collections::BTreeMap<String, String>,
    pub parsed_libs: Vec<String>,
}

impl InterpreterState {
    fn continue_(mut self) -> ftd::p1::Result<Interpreter> {
        if self.document_stack.is_empty() {
            panic!()
        }

        if (&self.document_stack[self.document_stack.len()-1]).processing_imports {
            let (state, module) = self.process_imports()?;
            if let Some(module) =  module {
                return Ok(Interpreter::StuckOnImport {state, module})
            }
            self = state;
            let l = self.document_stack.len() - 1; // Get the top of the stack
            self.document_stack[l].done_processing_imports()
        }

        todo!()
    }

    fn process_imports(mut self) -> ftd::p1::Result<(Self, Option<String>)> {
        let top = &self.document_stack[self.document_stack.len()-1];
        let p1 = &top.sections;

        let mut iteration_index = top.start_from;
        while iteration_index < p1.len() && p1[iteration_index].name == "import" {
            if p1[iteration_index].is_commented {
                iteration_index += 1;
                continue;
            }
            let (library_name, alias) = ftd::p2::utils::parse_import(
                &p1[iteration_index].caption,
                top.name.as_str(),
                p1[iteration_index].line_number,
            )?;

            self.current_doc_aliases.insert(alias, library_name.clone());

            if self.bag.contains_key(library_name.as_str()) {
                iteration_index += 1;
                continue
            }

            let last = self.document_stack.len()-1;
            self.document_stack[last].update_start_from(iteration_index);

            return Ok((self, Some(library_name)));
        }

        Ok((self, None))
    }
}

pub struct ParsedDocument {
    name: String,
    sections: Vec<ftd::p1::Section>,
    start_from: usize,
    processing_imports: bool
}

impl ParsedDocument {
    fn parse(id: &str, source: &str) -> ftd::p1::Result<ParsedDocument> {
        Ok(ParsedDocument {
            name: id.to_string(),
            sections: ftd::p1::parse(source, id)?,
            start_from: 0,
            processing_imports: true
        })
    }

    fn done_processing_imports(&mut self) {
        self.processing_imports = false;
    }

    fn update_start_from(&mut self, start_from: usize) {
        self.start_from = start_from;
    }
}

pub fn create() -> ParsedState {
    ParsedState::default()
}

#[derive(Default)]
pub struct ParsedState(InterpreterState);

enum Interpreter {
    StuckOnImport {
        module: String,
        state: InterpreterState,
    },
    StuckOnProcessor {
        state: InterpreterState,
        section: ftd::p1::Section,
    },
    Done {
        state: InterpreterState,
        instructions: Vec<ftd::Instruction>,
    },
}


impl ParsedState {
    fn interpret(mut self, id: &str, source: &str) -> ftd::p1::Result<Interpreter> {
        self.0.current_doc_aliases = ftd::p2::interpreter::default_aliases();
        self.0.document_stack.push(ParsedDocument::parse(id, source)?);
        self.0.continue_()
    }
}


