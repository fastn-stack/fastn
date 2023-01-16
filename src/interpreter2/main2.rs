#![allow(dead_code)]

/// The `InterpreterState` struct is a representation of the state of an interpreter. It contains
/// information about the interpreter's current state and its progress through the code being
/// interpreted.
///
/// The `InterpreterState` struct has the following fields:
///
/// - `id`: a String that represents the unique identifier of the interpreter.
///
/// - `bag`: an `ftd::Map` of `ftd::interpreter2::Thing`s that represents the bag of objects that
/// the interpreter has access to.
///
/// - `to_process`: a ToProcess struct that contains information about the elements that still need
/// to be processed by the interpreter.
///
/// - `pending_imports`: an `ftd::VecMap` of tuples containing a String and a usize that
/// represents the pending imports for the interpreter.
///
/// - `parsed_libs`: an `ftd::Map` of `ParsedDocument`s that represents the parsed libraries for the
/// interpreter.
///
/// - `instructions`: a `Vec` of `ftd::interpreter2::Component`s that represents the instructions
/// that the interpreter has processed.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct InterpreterState {
    pub id: String,
    pub bag: ftd::Map<ftd::interpreter2::Thing>,
    pub to_process: ToProcess,
    pub pending_imports: ftd::VecMap<(String, usize)>,
    pub parsed_libs: ftd::Map<ParsedDocument>,
    pub instructions: Vec<ftd::interpreter2::Component>,
}

/**
 * Struct to hold the items that need to be processed by the interpreter.
 *
 * # Fields
 *
 * `stack`: A vector of tuples containing a `String` representing the name of the document and a `Vec` of
 * tuples containing a `usize` representing the scan number and an `ftd::ast::AST` representing
 * the abstract syntax tree of the item to be processed.
 *
 * `contains`: A `HashSet` of tuples containing a `String` representing the name of the document and a `String`
 * representing the name of the item being processed. This field is used to track which items
 * have already been processed to avoid processing them multiple times.
 */
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ToProcess {
    pub stack: Vec<(String, Vec<(usize, ftd::ast::AST)>)>,
    pub contains: std::collections::HashSet<(String, String)>,
}

impl InterpreterState {
    /// The `new` function returns the new `InterpreterState` instance that it has created.
    fn new(id: String) -> InterpreterState {
        InterpreterState {
            id,
            bag: ftd::interpreter2::default::default_bag(),
            ..Default::default()
        }
    }

