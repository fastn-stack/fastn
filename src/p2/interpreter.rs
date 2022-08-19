#[derive(Debug, Default)]
pub struct InterpreterState {
    pub id: String,
    pub bag: ftd::Map<ftd::p2::Thing>,
    pub document_stack: Vec<ParsedDocument>,
    pub parsed_libs: ftd::Map<Vec<String>>,
}

impl InterpreterState {
    fn new(id: String) -> InterpreterState {
        InterpreterState {
            id,
            bag: ftd::p2::interpreter::default_bag(),
            ..Default::default()
        }
    }

    pub fn tdoc<'a>(
        &'a self,
        local_variables: &'a mut ftd::Map<ftd::p2::Thing>,
        referenced_local_variables: &'a mut ftd::Map<String>,
    ) -> ftd::p2::TDoc<'a> {
        let l = self.document_stack.len() - 1;
        ftd::p2::TDoc {
            name: &self.document_stack[l].name,
            aliases: &self.document_stack[l].doc_aliases,
            bag: &self.bag,
            local_variables,
            referenced_local_variables,
        }
    }

    fn library_in_the_bag(&self, name: &str) -> bool {
        self.parsed_libs.contains_key(name)
    }

    fn add_library_to_bag(&mut self, name: &str) {
        if !self.library_in_the_bag(name) {
            self.parsed_libs.insert(name.to_string(), vec![]);
        }
    }

    fn continue_(mut self) -> ftd::p1::Result<Interpreter> {
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
            self.document_stack[l].reorder(&self.bag)?;
        }
        let parsed_document = &mut self.document_stack[l];

        while let Some(p1) = parsed_document.sections.last_mut() {
            // first resolve the foreign_variables in the section before proceeding further
            let doc = ftd::p2::TDoc {
                name: &parsed_document.name,
                aliases: &parsed_document.doc_aliases,
                bag: &self.bag,
                local_variables: &mut Default::default(),
                referenced_local_variables: &mut Default::default(),
            };

            if let Some(variable) = Self::resolve_foreign_variable(
                p1,
                parsed_document.foreign_variable_prefix.as_slice(),
                &doc,
            )? {
                return Ok(Interpreter::StuckOnForeignVariable {
                    variable,
                    state: self,
                });
            }

            // Once the foreign_variables are resolved for the section, then pop and evaluate it.
            // This ensures that a section is evaluated once only.
            let p1 = parsed_document.sections.pop().unwrap();

            // while this is a specific to entire document, we are still creating it in a loop
            // because otherwise the self.interpret() call won't compile.

            let var_data = ftd::variable::VariableData::get_name_kind(
                &p1.name,
                &doc,
                p1.line_number,
                &parsed_document.var_types,
            );

            let mut thing = vec![];

            if p1.name.starts_with("record ") {
                // declare a record
                let d =
                    ftd::p2::Record::from_p1(p1.name.as_str(), &p1.header, &doc, p1.line_number)?;
                let name = doc.resolve_name(p1.line_number, &d.name.to_string())?;
                if self.bag.contains_key(name.as_str()) {
                    return ftd::e2(
                        format!("{} is already declared", d.name),
                        doc.name,
                        p1.line_number,
                    );
                }
                thing.push((name, ftd::p2::Thing::Record(d)));
            } else if p1.name.starts_with("or-type ") {
                // declare a record
                let d = ftd::OrType::from_p1(&p1, &doc)?;
                let name = doc.resolve_name(p1.line_number, &d.name.to_string())?;
                if self.bag.contains_key(name.as_str()) {
                    return ftd::e2(
                        format!("{} is already declared", d.name),
                        doc.name,
                        p1.line_number,
                    );
                }
                thing.push((name, ftd::p2::Thing::OrType(d)));
            } else if p1.name.starts_with("map ") {
                let d = ftd::Variable::map_from_p1(&p1, &doc)?;
                let name = doc.resolve_name(p1.line_number, &d.name.to_string())?;
                if self.bag.contains_key(name.as_str()) {
                    return ftd::e2(
                        format!("{} is already declared", d.name),
                        doc.name,
                        p1.line_number,
                    );
                }
                thing.push((name, ftd::p2::Thing::Variable(d)));
                // } else if_two_words(p1.name.as_str() {
                //   TODO: <record-name> <variable-name>: foo can be used to create a variable/
                //         Not sure if its a good idea tho.
                // }
            } else if p1.name == "container" {
                parsed_document
                    .instructions
                    .push(ftd::Instruction::ChangeContainer {
                        name: doc.resolve_name_with_instruction(
                            p1.line_number,
                            p1.caption(p1.line_number, doc.name)?.as_str(),
                            &parsed_document.instructions,
                        )?,
                    });
            } else if let Ok(ftd::variable::VariableData {
                type_: ftd::variable::Type::Component,
                ..
            }) = var_data
            {
                // declare a function
                let d = ftd::Component::from_p1(&p1, &doc)?;
                let name = doc.resolve_name(p1.line_number, &d.full_name.to_string())?;
                if self.bag.contains_key(name.as_str()) {
                    return ftd::e2(
                        format!("{} is already declared", d.full_name),
                        doc.name,
                        p1.line_number,
                    );
                }
                thing.push((name, ftd::p2::Thing::Component(d)));
                // processed_p1.push(p1.name.to_string());
            } else if let Ok(ref var_data) = var_data {
                let d = if p1
                    .header
                    .str(doc.name, p1.line_number, "$processor$")
                    .is_ok()
                {
                    // processor case: 1
                    return Ok(Interpreter::StuckOnProcessor {
                        state: self,
                        section: p1,
                    });
                } else if var_data.is_none() || var_data.is_optional() {
                    // declare and instantiate a variable
                    ftd::Variable::from_p1(&p1, &doc)?
                } else {
                    // declare and instantiate a list
                    ftd::Variable::list_from_p1(&p1, &doc)?
                };
                let name = doc.resolve_name(p1.line_number, &d.name)?;
                if self.bag.contains_key(name.as_str()) {
                    return ftd::e2(
                        format!("{} is already declared", d.name),
                        doc.name,
                        p1.line_number,
                    );
                }
                thing.push((name, ftd::p2::Thing::Variable(d)));
            } else if let ftd::p2::Thing::Variable(mut v) =
                doc.get_thing(p1.line_number, p1.name.as_str())?
            {
                assert!(
                    !(p1.header
                        .str_optional(doc.name, p1.line_number, "if")?
                        .is_some()
                        && p1
                            .header
                            .str_optional(doc.name, p1.line_number, "$processor$")?
                            .is_some())
                );
                let (doc_name, remaining) = ftd::p2::utils::get_doc_name_and_remaining(
                    doc.resolve_name(p1.line_number, p1.name.as_str())?.as_str(),
                )?;
                if remaining.is_some()
                    && p1
                        .header
                        .str_optional(doc.name, p1.line_number, "if")?
                        .is_some()
                {
                    return ftd::e2(
                        "Currently not supporting `if` for field value update.",
                        doc.name,
                        p1.line_number,
                    );
                }
                if let Some(expr) = p1.header.str_optional(doc.name, p1.line_number, "if")? {
                    let val = v.get_value(&p1, &doc)?;
                    v.conditions.push((
                        ftd::p2::Boolean::from_expression(
                            expr,
                            &doc,
                            &Default::default(),
                            (None, None),
                            p1.line_number,
                        )?,
                        val,
                    ));
                } else if p1
                    .header
                    .str_optional(doc.name, p1.line_number, "$processor$")?
                    .is_some()
                {
                    // processor case: 2
                    return Ok(Interpreter::StuckOnProcessor {
                        state: self,
                        section: p1.to_owned(),
                    });
                    // let start = std::time::Instant::now();
                    // let value = self.lib.process(p1, &doc)?;
                    // *d_processor = d_processor.saturating_add(std::time::Instant::now() - start);
                    // v.value = ftd::PropertyValue::Value { value };
                } else {
                    v.update_from_p1(&p1, &doc)?;
                }
                thing.push((
                    doc.resolve_name(p1.line_number, doc_name.as_str())?,
                    ftd::p2::Thing::Variable(doc.set_value(p1.line_number, p1.name.as_str(), v)?),
                ));
            } else {
                // cloning because https://github.com/rust-lang/rust/issues/59159
                match (doc.get_thing(p1.line_number, p1.name.as_str())?).clone() {
                    ftd::p2::Thing::Variable(_) => {
                        return ftd::e2(
                            format!("variable should have prefix $, found: `{}`", p1.name),
                            doc.name,
                            p1.line_number,
                        );
                    }
                    ftd::p2::Thing::Component(_) => {
                        if p1
                            .header
                            .str_optional(doc.name, p1.line_number, "$processor$")?
                            .is_some()
                        {
                            // processor case: 3
                            return Ok(Interpreter::StuckOnProcessor {
                                state: self,
                                section: p1.to_owned(),
                            });
                        }
                        if let Ok(loop_data) = p1.header.str(doc.name, p1.line_number, "$loop$") {
                            let section_to_subsection = ftd::p1::SubSection {
                                name: p1.name.to_string(),
                                caption: p1.caption.to_owned(),
                                header: p1.header.to_owned(),
                                body: p1.body.to_owned(),
                                is_commented: p1.is_commented,
                                line_number: p1.line_number,
                            };
                            parsed_document.instructions.push(
                                ftd::Instruction::RecursiveChildComponent {
                                    child: ftd::component::recursive_child_component(
                                        loop_data,
                                        &section_to_subsection,
                                        &doc,
                                        &Default::default(),
                                        None,
                                    )?,
                                },
                            );
                        } else {
                            let parent = ftd::ChildComponent::from_p1(
                                p1.line_number,
                                p1.name.as_str(),
                                &p1.header,
                                &p1.caption,
                                &p1.body,
                                &doc,
                                &Default::default(),
                            )?;

                            let mut children = vec![];

                            for sub in p1.sub_sections.0.iter() {
                                if let Ok(loop_data) =
                                    sub.header.str(doc.name, p1.line_number, "$loop$")
                                {
                                    children.push(ftd::component::recursive_child_component(
                                        loop_data,
                                        sub,
                                        &doc,
                                        &parent.arguments,
                                        None,
                                    )?);
                                } else {
                                    let root_name = ftd::p2::utils::get_root_component_name(
                                        &doc,
                                        parent.root.as_str(),
                                        sub.line_number,
                                    )?;
                                    let child = if root_name.eq("ftd#text") {
                                        ftd::p2::utils::get_markup_child(
                                            sub,
                                            &doc,
                                            &parent.arguments,
                                        )?
                                    } else {
                                        ftd::ChildComponent::from_p1(
                                            sub.line_number,
                                            sub.name.as_str(),
                                            &sub.header,
                                            &sub.caption,
                                            &sub.body,
                                            &doc,
                                            &parent.arguments,
                                        )?
                                    };
                                    children.push(child);
                                }
                            }

                            parsed_document
                                .instructions
                                .push(ftd::Instruction::Component { children, parent })
                        }
                    }
                    ftd::p2::Thing::Record(mut r) => {
                        r.add_instance(&p1, &doc)?;
                        thing.push((
                            doc.resolve_name(p1.line_number, &p1.name)?,
                            ftd::p2::Thing::Record(r),
                        ));
                    }
                    ftd::p2::Thing::OrType(_r) => {
                        // do we allow initialization of a record by name? nopes
                        return ftd::e2(
                            format!("'{}' is an or-type", p1.name.as_str()),
                            doc.name,
                            p1.line_number,
                        );
                    }
                    ftd::p2::Thing::OrTypeWithVariant { .. } => {
                        // do we allow initialization of a record by name? nopes
                        return ftd::e2(
                            format!("'{}' is an or-type variant", p1.name.as_str(),),
                            doc.name,
                            p1.line_number,
                        );
                    }
                };
            }
            self.bag.extend(thing);
        }

        if self.document_stack.len() > 1 {
            return self.continue_after_pop();
        }

        let mut rt = ftd::RT::from(
            &self.id,
            self.document_stack[0].get_doc_aliases(),
            self.bag,
            self.document_stack[0].instructions.clone(),
        );

        let main = if cfg!(test) {
            rt.render_()?
        } else {
            rt.render()?
        };

        let d = ftd::p2::document::Document {
            main,
            name: rt.name,
            data: rt.bag.clone(),
            aliases: rt.aliases,
            instructions: rt.instructions,
        };

        Ok(Interpreter::Done { document: d })
    }

    fn resolve_foreign_variable_name(name: &str) -> String {
        name.replace('.', "-")
    }

    fn resolve_foreign_variable(
        section: &mut ftd::p1::Section,
        foreign_variables: &[String],
        doc: &ftd::p2::TDoc,
    ) -> ftd::p1::Result<Option<String>> {
        if let Some(variable) = resolve_all_properties(
            &mut section.caption,
            &mut section.header,
            &mut section.body,
            section.line_number,
            foreign_variables,
            doc,
        )? {
            return Ok(Some(variable));
        }

        for subsection in section.sub_sections.0.iter_mut() {
            if let Some(variable) = resolve_all_properties(
                &mut subsection.caption,
                &mut subsection.header,
                &mut subsection.body,
                subsection.line_number,
                foreign_variables,
                doc,
            )? {
                return Ok(Some(variable));
            }
        }

        return Ok(None);

        fn resolve_all_properties(
            caption: &mut Option<String>,
            header: &mut ftd::p1::Header,
            body: &mut Option<(usize, String)>,
            line_number: usize,
            foreign_variables: &[String],
            doc: &ftd::p2::TDoc,
        ) -> ftd::p1::Result<Option<String>> {
            if let Some(ref mut caption) = caption {
                if let Some(cap) =
                    process_foreign_variables(caption, foreign_variables, doc, line_number)?
                {
                    return Ok(Some(cap));
                }
            }

            for (line_number, _, header) in header.0.iter_mut() {
                if let Some(h) =
                    process_foreign_variables(header, foreign_variables, doc, *line_number)?
                {
                    return Ok(Some(h));
                }
            }

            if let Some((line_number, ref mut body)) = body {
                if let Some(b) =
                    process_foreign_variables(body, foreign_variables, doc, *line_number)?
                {
                    return Ok(Some(b));
                }
            }

            Ok(None)
        }

        fn process_foreign_variables(
            value: &mut String,
            foreign_variables: &[String],
            doc: &ftd::p2::TDoc,
            line_number: usize,
        ) -> ftd::p1::Result<Option<String>> {
            if value.contains('#') {
                return Ok(None);
            }
            if let Some(val) = value.clone().strip_prefix('$') {
                if is_foreign_variable(val, foreign_variables, doc, line_number)? {
                    let val = doc.resolve_name(line_number, val)?;
                    *value = ftd::InterpreterState::resolve_foreign_variable_name(
                        format!("${}", val.as_str()).as_str(),
                    );
                    return Ok(Some(val));
                }
            }
            Ok(None)
        }

        fn is_foreign_variable(
            variable: &str,
            foreign_variables: &[String],
            doc: &ftd::p2::TDoc,
            line_number: usize,
        ) -> ftd::p1::Result<bool> {
            let var_name = doc.resolve_name(line_number, variable)?;

            if foreign_variables.iter().any(|v| var_name.starts_with(v)) {
                return Ok(true);
            }
            Ok(false)
        }
    }

    fn process_imports(
        top: &mut ParsedDocument,
        bag: &ftd::Map<ftd::p2::Thing>,
    ) -> ftd::p1::Result<Option<String>> {
        let mut iteration_index = 0;
        while iteration_index < top.sections.len() {
            if top.sections[iteration_index].name != "import" {
                iteration_index += 1;
                continue;
            }
            let (library_name, alias) = ftd::p2::utils::parse_import(
                &top.sections[iteration_index].caption,
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

    pub fn add_foreign_variable_prefix(&mut self, module: &str, prefix: Vec<String>) {
        if let Some(document) = self.document_stack.last_mut() {
            document
                .foreign_variable_prefix
                .extend_from_slice(prefix.as_slice());
        }
        self.parsed_libs.insert(module.to_string(), prefix);
    }

    pub fn continue_after_import(mut self, id: &str, source: &str) -> ftd::p1::Result<Interpreter> {
        self.document_stack.push(ParsedDocument::parse(id, source)?);
        self.continue_()
        // interpret then
        // handle top / start_from
    }

    pub fn continue_after_variable(
        mut self,
        variable: &str,
        value: ftd::Value,
    ) -> ftd::p1::Result<Interpreter> {
        let l = self.document_stack.len() - 1;
        let doc = ftd::p2::TDoc {
            name: &self.document_stack[l].name,
            aliases: &self.document_stack[l].doc_aliases,
            bag: &self.bag,
            local_variables: &mut Default::default(),
            referenced_local_variables: &mut Default::default(),
        };
        let var_name = ftd::InterpreterState::resolve_foreign_variable_name(
            doc.resolve_name(0, variable)?.as_str(),
        );
        self.bag.insert(
            var_name.clone(),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: var_name,
                value: ftd::PropertyValue::Value { value },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        self.continue_()
    }

    pub fn continue_after_pop(mut self) -> ftd::p1::Result<Interpreter> {
        self.document_stack.pop();
        self.continue_()
        // interpret then
        // handle top / start_from
    }

    pub fn continue_after_processor(
        mut self,
        p1: &ftd::p1::Section,
        value: ftd::Value,
    ) -> ftd::p1::Result<Interpreter> {
        let l = self.document_stack.len() - 1;
        let parsed_document = &mut self.document_stack[l];

        let doc = ftd::p2::TDoc {
            name: &parsed_document.name,
            aliases: &parsed_document.doc_aliases,
            bag: &self.bag,
            local_variables: &mut Default::default(),
            referenced_local_variables: &mut Default::default(),
        };

        let var_data = ftd::variable::VariableData::get_name_kind(
            &p1.name,
            &doc,
            p1.line_number,
            &parsed_document.var_types,
        );

        if let Ok(ftd::variable::VariableData {
            type_: ftd::variable::Type::Variable,
            name,
            ..
        }) = var_data
        {
            let name = doc.resolve_name(p1.line_number, &name)?;
            let variable = ftd::p2::Thing::Variable(ftd::Variable {
                name: name.clone(),
                value: ftd::PropertyValue::Value { value },
                conditions: vec![],
                flags: ftd::variable::VariableFlags::from_p1(&p1.header, doc.name, p1.line_number)?,
            });
            self.bag.insert(name, variable);
            return self.continue_();
        }

        match doc.get_thing(p1.line_number, p1.name.as_str())? {
            ftd::p2::Thing::Variable(mut v) => {
                // for case: 2
                let doc_name = ftd::p2::utils::get_doc_name_and_remaining(
                    doc.resolve_name(p1.line_number, p1.name.as_str())?.as_str(),
                )?
                .0;
                v.value = ftd::PropertyValue::Value { value };
                let key = doc.resolve_name(p1.line_number, doc_name.as_str())?;
                let variable =
                    ftd::p2::Thing::Variable(doc.set_value(p1.line_number, p1.name.as_str(), v)?);
                self.bag.insert(key, variable);
            }
            ftd::p2::Thing::Component(_) => {
                // for case: 3
                let mut p1 = p1.clone();
                Self::p1_from_processor(&mut p1, value);
                parsed_document.sections.push(p1.to_owned());
            }
            _ => todo!(), // throw error
        }
        self.continue_()
        // interpret then
        // handle top / start_from
    }

    pub(crate) fn p1_from_processor(p1: &mut ftd::p1::Section, value: ftd::Value) {
        for (idx, (_, k, _)) in p1.header.0.iter().enumerate() {
            if k.eq("$processor$") {
                p1.header.0.remove(idx);
                break;
            }
        }
        if let ftd::Value::Object { values } = value {
            for (k, v) in values {
                let v = if let ftd::PropertyValue::Value { value } = v {
                    if let Some(v) = value.to_string() {
                        v
                    } else {
                        continue;
                    }
                } else {
                    continue;
                };

                if k.eq("$body$") {
                    p1.body = Some((p1.line_number, v));
                } else if k.eq("$caption$") {
                    p1.caption = Some(v);
                } else {
                    p1.header.0.push((p1.line_number, k, v));
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParsedDocument {
    name: String,
    sections: Vec<ftd::p1::Section>,
    processing_imports: bool,
    doc_aliases: ftd::Map<String>,
    var_types: Vec<String>,
    foreign_variable_prefix: Vec<String>,
    instructions: Vec<ftd::Instruction>,
}

impl ParsedDocument {
    fn parse(id: &str, source: &str) -> ftd::p1::Result<ParsedDocument> {
        Ok(ParsedDocument {
            name: id.to_string(),
            sections: ftd::p1::parse(source, id)?,
            processing_imports: true,
            doc_aliases: ftd::p2::interpreter::default_aliases(),
            var_types: Default::default(),
            foreign_variable_prefix: vec![],
            instructions: vec![],
        })
    }

    fn done_processing_imports(&mut self) {
        self.processing_imports = false;
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
    /// uses [`Section::remove_comments()`] and [`Subsection::remove_comments()`] to remove comments
    /// in sections and subsections accordingly.
    ///
    /// [`parser`]: ftd::p1::parser::parse
    /// [`Section::remove_comments()`]: ftd::p1::section::Section::remove_comments
    /// [`SubSection::remove_comments()`]: ftd::p1::sub_section::SubSection::remove_comments
    fn ignore_comments(&mut self) {
        self.sections = self
            .sections
            .iter()
            .filter(|s| !s.is_commented)
            .map(|s| s.remove_comments())
            .collect::<Vec<ftd::p1::Section>>();
    }

    fn reorder(&mut self, bag: &ftd::Map<ftd::p2::Thing>) -> ftd::p1::Result<()> {
        let (mut new_p1, var_types) = ftd::p2::utils::reorder(
            &self.sections,
            &ftd::p2::TDoc {
                name: &self.name,
                aliases: &self.doc_aliases,
                bag,
                local_variables: &mut Default::default(),
                referenced_local_variables: &mut Default::default(),
            },
        )?;
        new_p1.reverse();
        self.sections = new_p1;
        self.var_types = var_types;
        Ok(())
    }

    pub fn get_doc_aliases(&self) -> ftd::Map<String> {
        self.doc_aliases.clone()
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum Interpreter {
    StuckOnImport {
        module: String,
        state: InterpreterState,
    },
    StuckOnProcessor {
        state: InterpreterState,
        section: ftd::p1::Section,
    },
    StuckOnForeignVariable {
        variable: String,
        state: InterpreterState,
    },
    Done {
        document: ftd::p2::Document,
    },
}

pub fn interpret(id: &str, source: &str) -> ftd::p1::Result<Interpreter> {
    let mut s = InterpreterState::new(id.to_string());
    s.document_stack.push(ParsedDocument::parse(id, source)?);
    s.continue_()
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Thing {
    Component(ftd::Component),
    Variable(ftd::Variable),
    Record(ftd::p2::Record),
    OrType(ftd::OrType),
    OrTypeWithVariant { e: ftd::OrType, variant: String },
    // Library -> Name of library successfully parsed
}

pub fn default_bag() -> ftd::Map<ftd::p2::Thing> {
    let record = |n: &str, r: &str| (n.to_string(), ftd::p2::Kind::record(r));
    let color = |n: &str| record(n, "ftd#color");
    std::iter::IntoIterator::into_iter([
        (
            "ftd#row".to_string(),
            ftd::p2::Thing::Component(ftd::p2::element::row_function()),
        ),
        (
            "ftd#column".to_string(),
            ftd::p2::Thing::Component(ftd::p2::element::column_function()),
        ),
        (
            "ftd#text-block".to_string(),
            ftd::p2::Thing::Component(ftd::p2::element::text_function()),
        ),
        (
            "ftd#code".to_string(),
            ftd::p2::Thing::Component(ftd::p2::element::code_function()),
        ),
        (
            "ftd#image".to_string(),
            ftd::p2::Thing::Component(ftd::p2::element::image_function()),
        ),
        (
            "ftd#iframe".to_string(),
            ftd::p2::Thing::Component(ftd::p2::element::iframe_function()),
        ),
        (
            "ftd#integer".to_string(),
            ftd::p2::Thing::Component(ftd::p2::element::integer_function()),
        ),
        (
            "ftd#decimal".to_string(),
            ftd::p2::Thing::Component(ftd::p2::element::decimal_function()),
        ),
        (
            "ftd#boolean".to_string(),
            ftd::p2::Thing::Component(ftd::p2::element::boolean_function()),
        ),
        (
            "ftd#scene".to_string(),
            ftd::p2::Thing::Component(ftd::p2::element::scene_function()),
        ),
        (
            "ftd#grid".to_string(),
            ftd::p2::Thing::Component(ftd::p2::element::grid_function()),
        ),
        (
            "ftd#text".to_string(),
            ftd::p2::Thing::Component(ftd::p2::element::markup_function()),
        ),
        (
            "ftd#input".to_string(),
            ftd::p2::Thing::Component(ftd::p2::element::input_function()),
        ),
        (
            "ftd#null".to_string(),
            ftd::p2::Thing::Component(ftd::p2::element::null()),
        ),
        (
            "ftd#dark-mode".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: "ftd#dark-mode".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Boolean { value: false },
                },
                conditions: vec![],
                flags: ftd::VariableFlags {
                    always_include: Some(true),
                },
            }),
        ),
        (
            "ftd#system-dark-mode".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: "ftd#system-dark-mode".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Boolean { value: false },
                },
                conditions: vec![],
                flags: ftd::VariableFlags {
                    always_include: Some(true),
                },
            }),
        ),
        (
            "ftd#follow-system-dark-mode".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: "ftd#follow-system-dark-mode".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Boolean { value: true },
                },
                conditions: vec![],
                flags: ftd::VariableFlags {
                    always_include: Some(true),
                },
            }),
        ),
        (
            "ftd#device".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: "ftd#device".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "desktop".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: ftd::VariableFlags {
                    always_include: Some(true),
                },
            }),
        ),
        (
            "ftd#mobile-breakpoint".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: "ftd#mobile-breakpoint".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 768 },
                },
                conditions: vec![],
                flags: ftd::VariableFlags {
                    always_include: Some(true),
                },
            }),
        ),
        (
            "ftd#desktop-breakpoint".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: "ftd#desktop-breakpoint".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 1440 },
                },
                conditions: vec![],
                flags: ftd::VariableFlags {
                    always_include: Some(true),
                },
            }),
        ),
        (
            "ftd#image-src".to_string(),
            ftd::p2::Thing::Record(ftd::p2::Record {
                name: "ftd#image-src".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    ("light".to_string(), ftd::p2::Kind::caption()),
                    ("dark".to_string(), ftd::p2::Kind::string()),
                ])
                .collect(),
                instances: Default::default(),
                order: vec!["light".to_string(), "dark".to_string()],
            }),
        ),
        (
            "ftd#color".to_string(),
            ftd::p2::Thing::Record(ftd::p2::Record {
                name: "ftd#color".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    ("light".to_string(), ftd::p2::Kind::caption()),
                    ("dark".to_string(), ftd::p2::Kind::string()),
                ])
                .collect(),
                instances: Default::default(),
                order: vec!["light".to_string(), "dark".to_string()],
            }),
        ),
        (
            "ftd#font-size".to_string(),
            ftd::p2::Thing::Record(ftd::p2::Record {
                name: "ftd#font-size".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    ("line-height".to_string(), ftd::p2::Kind::integer()),
                    ("size".to_string(), ftd::p2::Kind::integer()),
                    (
                        "letter-spacing".to_string(),
                        ftd::p2::Kind::integer().set_default(Some("0".to_string())),
                    ),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![
                    "line-height".to_string(),
                    "size".to_string(),
                    "letter-spacing".to_string(),
                ],
            }),
        ),
        (
            "ftd#type".to_string(),
            ftd::p2::Thing::Record(ftd::p2::Record {
                name: "ftd#type".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    ("font".to_string(), ftd::p2::Kind::caption()),
                    (
                        "desktop".to_string(),
                        ftd::p2::Kind::record("ftd#font-size"),
                    ),
                    ("mobile".to_string(), ftd::p2::Kind::record("ftd#font-size")),
                    ("xl".to_string(), ftd::p2::Kind::record("ftd#font-size")),
                    (
                        "weight".to_string(),
                        ftd::p2::Kind::integer().set_default(Some("400".to_string())),
                    ),
                    ("style".to_string(), ftd::p2::Kind::string().into_optional()),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![
                    "font".to_string(),
                    "desktop".to_string(),
                    "mobile".to_string(),
                    "xl".to_string(),
                    "weight".to_string(),
                    "style".to_string(),
                ],
            }),
        ),
        (
            "ftd#btb".to_string(),
            ftd::p2::Thing::Record(ftd::p2::Record {
                name: "ftd#btb".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    color("base"),
                    color("text"),
                    color("border"),
                ])
                .collect(),
                instances: Default::default(),
                order: vec!["base".to_string(), "text".to_string(), "border".to_string()],
            }),
        ),
        (
            "ftd#pst".to_string(),
            ftd::p2::Thing::Record(ftd::p2::Record {
                name: "ftd#pst".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    color("primary"),
                    color("secondary"),
                    color("tertiary"),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![
                    "primary".to_string(),
                    "secondary".to_string(),
                    "tertiary".to_string(),
                ],
            }),
        ),
        (
            "ftd#background-colors".to_string(),
            ftd::p2::Thing::Record(ftd::p2::Record {
                name: "ftd#background-colors".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    color("base"),
                    color("step-1"),
                    color("step-2"),
                    color("overlay"),
                    color("code"),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![
                    "base".to_string(),
                    "step-1".to_string(),
                    "step-2".to_string(),
                    "overlay".to_string(),
                    "code".to_string(),
                ],
            }),
        ),
        (
            "ftd#custom-colors".to_string(),
            ftd::p2::Thing::Record(ftd::p2::Record {
                name: "ftd#custom-colors".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    color("one"),
                    color("two"),
                    color("three"),
                    color("four"),
                    color("five"),
                    color("six"),
                    color("seven"),
                    color("eight"),
                    color("nine"),
                    color("ten"),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![
                    "one".to_string(),
                    "two".to_string(),
                    "three".to_string(),
                    "four".to_string(),
                    "five".to_string(),
                    "six".to_string(),
                    "seven".to_string(),
                    "eight".to_string(),
                    "nine".to_string(),
                    "ten".to_string(),
                ],
            }),
        ),
        (
            "ftd#cta-colors".to_string(),
            ftd::p2::Thing::Record(ftd::p2::Record {
                name: "ftd#cta-colors".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    color("base"),
                    color("hover"),
                    color("pressed"),
                    color("disabled"),
                    color("focused"),
                    color("border"),
                    color("text"),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![
                    "base".to_string(),
                    "hover".to_string(),
                    "pressed".to_string(),
                    "disabled".to_string(),
                    "focused".to_string(),
                    "border".to_string(),
                    "text".to_string(),
                ],
            }),
        ),
        (
            "ftd#color-scheme".to_string(),
            ftd::p2::Thing::Record(ftd::p2::Record {
                name: "ftd#color-scheme".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    record("background", "ftd#background-colors"),
                    color("border"),
                    color("border-strong"),
                    color("text"),
                    color("text-strong"),
                    color("shadow"),
                    color("scrim"),
                    record("cta-primary", "ftd#cta-colors"),
                    record("cta-secondary", "ftd#cta-colors"),
                    record("cta-tertiary", "ftd#cta-colors"),
                    record("cta-danger", "ftd#cta-colors"),
                    record("accent", "ftd#pst"),
                    record("error", "ftd#btb"),
                    record("success", "ftd#btb"),
                    record("info", "ftd#btb"),
                    record("warning", "ftd#btb"),
                    record("custom", "ftd#custom-colors"),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![
                    "background".to_string(),
                    "border".to_string(),
                    "border-strong".to_string(),
                    "text".to_string(),
                    "text-strong".to_string(),
                    "shadow".to_string(),
                    "scrim".to_string(),
                    "cta-primary".to_string(),
                    "cta-secondary".to_string(),
                    "cta-tertiary".to_string(),
                    "cta-danger".to_string(),
                    "accent".to_string(),
                    "error".to_string(),
                    "success".to_string(),
                    "info".to_string(),
                    "warning".to_string(),
                    "custom".to_string(),
                ],
            }),
        ),
    ])
    .collect()
}

