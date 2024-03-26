#[derive(Debug, Default)]
pub struct InterpreterState {
    pub id: String,
    pub bag: ftd::Map<ftd::ftd2021::interpreter::Thing>,
    pub document_stack: Vec<ParsedDocument>,
    pub parsed_libs: ftd::Map<Vec<String>>,
}

impl InterpreterState {
    fn new(id: String) -> InterpreterState {
        InterpreterState {
            id,
            // bag: ftd::p2::interpreter::default_bag(),
            ..Default::default()
        }
    }

    fn continue_(mut self) -> ftd::ftd2021::interpreter::Result<Interpreter> {
        if self.document_stack.is_empty() {
            panic!()
        }

        let l = self.document_stack.len() - 1; // Get the top of the stack

        // Removing commented parts from the parsed document
        self.document_stack[l].ignore_comments();
        // beyond this point commented things will no longer exist in the parsed document

        if self.document_stack[l].processing_imports {
            // Check for all the imports
            // break the loop only when there's no more `import` statement
            loop {
                let top = &mut self.document_stack[l];
                let module = Self::process_imports(top, &self.bag)?;
                if let Some(module) = module {
                    if !self.library_in_the_bag(module.as_str()) {
                        self.add_library_to_bag(module.as_str());
                        return Ok(Interpreter::StuckOnImport {
                            state: self,
                            module,
                        });
                    }
                    if let Some(foreign_var_prefix) = self.parsed_libs.get(module.as_str()) {
                        self.document_stack[l]
                            .foreign_variable_prefix
                            .extend_from_slice(foreign_var_prefix.as_slice());
                    }
                } else {
                    break;
                }
            }
            self.document_stack[l].done_processing_imports();
            //Todo: self.document_stack[l].reorder(&self.bag)?;
        }

        let parsed_document = &mut self.document_stack[l];

        while let Some(_p1) = parsed_document.sections.last_mut() {
            // StuckOnForeignVariable

            let doc = ftd::ftd2021::interpreter::TDoc {
                name: &parsed_document.name,
                aliases: &parsed_document.doc_aliases,
                bag: &self.bag,
            };

            // TODO: first resolve the foreign_variables in the section before proceeding further

            let p1 = parsed_document.sections.pop().unwrap();

            let variable = ftd::ftd2021::interpreter::Variable::from_p1_section(&p1, doc.name)?;
            let variable_name = doc.resolve_name(variable.name.as_str());
            self.bag.insert(
                variable_name,
                ftd::ftd2021::interpreter::Thing::Variable(variable),
            );
        }

        let document = Document {
            name: self.id,
            data: self.bag,
            aliases: self.document_stack[0].get_doc_aliases(),
            instructions: self.document_stack[0].instructions.clone(),
        };

        Ok(Interpreter::Done { document })
    }

    pub fn continue_after_import(
        mut self,
        id: &str,
        source: &str,
    ) -> ftd::ftd2021::interpreter::Result<Interpreter> {
        self.document_stack.push(ParsedDocument::parse(id, source)?);
        self.continue_()
    }

    fn library_in_the_bag(&self, name: &str) -> bool {
        self.parsed_libs.contains_key(name)
    }

    fn add_library_to_bag(&mut self, name: &str) {
        if !self.library_in_the_bag(name) {
            self.parsed_libs.insert(name.to_string(), vec![]);
        }
    }

    fn process_imports(
        top: &mut ParsedDocument,
        bag: &ftd::Map<ftd::ftd2021::interpreter::Thing>,
    ) -> ftd::ftd2021::interpreter::Result<Option<String>> {
        let mut iteration_index = 0;
        while iteration_index < top.sections.len() {
            if top.sections[iteration_index].name != "import" {
                iteration_index += 1;
                continue;
            }
            let (library_name, alias) = ftd::ftd2021::interpreter::utils::parse_import(
                &top.sections[iteration_index]
                    .caption
                    .as_ref()
                    .map_or(Ok(None), |v| v.get_value(top.name.as_str()).map(Some))?
                    .flatten(),
                top.name.as_str(),
                top.sections[iteration_index].line_number,
            )?;

            top.doc_aliases.insert(alias, library_name.clone());

            if bag.contains_key(library_name.as_str()) {
                iteration_index += 1;
                continue;
            }

            top.sections.remove(iteration_index);
            return Ok(Some(library_name));
        }

        Ok(None)
    }
}

pub fn interpret(id: &str, source: &str) -> ftd::ftd2021::interpreter::Result<Interpreter> {
    let mut s = InterpreterState::new(id.to_string());
    s.document_stack.push(ParsedDocument::parse(id, source)?);
    s.continue_()
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum Interpreter {
    StuckOnImport {
        module: String,
        state: InterpreterState,
    },
    Done {
        document: Document,
    },
}

#[derive(Debug, Clone)]
pub struct ParsedDocument {
    name: String,
    sections: Vec<ftd_p1::Section>,
    processing_imports: bool,
    doc_aliases: ftd::Map<String>,
    foreign_variable_prefix: Vec<String>,
    instructions: Vec<ftd::Instruction>,
}

impl ParsedDocument {
    fn parse(id: &str, source: &str) -> ftd::ftd2021::interpreter::Result<ParsedDocument> {
        Ok(ParsedDocument {
            name: id.to_string(),
            sections: ftd_p1::parse(source, id)?,
            processing_imports: true,
            doc_aliases: default_aliases(),
            foreign_variable_prefix: vec![],
            instructions: vec![],
        })
    }

    /// Filters out commented parts from the parsed document.
    ///
    /// # Comments are ignored for
    /// 1.  /-- section: caption
    ///
    /// 2.  /section-header: value
    ///
    /// 3.  /body
    ///
    /// 4.  /--- subsection: caption
    ///
    /// 5.  /sub-section-header: value
    ///
    /// ## Note: To allow ["/content"] inside body, use ["\\/content"].
    ///
    /// Only '/' comments are ignored here.
    /// ';' comments are ignored inside the [`parser`] itself.
    ///
    /// uses [`Section::remove_comments()`] and [`SubSection::remove_comments()`] to remove comments
    /// in sections and sub_sections accordingly.
    ///
    /// [`parser`]: ftd_p1::parser::parse
    /// [`Section::remove_comments()`]: ftd_p1::section::Section::remove_comments
    /// [`SubSection::remove_comments()`]: ftd_p1::sub_section::SubSection::remove_comments
    fn ignore_comments(&mut self) {
        self.sections = self
            .sections
            .iter()
            .filter_map(|s| s.remove_comments())
            .collect::<Vec<ftd_p1::Section>>();
    }

    fn done_processing_imports(&mut self) {
        self.processing_imports = false;
    }

    pub fn get_doc_aliases(&self) -> ftd::Map<String> {
        self.doc_aliases.clone()
    }
}

pub fn default_aliases() -> ftd::Map<String> {
    std::iter::IntoIterator::into_iter([("ftd".to_string(), "ftd".to_string())]).collect()
}

#[derive(Debug, Default, PartialEq)]
pub struct Document {
    pub data: ftd::Map<ftd::ftd2021::interpreter::Thing>,
    pub name: String,
    pub instructions: Vec<ftd::Instruction>,
    pub aliases: ftd::Map<String>,
}