    /**
    The `tdoc` method is a function that is defined within the `InterpreterState` struct. It
    takes in two parameters:

    - `doc_name`: a reference to a string slice representing the name of the document
    - `line_number`: a usize representing the line number

    The `tdoc` method first retrieves the `parsed_document` from the `parsed_libs` field of the
    `InterpreterState` struct using the `doc_name` parameter. If the document is not found, the
    `Error` variant is returned with a `ParseError`. If the document is found, a new `TDoc`
    struct is constructed. The `TDoc` struct contains a name field, an `aliases` field that
    is a reference to a map of strings representing the aliases of the document, and a `bag`
    field that is either a reference to the `bag` of the `InterpreterState` struct or a mutable
    reference to the `InterpreterState` struct itself. The new `TDoc` struct is then returned as
    the `Ok` variant of the
    `Result`.
    **/
    pub fn tdoc<'a>(
        &'a self,
        doc_name: &'a str,
        line_number: usize,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::TDoc<'a>> {
        let parsed_document =
            self.parsed_libs
                .get(doc_name)
                .ok_or(ftd::interpreter2::Error::ParseError {
                    message: format!("Cannot find this document: `{}`", doc_name),
                    doc_id: doc_name.to_string(),
                    line_number,
                })?;
        Ok(ftd::interpreter2::TDoc::new(
            &parsed_document.name,
            &parsed_document.doc_aliases,
            &self.bag,
        ))
    }

    pub fn get_current_processing_module(&self) -> Option<String> {
        self.to_process.stack.last().map(|v| v.0.clone())
    }

    /// Increments the scan count of the first element in the
    /// AST stack of the `to_process` field of `InterpreterState` instance.
    pub fn increase_scan_count(&mut self) {
        if let Some((_, asts)) = self.to_process.stack.last_mut() {
            if let Some(ast) = asts.first_mut() {
                ast.0 += 1;
            }
        }
    }

    #[tracing::instrument(name = "continue_processing", skip_all)]
    pub fn continue_processing(mut self) -> ftd::interpreter2::Result<Interpreter> {
        while let Some((doc_name, number_of_scan, ast)) = self.get_next_ast() {
            if let Some(interpreter) = self.resolve_pending_imports()? {
                return Ok(interpreter);
            }

            self.increase_scan_count();
            let parsed_document = self.parsed_libs.get(doc_name.as_str()).unwrap();
            let name = parsed_document.name.to_string();
            let aliases = parsed_document.doc_aliases.clone();

            let ast_full_name = ftd::interpreter2::utils::resolve_name(
                ast.name().as_str(),
                &parsed_document.name,
                &parsed_document.doc_aliases,
            );
            let is_in_bag = self.bag.contains_key(&ast_full_name);
            let state = &mut self;

            let mut doc = ftd::interpreter2::TDoc::new_state(&name, &aliases, state);
            if ast.is_record() {
                if !is_in_bag {
                    if number_of_scan.eq(&1) {
                        ftd::interpreter2::Record::scan_ast(ast, &mut doc)?;
                        continue;
                    } else {
                        match ftd::interpreter2::Record::from_ast(ast, &mut doc)? {
                            ftd::interpreter2::StateWithThing::State(s) => {
                                return Ok(s.into_interpreter(self))
                            }
                            ftd::interpreter2::StateWithThing::Thing(record) => {
                                self.bag.insert(
                                    record.name.to_string(),
                                    ftd::interpreter2::Thing::Record(record),
                                );
                            }
                            ftd::interpreter2::StateWithThing::Continue => continue,
                        }
                    }
                }
            } else if ast.is_or_type() {
                if !is_in_bag {
                    if number_of_scan.eq(&1) {
                        ftd::interpreter2::OrType::scan_ast(ast, &mut doc)?;
                        continue;
                    } else {
                        match ftd::interpreter2::OrType::from_ast(ast, &mut doc)? {
                            ftd::interpreter2::StateWithThing::State(s) => {
                                return Ok(s.into_interpreter(self))
                            }
                            ftd::interpreter2::StateWithThing::Thing(or_type) => {
                                self.bag.insert(
                                    or_type.name.to_string(),
                                    ftd::interpreter2::Thing::OrType(or_type),
                                );
                            }
                            ftd::interpreter2::StateWithThing::Continue => continue,
                        }
                    }
                }
            } else if ast.is_function() {
                if !is_in_bag {
                    if number_of_scan.eq(&1) {
                        ftd::interpreter2::Function::scan_ast(ast, &mut doc)?;
                        continue;
                    } else {
                        match ftd::interpreter2::Function::from_ast(ast, &mut doc)? {
                            ftd::interpreter2::StateWithThing::State(s) => {
                                return Ok(s.into_interpreter(self))
                            }
                            ftd::interpreter2::StateWithThing::Thing(function) => {
                                self.bag.insert(
                                    function.name.to_string(),
                                    ftd::interpreter2::Thing::Function(function),
                                );
                            }
                            ftd::interpreter2::StateWithThing::Continue => continue,
                        }
                    }
                }
            } else if ast.is_variable_definition() {
                if !is_in_bag {
                    if number_of_scan.eq(&1) {
                        ftd::interpreter2::Variable::scan_ast(ast, &mut doc)?;
                        continue;
                    } else {
                        match ftd::interpreter2::Variable::from_ast(ast, &mut doc, number_of_scan)?
                        {
                            ftd::interpreter2::StateWithThing::State(s) => {
                                return Ok(s.into_interpreter(self))
                            }
                            ftd::interpreter2::StateWithThing::Thing(variable) => {
                                self.bag.insert(
                                    variable.name.to_string(),
                                    ftd::interpreter2::Thing::Variable(variable),
                                );
                            }
                            ftd::interpreter2::StateWithThing::Continue => continue,
                        }
                    }
                }
            } else if ast.is_variable_invocation() {
                if number_of_scan.eq(&1) {
                    ftd::interpreter2::Variable::scan_update_from_ast(ast, &mut doc)?;
                    continue;
                } else {
                    match ftd::interpreter2::Variable::update_from_ast(ast, &mut doc)? {
                        ftd::interpreter2::StateWithThing::State(s) => {
                            return Ok(s.into_interpreter(self))
                        }
                        ftd::interpreter2::StateWithThing::Thing(variable) => {
                            self.bag.insert(
                                variable.name.to_string(),
                                ftd::interpreter2::Thing::Variable(variable),
                            );
                        }
                        ftd::interpreter2::StateWithThing::Continue => continue,
                    }
                }
            } else if ast.is_component_definition() {
                if !is_in_bag {
                    if number_of_scan.eq(&1) {
                        ftd::interpreter2::ComponentDefinition::scan_ast(ast, &mut doc)?;
                        continue;
                    } else {
                        match ftd::interpreter2::ComponentDefinition::from_ast(ast, &mut doc)? {
                            ftd::interpreter2::StateWithThing::State(s) => {
                                return Ok(s.into_interpreter(self))
                            }
                            ftd::interpreter2::StateWithThing::Thing(component) => {
                                self.bag.insert(
                                    component.name.to_string(),
                                    ftd::interpreter2::Thing::Component(component),
                                );
                            }
                            ftd::interpreter2::StateWithThing::Continue => continue,
                        }
                    }
                }
            } else if ast.is_component() {
                if number_of_scan.eq(&1) {
                    ftd::interpreter2::Component::scan_ast(ast, &mut doc)?;
                    continue;
                } else {
                    match ftd::interpreter2::Component::from_ast(ast, &mut doc)? {
                        ftd::interpreter2::StateWithThing::State(s) => {
                            return Ok(s.into_interpreter(self))
                        }
                        ftd::interpreter2::StateWithThing::Thing(component) => {
                            self.instructions.push(component);
                        }
                        ftd::interpreter2::StateWithThing::Continue => continue,
                    }
                }
            }
            self.remove_last();
        }

        if self.to_process.stack.is_empty() {
            let document = Document {
                data: self.bag,
                aliases: self
                    .parsed_libs
                    .get(self.id.as_str())
                    .unwrap()
                    .doc_aliases
                    .clone(),
                tree: self.instructions,
                name: self.id,
            };

            Ok(Interpreter::Done { document })
        } else {
            self.continue_processing()
        }
    }

    /*pub fn continue_(mut self) -> ftd::interpreter2::Result<Interpreter> {
        if let Some(interpreter) = self.resolve_pending_imports()? {
            return Ok(interpreter);
        }
        if let Some((id, ast_to_process)) = self.to_process.stack.last() {
            let parsed_document = self.parsed_libs.get(id).unwrap();
            let name = parsed_document.name.to_string();
            let aliases = parsed_document.doc_aliases.clone();
            if let Some((number_of_scan, ast)) = ast_to_process.first() {
                let ast = ast.clone();
                let number_of_scan = *number_of_scan;
                if !number_of_scan.gt(&1) {
                    self.increase_scan_count();
                }
                let ast_full_name =
                    ftd::interpreter2::utils::resolve_name(ast.name().as_str(), &name, &aliases);
                let is_in_bag = self.bag.contains_key(&ast_full_name);

                let state = &mut self;

                let mut doc = ftd::interpreter2::TDoc::new_state(&name, &aliases, state);

                if ast.is_record() {
                    if !is_in_bag {
                        if number_of_scan.eq(&0) || number_of_scan.gt(&1) {
                            match ftd::interpreter2::Record::from_ast(ast, &doc)? {
                                ftd::interpreter2::StateWithThing::State(s) => return Ok(s),
                                ftd::interpreter2::StateWithThing::Thing(record) => {
                                    self.bag.insert(
                                        record.name.to_string(),
                                        ftd::interpreter2::Thing::Record(record),
                                    );
                                }
                            }
                        } else {
                            ftd::interpreter2::Record::scan_ast(ast, &mut doc)?;
                            return self.continue_();
                        }
                    }
                } else if ast.is_or_type() {
                    if !is_in_bag {
                        if number_of_scan.eq(&0) || number_of_scan.gt(&1) {
                            match ftd::interpreter2::OrType::from_ast(ast, &doc)? {
                                ftd::interpreter2::StateWithThing::State(s) => return Ok(s),
                                ftd::interpreter2::StateWithThing::Thing(or_type) => {
                                    self.bag.insert(
                                        or_type.name.to_string(),
                                        ftd::interpreter2::Thing::OrType(or_type),
                                    );
                                }
                            }
                        } else {
                            ftd::interpreter2::OrType::scan_ast(ast, &mut doc)?;
                            return self.continue_();
                        }
                    }
                } else if ast.is_function() {
                    if !is_in_bag {
                        if number_of_scan.eq(&0) || number_of_scan.gt(&1) {
                            match ftd::interpreter2::Function::from_ast(ast, &doc)? {
                                ftd::interpreter2::StateWithThing::State(s) => return Ok(s),
                                ftd::interpreter2::StateWithThing::Thing(function) => {
                                    self.bag.insert(
                                        function.name.to_string(),
                                        ftd::interpreter2::Thing::Function(function),
                                    );
                                }
                            }
                        } else {
                            ftd::interpreter2::Function::scan_ast(ast, &mut doc)?;
                            return self.continue_();
                        }
                    }
                } else if ast.is_variable_definition() {
                    if !is_in_bag {
                        if number_of_scan.eq(&0) || number_of_scan.gt(&1) {
                            match ftd::interpreter2::Variable::from_ast(ast, &doc)? {
                                ftd::interpreter2::StateWithThing::State(s) => return Ok(s),
                                ftd::interpreter2::StateWithThing::Thing(variable) => {
                                    self.bag.insert(
                                        variable.name.to_string(),
                                        ftd::interpreter2::Thing::Variable(variable),
                                    );
                                }
                            }
                        } else {
                            ftd::interpreter2::Variable::scan_ast(ast, &mut doc)?;
                            return self.continue_();
                        }
                    }
                } else if ast.is_variable_invocation() {
                    if number_of_scan.eq(&0) || number_of_scan.gt(&1) {
                        match ftd::interpreter2::Variable::update_from_ast(ast, &doc)? {
                            ftd::interpreter2::StateWithThing::State(s) => return Ok(s),
                            ftd::interpreter2::StateWithThing::Thing(variable) => {
                                self.bag.insert(
                                    variable.name.to_string(),
                                    ftd::interpreter2::Thing::Variable(variable),
                                );
                            }
                        }
                    } else {
                        ftd::interpreter2::Variable::scan_update_from_ast(ast, &mut doc)?;
                        return self.continue_();
                    }
                } else if ast.is_component_definition() {
                    if !is_in_bag {
                        if number_of_scan.eq(&0) || number_of_scan.gt(&1) {
                            match ftd::interpreter2::ComponentDefinition::from_ast(ast, &doc)? {
                                ftd::interpreter2::StateWithThing::State(s) => return Ok(s),
                                ftd::interpreter2::StateWithThing::Thing(component) => {
                                    self.bag.insert(
                                        component.name.to_string(),
                                        ftd::interpreter2::Thing::Component(component),
                                    );
                                }
                            }
                        } else {
                            ftd::interpreter2::ComponentDefinition::scan_ast(ast, &mut doc)?;
                            return self.continue_();
                        }
                    }
                } else if ast.is_component() {
                    if number_of_scan.eq(&0) || number_of_scan.gt(&1) {
                        match ftd::interpreter2::Component::from_ast(ast, &doc)? {
                            ftd::interpreter2::StateWithThing::State(s) => return Ok(s),
                            ftd::interpreter2::StateWithThing::Thing(component) => {
                                self.instructions.push(component);
                            }
                        }
                    } else {
                        ftd::interpreter2::Component::scan_ast(ast, &mut doc)?;
                        return self.continue_();
                    }
                }
                self.remove_last();
            }
        }

        if self
            .to_process
            .stack
            .last()
            .map(|v| v.1.is_empty())
            .unwrap_or(false)
        {
            self.to_process.stack.pop();
        }

        if self.to_process.stack.is_empty() {
            let document = Document {
                data: self.bag,
                aliases: self
                    .parsed_libs
                    .get(self.id.as_str())
                    .unwrap()
                    .doc_aliases
                    .clone(),
                tree: self.instructions,
                name: self.id,
            };

            Ok(Interpreter::Done { document })
        } else {
            self.continue_()
        }
    }*/

    /// Returns (doc_name, number_of_scan, last_ast)
    ///
    /// The peek_stack method defined in this code is a method on the InterpreterState struct.
    /// It returns an Option that contains a tuple of a String, an usize, and a reference to an
    /// ftd::ast::AST.
    ///
    /// The method looks at the last element in the stack field of the to_process field of the
    /// InterpreterState instance it is called on. If the last element exists, it looks at the
    /// first element in the asts field of the last element. If the first element exists, the
    /// method returns a tuple containing the doc_name as a String, the `number_of_scan` as an
    /// usize, and the ast as a reference to an ftd::ast::AST. If either the last element of the
    /// stack or the first element of the asts field do not exist, the method returns None.
    pub fn peek_stack(&self) -> Option<(String, usize, &ftd::ast::AST)> {
        if let Some((doc_name, asts)) = self.to_process.stack.last() {
            if let Some((number_of_scan, ast)) = asts.first() {
                return Some((doc_name.to_string(), *number_of_scan, ast));
            }
        }
        None
    }

    /// Returns (doc_name, number_of_scan, last_ast)
    ///
    /// The `get_next_ast` method retrieves the next available AST (abstract syntax tree) from
    /// the `InterpreterState` struct. It does this by first checking if there are any ASTs
    /// remaining in the `to_process` field's stack field. If there are, it returns the first one
    /// in the asts vector. If there are no ASTs remaining in the current stack element, it
    /// checks if the stack element is empty. If it is, it removes it from the stack and
    /// continues the loop. If the stack is empty, it returns None.
    pub fn get_next_ast(&mut self) -> Option<(String, usize, ftd::ast::AST)> {
        loop {
            if let Some((doc_name, asts)) = self.to_process.stack.last() {
                if let Some((number_of_scan, ast)) = asts.first() {
                    return Some((doc_name.to_string(), *number_of_scan, ast.clone()));
                }
            }

            if self
                .to_process
                .stack
                .last()
                .map(|v| v.1.is_empty())
                .unwrap_or(false)
            {
                self.to_process.stack.pop();
            }

            if self.to_process.stack.is_empty() {
                return None;
            }
        }
    }

    pub fn remove_last(&mut self) {
        let mut pop_last = false;
        if let Some((doc_name, asts)) = self.to_process.stack.last_mut() {
            if !asts.is_empty() {
                let (_, ast) = asts.remove(0);
                let document = self.parsed_libs.get(doc_name).unwrap();
                let ast_full_name = ftd::interpreter2::utils::resolve_name(
                    ast.name().as_str(),
                    document.name.as_str(),
                    &document.doc_aliases,
                );
                let (doc_name, thing_name, _remaining) = // Todo: use remaining
                    ftd::interpreter2::utils::get_doc_name_and_thing_name_and_remaining(
                        ast_full_name.as_str(),
                        doc_name,
                        ast.line_number(),
                    );
                self.to_process.contains.remove(&(
                    document.name.to_string(),
                    format!("{}#{}", doc_name, thing_name),
                ));
            }
            if asts.is_empty() {
                pop_last = true;
            }
        }
        if pop_last {
            self.to_process.stack.pop();
        }
    }

    pub fn resolve_pending_imports(&mut self) -> ftd::interpreter2::Result<Option<Interpreter>> {
        if let Some(module) = self.pending_imports.value.keys().next().cloned() {
            if self.parsed_libs.contains_key(module.as_str()) {
                return Ok(Some(self.resolve_import_things(module.as_str())?));
            }
            return Ok(Some(ftd::interpreter2::Interpreter::StuckOnImport {
                module: module.to_string(),
                state: self.clone(),
            }));
        }

        Ok(None)
    }

    pub fn resolve_import_things(
        &mut self,
        module: &str,
    ) -> ftd::interpreter2::Result<Interpreter> {
        use itertools::Itertools;
        let document = self.parsed_libs.get(module).unwrap();
        let mut is_all_thing_resolved = false;
        if let Some(thing_names) = self.pending_imports.value.get_mut(module) {
            while let Some((name, line_number)) = thing_names.last() {
                let (doc_name, thing_name, remaining) = // Todo: use remaining
                    ftd::interpreter2::utils::get_doc_name_and_thing_name_and_remaining(
                        name.as_str(),
                        module,
                        *line_number,
                    );
                let ast_for_thing = document
                    .ast
                    .iter()
                    .filter(|v| {
                        !v.is_component()
                            && (v.name().eq(&thing_name)
                                || v.name().starts_with(format!("{}.", thing_name).as_str()))
                    })
                    .map(|v| (0, v.to_owned()))
                    .collect_vec();

                if !ast_for_thing.is_empty() {
                    self.to_process
                        .contains
                        .insert((doc_name.to_string(), format!("{}#{}", doc_name, thing_name)));
                    self.to_process.stack.push((doc_name, ast_for_thing));
                } else {
                    let found_foreign_variable =
                        document.foreign_variable.iter().any(|v| thing_name.eq(v));
                    if found_foreign_variable && !self.bag.contains_key(name.as_str()) {
                        return Ok(ftd::interpreter2::Interpreter::StuckOnForeignVariable {
                            module: doc_name,
                            state: self.clone(),
                            variable: remaining
                                .map(|v| format!("{}.{}", thing_name, v))
                                .unwrap_or(thing_name),
                        });
                    } else if document.foreign_function.iter().any(|v| thing_name.eq(v)) {
                        thing_names.pop();
                        continue;
                    } else if !found_foreign_variable {
                        return ftd::interpreter2::utils::e2(
                            format!("`{}` not found", name),
                            name,
                            *line_number,
                        );
                    }
                }
                thing_names.pop();
            }
            if thing_names.is_empty() {
                is_all_thing_resolved = true;
            }
        }
        if is_all_thing_resolved {
            self.pending_imports.value.remove(module);
        }

        self.clone().continue_processing()
    }

    #[tracing::instrument(skip_all)]
    pub fn continue_after_import(
        mut self,
        module: &str,
        source: &str,
        foreign_variable: Vec<String>,
        foreign_function: Vec<String>,
        ignore_line_numbers: usize,
    ) -> ftd::interpreter2::Result<Interpreter> {
        let mut document =
            ParsedDocument::parse_with_line_number(module, source, ignore_line_numbers)?;
        document.add_foreign_function(foreign_function);
        document.add_foreign_variable(foreign_variable);
        self.parsed_libs.insert(module.to_string(), document);
        self.continue_processing()
    }

    #[tracing::instrument(skip_all)]
    pub fn continue_after_processor(
        mut self,
        value: ftd::interpreter2::Value,
    ) -> ftd::interpreter2::Result<Interpreter> {
        let (id, ast_to_process) = self.to_process.stack.last().unwrap(); //TODO: remove unwrap & throw error
        let parsed_document = self.parsed_libs.get(id).unwrap();
        let name = parsed_document.name.to_string();
        let aliases = parsed_document.doc_aliases.clone();
        let ast = ast_to_process.first().unwrap().clone().1; // TODO: remove unwrap
        let mut doc = ftd::interpreter2::TDoc::new_state(&name, &aliases, &mut self);
        let variable_definition = ast.get_variable_definition(doc.name)?;
        let name = doc.resolve_name(variable_definition.name.as_str());
        let kind = match ftd::interpreter2::KindData::from_ast_kind(
            variable_definition.kind,
            &Default::default(),
            &mut doc,
            variable_definition.line_number,
        )? {
            StateWithThing::Thing(t) => t,
            StateWithThing::State(s) => return Ok(s.into_interpreter(self)),
            StateWithThing::Continue => return self.continue_processing(),
        };

        let value =
            value.into_property_value(variable_definition.mutable, variable_definition.line_number);

        let variable = ftd::interpreter2::Variable {
            name,
            kind,
            mutable: variable_definition.mutable,
            value,
            conditional_value: vec![],
            line_number: variable_definition.line_number,
            is_static: true,
        }
        .set_static(&doc);
        ftd::interpreter2::utils::validate_variable(&variable, &doc)?;
        self.bag.insert(
            variable.name.to_string(),
            ftd::interpreter2::Thing::Variable(variable),
        );
        self.remove_last();
        self.continue_processing()
    }

    #[tracing::instrument(skip_all)]
    pub fn continue_after_variable(
        mut self,
        module: &str,
        variable: &str,
        value: ftd::interpreter2::Value,
    ) -> ftd::interpreter2::Result<Interpreter> {
        let parsed_document = self.parsed_libs.get(module).unwrap();
        let name = parsed_document.name.to_string();
        let aliases = parsed_document.doc_aliases.clone();
        let doc = ftd::interpreter2::TDoc::new_state(&name, &aliases, &mut self);
        let var_name = doc.resolve_name(variable);
        let variable = ftd::interpreter2::Variable {
            name: var_name,
            kind: value.kind().into_kind_data(),
            mutable: false,
            value: value.into_property_value(false, 0),
            conditional_value: vec![],
            line_number: 0,
            is_static: true,
        }
        .set_static(&doc);
        ftd::interpreter2::utils::validate_variable(&variable, &doc)?;
        self.bag.insert(
            variable.name.to_string(),
            ftd::interpreter2::Thing::Variable(variable),
        );
        self.continue_processing()
    }
}