pub fn default_aliases() -> ftd::Map<String> {
    std::iter::IntoIterator::into_iter([("ftd".to_string(), "ftd".to_string())]).collect()
}

pub fn default_column() -> ftd::Column {
    ftd::Column {
        common: ftd::Common {
            width: Some(ftd::Length::Fill),
            height: Some(ftd::Length::Fill),
            position: Some(ftd::Position::Center),
            ..Default::default()
        },
        spacing: None,
        ..Default::default()
    }
}

// #[cfg(test)]
// pub fn elapsed(e: std::time::Duration) -> String {
//     // NOTE: there is a copy of this function in ftd also
//     let nanos = e.subsec_nanos();
//     let fraction = match nanos {
//         t if nanos < 1000 => format!("{}ns", t),
//         t if nanos < 1_000_000 => format!("{:.*}µs", 3, f64::from(t) / 1000.0),
//         t => format!("{:.*}ms", 3, f64::from(t) / 1_000_000.0),
//     };
//     let secs = e.as_secs();
//     match secs {
//         _ if secs == 0 => fraction,
//         t if secs < 5 => format!("{}.{:06}s", t, nanos / 1000),
//         t if secs < 60 => format!("{}.{:03}s", t, nanos / 1_000_000),
//         t if secs < 3600 => format!("{}m {}s", t / 60, t % 60),
//         t if secs < 86400 => format!("{}h {}m", t / 3600, (t % 3600) / 60),
//         t => format!("{}s", t),
//     }
// }
