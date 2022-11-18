/*#![allow(dead_code)]

#[derive(Debug, Default)]
pub struct InterpreterState {
    pub id: String,
    pub bag: ftd::Map<ftd::interpreter2::Thing>,
    pub document_stack: Vec<ParsedDocument>,
    pub parsed_libs: ftd::Map<Vec<String>>,
}

impl InterpreterState {
    fn new(id: String) -> InterpreterState {
        InterpreterState {
            id,
            bag: ftd::interpreter2::default::default_bag(),
            ..Default::default()
        }
    }

    fn continue_(mut self) -> ftd::interpreter2::Result<Interpreter> {
        if self.document_stack.is_empty() {
            panic!()
        }

        let l = self.document_stack.len() - 1; // Get the top of the stack

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
            self.document_stack[l].reorder(&self.bag)?;
        }

        let parsed_document = &mut self.document_stack[l];

        while let Some(_p1) = parsed_document.ast.last_mut() {
            // StuckOnForeignVariable

            let doc = ftd::interpreter2::TDoc {
                name: &parsed_document.name,
                aliases: &parsed_document.doc_aliases,
                bag: &self.bag,
            };

            // TODO: first resolve the foreign_variables in the section before proceeding further

            let ast = parsed_document.ast.pop().unwrap();

            if ast.is_record() {
                let record = ftd::interpreter2::Record::from_ast(ast, &doc)?;
                self.bag.insert(
                    record.name.to_string(),
                    ftd::interpreter2::Thing::Record(record),
                );
            } else if ast.is_function() {
                let function = ftd::interpreter2::Function::from_ast(ast, &doc)?;
                self.bag.insert(
                    function.name.to_string(),
                    ftd::interpreter2::Thing::Function(function),
                );
            } else if ast.is_variable_definition() {
                let variable = ftd::interpreter2::Variable::from_ast(ast, &doc)?;
                self.bag.insert(
                    variable.name.to_string(),
                    ftd::interpreter2::Thing::Variable(variable),
                );
            } else if ast.is_variable_invocation() {
                let variable = ftd::interpreter2::Variable::update_from_ast(ast, &doc)?;
                self.bag.insert(
                    variable.name.to_string(),
                    ftd::interpreter2::Thing::Variable(variable),
                );
            } else if ast.is_component_definition() {
                let component = ftd::interpreter2::ComponentDefinition::from_ast(ast, &doc)?;
                self.bag.insert(
                    component.name.to_string(),
                    ftd::interpreter2::Thing::Component(component),
                );
            } else if ast.is_component() {
                let component = ftd::interpreter2::Component::from_ast(ast, &doc)?;
                parsed_document.instructions.push(component);
            }
        }

        let document = Document {
            name: self.id,
            data: self.bag,
            aliases: self.document_stack[0].get_doc_aliases(),
            tree: self.document_stack[0].instructions.clone(),
        };

        Ok(Interpreter::Done { document })
    }

    pub fn continue_after_import(
        mut self,
        id: &str,
        source: &str,
    ) -> ftd::interpreter2::Result<Interpreter> {
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
        bag: &ftd::Map<ftd::interpreter2::Thing>,
    ) -> ftd::interpreter2::Result<Option<String>> {
        let mut iteration_index = 0;
        while iteration_index < top.ast.len() {
            let (library_name, alias) =
                if let ftd::ast::AST::Import(ftd::ast::Import { module, alias, .. }) =
                    &top.ast[iteration_index]
                {
                    (module.to_owned(), alias.to_owned())
                } else {
                    iteration_index += 1;
                    continue;
                };

            top.doc_aliases
                .insert(alias.to_string(), library_name.clone());

            if bag.contains_key(library_name.as_str()) {
                iteration_index += 1;
                continue;
            }

            top.ast.remove(iteration_index);
            return Ok(Some(library_name));
        }

        Ok(None)
    }
}

pub fn interpret(id: &str, source: &str) -> ftd::interpreter2::Result<Interpreter> {
    let mut s = InterpreterState::new(id.to_string());
    s.document_stack.push(ParsedDocument::parse(id, source)?);
    s.continue_()
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ParsedDocument {
    name: String,
    ast: Vec<ftd::ast::AST>,
    processing_imports: bool,
    doc_aliases: ftd::Map<String>,
    foreign_variable_prefix: Vec<String>,
    instructions: Vec<ftd::interpreter2::Component>,
}

impl ParsedDocument {
    fn parse(id: &str, source: &str) -> ftd::interpreter2::Result<ParsedDocument> {
        Ok(ParsedDocument {
            name: id.to_string(),
            ast: ftd::ast::AST::from_sections(ftd::p11::parse(source, id)?.as_slice(), id)?,
            processing_imports: true,
            doc_aliases: ftd::interpreter2::default::default_aliases(),
            foreign_variable_prefix: vec![],
            instructions: vec![],
        })
    }

    fn done_processing_imports(&mut self) {
        self.processing_imports = false;
    }

    fn reorder(
        &mut self,
        _bag: &ftd::Map<ftd::interpreter2::Thing>,
    ) -> ftd::interpreter2::Result<()> {
        // TODO: reorder
        self.ast.reverse();
        Ok(())
    }

    pub fn get_doc_aliases(&self) -> ftd::Map<String> {
        self.doc_aliases.clone()
    }
}

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

#[derive(Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Document {
    pub data: ftd::Map<ftd::interpreter2::Thing>,
    pub name: String,
    pub tree: Vec<ftd::interpreter2::Component>,
    pub aliases: ftd::Map<String>,
}
*/