pub fn interpret<'a>(id: &'a str, source: &'a str) -> ftd::interpreter2::Result<Interpreter> {
    interpret_with_line_number(id, source, 0)
}

#[tracing::instrument(skip_all)]
pub fn interpret_with_line_number<'a>(
    id: &'a str,
    source: &'a str,
    line_number: usize,
) -> ftd::interpreter2::Result<Interpreter> {
    use itertools::Itertools;

    tracing::info!(msg = "ftd: interpreting", doc = id);

    let mut s = InterpreterState::new(id.to_string());
    s.parsed_libs.insert(
        id.to_string(),
        ParsedDocument::parse_with_line_number(id, source, line_number)?,
    );
    s.to_process.stack.push((
        id.to_string(),
        s.parsed_libs
            .get(id)
            .unwrap()
            .ast
            .iter()
            .filter_map(|v| {
                if v.is_component() {
                    Some((0, v.to_owned()))
                } else {
                    None
                }
            })
            .collect_vec(),
    ));

    s.continue_processing()
}

#[derive(Debug, Clone, Default, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ParsedDocument {
    pub name: String,
    pub ast: Vec<ftd::ast::AST>,
    pub processing_imports: bool,
    pub doc_aliases: ftd::Map<String>,
    pub foreign_variable: Vec<String>,
    pub foreign_function: Vec<String>,
    pub instructions: Vec<ftd::interpreter2::Component>,
}

impl ParsedDocument {
    fn parse(id: &str, source: &str) -> ftd::interpreter2::Result<ParsedDocument> {
        ParsedDocument::parse_with_line_number(id, source, 0)
    }

    #[tracing::instrument(name = "parse_with_line_number", skip_all)]
    fn parse_with_line_number(
        id: &str,
        source: &str,
        line_number: usize,
    ) -> ftd::interpreter2::Result<ParsedDocument> {
        let ast = ftd::ast::AST::from_sections(
            ftd::p11::parse_with_line_number(source, id, line_number)?.as_slice(),
            id,
        )?;
        let doc_aliases = {
            let mut doc_aliases = ftd::interpreter2::default::default_aliases();
            for ast in ast.iter().filter(|v| v.is_import()) {
                if let ftd::ast::AST::Import(ftd::ast::Import { module, alias, .. }) = ast {
                    doc_aliases.insert(alias.to_string(), module.to_string());
                }
            }
            doc_aliases
        };
        Ok(ParsedDocument {
            name: id.to_string(),
            ast,
            processing_imports: true,
            doc_aliases,
            foreign_variable: vec![],
            foreign_function: vec![],
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

    pub fn add_foreign_variable(&mut self, foreign_variable: Vec<String>) {
        self.foreign_variable.extend(foreign_variable);
    }

    pub fn add_foreign_function(&mut self, foreign_function: Vec<String>) {
        self.foreign_function.extend(foreign_function);
    }
}

/// Interpreter enum that represents different states that an interpreter can be in during its
/// execution. The states are:
///
/// StuckOnImport: The interpreter is currently waiting onan import to be resolved. The module
/// field indicates the name of the module that is being imported, and the state field holds the
/// current state of the interpreter.
///
/// Done: The interpreter has completed its execution and the resulting Document is stored in the
/// document field.
///
/// StuckOnProcessor: The interpreter is currently stuck on processing an AST and is waiting on a
/// processor to finish its execution. The state, ast, module, and processor fields hold the
/// current state of the interpreter, the AST being processed, the name of the module containing
/// the processor, and the name of the processor, respectively.
///
/// StuckOnForeignVariable: The interpreter is currently stuck on processing a foreign variable.
/// The state, module, and variable fields hold the current state of the interpreter, the name of
/// the module containing the variable, and the name of the variable, respectively.
#[derive(Debug)]
pub enum Interpreter {
    StuckOnImport {
        module: String,
        state: InterpreterState,
    },
    Done {
        document: Document,
    },
    StuckOnProcessor {
        state: InterpreterState,
        ast: ftd::ast::AST,
        module: String,
        processor: String,
    },
    StuckOnForeignVariable {
        state: InterpreterState,
        module: String,
        variable: String,
    },
}

#[derive(Debug)]
pub enum InterpreterWithoutState {
    StuckOnImport {
        module: String,
    },
    Done {
        document: Document,
    },
    StuckOnProcessor {
        ast: ftd::ast::AST,
        module: String,
        processor: String,
    },
    StuckOnForeignVariable {
        module: String,
        variable: String,
    },
}

impl InterpreterWithoutState {
    pub fn into_interpreter(self, state: InterpreterState) -> Interpreter {
        match self {
            InterpreterWithoutState::StuckOnImport { module } => {
                Interpreter::StuckOnImport { module, state }
            }
            InterpreterWithoutState::Done { document } => Interpreter::Done { document },
            InterpreterWithoutState::StuckOnProcessor {
                ast,
                module,
                processor,
            } => Interpreter::StuckOnProcessor {
                ast,
                module,
                state,
                processor,
            },
            InterpreterWithoutState::StuckOnForeignVariable { module, variable } => {
                Interpreter::StuckOnForeignVariable {
                    variable,
                    module,
                    state,
                }
            }
        }
    }
}

#[derive(Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Document {
    pub data: ftd::Map<ftd::interpreter2::Thing>,
    pub name: String,
    pub tree: Vec<ftd::interpreter2::Component>,
    pub aliases: ftd::Map<String>,
}

#[derive(Debug)]
pub enum StateWithThing<T> {
    Thing(T),
    State(InterpreterWithoutState),
    Continue,
}

impl<T> StateWithThing<T> {
    pub fn new_thing(thing: T) -> StateWithThing<T> {
        StateWithThing::Thing(thing)
    }

    pub fn new_state(state: InterpreterWithoutState) -> StateWithThing<T> {
        StateWithThing::State(state)
    }

    pub fn new_continue() -> StateWithThing<T> {
        StateWithThing::Continue
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> StateWithThing<U> {
        let thing = try_state!(self);
        StateWithThing::new_thing(f(thing))
    }

    pub fn into_optional(self) -> Option<T> {
        match self {
            ftd::interpreter2::StateWithThing::State(_)
            | ftd::interpreter2::StateWithThing::Continue => None,
            ftd::interpreter2::StateWithThing::Thing(t) => Some(t),
        }
    }
}
