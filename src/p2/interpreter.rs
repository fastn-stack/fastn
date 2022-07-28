#[derive(Debug, Default)]
pub struct InterpreterState {
    pub id: String,
    pub bag: std::collections::BTreeMap<String, ftd::p2::Thing>,
    pub document_stack: Vec<ParsedDocument>,
    pub parsed_libs: std::collections::BTreeMap<String, Vec<String>>,
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
        local_variables: &'a mut std::collections::BTreeMap<String, ftd::p2::Thing>,
        referenced_local_variables: &'a mut std::collections::BTreeMap<String, String>,
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
                let name = doc.resolve_name(p1.line_number, &d.name.to_string())?;
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
                            doc.resolve_name(p1.line_number, &p1.name.to_string())?,
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

        let mut d = ftd::p2::document::Document {
            main,
            name: rt.name,
            data: rt.bag.clone(),
            aliases: rt.aliases,
            instructions: rt.instructions,
        };

        d.data.extend(rt.bag);

        // d.main = rt.render()?;
        // d.data.extend(rt.bag);

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
        bag: &std::collections::BTreeMap<String, ftd::p2::Thing>,
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
    doc_aliases: std::collections::BTreeMap<String, String>,
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

    fn reorder(
        &mut self,
        bag: &std::collections::BTreeMap<String, ftd::p2::Thing>,
    ) -> ftd::p1::Result<()> {
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

    pub fn get_doc_aliases(&self) -> std::collections::BTreeMap<String, String> {
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

pub fn default_bag() -> std::collections::BTreeMap<String, ftd::p2::Thing> {
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
                ])
                .collect(),
                instances: Default::default(),
                order: vec![
                    "base".to_string(),
                    "step-1".to_string(),
                    "step-2".to_string(),
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

pub fn default_aliases() -> std::collections::BTreeMap<String, String> {
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

#[cfg(test)]
mod test {
    use ftd::test::*;
    use ftd::{markdown_line, Instruction};

    /// inserts mapping of root_id -> integer variable (thing) in the bag
    ///
    /// bag\[root_id\] = integer variable
    ///
    /// root_id = \[doc_id\]#\[var_name\]@\[level\]
    fn insert_integer_by_root(
        root: &str,
        val: i64,
        bag: &mut std::collections::BTreeMap<String, ftd::p2::Thing>,
    ) {
        // root => [doc_id]#[var_name]@[level]
        // root_parts = [ doc_id , var_name, level ]
        let root_parts: Vec<&str> = root.trim().split(|ch| ch == '#' || ch == '@').collect();
        let var_name = root_parts[1];

        let integer_thing = ftd::p2::Thing::Variable(ftd::Variable {
            name: format!("{}", var_name),
            value: ftd::PropertyValue::Value {
                value: ftd::Value::Integer { value: val },
            },
            conditions: vec![],
            flags: Default::default(),
        });

        bag.insert(root.to_string(), integer_thing);
    }

    #[test]
    fn basic_1() {
        let mut bag = super::default_bag();
        bag.insert(
            "foo/bar#foo".to_string(),
            ftd::p2::Thing::Component(ftd::Component {
                root: "ftd#text".to_string(),
                full_name: s("foo/bar#foo"),
                properties: std::iter::IntoIterator::into_iter([(
                    s("text"),
                    ftd::component::Property {
                        default: Some(ftd::PropertyValue::Value {
                            value: ftd::Value::String {
                                text: s("hello"),
                                source: ftd::TextSource::Header,
                            },
                        }),
                        conditions: vec![],
                        ..Default::default()
                    },
                )])
                .collect(),
                ..Default::default()
            }),
        );
        bag.insert(
            "foo/bar#x".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "x".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 10 },
                },
                conditions: vec![],
            }),
        );

        p!(
            "
            -- ftd.text foo:
            text: hello

            -- integer x: 10
            ",
            (bag, super::default_column()),
        );
    }

    #[test]
    fn conditional_attribute() {
        let mut bag = super::default_bag();
        bag.insert(
            "foo/bar#foo".to_string(),
            ftd::p2::Thing::Component(ftd::Component {
                full_name: s("foo/bar#foo"),
                root: "ftd#text".to_string(),
                arguments: std::iter::IntoIterator::into_iter([(
                    s("name"),
                    ftd::p2::Kind::caption(),
                )])
                .collect(),
                properties: std::iter::IntoIterator::into_iter([
                    (
                        s("color"),
                        ftd::component::Property {
                            default: Some(ftd::PropertyValue::Reference {
                                name: s("foo/bar#white"),
                                kind: ftd::p2::Kind::Optional {
                                    kind: Box::new(ftd::p2::Kind::Record {
                                        name: s("ftd#color"),
                                        default: None,
                                        is_reference: false,
                                    }),
                                    is_reference: false,
                                },
                            }),
                            conditions: vec![
                                (
                                    ftd::p2::Boolean::Equal {
                                        left: ftd::PropertyValue::Reference {
                                            name: "foo/bar#present".to_string(),
                                            kind: ftd::p2::Kind::boolean(),
                                        },
                                        right: ftd::PropertyValue::Value {
                                            value: ftd::Value::Boolean { value: true },
                                        },
                                    },
                                    ftd::PropertyValue::Reference {
                                        name: s("foo/bar#green"),
                                        kind: ftd::p2::Kind::Optional {
                                            kind: Box::new(ftd::p2::Kind::Record {
                                                name: s("ftd#color"),
                                                default: None,
                                                is_reference: false,
                                            }),
                                            is_reference: false,
                                        },
                                    },
                                ),
                                (
                                    ftd::p2::Boolean::Equal {
                                        left: ftd::PropertyValue::Reference {
                                            name: "foo/bar#present".to_string(),
                                            kind: ftd::p2::Kind::boolean(),
                                        },
                                        right: ftd::PropertyValue::Value {
                                            value: ftd::Value::Boolean { value: false },
                                        },
                                    },
                                    ftd::PropertyValue::Reference {
                                        name: s("foo/bar#red"),
                                        kind: ftd::p2::Kind::Optional {
                                            kind: Box::new(ftd::p2::Kind::Record {
                                                name: s("ftd#color"),
                                                default: None,
                                                is_reference: false,
                                            }),
                                            is_reference: false,
                                        },
                                    },
                                ),
                            ],
                            ..Default::default()
                        },
                    ),
                    (
                        s("text"),
                        ftd::component::Property {
                            default: Some(ftd::PropertyValue::Variable {
                                name: "name".to_string(),
                                kind: ftd::p2::Kind::caption_or_body(),
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                ])
                .collect(),
                ..Default::default()
            }),
        );

        bag.insert(
            s("foo/bar#green"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("green"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("ftd#color"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("dark"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("green"),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("light"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("green"),
                                        source: ftd::TextSource::Caption,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        bag.insert(
            s("foo/bar#red"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("red"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("ftd#color"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("dark"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("red"),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("light"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("red"),
                                        source: ftd::TextSource::Caption,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        bag.insert(
            s("foo/bar#white"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("white"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("ftd#color"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("dark"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("white"),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("light"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("white"),
                                        source: ftd::TextSource::Caption,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        bag.insert(
            "foo/bar#name@0".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "name".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("hello"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#present".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "present".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Boolean { value: false },
                },
                conditions: vec![],
            }),
        );

        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("hello"),
                line: true,
                common: ftd::Common {
                    color: Some(ftd::Color {
                        light: ftd::ColorValue {
                            r: 255,
                            g: 0,
                            b: 0,
                            alpha: 1.0,
                        },
                        dark: ftd::ColorValue {
                            r: 255,
                            g: 0,
                            b: 0,
                            alpha: 1.0,
                        },
                        reference: Some(s("foo/bar#red")),
                    }),
                    conditional_attribute: std::iter::IntoIterator::into_iter([(
                        s("color"),
                        ftd::ConditionalAttribute {
                            attribute_type: ftd::AttributeType::Style,
                            conditions_with_value: vec![
                                (
                                    ftd::Condition {
                                        variable: s("foo/bar#present"),
                                        value: serde_json::Value::Bool(true),
                                    },
                                    ftd::ConditionalValue {
                                        value: serde_json::from_str("{\"$kind$\":\"light\",\"dark\":\"rgba(0,128,0,1)\",\"light\":\"rgba(0,128,0,1)\"}").unwrap(),
                                        important: false,
                                        reference: Some(s("foo/bar#green")),
                                    },
                                ),
                                (
                                    ftd::Condition {
                                        variable: s("foo/bar#present"),
                                        value: serde_json::Value::Bool(false),
                                    },
                                    ftd::ConditionalValue {
                                        value: serde_json::from_str("{\"$kind$\":\"light\",\"dark\":\"rgba(255,0,0,1)\",\"light\":\"rgba(255,0,0,1)\"}").unwrap(),
                                        important: false,
                                        reference: Some(s("foo/bar#red")),
                                    },
                                ),
                            ],
                            default: Some(ftd::ConditionalValue {
                                value: serde_json::from_str("{\"$kind$\":\"light\",\"dark\":\"rgba(255,255,255,1)\",\"light\":\"rgba(255,255,255,1)\"}").unwrap(),
                                important: false,
                                reference: Some(s("foo/bar#white")),
                            }),
                        },
                    )])
                    .collect(),
                    reference: Some(s("foo/bar#name@0")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        p!(
            "
            -- boolean present: false

            -- ftd.color red: red
            dark: red

            -- ftd.color green: green
            dark: green

            -- ftd.color white: white
            dark: white

            -- ftd.text foo:
            caption name:
            color: $white
            color if $present: $green
            color if not $present: $red
            text: $name

            -- foo: hello
            ",
            (bag, main),
        );
    }

    #[test]
    fn creating_a_tree() {
        let mut bag = super::default_bag();

        bag.insert(
            "foo/bar#ft_toc".to_string(),
            ftd::p2::Thing::Component(ftd::Component {
                root: "ftd#column".to_string(),
                full_name: "foo/bar#ft_toc".to_string(),
                arguments: Default::default(),
                properties: Default::default(),
                instructions: vec![
                    ftd::component::Instruction::ChildComponent {
                        child: ftd::component::ChildComponent {
                            events: vec![],
                            root: "foo/bar#table-of-content".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("id"),
                                ftd::component::Property {
                                    default: Some(ftd::PropertyValue::Value {
                                        value: ftd::variable::Value::String {
                                            text: "toc_main".to_string(),
                                            source: ftd::TextSource::Header,
                                        },
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            arguments: Default::default(),
                            is_recursive: false,
                            ..Default::default()
                        },
                    },
                    ftd::component::Instruction::ChildComponent {
                        child: ftd::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "foo/bar#parent".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("active"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::variable::Value::Boolean { value: true },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("id"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::variable::Value::String {
                                                text: "/welcome/".to_string(),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("name"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::variable::Value::String {
                                                text: "5PM Tasks".to_string(),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::component::Instruction::ChildComponent {
                        child: ftd::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "foo/bar#parent".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("id"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::variable::Value::String {
                                                text: "/Building/".to_string(),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("name"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::variable::Value::String {
                                                text: "Log".to_string(),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::component::Instruction::ChildComponent {
                        child: ftd::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "foo/bar#parent".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("id"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::variable::Value::String {
                                                text: "/ChildBuilding/".to_string(),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("name"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::variable::Value::String {
                                                text: "ChildLog".to_string(),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::component::Instruction::ChangeContainer {
                        name: "/welcome/".to_string(),
                    },
                    ftd::component::Instruction::ChildComponent {
                        child: ftd::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "foo/bar#parent".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("id"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::variable::Value::String {
                                                text: "/Building2/".to_string(),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("name"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::variable::Value::String {
                                                text: "Log2".to_string(),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                ],
                kernel: false,
                ..Default::default()
            }),
        );

        bag.insert(
            "foo/bar#parent".to_string(),
            ftd::p2::Thing::Component(ftd::Component {
                root: "ftd#column".to_string(),
                full_name: "foo/bar#parent".to_string(),
                arguments: std::iter::IntoIterator::into_iter([
                    (
                        s("active"),
                        ftd::p2::Kind::Optional {
                            kind: Box::new(ftd::p2::Kind::boolean()),
                            is_reference: false,
                        },
                    ),
                    (s("id"), ftd::p2::Kind::string()),
                    (s("name"), ftd::p2::Kind::caption()),
                ])
                .collect(),
                properties: std::iter::IntoIterator::into_iter([
                    (
                        s("id"),
                        ftd::component::Property {
                            default: Some(ftd::PropertyValue::Variable {
                                name: "id".to_string(),
                                kind: ftd::p2::Kind::Optional {
                                    kind: Box::new(ftd::p2::Kind::string()),
                                    is_reference: false,
                                },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                    (
                        s("open"),
                        ftd::component::Property {
                            default: Some(ftd::PropertyValue::Value {
                                value: ftd::Value::Boolean { value: true },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                    (
                        s("width"),
                        ftd::component::Property {
                            default: Some(ftd::PropertyValue::Value {
                                value: ftd::variable::Value::String {
                                    text: "fill".to_string(),
                                    source: ftd::TextSource::Header,
                                },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                ])
                .collect(),
                instructions: vec![
                    ftd::component::Instruction::ChildComponent {
                        child: ftd::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: Some(ftd::p2::Boolean::IsNotNull {
                                value: ftd::PropertyValue::Variable {
                                    name: "active".to_string(),
                                    kind: ftd::p2::Kind::Optional {
                                        kind: Box::new(ftd::p2::Kind::boolean()),
                                        is_reference: false,
                                    },
                                },
                            }),
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("color"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Reference {
                                            name: s("foo/bar#white"),
                                            kind: ftd::p2::Kind::Optional {
                                                kind: Box::new(ftd::p2::Kind::Record {
                                                    name: s("ftd#color"),
                                                    default: None,
                                                    is_reference: false,
                                                }),
                                                is_reference: false,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("text"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Variable {
                                            name: "name".to_string(),
                                            kind: ftd::p2::Kind::caption_or_body(),
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::component::Instruction::ChildComponent {
                        child: ftd::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: Some(ftd::p2::Boolean::IsNull {
                                value: ftd::PropertyValue::Variable {
                                    name: "active".to_string(),
                                    kind: ftd::p2::Kind::Optional {
                                        kind: Box::new(ftd::p2::Kind::boolean()),
                                        is_reference: false,
                                    },
                                },
                            }),
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("color"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Reference {
                                            name: s("foo/bar#4D4D4D"),
                                            kind: ftd::p2::Kind::Optional {
                                                kind: Box::new(ftd::p2::Kind::Record {
                                                    name: s("ftd#color"),
                                                    default: None,
                                                    is_reference: false,
                                                }),
                                                is_reference: false,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("text"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Variable {
                                            name: "name".to_string(),
                                            kind: ftd::p2::Kind::caption_or_body(),
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                ],
                kernel: false,
                ..Default::default()
            }),
        );

        bag.insert(
            "foo/bar#table-of-content".to_string(),
            ftd::p2::Thing::Component(ftd::Component {
                root: "ftd#column".to_string(),
                full_name: "foo/bar#table-of-content".to_string(),
                arguments: std::iter::IntoIterator::into_iter([(s("id"), ftd::p2::Kind::string())])
                    .collect(),
                properties: std::iter::IntoIterator::into_iter([
                    (
                        s("height"),
                        ftd::component::Property {
                            default: Some(ftd::PropertyValue::Value {
                                value: ftd::variable::Value::String {
                                    text: "fill".to_string(),
                                    source: ftd::TextSource::Header,
                                },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                    (
                        s("id"),
                        ftd::component::Property {
                            default: Some(ftd::PropertyValue::Variable {
                                name: "id".to_string(),
                                kind: ftd::p2::Kind::Optional {
                                    kind: Box::new(ftd::p2::Kind::string()),
                                    is_reference: false,
                                },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                    (
                        s("width"),
                        ftd::component::Property {
                            default: Some(ftd::PropertyValue::Value {
                                value: ftd::variable::Value::String {
                                    text: "300".to_string(),
                                    source: ftd::TextSource::Header,
                                },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                ])
                .collect(),
                instructions: vec![],
                kernel: false,
                ..Default::default()
            }),
        );

        bag.insert(
            "foo/bar#toc-heading".to_string(),
            ftd::p2::Thing::Component(ftd::Component {
                root: "ftd#text".to_string(),
                full_name: "foo/bar#toc-heading".to_string(),
                arguments: std::iter::IntoIterator::into_iter([(
                    s("text"),
                    ftd::p2::Kind::caption(),
                )])
                .collect(),
                properties: std::iter::IntoIterator::into_iter([
                    (
                        s("line-clamp"),
                        ftd::component::Property {
                            default: Some(ftd::PropertyValue::Value {
                                value: ftd::variable::Value::Integer { value: 16 },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                    (
                        s("text"),
                        ftd::component::Property {
                            default: Some(ftd::PropertyValue::Variable {
                                name: "text".to_string(),
                                kind: ftd::p2::Kind::caption_or_body(),
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                ])
                .collect(),
                ..Default::default()
            }),
        );

        bag.insert(
            s("foo/bar#active@0,0,0"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("active"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Boolean { value: true },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#active@0,0,0,0,2"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("active"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Optional {
                        data: Box::new(None),
                        kind: ftd::p2::Kind::boolean(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#active@0,0,0,2"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("active"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Optional {
                        data: Box::new(None),
                        kind: ftd::p2::Kind::boolean(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#active@0,0,0,3"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("active"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Optional {
                        data: Box::new(None),
                        kind: ftd::p2::Kind::boolean(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#id@0,0"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("id"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("toc_main"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#id@0,0,0"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("id"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("/welcome/"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#id@0,0,0,2"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("id"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("/Building/"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#id@0,0,0,0,2"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("id"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("/ChildBuilding/"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#id@0,0,0,3"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("id"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("/Building2/"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#name@0,0,0,2"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("name"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Log"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#name@0,0,0,0,2"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("name"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("ChildLog"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#name@0,0,0"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("name"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("5PM Tasks"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#name@0,0,0,3"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("name"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Log2"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        bag.insert(
            s("foo/bar#4D4D4D"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("4D4D4D"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("ftd#color"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("dark"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("#4D4D4D"),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("light"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("#4D4D4D"),
                                        source: ftd::TextSource::Caption,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        bag.insert(
            s("foo/bar#white"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("white"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("ftd#color"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("dark"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("white"),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("light"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("white"),
                                        source: ftd::TextSource::Caption,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        let children = vec![
            ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("5PM Tasks"),
                line: true,
                common: ftd::Common {
                    color: Some(ftd::Color {
                        light: ftd::ColorValue {
                            r: 255,
                            g: 255,
                            b: 255,
                            alpha: 1.0,
                        },
                        dark: ftd::ColorValue {
                            r: 255,
                            g: 255,
                            b: 255,
                            alpha: 1.0,
                        },
                        reference: Some(s("foo/bar#white")),
                    }),
                    reference: Some(s("foo/bar#name@0,0,0")),
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#active@0,0,0"),
                        value: serde_json::Value::String(s("$IsNotNull$")),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }),
            ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("5PM Tasks"),
                line: true,
                common: ftd::Common {
                    color: Some(ftd::Color {
                        light: ftd::ColorValue {
                            r: 77,
                            g: 77,
                            b: 77,
                            alpha: 1.0,
                        },
                        dark: ftd::ColorValue {
                            r: 77,
                            g: 77,
                            b: 77,
                            alpha: 1.0,
                        },
                        reference: Some(s("foo/bar#4D4D4D")),
                    }),
                    reference: Some(s("foo/bar#name@0,0,0")),
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#active@0,0,0"),
                        value: serde_json::Value::String(s("$IsNull$")),
                    }),
                    is_not_visible: true,
                    ..Default::default()
                },
                ..Default::default()
            }),
            ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("Log"),
                            line: true,
                            common: ftd::Common {
                                color: Some(ftd::Color {
                                    light: ftd::ColorValue {
                                        r: 255,
                                        g: 255,
                                        b: 255,
                                        alpha: 1.0,
                                    },
                                    dark: ftd::ColorValue {
                                        r: 255,
                                        g: 255,
                                        b: 255,
                                        alpha: 1.0,
                                    },
                                    reference: Some(s("foo/bar#white")),
                                }),
                                reference: Some(s("foo/bar#name@0,0,0,2")),
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#active@0,0,0,2"),
                                    value: serde_json::Value::String(s("$IsNotNull$")),
                                }),
                                is_not_visible: true,
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("Log"),
                            line: true,
                            common: ftd::Common {
                                color: Some(ftd::Color {
                                    light: ftd::ColorValue {
                                        r: 77,
                                        g: 77,
                                        b: 77,
                                        alpha: 1.0,
                                    },
                                    dark: ftd::ColorValue {
                                        r: 77,
                                        g: 77,
                                        b: 77,
                                        alpha: 1.0,
                                    },
                                    reference: Some(s("foo/bar#4D4D4D")),
                                }),
                                reference: Some(s("foo/bar#name@0,0,0,2")),
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#active@0,0,0,2"),
                                    value: serde_json::Value::String(s("$IsNull$")),
                                }),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                external_children: Default::default(),
                                children: vec![
                                    ftd::Element::Markup(ftd::Markups {
                                        text: ftd::markdown_line("ChildLog"),
                                        line: true,
                                        common: ftd::Common {
                                            color: Some(ftd::Color {
                                                light: ftd::ColorValue {
                                                    r: 255,
                                                    g: 255,
                                                    b: 255,
                                                    alpha: 1.0,
                                                },
                                                dark: ftd::ColorValue {
                                                    r: 255,
                                                    g: 255,
                                                    b: 255,
                                                    alpha: 1.0,
                                                },
                                                reference: Some(s("foo/bar#white")),
                                            }),
                                            reference: Some(s("foo/bar#name@0,0,0,0,2")),
                                            condition: Some(ftd::Condition {
                                                variable: s("foo/bar#active@0,0,0,0,2"),
                                                value: serde_json::Value::String(s("$IsNotNull$")),
                                            }),
                                            is_not_visible: true,
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    }),
                                    ftd::Element::Markup(ftd::Markups {
                                        text: ftd::markdown_line("ChildLog"),
                                        line: true,
                                        common: ftd::Common {
                                            color: Some(ftd::Color {
                                                light: ftd::ColorValue {
                                                    r: 77,
                                                    g: 77,
                                                    b: 77,
                                                    alpha: 1.0,
                                                },
                                                dark: ftd::ColorValue {
                                                    r: 77,
                                                    g: 77,
                                                    b: 77,
                                                    alpha: 1.0,
                                                },
                                                reference: Some(s("foo/bar#4D4D4D")),
                                            }),
                                            reference: Some(s("foo/bar#name@0,0,0,0,2")),
                                            condition: Some(ftd::Condition {
                                                variable: s("foo/bar#active@0,0,0,0,2"),
                                                value: serde_json::Value::String(s("$IsNull$")),
                                            }),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    }),
                                ],
                                open: Some(true),
                                ..Default::default()
                            },
                            common: ftd::Common {
                                data_id: Some(s("/ChildBuilding/")),
                                width: Some(ftd::Length::Fill),
                                ..Default::default()
                            },
                        }),
                    ],
                    external_children: Default::default(),
                    open: Some(true),
                    ..Default::default()
                },
                common: ftd::Common {
                    data_id: Some(s("/Building/")),
                    width: Some(ftd::Length::Fill),
                    ..Default::default()
                },
            }),
            ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    external_children: Default::default(),
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("Log2"),
                            line: true,
                            common: ftd::Common {
                                color: Some(ftd::Color {
                                    light: ftd::ColorValue {
                                        r: 255,
                                        g: 255,
                                        b: 255,
                                        alpha: 1.0,
                                    },
                                    dark: ftd::ColorValue {
                                        r: 255,
                                        g: 255,
                                        b: 255,
                                        alpha: 1.0,
                                    },
                                    reference: Some(s("foo/bar#white")),
                                }),
                                reference: Some(s("foo/bar#name@0,0,0,3")),
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#active@0,0,0,3"),
                                    value: serde_json::Value::String(s("$IsNotNull$")),
                                }),
                                is_not_visible: true,
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("Log2"),
                            line: true,
                            common: ftd::Common {
                                color: Some(ftd::Color {
                                    light: ftd::ColorValue {
                                        r: 77,
                                        g: 77,
                                        b: 77,
                                        alpha: 1.0,
                                    },
                                    dark: ftd::ColorValue {
                                        r: 77,
                                        g: 77,
                                        b: 77,
                                        alpha: 1.0,
                                    },
                                    reference: Some(s("foo/bar#4D4D4D")),
                                }),
                                reference: Some(s("foo/bar#name@0,0,0,3")),
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#active@0,0,0,3"),
                                    value: serde_json::Value::String(s("$IsNull$")),
                                }),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ],
                    open: Some(true),
                    ..Default::default()
                },
                common: ftd::Common {
                    data_id: Some(s("/Building2/")),
                    width: Some(ftd::Length::Fill),
                    ..Default::default()
                },
            }),
        ];

        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Column(ftd::Column {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![ftd::Element::Column(ftd::Column {
                                spacing: None,
                                container: ftd::Container {
                                    children,
                                    external_children: Default::default(),
                                    open: Some(true),
                                    ..Default::default()
                                },
                                common: ftd::Common {
                                    data_id: Some(s("/welcome/")),
                                    width: Some(ftd::Length::Fill),
                                    ..Default::default()
                                },
                            })],
                            ..Default::default()
                        },
                        common: ftd::Common {
                            data_id: Some(s("toc_main")),
                            height: Some(ftd::Length::Fill),
                            width: Some(ftd::Length::Px { value: 300 }),
                            ..Default::default()
                        },
                    })],
                    ..Default::default()
                },
                ..Default::default()
            }));

        p!(
            r"
            -- ftd.color white: white
            dark: white
            
            -- ftd.color 4D4D4D: #4D4D4D
            dark: #4D4D4D

            -- ftd.text toc-heading:
            caption text:
            text: $text
            line-clamp: 16


            -- ftd.column table-of-content:
            string id:
            id: $id
            width: 300
            height: fill


            -- ftd.column parent:
            string id:
            caption name:
            optional boolean active:
            id: $id
            width: fill
            open: true

            --- ftd.text:
            if: $active is not null
            text: $name
            color: $white

            --- ftd.text:
            if: $active is null
            text: $name
            color: $4D4D4D


            -- ftd.column ft_toc:

            --- table-of-content:
            id: toc_main

            --- parent:
            id: /welcome/
            name: 5PM Tasks
            active: true

            --- parent:
            id: /Building/
            name: Log

            --- parent:
            id: /ChildBuilding/
            name: ChildLog

            --- container: \/welcome/

            --- parent:
            id: /Building2/
            name: Log2


            -- ft_toc:
            ",
            (bag, main),
        );
    }

    #[test]
    fn creating_a_tree_using_import() {
        let mut bag = super::default_bag();

        bag.insert(
            "creating-a-tree#ft_toc".to_string(),
            ftd::p2::Thing::Component(ftd::Component {
                root: "ftd#column".to_string(),
                full_name: "creating-a-tree#ft_toc".to_string(),
                arguments: Default::default(),
                properties: Default::default(),
                instructions: vec![
                    ftd::component::Instruction::ChildComponent {
                        child: ftd::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "creating-a-tree#table-of-content".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("id"),
                                ftd::component::Property {
                                    default: Some(ftd::PropertyValue::Value {
                                        value: ftd::variable::Value::String {
                                            text: "toc_main".to_string(),
                                            source: ftd::TextSource::Header,
                                        },
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::component::Instruction::ChildComponent {
                        child: ftd::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "creating-a-tree#parent".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("active"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::variable::Value::Boolean { value: true },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("id"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::variable::Value::String {
                                                text: "/welcome/".to_string(),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("name"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::variable::Value::String {
                                                text: "5PM Tasks".to_string(),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::component::Instruction::ChildComponent {
                        child: ftd::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "creating-a-tree#parent".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("id"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::variable::Value::String {
                                                text: "/Building/".to_string(),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("name"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::variable::Value::String {
                                                text: "Log".to_string(),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::component::Instruction::ChildComponent {
                        child: ftd::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "creating-a-tree#parent".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("id"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::variable::Value::String {
                                                text: "/ChildBuilding/".to_string(),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("name"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::variable::Value::String {
                                                text: "ChildLog".to_string(),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::component::Instruction::ChangeContainer {
                        name: "/welcome/".to_string(),
                    },
                    ftd::component::Instruction::ChildComponent {
                        child: ftd::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "creating-a-tree#parent".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("id"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::variable::Value::String {
                                                text: "/Building2/".to_string(),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("name"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::variable::Value::String {
                                                text: "Log2".to_string(),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                ],
                kernel: false,
                ..Default::default()
            }),
        );

        bag.insert(
            "creating-a-tree#parent".to_string(),
            ftd::p2::Thing::Component(ftd::Component {
                root: "ftd#column".to_string(),
                full_name: "creating-a-tree#parent".to_string(),
                arguments: std::iter::IntoIterator::into_iter([
                    (
                        s("active"),
                        ftd::p2::Kind::Optional {
                            kind: Box::new(ftd::p2::Kind::boolean()),
                            is_reference: false,
                        },
                    ),
                    (s("id"), ftd::p2::Kind::string()),
                    (s("name"), ftd::p2::Kind::caption()),
                ])
                .collect(),
                properties: std::iter::IntoIterator::into_iter([
                    (
                        s("id"),
                        ftd::component::Property {
                            default: Some(ftd::PropertyValue::Variable {
                                name: "id".to_string(),
                                kind: ftd::p2::Kind::Optional {
                                    kind: Box::new(ftd::p2::Kind::string()),
                                    is_reference: false,
                                },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                    (
                        s("open"),
                        ftd::component::Property {
                            default: Some(ftd::PropertyValue::Value {
                                value: ftd::Value::Boolean { value: true },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                    (
                        s("width"),
                        ftd::component::Property {
                            default: Some(ftd::PropertyValue::Value {
                                value: ftd::variable::Value::String {
                                    text: "fill".to_string(),
                                    source: ftd::TextSource::Header,
                                },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                ])
                .collect(),
                instructions: vec![
                    ftd::component::Instruction::ChildComponent {
                        child: ftd::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: Some(ftd::p2::Boolean::IsNotNull {
                                value: ftd::PropertyValue::Variable {
                                    name: "active".to_string(),
                                    kind: ftd::p2::Kind::Optional {
                                        kind: Box::new(ftd::p2::Kind::boolean()),
                                        is_reference: false,
                                    },
                                },
                            }),
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("color"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Reference {
                                            name: s("creating-a-tree#white"),
                                            kind: ftd::p2::Kind::Optional {
                                                kind: Box::new(ftd::p2::Kind::Record {
                                                    name: s("ftd#color"),
                                                    default: None,
                                                    is_reference: false,
                                                }),
                                                is_reference: false,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("text"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Variable {
                                            name: "name".to_string(),
                                            kind: ftd::p2::Kind::caption_or_body(),
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::component::Instruction::ChildComponent {
                        child: ftd::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: Some(ftd::p2::Boolean::IsNull {
                                value: ftd::PropertyValue::Variable {
                                    name: "active".to_string(),
                                    kind: ftd::p2::Kind::Optional {
                                        kind: Box::new(ftd::p2::Kind::boolean()),
                                        is_reference: false,
                                    },
                                },
                            }),
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("color"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Reference {
                                            name: s("creating-a-tree#4D4D4D"),
                                            kind: ftd::p2::Kind::Optional {
                                                kind: Box::new(ftd::p2::Kind::Record {
                                                    name: s("ftd#color"),
                                                    default: None,
                                                    is_reference: false,
                                                }),
                                                is_reference: false,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("text"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Variable {
                                            name: "name".to_string(),
                                            kind: ftd::p2::Kind::caption_or_body(),
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                ],
                kernel: false,
                ..Default::default()
            }),
        );

        bag.insert(
            "creating-a-tree#table-of-content".to_string(),
            ftd::p2::Thing::Component(ftd::Component {
                root: "ftd#column".to_string(),
                full_name: "creating-a-tree#table-of-content".to_string(),
                arguments: std::iter::IntoIterator::into_iter([(s("id"), ftd::p2::Kind::string())])
                    .collect(),
                properties: std::iter::IntoIterator::into_iter([
                    (
                        s("height"),
                        ftd::component::Property {
                            default: Some(ftd::PropertyValue::Value {
                                value: ftd::variable::Value::String {
                                    text: "fill".to_string(),
                                    source: ftd::TextSource::Header,
                                },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                    (
                        s("id"),
                        ftd::component::Property {
                            default: Some(ftd::PropertyValue::Variable {
                                name: "id".to_string(),
                                kind: ftd::p2::Kind::Optional {
                                    kind: Box::new(ftd::p2::Kind::string()),
                                    is_reference: false,
                                },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                    (
                        s("width"),
                        ftd::component::Property {
                            default: Some(ftd::PropertyValue::Value {
                                value: ftd::variable::Value::String {
                                    text: "300".to_string(),
                                    source: ftd::TextSource::Header,
                                },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                ])
                .collect(),
                instructions: vec![],
                kernel: false,
                ..Default::default()
            }),
        );

        bag.insert(
            "creating-a-tree#toc-heading".to_string(),
            ftd::p2::Thing::Component(ftd::Component {
                root: "ftd#text".to_string(),
                full_name: "creating-a-tree#toc-heading".to_string(),
                arguments: std::iter::IntoIterator::into_iter([(
                    s("text"),
                    ftd::p2::Kind::caption(),
                )])
                .collect(),
                properties: std::iter::IntoIterator::into_iter([(
                    s("text"),
                    ftd::component::Property {
                        default: Some(ftd::PropertyValue::Variable {
                            name: "text".to_string(),
                            kind: ftd::p2::Kind::caption_or_body(),
                        }),
                        conditions: vec![],
                        ..Default::default()
                    },
                )])
                .collect(),
                ..Default::default()
            }),
        );
        bag.insert(
            s("foo/bar#active@0,0,0"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("active"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Boolean { value: true },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#active@0,0,0,2"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("active"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Optional {
                        data: Box::new(None),
                        kind: ftd::p2::Kind::boolean(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#active@0,0,0,0,2"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("active"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Optional {
                        data: Box::new(None),
                        kind: ftd::p2::Kind::boolean(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#active@0,0,0,3"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("active"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Optional {
                        data: Box::new(None),
                        kind: ftd::p2::Kind::boolean(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#id@0,0"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("id"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("toc_main"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#id@0,0,0"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("id"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("/welcome/"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#id@0,0,0,2"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("id"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("/Building/"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#id@0,0,0,0,2"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("id"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("/ChildBuilding/"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#id@0,0,0,3"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("id"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("/Building2/"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#name@0,0,0,2"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("name"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Log"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#name@0,0,0,0,2"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("name"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("ChildLog"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#name@0,0,0"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("name"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("5PM Tasks"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#name@0,0,0,3"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("name"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Log2"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("creating-a-tree#4D4D4D"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("4D4D4D"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("ftd#color"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("dark"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("#4D4D4D"),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("light"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("#4D4D4D"),
                                        source: ftd::TextSource::Caption,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        bag.insert(
            s("creating-a-tree#white"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("white"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("ftd#color"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("dark"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("white"),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("light"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("white"),
                                        source: ftd::TextSource::Caption,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        let children = vec![
            ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("5PM Tasks"),
                line: true,
                common: ftd::Common {
                    color: Some(ftd::Color {
                        light: ftd::ColorValue {
                            r: 255,
                            g: 255,
                            b: 255,
                            alpha: 1.0,
                        },
                        dark: ftd::ColorValue {
                            r: 255,
                            g: 255,
                            b: 255,
                            alpha: 1.0,
                        },
                        reference: Some(s("creating-a-tree#white")),
                    }),
                    reference: Some(s("foo/bar#name@0,0,0")),
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#active@0,0,0"),
                        value: serde_json::Value::String(s("$IsNotNull$")),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }),
            ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("5PM Tasks"),
                line: true,
                common: ftd::Common {
                    color: Some(ftd::Color {
                        light: ftd::ColorValue {
                            r: 77,
                            g: 77,
                            b: 77,
                            alpha: 1.0,
                        },
                        dark: ftd::ColorValue {
                            r: 77,
                            g: 77,
                            b: 77,
                            alpha: 1.0,
                        },
                        reference: Some(s("creating-a-tree#4D4D4D")),
                    }),
                    reference: Some(s("foo/bar#name@0,0,0")),
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#active@0,0,0"),
                        value: serde_json::Value::String(s("$IsNull$")),
                    }),
                    is_not_visible: true,
                    ..Default::default()
                },
                ..Default::default()
            }),
            ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("Log"),
                            line: true,
                            common: ftd::Common {
                                color: Some(ftd::Color {
                                    light: ftd::ColorValue {
                                        r: 255,
                                        g: 255,
                                        b: 255,
                                        alpha: 1.0,
                                    },
                                    dark: ftd::ColorValue {
                                        r: 255,
                                        g: 255,
                                        b: 255,
                                        alpha: 1.0,
                                    },
                                    reference: Some(s("creating-a-tree#white")),
                                }),
                                reference: Some(s("foo/bar#name@0,0,0,2")),
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#active@0,0,0,2"),
                                    value: serde_json::Value::String(s("$IsNotNull$")),
                                }),
                                is_not_visible: true,
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("Log"),
                            line: true,
                            common: ftd::Common {
                                color: Some(ftd::Color {
                                    light: ftd::ColorValue {
                                        r: 77,
                                        g: 77,
                                        b: 77,
                                        alpha: 1.0,
                                    },
                                    dark: ftd::ColorValue {
                                        r: 77,
                                        g: 77,
                                        b: 77,
                                        alpha: 1.0,
                                    },
                                    reference: Some(s("creating-a-tree#4D4D4D")),
                                }),
                                reference: Some(s("foo/bar#name@0,0,0,2")),
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#active@0,0,0,2"),
                                    value: serde_json::Value::String(s("$IsNull$")),
                                }),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                external_children: Default::default(),
                                children: vec![
                                    ftd::Element::Markup(ftd::Markups {
                                        text: ftd::markdown_line("ChildLog"),
                                        line: true,
                                        common: ftd::Common {
                                            color: Some(ftd::Color {
                                                light: ftd::ColorValue {
                                                    r: 255,
                                                    g: 255,
                                                    b: 255,
                                                    alpha: 1.0,
                                                },
                                                dark: ftd::ColorValue {
                                                    r: 255,
                                                    g: 255,
                                                    b: 255,
                                                    alpha: 1.0,
                                                },
                                                reference: Some(s("creating-a-tree#white")),
                                            }),
                                            reference: Some(s("foo/bar#name@0,0,0,0,2")),
                                            condition: Some(ftd::Condition {
                                                variable: s("foo/bar#active@0,0,0,0,2"),
                                                value: serde_json::Value::String(s("$IsNotNull$")),
                                            }),
                                            is_not_visible: true,
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    }),
                                    ftd::Element::Markup(ftd::Markups {
                                        text: ftd::markdown_line("ChildLog"),
                                        line: true,
                                        common: ftd::Common {
                                            color: Some(ftd::Color {
                                                light: ftd::ColorValue {
                                                    r: 77,
                                                    g: 77,
                                                    b: 77,
                                                    alpha: 1.0,
                                                },
                                                dark: ftd::ColorValue {
                                                    r: 77,
                                                    g: 77,
                                                    b: 77,
                                                    alpha: 1.0,
                                                },
                                                reference: Some(s("creating-a-tree#4D4D4D")),
                                            }),
                                            reference: Some(s("foo/bar#name@0,0,0,0,2")),
                                            condition: Some(ftd::Condition {
                                                variable: s("foo/bar#active@0,0,0,0,2"),
                                                value: serde_json::Value::String(s("$IsNull$")),
                                            }),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    }),
                                ],
                                open: Some(true),
                                ..Default::default()
                            },
                            common: ftd::Common {
                                data_id: Some(s("/ChildBuilding/")),
                                width: Some(ftd::Length::Fill),
                                ..Default::default()
                            },
                        }),
                    ],
                    external_children: Default::default(),
                    open: Some(true),
                    ..Default::default()
                },
                common: ftd::Common {
                    data_id: Some(s("/Building/")),
                    width: Some(ftd::Length::Fill),
                    ..Default::default()
                },
            }),
            ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    external_children: Default::default(),
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("Log2"),
                            line: true,
                            common: ftd::Common {
                                color: Some(ftd::Color {
                                    light: ftd::ColorValue {
                                        r: 255,
                                        g: 255,
                                        b: 255,
                                        alpha: 1.0,
                                    },
                                    dark: ftd::ColorValue {
                                        r: 255,
                                        g: 255,
                                        b: 255,
                                        alpha: 1.0,
                                    },
                                    reference: Some(s("creating-a-tree#white")),
                                }),
                                reference: Some(s("foo/bar#name@0,0,0,3")),
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#active@0,0,0,3"),
                                    value: serde_json::Value::String(s("$IsNotNull$")),
                                }),
                                is_not_visible: true,
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("Log2"),
                            line: true,
                            common: ftd::Common {
                                color: Some(ftd::Color {
                                    light: ftd::ColorValue {
                                        r: 77,
                                        g: 77,
                                        b: 77,
                                        alpha: 1.0,
                                    },
                                    dark: ftd::ColorValue {
                                        r: 77,
                                        g: 77,
                                        b: 77,
                                        alpha: 1.0,
                                    },
                                    reference: Some(s("creating-a-tree#4D4D4D")),
                                }),
                                reference: Some(s("foo/bar#name@0,0,0,3")),
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#active@0,0,0,3"),
                                    value: serde_json::Value::String(s("$IsNull$")),
                                }),
                                ..Default::default()
                            },

                            ..Default::default()
                        }),
                    ],
                    open: Some(true),
                    ..Default::default()
                },
                common: ftd::Common {
                    data_id: Some(s("/Building2/")),
                    width: Some(ftd::Length::Fill),
                    ..Default::default()
                },
            }),
        ];

        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Column(ftd::Column {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![ftd::Element::Column(ftd::Column {
                                spacing: None,
                                container: ftd::Container {
                                    children,
                                    external_children: Default::default(),
                                    open: Some(true),
                                    ..Default::default()
                                },
                                common: ftd::Common {
                                    data_id: Some(s("/welcome/")),
                                    width: Some(ftd::Length::Fill),
                                    ..Default::default()
                                },
                            })],
                            ..Default::default()
                        },
                        common: ftd::Common {
                            data_id: Some(s("toc_main")),
                            height: Some(ftd::Length::Fill),
                            width: Some(ftd::Length::Px { value: 300 }),
                            ..Default::default()
                        },
                    })],
                    ..Default::default()
                },
                ..Default::default()
            }));

        p!(
            "
            -- import: creating-a-tree as ft

            -- ft.ft_toc:
            ",
            (bag, main),
        );
    }

    #[test]
    fn reference() {
        let mut bag = super::default_bag();

        bag.insert(
            s("reference#f3f3f3"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("f3f3f3"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("ftd#color"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("dark"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("#f3f3f3"),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("light"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("#f3f3f3"),
                                        source: ftd::TextSource::Caption,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        bag.insert(
            "fifthtry/ft#dark-mode".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "dark-mode".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Boolean { value: true },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "fifthtry/ft#toc".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "toc".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "not set".to_string(),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "fifthtry/ft#markdown".to_string(),
            ftd::p2::Thing::Component(ftd::Component {
                root: "ftd#text".to_string(),
                full_name: "fifthtry/ft#markdown".to_string(),
                arguments: std::iter::IntoIterator::into_iter([(s("body"), ftd::p2::Kind::body())])
                    .collect(),
                properties: std::iter::IntoIterator::into_iter([(
                    s("text"),
                    ftd::component::Property {
                        default: Some(ftd::PropertyValue::Variable {
                            name: "body".to_string(),
                            kind: ftd::p2::Kind::caption_or_body(),
                        }),
                        conditions: vec![],
                        ..Default::default()
                    },
                )])
                .collect(),
                ..Default::default()
            }),
        );

        bag.insert(
            "reference#name".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "name".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "John smith".to_string(),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "reference#test-component".to_string(),
            ftd::p2::Thing::Component(ftd::Component {
                root: "ftd#column".to_string(),
                full_name: "reference#test-component".to_string(),
                arguments: Default::default(),
                properties: std::iter::IntoIterator::into_iter([
                    (
                        s("background-color"),
                        ftd::component::Property {
                            default: Some(ftd::PropertyValue::Reference {
                                name: s("reference#f3f3f3"),
                                kind: ftd::p2::Kind::Optional {
                                    kind: Box::new(ftd::p2::Kind::Record {
                                        name: s("ftd#color"),
                                        default: None,
                                        is_reference: false,
                                    }),
                                    is_reference: false,
                                },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                    (
                        s("width"),
                        ftd::component::Property {
                            default: Some(ftd::PropertyValue::Value {
                                value: ftd::variable::Value::String {
                                    text: "200".to_string(),
                                    source: ftd::TextSource::Header,
                                },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                ])
                .collect(),
                instructions: vec![ftd::component::Instruction::ChildComponent {
                    child: ftd::component::ChildComponent {
                        is_recursive: false,
                        events: vec![],
                        root: "ftd#text".to_string(),
                        condition: None,
                        properties: std::iter::IntoIterator::into_iter([(
                            s("text"),
                            ftd::component::Property {
                                default: Some(ftd::PropertyValue::Reference {
                                    name: "reference#name".to_string(),
                                    kind: ftd::p2::Kind::caption_or_body(),
                                }),
                                conditions: vec![],
                                ..Default::default()
                            },
                        )])
                        .collect(),
                        ..Default::default()
                    },
                }],
                kernel: false,
                ..Default::default()
            }),
        );
        let title = ftd::Markups {
            text: ftd::markdown_line("John smith"),
            line: true,
            common: ftd::Common {
                reference: Some(s("reference#name")),
                ..Default::default()
            },
            ..Default::default()
        };

        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                common: ftd::Common {
                    width: Some(ftd::Length::Px { value: 200 }),
                    background_color: Some(ftd::Color {
                        light: ftd::ColorValue {
                            r: 243,
                            g: 243,
                            b: 243,
                            alpha: 1.0,
                        },
                        dark: ftd::ColorValue {
                            r: 243,
                            g: 243,
                            b: 243,
                            alpha: 1.0,
                        },
                        reference: Some(s("reference#f3f3f3")),
                    }),
                    ..Default::default()
                },
                container: ftd::Container {
                    children: vec![ftd::Element::Markup(title)],
                    ..Default::default()
                },
            }));

        p!(
            "
            -- import: reference as ct

            -- ct.test-component:
            ",
            (bag, main),
        );
    }

    #[test]
    fn text() {
        let mut bag = super::default_bag();

        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@0", -1, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@1", -1, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@2", -1, &mut bag);

        insert_integer_by_root("foo/bar#CHILDREN-COUNT@0", 0, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT@1", 0, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT@2", 0, &mut bag);

        insert_integer_by_root("foo/bar#SIBLING-INDEX-0@0", 0, &mut bag);
        insert_integer_by_root("foo/bar#SIBLING-INDEX-0@1", 1, &mut bag);
        insert_integer_by_root("foo/bar#SIBLING-INDEX-0@2", 2, &mut bag);

        insert_integer_by_root("foo/bar#SIBLING-INDEX@0", 1, &mut bag);
        insert_integer_by_root("foo/bar#SIBLING-INDEX@1", 2, &mut bag);
        insert_integer_by_root("foo/bar#SIBLING-INDEX@2", 3, &mut bag);

        bag.insert(
            "foo/bar#foo".to_string(),
            ftd::p2::Thing::Component(ftd::Component {
                full_name: s("foo/bar#foo"),
                root: "ftd#text".to_string(),
                arguments: std::iter::IntoIterator::into_iter([(
                    s("name"),
                    ftd::p2::Kind::caption_or_body(),
                )])
                .collect(),
                properties: std::iter::IntoIterator::into_iter([(
                    s("text"),
                    ftd::component::Property {
                        default: Some(ftd::PropertyValue::Variable {
                            name: "name".to_string(),
                            kind: ftd::p2::Kind::caption_or_body(),
                        }),
                        conditions: vec![],
                        ..Default::default()
                    },
                )])
                .collect(),
                invocations: vec![
                    std::iter::IntoIterator::into_iter([(
                        s("name"),
                        ftd::Value::String {
                            text: s("hello"),
                            source: ftd::TextSource::Caption,
                        },
                    )])
                    .collect(),
                    std::iter::IntoIterator::into_iter([(
                        s("name"),
                        ftd::Value::String {
                            text: s("world"),
                            source: ftd::TextSource::Header,
                        },
                    )])
                    .collect(),
                    std::iter::IntoIterator::into_iter([(
                        s("name"),
                        ftd::Value::String {
                            text: s("yo yo"),
                            source: ftd::TextSource::Body,
                        },
                    )])
                    .collect(),
                ],
                line_number: 1,
                ..Default::default()
            }),
        );
        bag.insert(
            "foo/bar#name@0".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "name".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "hello".to_string(),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#name@1".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "name".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "world".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#name@2".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "name".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "yo yo".to_string(),
                        source: ftd::TextSource::Body,
                    },
                },
                conditions: vec![],
            }),
        );

        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("hello"),
                line: true,
                common: ftd::Common {
                    reference: Some(s("foo/bar#name@0")),
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("world"),
                line: true,
                common: ftd::Common {
                    reference: Some(s("foo/bar#name@1")),
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("yo yo"),
                line: true,
                common: ftd::Common {
                    reference: Some(s("foo/bar#name@2")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.text foo:
                caption or body name:
                text: $name

                -- foo: hello

                -- foo:
                name: world

                -- foo:

                yo yo
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn row() {
        let mut main = super::default_column();
        let mut row = ftd::Row {
            common: ftd::Common {
                data_id: Some("the-row".to_string()),
                id: Some("the-row".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        row.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("hello"),
                line: true,
                ..Default::default()
            }));
        row.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("world"),
                line: true,
                ..Default::default()
            }));
        row.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("row child three"),
                line: true,
                ..Default::default()
            }));
        main.container.children.push(ftd::Element::Row(row));
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("back in main"),
                line: true,
                ..Default::default()
            }));

        p!(
            "
            -- ftd.row:
            id: the-row

            -- ftd.text:
            text: hello

            -- ftd.text:
            text: world

            -- container: ftd.main

            -- ftd.text:
            text: back in main

            -- container: the-row

            -- ftd.text:
            text: row child three
        ",
            (super::default_bag(), main),
        );
    }

    #[test]
    fn sub_function() {
        let mut main = super::default_column();
        let mut row: ftd::Row = Default::default();
        row.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("hello"),
                line: true,
                ..Default::default()
            }));
        row.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("world"),
                line: true,
                ..Default::default()
            }));
        main.container.children.push(ftd::Element::Row(row));
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("back in main"),
                line: true,
                ..Default::default()
            }));

        p!(
            "
            -- ftd.row:

            --- ftd.text:
            text: hello

            --- ftd.text:
            text: world

            -- ftd.text:
            text: back in main
        ",
            (super::default_bag(), main),
        );
    }

    #[test]
    fn list_of_numbers() {
        let mut bag = super::default_bag();
        bag.insert(
            "foo/bar#numbers".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "foo/bar#numbers".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::List {
                        data: vec![
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Integer { value: 20 },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Integer { value: 30 },
                            },
                        ],
                        kind: ftd::p2::Kind::integer(),
                    },
                },
                conditions: vec![],
            }),
        );

        p!(
            "
            -- integer list numbers:

            -- numbers: 20
            -- numbers: 30
            ",
            (bag, super::default_column()),
        );
    }

    #[test]
    fn list_of_records() {
        let mut bag = super::default_bag();
        bag.insert(
            "foo/bar#point".to_string(),
            ftd::p2::Thing::Record(ftd::p2::Record {
                name: "foo/bar#point".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    (s("x"), ftd::p2::Kind::integer()),
                    (s("y"), ftd::p2::Kind::integer()),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![s("x"), s("y")],
            }),
        );

        bag.insert(
            "foo/bar#points".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "foo/bar#points".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::List {
                        data: vec![
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: s("foo/bar#point"),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("x"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::Integer { value: 10 },
                                            },
                                        ),
                                        (
                                            s("y"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::Integer { value: 20 },
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: s("foo/bar#point"),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("x"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::Integer { value: 0 },
                                            },
                                        ),
                                        (
                                            s("y"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::Integer { value: 0 },
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                        ],
                        kind: ftd::p2::Kind::Record {
                            name: s("foo/bar#point"),
                            default: None,
                            is_reference: false,
                        },
                    },
                },
                conditions: vec![],
            }),
        );

        p!(
            "
            -- record point:
            integer x:
            integer y:

            -- point list points:

            -- points:
            x: 10
            y: 20

            -- points:
            x: 0
            y: 0
            ",
            (bag, super::default_column()),
        );
    }

    #[test]
    #[ignore]
    fn list_with_reference() {
        let mut bag = super::default_bag();
        bag.insert(
            "foo/bar#numbers".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "foo/bar#numbers".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::List {
                        data: vec![
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Integer { value: 20 },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Integer { value: 30 },
                            },
                            // TODO: third element
                        ],
                        kind: ftd::p2::Kind::integer(),
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#x".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "x".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 20 },
                },
                conditions: vec![],
            }),
        );

        p!(
            "
            -- integer list numbers:

            -- numbers: 20
            -- numbers: 30

            -- integer x: 20

            -- numbers: $x
            ",
            (bag, super::default_column()),
        );
    }

    fn white_two_image_bag(
        about_optional: bool,
    ) -> std::collections::BTreeMap<String, ftd::p2::Thing> {
        let mut bag = super::default_bag();
        bag.insert(
            s("foo/bar#white-two-image"),
            ftd::p2::Thing::Component(ftd::Component {
                invocations: Default::default(),
                full_name: "foo/bar#white-two-image".to_string(),
                root: s("ftd#column"),
                arguments: std::iter::IntoIterator::into_iter([
                    (s("about"), {
                        let s = ftd::p2::Kind::body();
                        if about_optional {
                            s.into_optional()
                        } else {
                            s
                        }
                    }),
                    (s("src"), {
                        let s = ftd::p2::Kind::Record {
                            name: s("ftd#image-src"),
                            default: None,
                            is_reference: false,
                        };
                        if about_optional {
                            s.into_optional()
                        } else {
                            s
                        }
                    }),
                    (s("title"), ftd::p2::Kind::caption()),
                ])
                .collect(),
                properties: std::iter::IntoIterator::into_iter([(
                    s("padding"),
                    ftd::component::Property {
                        default: Some(ftd::PropertyValue::Value {
                            value: ftd::Value::Integer { value: 30 },
                        }),
                        conditions: vec![],
                        ..Default::default()
                    },
                )])
                .collect(),
                kernel: false,
                instructions: vec![
                    ftd::Instruction::ChildComponent {
                        child: ftd::ChildComponent {
                            events: vec![],
                            condition: None,
                            root: s("ftd#text"),
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("text"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Variable {
                                            name: s("title"),
                                            kind: ftd::p2::Kind::caption_or_body(),
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("align"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::Value::String {
                                                source: ftd::TextSource::Header,
                                                text: s("center"),
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::Instruction::ChildComponent {
                        child: ftd::ChildComponent {
                            events: vec![],
                            condition: if about_optional {
                                Some(ftd::p2::Boolean::IsNotNull {
                                    value: ftd::PropertyValue::Variable {
                                        name: s("about"),
                                        kind: ftd::p2::Kind::body().into_optional(),
                                    },
                                })
                            } else {
                                None
                            },
                            root: s("ftd#text"),
                            properties: std::iter::IntoIterator::into_iter([(
                                s("text"),
                                ftd::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: s("about"),
                                        kind: ftd::p2::Kind::caption_or_body(),
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::Instruction::ChildComponent {
                        child: ftd::ChildComponent {
                            events: vec![],
                            condition: if about_optional {
                                Some(ftd::p2::Boolean::IsNotNull {
                                    value: ftd::PropertyValue::Variable {
                                        name: s("src"),
                                        kind: ftd::p2::Kind::Record {
                                            name: s("ftd#image-src"),
                                            default: None,
                                            is_reference: false,
                                        }
                                        .into_optional(),
                                    },
                                })
                            } else {
                                None
                            },
                            root: s("ftd#image"),
                            properties: std::iter::IntoIterator::into_iter([(
                                s("src"),
                                ftd::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: s("src"),
                                        kind: ftd::p2::Kind::Record {
                                            name: s("ftd#image-src"),
                                            default: None,
                                            is_reference: false,
                                        },
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                ],
                ..Default::default()
            }),
        );
        bag
    }

    #[test]
    fn components() {
        let title = ftd::Markups {
            text: ftd::markup_line("What kind of documentation?"),
            line: true,
            common: ftd::Common {
                position: Some(ftd::Position::Center),
                reference: Some(s("foo/bar#title@0")),
                ..Default::default()
            },
            ..Default::default()
        };
        let about = ftd::Markups {
            text: ftd::markup_line(
                indoc::indoc!(
                    "
                    UI screens, behaviour and journeys, database tables, APIs, how to
                    contribute to, deploy, or monitor microservice, everything that
                    makes web or mobile product teams productive.
                    "
                )
                .trim(),
            ),
            common: ftd::Common {
                reference: Some(s("foo/bar#about@0")),
                ..Default::default()
            },
            line: true,
            ..Default::default()
        };

        let image = ftd::Image {
            src: i(
                "/static/home/document-type-min.png",
                Some(s("foo/bar#src@0")),
            ),
            common: ftd::Common {
                reference: Some(s("foo/bar#src@0")),
                ..Default::default()
            },
            ..Default::default()
        };

        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                common: ftd::Common {
                    padding: Some(30),
                    ..Default::default()
                },
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(title),
                        ftd::Element::Markup(about),
                        ftd::Element::Image(image),
                    ],
                    ..Default::default()
                },
            }));

        let mut bag = white_two_image_bag(false);
        bag.insert(
            "foo/bar#about@0".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "about".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("UI screens, behaviour and journeys, database tables, APIs, how to\ncontribute to, deploy, or monitor microservice, everything that\nmakes web or mobile product teams productive."),
                        source: ftd::TextSource::Body,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#src0".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "src0".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("ftd#image-src"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("dark"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "/static/home/document-type-min.png".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("light"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "/static/home/document-type-min.png".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#src@0".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "src".to_string(),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#src0"),
                    kind: ftd::p2::Kind::Record {
                        name: s("ftd#image-src"),
                        default: None,
                        is_reference: false,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#title@0".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "title".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("What kind of documentation?"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );

        p!(
            "
            -- ftd.image-src src0: 
            light: /static/home/document-type-min.png
            dark: /static/home/document-type-min.png

            -- ftd.column white-two-image:
            caption title:
            body about:
            ftd.image-src src:
            padding: 30

            --- ftd.text:
            text: $title
            align: center

            --- ftd.text:
            text: $about

            --- ftd.image:
            src: $src

            -- white-two-image: What kind of documentation?
            src: $src0

            UI screens, behaviour and journeys, database tables, APIs, how to
            contribute to, deploy, or monitor microservice, everything that
            makes web or mobile product teams productive.
            ",
            (bag, main),
        );
    }

    #[test]
    fn conditional_body() {
        let title = ftd::Markups {
            text: ftd::markup_line("What kind of documentation?"),
            common: ftd::Common {
                position: Some(ftd::Position::Center),
                reference: Some(s("foo/bar#title@0")),
                ..Default::default()
            },
            line: true,
            ..Default::default()
        };
        let second_title = ftd::Markups {
            text: ftd::markup_line("second call"),
            common: ftd::Common {
                position: Some(ftd::Position::Center),
                reference: Some(s("foo/bar#title@1")),
                ..Default::default()
            },
            line: true,
            ..Default::default()
        };
        let about = ftd::Markups {
            text: ftd::markup_line(
                indoc::indoc!(
                    "
                    UI screens, behaviour and journeys, database tables, APIs, how to
                    contribute to, deploy, or monitor microservice, everything that
                    makes web or mobile product teams productive.
                    "
                )
                .trim(),
            ),
            common: ftd::Common {
                reference: Some(s("foo/bar#about@0")),
                condition: Some(ftd::Condition {
                    variable: s("foo/bar#about@0"),
                    value: serde_json::Value::String(s("$IsNotNull$")),
                }),
                ..Default::default()
            },
            line: true,
            ..Default::default()
        };
        let second_about = ftd::Markups {
            text: ftd::markup_line(""),
            common: ftd::Common {
                reference: Some(s("foo/bar#about@1")),
                condition: Some(ftd::Condition {
                    variable: s("foo/bar#about@1"),
                    value: serde_json::Value::String(s("$IsNotNull$")),
                }),
                is_not_visible: true,
                ..Default::default()
            },
            line: true,
            ..Default::default()
        };
        let image = ftd::Image {
            src: i(
                "/static/home/document-type-min.png",
                Some(s("foo/bar#src@0")),
            ),
            common: ftd::Common {
                reference: Some(s("foo/bar#src@0")),
                condition: Some(ftd::Condition {
                    variable: s("foo/bar#src@0"),
                    value: serde_json::Value::String(s("$IsNotNull$")),
                }),
                ..Default::default()
            },
            ..Default::default()
        };
        let second_image = ftd::Image {
            src: i("second-image.png", Some(s("foo/bar#src@1"))),
            common: ftd::Common {
                reference: Some(s("foo/bar#src@1")),
                condition: Some(ftd::Condition {
                    variable: s("foo/bar#src@1"),
                    value: serde_json::Value::String(s("$IsNotNull$")),
                }),
                ..Default::default()
            },
            ..Default::default()
        };

        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                common: ftd::Common {
                    padding: Some(30),
                    ..Default::default()
                },
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(title),
                        ftd::Element::Markup(about),
                        ftd::Element::Image(image),
                    ],
                    ..Default::default()
                },
            }));
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                common: ftd::Common {
                    padding: Some(30),
                    ..Default::default()
                },
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(second_title),
                        ftd::Element::Markup(second_about),
                        ftd::Element::Image(second_image),
                    ],
                    ..Default::default()
                },
            }));

        let mut bag = white_two_image_bag(true);
        bag.insert(
            "foo/bar#src0".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "src0".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("ftd#image-src"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("dark"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "/static/home/document-type-min.png".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("light"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "/static/home/document-type-min.png".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#src1".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "src1".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("ftd#image-src"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("dark"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "second-image.png".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("light"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "second-image.png".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#about@0".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "about".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("UI screens, behaviour and journeys, database tables, APIs, how to\ncontribute to, deploy, or monitor microservice, everything that\nmakes web or mobile product teams productive."),
                        source: ftd::TextSource::Body,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#about@1".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "about".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Optional {
                        data: Box::new(None),
                        kind: ftd::p2::Kind::body(),
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#src@0".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "src".to_string(),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#src0"),
                    kind: ftd::p2::Kind::Optional {
                        kind: Box::new(ftd::p2::Kind::Record {
                            name: s("ftd#image-src"),
                            default: None,
                            is_reference: false,
                        }),
                        is_reference: false,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#src@1".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "src".to_string(),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#src1"),
                    kind: ftd::p2::Kind::Optional {
                        kind: Box::new(ftd::p2::Kind::Record {
                            name: s("ftd#image-src"),
                            default: None,
                            is_reference: false,
                        }),
                        is_reference: false,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#title@0".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "title".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("What kind of documentation?"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#title@1".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "title".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("second call"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );

        p!(
            "
            -- ftd.image-src src0: 
            light: /static/home/document-type-min.png
            dark: /static/home/document-type-min.png

            -- ftd.image-src src1: 
            light: second-image.png
            dark: second-image.png
            
            -- ftd.column white-two-image:
            caption title:
            optional body about:
            optional ftd.image-src src:
            padding: 30

            --- ftd.text:
            text: $title
            align: center

            --- ftd.text:
            if: $about is not null
            text: $about

            --- ftd.image:
            if: $src is not null
            src: $src

            -- white-two-image: What kind of documentation?
            src: $src0

            UI screens, behaviour and journeys, database tables, APIs, how to
            contribute to, deploy, or monitor microservice, everything that
            makes web or mobile product teams productive.

            -- white-two-image: second call
            src: $src1
            ",
            (bag, main),
        );
    }

    #[test]
    fn conditional_header() {
        let title = ftd::Markups {
            text: ftd::markup_line("What kind of documentation?"),
            common: ftd::Common {
                position: Some(ftd::Position::Center),
                reference: Some(s("foo/bar#title@0")),
                ..Default::default()
            },
            line: true,
            ..Default::default()
        };
        let second_title = ftd::Markups {
            text: ftd::markup_line("second call"),
            common: ftd::Common {
                position: Some(ftd::Position::Center),
                reference: Some(s("foo/bar#title@1")),
                ..Default::default()
            },
            line: true,
            ..Default::default()
        };
        let third_title = ftd::Markups {
            text: ftd::markup_line("third call"),
            common: ftd::Common {
                position: Some(ftd::Position::Center),
                reference: Some(s("foo/bar#title@2")),
                ..Default::default()
            },
            line: true,
            ..Default::default()
        };
        let about = ftd::Markups {
            text: ftd::markup_line(
                indoc::indoc!(
                    "
                    UI screens, behaviour and journeys, database tables, APIs, how to
                    contribute to, deploy, or monitor microservice, everything that
                    makes web or mobile product teams productive.
                    "
                )
                .trim(),
            ),
            common: ftd::Common {
                condition: Some(ftd::Condition {
                    variable: s("foo/bar#about@0"),
                    value: serde_json::Value::String(s("$IsNotNull$")),
                }),
                reference: Some(s("foo/bar#about@0")),
                ..Default::default()
            },
            line: true,
            ..Default::default()
        };
        let second_about = ftd::Markups {
            text: ftd::markup_line(""),
            common: ftd::Common {
                condition: Some(ftd::Condition {
                    variable: s("foo/bar#about@1"),
                    value: serde_json::Value::String(s("$IsNotNull$")),
                }),
                reference: Some(s("foo/bar#about@1")),
                is_not_visible: true,
                ..Default::default()
            },
            line: true,
            ..Default::default()
        };
        let third_about = ftd::Markups {
            text: ftd::markup_line(""),
            common: ftd::Common {
                condition: Some(ftd::Condition {
                    variable: s("foo/bar#about@2"),
                    value: serde_json::Value::String(s("$IsNotNull$")),
                }),
                reference: Some(s("foo/bar#about@2")),
                is_not_visible: true,
                ..Default::default()
            },
            line: true,
            ..Default::default()
        };
        let image = ftd::Image {
            src: i(
                "/static/home/document-type-min.png",
                Some(s("foo/bar#src@0")),
            ),
            common: ftd::Common {
                reference: Some(s("foo/bar#src@0")),
                condition: Some(ftd::Condition {
                    variable: s("foo/bar#src@0"),
                    value: serde_json::Value::String(s("$IsNotNull$")),
                }),
                ..Default::default()
            },
            ..Default::default()
        };
        let second_image = ftd::Image {
            src: i("second-image.png", Some(s("foo/bar#src@1"))),
            common: ftd::Common {
                reference: Some(s("foo/bar#src@1")),
                condition: Some(ftd::Condition {
                    variable: s("foo/bar#src@1"),
                    value: serde_json::Value::String(s("$IsNotNull$")),
                }),
                ..Default::default()
            },
            ..Default::default()
        };
        let third_image = ftd::Image {
            src: i("", Some(s("foo/bar#src@2"))),
            common: ftd::Common {
                reference: Some(s("foo/bar#src@2")),
                condition: Some(ftd::Condition {
                    variable: s("foo/bar#src@2"),
                    value: serde_json::Value::String(s("$IsNotNull$")),
                }),
                is_not_visible: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                common: ftd::Common {
                    padding: Some(30),
                    ..Default::default()
                },
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(title),
                        ftd::Element::Markup(about),
                        ftd::Element::Image(image),
                    ],
                    ..Default::default()
                },
            }));
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                common: ftd::Common {
                    padding: Some(30),
                    ..Default::default()
                },
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(second_title),
                        ftd::Element::Markup(second_about),
                        ftd::Element::Image(second_image),
                    ],
                    ..Default::default()
                },
            }));
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                common: ftd::Common {
                    padding: Some(30),
                    ..Default::default()
                },
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(third_title),
                        ftd::Element::Markup(third_about),
                        ftd::Element::Image(third_image),
                    ],
                    ..Default::default()
                },
            }));

        let mut bag = white_two_image_bag(true);
        bag.insert(
            "foo/bar#src0".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "src0".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("ftd#image-src"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("dark"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "/static/home/document-type-min.png".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("light"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "/static/home/document-type-min.png".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#src1".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "src1".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("ftd#image-src"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("dark"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "second-image.png".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("light"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "second-image.png".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#about@0".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "about".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("UI screens, behaviour and journeys, database tables, APIs, how to\ncontribute to, deploy, or monitor microservice, everything that\nmakes web or mobile product teams productive."),
                        source: ftd::TextSource::Body,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#about@1".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "about".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Optional {
                        data: Box::new(None),
                        kind: ftd::p2::Kind::body(),
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#about@2".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "about".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Optional {
                        data: Box::new(None),
                        kind: ftd::p2::Kind::body(),
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#src@0".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "src".to_string(),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#src0"),
                    kind: ftd::p2::Kind::Optional {
                        kind: Box::new(ftd::p2::Kind::Record {
                            name: s("ftd#image-src"),
                            default: None,
                            is_reference: false,
                        }),
                        is_reference: false,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#src@1".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "src".to_string(),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#src1"),
                    kind: ftd::p2::Kind::Optional {
                        kind: Box::new(ftd::p2::Kind::Record {
                            name: s("ftd#image-src"),
                            default: None,
                            is_reference: false,
                        }),
                        is_reference: false,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#src@2".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "src".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Optional {
                        data: Box::new(None),
                        kind: ftd::p2::Kind::Record {
                            name: s("ftd#image-src"),
                            default: None,
                            is_reference: false,
                        },
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#title@0".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "title".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("What kind of documentation?"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#title@1".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "title".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("second call"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#title@2".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "title".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("third call"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );
        p!(
            "
            -- ftd.image-src src0: 
            light: /static/home/document-type-min.png
            dark: /static/home/document-type-min.png

            -- ftd.image-src src1: 
            light: second-image.png
            dark: second-image.png

            -- ftd.column white-two-image:
            caption title:
            optional body about:
            optional ftd.image-src src:
            padding: 30

            --- ftd.text:
            text: $title
            align: center

            --- ftd.text:
            if: $about is not null
            text: $about

            --- ftd.image:
            if: $src is not null
            src: $src

            -- white-two-image: What kind of documentation?
            src: $src0

            UI screens, behaviour and journeys, database tables, APIs, how to
            contribute to, deploy, or monitor microservice, everything that
            makes web or mobile product teams productive.

            -- white-two-image: second call
            src: $src1

            -- white-two-image: third call
            ",
            (bag, main),
        );
    }

    #[test]
    fn markdown() {
        let mut bag = super::default_bag();
        bag.insert(
            s("fifthtry/ft#markdown"),
            ftd::p2::Thing::Component(ftd::Component {
                invocations: Default::default(),
                full_name: "fifthtry/ft#markdown".to_string(),
                root: s("ftd#text"),
                arguments: std::iter::IntoIterator::into_iter([(s("body"), ftd::p2::Kind::body())])
                    .collect(),
                properties: std::iter::IntoIterator::into_iter([(
                    s("text"),
                    ftd::component::Property {
                        default: Some(ftd::PropertyValue::Variable {
                            name: s("body"),
                            kind: ftd::p2::Kind::string().string_any(),
                        }),
                        conditions: vec![],
                        ..Default::default()
                    },
                )])
                .collect(),
                ..Default::default()
            }),
        );
        bag.insert(
            s("fifthtry/ft#dark-mode"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("dark-mode"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Boolean { value: true },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("fifthtry/ft#toc"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("toc"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "not set".to_string(),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#h0"),
            ftd::p2::Thing::Component(ftd::Component {
                invocations: Default::default(),
                full_name: "foo/bar#h0".to_string(),
                root: s("ftd#column"),
                arguments: std::iter::IntoIterator::into_iter([
                    (s("body"), ftd::p2::Kind::body().into_optional()),
                    (s("title"), ftd::p2::Kind::caption()),
                ])
                .collect(),
                instructions: vec![
                    ftd::Instruction::ChildComponent {
                        child: ftd::ChildComponent {
                            events: vec![],
                            condition: None,
                            root: s("ftd#text"),
                            properties: std::iter::IntoIterator::into_iter([(
                                s("text"),
                                ftd::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: s("title"),
                                        kind: ftd::p2::Kind::caption_or_body(),
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::Instruction::ChildComponent {
                        child: ftd::ChildComponent {
                            events: vec![],
                            condition: Some(ftd::p2::Boolean::IsNotNull {
                                value: ftd::PropertyValue::Variable {
                                    name: s("body"),
                                    kind: ftd::p2::Kind::body().into_optional(),
                                },
                            }),
                            root: s("fifthtry/ft#markdown"),
                            properties: std::iter::IntoIterator::into_iter([(
                                s("body"),
                                ftd::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: s("body"),
                                        kind: ftd::p2::Kind::body(),
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                ],
                ..Default::default()
            }),
        );
        bag.insert(
            s("foo/bar#body@0"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("body"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "what about the body?".to_string(),
                        source: ftd::TextSource::Body,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#body@0,1"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("body"),
                value: ftd::PropertyValue::Variable {
                    name: "foo/bar#body@0".to_string(),
                    kind: ftd::p2::Kind::body(),
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#body@1"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("body"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Optional {
                        data: Box::new(None),
                        kind: ftd::p2::Kind::body(),
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#body@1,1"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("body"),
                value: ftd::PropertyValue::Variable {
                    name: "foo/bar#body@1".to_string(),
                    kind: ftd::p2::Kind::body(),
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#title@0"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("title"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "hello".to_string(),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#title@1"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("title"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "heading without body".to_string(),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );

        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("hello"),
                            line: true,
                            common: ftd::Common {
                                reference: Some(s("foo/bar#title@0")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("what about the body?"),
                            line: true,
                            common: ftd::Common {
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#body@0"),
                                    value: serde_json::Value::String(s("$IsNotNull$")),
                                }),
                                reference: Some(s("foo/bar#body@0,1")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("heading without body"),
                            line: true,
                            common: ftd::Common {
                                reference: Some(s("foo/bar#title@1")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line(""),
                            line: true,
                            common: ftd::Common {
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#body@1"),
                                    value: serde_json::Value::String(s("$IsNotNull$")),
                                }),
                                reference: Some(s("foo/bar#body@1,1")),
                                is_not_visible: true,
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        p!(
            "
            -- import: fifthtry/ft

            -- ftd.column h0:
            caption title:
            optional body body:

            --- ftd.text:
            text: $title

            --- ft.markdown:
            if: $body is not null
            body: $body

            -- h0: hello

            what about the body?

            -- h0: heading without body
            ",
            (bag, main),
        );
    }

    #[test]
    fn width() {
        let mut bag = super::default_bag();

        bag.insert(
            s("foo/bar#src@0"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("src"),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#src0"),
                    kind: ftd::p2::Kind::Record {
                        name: s("ftd#image-src"),
                        default: None,
                        is_reference: false,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#src@1"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("src"),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#src1"),
                    kind: ftd::p2::Kind::Record {
                        name: s("ftd#image-src"),
                        default: None,
                        is_reference: false,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#width@0"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("width"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Optional {
                        data: Box::new(None),
                        kind: ftd::p2::Kind::string(),
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#width@1"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("width"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "300".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#src0".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "src0".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("ftd#image-src"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("dark"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "foo.png".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("light"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "foo.png".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#src1".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "src1".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("ftd#image-src"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("dark"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "bar.png".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("light"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "bar.png".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            s("foo/bar#image"),
            ftd::p2::Thing::Component(ftd::Component {
                invocations: Default::default(),
                full_name: "foo/bar#image".to_string(),
                root: s("ftd#column"),
                arguments: std::iter::IntoIterator::into_iter([
                    (s("width"), ftd::p2::Kind::string().into_optional()),
                    (
                        s("src"),
                        ftd::p2::Kind::Record {
                            name: s("ftd#image-src"),
                            default: None,
                            is_reference: false,
                        },
                    ),
                ])
                .collect(),
                instructions: vec![ftd::Instruction::ChildComponent {
                    child: ftd::ChildComponent {
                        events: vec![],
                        condition: None,
                        root: s("ftd#image"),
                        properties: std::iter::IntoIterator::into_iter([
                            (
                                s("src"),
                                ftd::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: s("src"),
                                        kind: ftd::p2::Kind::Record {
                                            name: s("ftd#image-src"),
                                            default: None,
                                            is_reference: false,
                                        },
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            ),
                            (
                                s("width"),
                                ftd::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: s("width"),
                                        kind: ftd::p2::Kind::string().into_optional(),
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            ),
                        ])
                        .collect(),
                        ..Default::default()
                    },
                }],
                ..Default::default()
            }),
        );

        let mut main = super::default_column();

        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Image(ftd::Image {
                        src: i("foo.png", Some(s("foo/bar#src@0"))),
                        common: ftd::Common {
                            reference: Some(s("foo/bar#src@0")),
                            ..Default::default()
                        },
                        ..Default::default()
                    })],
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Image(ftd::Image {
                        src: i("bar.png", Some(s("foo/bar#src@1"))),
                        common: ftd::Common {
                            reference: Some(s("foo/bar#src@1")),
                            width: Some(ftd::Length::Px { value: 300 }),
                            ..Default::default()
                        },
                        ..Default::default()
                    })],
                    ..Default::default()
                },
                ..Default::default()
            }));

        p!(
            "
            -- ftd.image-src src0: 
            light: foo.png
            dark: foo.png

            -- ftd.image-src src1: 
            light: bar.png
            dark: bar.png

            -- ftd.column image:
            ftd.image-src src:
            optional string width:

            --- ftd.image:
            src: $src
            width: $width

            -- image:
            src: $src0

            -- image:
            src: $src1
            width: 300
            ",
            (bag, main),
        );
    }

    #[test]
    fn decimal() {
        let mut bag = super::default_bag();

        bag.insert(
            "foo/bar#foo".to_string(),
            ftd::p2::Thing::Component(ftd::Component {
                full_name: s("foo/bar#foo"),
                root: "ftd#row".to_string(),
                instructions: vec![
                    ftd::Instruction::ChildComponent {
                        child: ftd::ChildComponent {
                            events: vec![],
                            condition: None,
                            root: s("ftd#decimal"),
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("value"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::Value::Decimal { value: 0.06 },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("format"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::Value::String {
                                                text: s(".1f"),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::Instruction::ChildComponent {
                        child: ftd::ChildComponent {
                            events: vec![],
                            condition: None,
                            root: s("ftd#decimal"),
                            properties: std::iter::IntoIterator::into_iter([(
                                s("value"),
                                ftd::component::Property {
                                    default: Some(ftd::PropertyValue::Value {
                                        value: ftd::Value::Decimal { value: 0.01 },
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                ],
                arguments: std::iter::IntoIterator::into_iter([(s("x"), ftd::p2::Kind::integer())])
                    .collect(),
                ..Default::default()
            }),
        );
        bag.insert(
            "foo/bar#x@0".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "x".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 10 },
                },
                conditions: vec![],
            }),
        );

        let mut main = super::default_column();
        let mut row: ftd::Row = Default::default();
        row.container
            .children
            .push(ftd::Element::Decimal(ftd::Text {
                text: ftd::markdown_line("0.1"),
                line: false,
                ..Default::default()
            }));
        row.container
            .children
            .push(ftd::Element::Decimal(ftd::Text {
                text: ftd::markdown_line("0.01"),
                line: false,
                ..Default::default()
            }));
        main.container.children.push(ftd::Element::Row(row));

        p!(
            "
            -- ftd.row foo:
            integer x:

            --- ftd.decimal:
            value: 0.06
            format: .1f

            --- ftd.decimal:
            value: 0.01

            -- foo:
            x: 10
        ",
            (bag, main),
        );
    }

    #[test]
    fn integer() {
        let mut bag = super::default_bag();

        bag.insert(
            "foo/bar#x@0".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "x".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 10 },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#foo".to_string(),
            ftd::p2::Thing::Component(ftd::Component {
                full_name: s("foo/bar#foo"),
                root: "ftd#row".to_string(),
                instructions: vec![
                    ftd::Instruction::ChildComponent {
                        child: ftd::ChildComponent {
                            events: vec![],
                            condition: None,
                            root: s("ftd#integer"),
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("value"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::Value::Integer { value: 3 },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("format"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::Value::String {
                                                text: s("b"),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::Instruction::ChildComponent {
                        child: ftd::ChildComponent {
                            events: vec![],
                            condition: None,
                            root: s("ftd#integer"),
                            properties: std::iter::IntoIterator::into_iter([(
                                s("value"),
                                ftd::component::Property {
                                    default: Some(ftd::PropertyValue::Value {
                                        value: ftd::Value::Integer { value: 14 },
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                ],
                arguments: std::iter::IntoIterator::into_iter([(s("x"), ftd::p2::Kind::integer())])
                    .collect(),
                ..Default::default()
            }),
        );

        let mut main = super::default_column();
        let mut row: ftd::Row = Default::default();
        row.container
            .children
            .push(ftd::Element::Integer(ftd::Text {
                text: ftd::markdown_line("11"),
                line: false,
                ..Default::default()
            }));
        row.container
            .children
            .push(ftd::Element::Integer(ftd::Text {
                text: ftd::markdown_line("14"),
                line: false,
                ..Default::default()
            }));

        main.container.children.push(ftd::Element::Row(row));

        p!(
            "
            -- ftd.row foo:
            integer x:

            --- ftd.integer:
            value: 3
            format: b

            --- ftd.integer:
            value: 14

            -- foo:
            x: 10
        ",
            (bag, main),
        );
    }

    #[test]
    fn boolean() {
        let mut bag = super::default_bag();

        bag.insert(
            "foo/bar#foo".to_string(),
            ftd::p2::Thing::Component(ftd::Component {
                full_name: s("foo/bar#foo"),
                root: "ftd#row".to_string(),
                instructions: vec![
                    ftd::Instruction::ChildComponent {
                        child: ftd::ChildComponent {
                            events: vec![],
                            condition: None,
                            root: s("ftd#boolean"),
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("value"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::Value::Boolean { value: true },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("true"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::Value::String {
                                                text: s("show this when value is true"),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("false"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::Value::String {
                                                text: s("show this when value is false"),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::Instruction::ChildComponent {
                        child: ftd::ChildComponent {
                            events: vec![],
                            condition: None,
                            root: s("ftd#boolean"),
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("value"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::Value::Boolean { value: false },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("true"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::Value::String {
                                                text: s("show this when value is true"),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("false"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::Value::String {
                                                text: s("show this when value is false"),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                ],
                arguments: std::iter::IntoIterator::into_iter([(s("x"), ftd::p2::Kind::integer())])
                    .collect(),
                ..Default::default()
            }),
        );
        bag.insert(
            "foo/bar#x@0".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "x".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 10 },
                },
                conditions: vec![],
            }),
        );

        let mut main = super::default_column();
        let mut row: ftd::Row = Default::default();
        row.container
            .children
            .push(ftd::Element::Boolean(ftd::Text {
                text: ftd::markdown_line("show this when value is true"),
                line: false,
                ..Default::default()
            }));
        row.container
            .children
            .push(ftd::Element::Boolean(ftd::Text {
                text: ftd::markdown_line("show this when value is false"),
                line: false,
                ..Default::default()
            }));
        main.container.children.push(ftd::Element::Row(row));

        p!(
            "
            -- ftd.row foo:
            integer x:

            --- ftd.boolean:
            value: true
            true:  show this when value is true
            false: show this when value is false

            --- ftd.boolean:
            value: false
            true:  show this when value is true
            false: show this when value is false

            -- foo:
            x: 10
        ",
            (bag, main),
        );
    }

    #[test]
    fn boolean_expression() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("present is true"),
                line: true,
                common: ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: "foo/bar#present".to_string(),
                        value: serde_json::Value::Bool(true),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("present is false"),
                line: true,
                common: ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: "foo/bar#present".to_string(),
                        value: serde_json::Value::Bool(false),
                    }),
                    is_not_visible: true,
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("dark-mode is true"),
                line: true,
                common: ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: "fifthtry/ft#dark-mode".to_string(),
                        value: serde_json::Value::Bool(true),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("dark-mode is false"),
                line: true,
                common: ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: "fifthtry/ft#dark-mode".to_string(),
                        value: serde_json::Value::Bool(false),
                    }),
                    is_not_visible: true,
                    ..Default::default()
                },
                ..Default::default()
            }));

        let mut column: ftd::Column = Default::default();
        column
            .container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("inner present false"),
                line: true,
                common: ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: "foo/bar#present".to_string(),
                        value: serde_json::Value::Bool(false),
                    }),
                    is_not_visible: true,
                    ..Default::default()
                },
                ..Default::default()
            }));

        column
            .container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("inner present true"),
                line: true,
                common: ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: "foo/bar#present".to_string(),
                        value: serde_json::Value::Bool(true),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container.children.push(ftd::Element::Column(column));

        let mut column: ftd::Column = Default::default();
        column
            .container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("argument present false"),
                line: true,
                common: ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#present@5"),
                        value: serde_json::Value::Bool(false),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }));
        column
            .container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("argument present true"),
                line: true,
                common: ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#present@5"),
                        value: serde_json::Value::Bool(true),
                    }),
                    is_not_visible: true,
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container.children.push(ftd::Element::Column(column));

        let mut column: ftd::Column = Default::default();
        column
            .container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("argument present false"),
                line: true,
                common: ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#present@6"),
                        value: serde_json::Value::Bool(false),
                    }),
                    is_not_visible: true,
                    ..Default::default()
                },
                ..Default::default()
            }));
        column
            .container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("argument present true"),
                line: true,
                common: ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#present@6"),
                        value: serde_json::Value::Bool(true),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container.children.push(ftd::Element::Column(column));

        let mut column: ftd::Column = Default::default();
        column
            .container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("foo2 dark-mode is true"),
                line: true,
                common: ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: "fifthtry/ft#dark-mode".to_string(),
                        value: serde_json::Value::Bool(true),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }));

        column
            .container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("foo2 dark-mode is false"),
                line: true,
                common: ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: "fifthtry/ft#dark-mode".to_string(),
                        value: serde_json::Value::Bool(false),
                    }),
                    is_not_visible: true,
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container.children.push(ftd::Element::Column(column));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("hello literal truth"),
                line: true,
                ..Default::default()
            }));

        main.container.children.push(ftd::Element::Null);

        p!(
            "
            -- import: fifthtry/ft
            -- boolean present: true

            -- ftd.text: present is true
            if: $present

            -- ftd.text: present is false
            if: not $present

            -- ftd.text: dark-mode is true
            if: $ft.dark-mode

            -- ftd.text: dark-mode is false
            if: not $ft.dark-mode

            -- ftd.column foo:

            --- ftd.text: inner present false
            if: not $present

            --- ftd.text: inner present true
            if: $present

            -- foo:

            -- ftd.column bar:
            boolean present:

            --- ftd.text: argument present false
            if: not $present

            --- ftd.text: argument present true
            if: $present

            -- bar:
            present: false

            -- bar:
            present: $ft.dark-mode

            -- ftd.column foo2:

            --- ftd.text: foo2 dark-mode is true
            if: $ft.dark-mode

            --- ftd.text: foo2 dark-mode is false
            if: not $ft.dark-mode

            -- foo2:

            -- ftd.text: hello literal truth
            if: true

            -- ftd.text: never see light of the day
            if: false
        ",
            (Default::default(), main),
        );
    }

    #[test]
    fn inner_container() {
        let mut bag = super::default_bag();

        bag.insert(
            "foo/bar#foo".to_string(),
            ftd::p2::Thing::Component(ftd::Component {
                root: "ftd#column".to_string(),
                full_name: "foo/bar#foo".to_string(),
                instructions: vec![
                    ftd::component::Instruction::ChildComponent {
                        child: ftd::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#row".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("id"),
                                ftd::component::Property {
                                    default: Some(ftd::PropertyValue::Value {
                                        value: ftd::variable::Value::String {
                                            text: "r1".to_string(),
                                            source: ftd::TextSource::Header,
                                        },
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::component::Instruction::ChildComponent {
                        child: ftd::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#row".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("id"),
                                ftd::component::Property {
                                    default: Some(ftd::PropertyValue::Value {
                                        value: ftd::variable::Value::String {
                                            text: "r2".to_string(),
                                            source: ftd::TextSource::Header,
                                        },
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                ],
                ..Default::default()
            }),
        );
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Row(ftd::Row {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![
                                ftd::Element::Row(ftd::Row {
                                    spacing: None,
                                    common: ftd::Common {
                                        data_id: Some(s("r2")),
                                        id: Some(s("foo-1:r2")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                                ftd::Element::Markup(ftd::Markups {
                                    text: ftd::markdown_line("hello"),
                                    line: true,
                                    ..Default::default()
                                }),
                            ],
                            ..Default::default()
                        },
                        common: ftd::Common {
                            data_id: Some(s("r1")),
                            id: Some(s("foo-1:r1")),
                            ..Default::default()
                        },
                    })],
                    ..Default::default()
                },
                common: ftd::Common {
                    data_id: Some(s("foo-1")),
                    id: Some(s("foo-1")),
                    ..Default::default()
                },
            }));

        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Row(ftd::Row {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![ftd::Element::Row(ftd::Row {
                                spacing: None,
                                common: ftd::Common {
                                    data_id: Some(s("r2")),
                                    id: Some(s("foo-2:r2")),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })],
                            ..Default::default()
                        },
                        common: ftd::Common {
                            data_id: Some(s("r1")),
                            id: Some(s("foo-2:r1")),
                            ..Default::default()
                        },
                    })],
                    ..Default::default()
                },
                common: ftd::Common {
                    data_id: Some(s("foo-2")),
                    id: Some(s("foo-2")),
                    ..Default::default()
                },
            }));

        p!(
            "
            -- ftd.column foo:

            --- ftd.row:
            id: r1

            --- ftd.row:
            id: r2

            -- foo:
            id: foo-1

            -- foo:
            id: foo-2

            -- container: foo-1.r1

            -- ftd.text: hello
            ",
            (bag, main),
        );
    }

    #[test]
    fn inner_container_using_import() {
        let mut bag = super::default_bag();

        bag.insert(
            "inner_container#foo".to_string(),
            ftd::p2::Thing::Component(ftd::Component {
                root: "ftd#column".to_string(),
                full_name: "inner_container#foo".to_string(),
                instructions: vec![
                    ftd::component::Instruction::ChildComponent {
                        child: ftd::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#row".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("id"),
                                ftd::component::Property {
                                    default: Some(ftd::PropertyValue::Value {
                                        value: ftd::variable::Value::String {
                                            text: "r1".to_string(),
                                            source: ftd::TextSource::Header,
                                        },
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::component::Instruction::ChildComponent {
                        child: ftd::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#row".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("id"),
                                ftd::component::Property {
                                    default: Some(ftd::PropertyValue::Value {
                                        value: ftd::variable::Value::String {
                                            text: "r2".to_string(),
                                            source: ftd::TextSource::Header,
                                        },
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                ],
                ..Default::default()
            }),
        );
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Row(ftd::Row {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![
                                ftd::Element::Row(ftd::Row {
                                    spacing: None,
                                    common: ftd::Common {
                                        data_id: Some(s("r2")),
                                        id: Some(s("foo-1:r2")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                                ftd::Element::Markup(ftd::Markups {
                                    text: ftd::markdown_line("hello"),
                                    line: true,
                                    ..Default::default()
                                }),
                            ],
                            ..Default::default()
                        },
                        common: ftd::Common {
                            data_id: Some(s("r1")),
                            id: Some(s("foo-1:r1")),
                            ..Default::default()
                        },
                    })],
                    ..Default::default()
                },
                common: ftd::Common {
                    data_id: Some(s("foo-1")),
                    id: Some(s("foo-1")),
                    ..Default::default()
                },
            }));

        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Row(ftd::Row {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![ftd::Element::Row(ftd::Row {
                                spacing: None,
                                common: ftd::Common {
                                    data_id: Some(s("r2")),
                                    id: Some(s("foo-2:r2")),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })],
                            ..Default::default()
                        },
                        common: ftd::Common {
                            data_id: Some(s("r1")),
                            id: Some(s("foo-2:r1")),
                            ..Default::default()
                        },
                    })],
                    ..Default::default()
                },
                common: ftd::Common {
                    data_id: Some(s("foo-2")),
                    id: Some(s("foo-2")),
                    ..Default::default()
                },
            }));

        p!(
            "
            -- import: inner_container as ic

            -- ic.foo:
            id: foo-1

            -- ic.foo:
            id: foo-2

            -- container: foo-1.r1

            -- ftd.text: hello
            ",
            (bag, main),
        );
    }

    #[test]
    fn open_container_with_id() {
        let mut external_children = super::default_column();
        external_children.container.children = vec![ftd::Element::Markup(ftd::Markups {
            text: ftd::markdown_line("hello"),
            line: true,
            ..Default::default()
        })];

        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    external_children: Some((
                        s("some-child"),
                        vec![vec![0, 0]],
                        vec![ftd::Element::Column(external_children)],
                    )),
                    children: vec![ftd::Element::Row(ftd::Row {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![ftd::Element::Row(ftd::Row {
                                spacing: None,
                                common: ftd::Common {
                                    data_id: Some(s("some-child")),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })],
                            ..Default::default()
                        },
                        ..Default::default()
                    })],
                    open: Some(true),
                    append_at: Some(s("some-child")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let mut bag = super::default_bag();

        bag.insert(
            "foo/bar#foo".to_string(),
            ftd::p2::Thing::Component(ftd::Component {
                root: "ftd#column".to_string(),
                full_name: s("foo/bar#foo"),
                properties: std::iter::IntoIterator::into_iter([
                    (
                        s("append-at"),
                        ftd::component::Property {
                            default: Some(ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: s("some-child"),
                                    source: ftd::TextSource::Header,
                                },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                    (
                        s("open"),
                        ftd::component::Property {
                            default: Some(ftd::PropertyValue::Value {
                                value: ftd::Value::Boolean { value: true },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                ])
                .collect(),
                instructions: vec![
                    ftd::component::Instruction::ChildComponent {
                        child: ftd::component::ChildComponent {
                            events: vec![],
                            root: "ftd#row".to_string(),
                            condition: None,
                            ..Default::default()
                        },
                    },
                    ftd::component::Instruction::ChildComponent {
                        child: ftd::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#row".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("id"),
                                ftd::component::Property {
                                    default: Some(ftd::PropertyValue::Value {
                                        value: ftd::variable::Value::String {
                                            text: "some-child".to_string(),
                                            source: ftd::TextSource::Header,
                                        },
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                ],
                ..Default::default()
            }),
        );

        p!(
            "
            -- ftd.column foo:
            open: true
            append-at: some-child

            --- ftd.row:

            --- ftd.row:
            id: some-child

            -- foo:

            -- ftd.text: hello
            ",
            (bag, main),
        );
    }

    #[test]
    fn open_container_with_if() {
        let mut external_children = super::default_column();
        external_children.container.children = vec![
            ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("hello"),
                line: true,
                ..Default::default()
            }),
            ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("hello1"),
                line: true,
                ..Default::default()
            }),
        ];

        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("Start Browser"),
                line: true,
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Column(ftd::Column {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![ftd::Element::Column(ftd::Column {
                                spacing: None,
                                container: ftd::Container {
                                    children: vec![
                                        ftd::Element::Column(ftd::Column {
                                            spacing: None,
                                            container: ftd::Container {
                                                children: vec![ftd::Element::Markup(
                                                    ftd::Markups {
                                                        text: ftd::markdown_line("Mobile Display"),
                                                        line: true,
                                                        common: ftd::Common {
                                                            data_id: Some(s("mobile-display")),
                                                            id: Some(s(
                                                                "foo-id:some-child:mobile-display",
                                                            )),
                                                            ..Default::default()
                                                        },
                                                        ..Default::default()
                                                    },
                                                )],
                                                ..Default::default()
                                            },
                                            common: ftd::Common {
                                                condition: Some(ftd::Condition {
                                                    variable: s("foo/bar#mobile"),
                                                    value: serde_json::Value::Bool(true),
                                                }),
                                                data_id: Some(s("some-child")),
                                                id: Some(s("foo-id:some-child")),
                                                ..Default::default()
                                            },
                                        }),
                                        ftd::Element::Column(ftd::Column {
                                            spacing: None,
                                            container: ftd::Container {
                                                children: vec![ftd::Element::Markup(
                                                    ftd::Markups {
                                                        text: ftd::markdown_line("Desktop Display"),
                                                        line: true,
                                                        ..Default::default()
                                                    },
                                                )],
                                                ..Default::default()
                                            },
                                            common: ftd::Common {
                                                condition: Some(ftd::Condition {
                                                    variable: s("foo/bar#mobile"),
                                                    value: serde_json::Value::Bool(false),
                                                }),
                                                is_not_visible: true,
                                                data_id: Some(s("some-child")),
                                                id: Some(s("foo-id:some-child")),
                                                ..Default::default()
                                            },
                                        }),
                                    ],
                                    external_children: Some((
                                        s("some-child"),
                                        vec![vec![0], vec![1]],
                                        vec![ftd::Element::Column(external_children)],
                                    )),
                                    open: Some(true),
                                    append_at: Some(s("some-child")),
                                    ..Default::default()
                                },
                                common: ftd::Common {
                                    id: Some(s("foo-id")),
                                    data_id: Some(s("foo-id")),
                                    ..Default::default()
                                },
                            })],
                            ..Default::default()
                        },
                        common: ftd::Common {
                            data_id: Some(s("c2")),
                            id: Some(s("c2")),
                            ..Default::default()
                        },
                    })],
                    ..Default::default()
                },
                common: ftd::Common {
                    data_id: Some(s("c1")),
                    id: Some(s("c1")),
                    ..Default::default()
                },
            }));

        let mut bag = super::default_bag();
        bag.insert(
            s("foo/bar#desktop-display"),
            ftd::p2::Thing::Component(ftd::Component {
                root: "ftd#column".to_string(),
                full_name: s("foo/bar#desktop-display"),
                arguments: std::iter::IntoIterator::into_iter([(
                    s("id"),
                    ftd::p2::Kind::optional(ftd::p2::Kind::string()),
                )])
                .collect(),
                properties: std::iter::IntoIterator::into_iter([(
                    s("id"),
                    ftd::component::Property {
                        default: Some(ftd::PropertyValue::Variable {
                            name: "id".to_string(),
                            kind: ftd::p2::Kind::Optional {
                                kind: Box::new(ftd::p2::Kind::string()),
                                is_reference: false,
                            },
                        }),
                        conditions: vec![],
                        ..Default::default()
                    },
                )])
                .collect(),
                instructions: vec![ftd::component::Instruction::ChildComponent {
                    child: ftd::component::ChildComponent {
                        is_recursive: false,
                        events: vec![],
                        root: "ftd#text".to_string(),
                        condition: None,
                        properties: std::iter::IntoIterator::into_iter([(
                            s("text"),
                            ftd::component::Property {
                                default: Some(ftd::PropertyValue::Value {
                                    value: ftd::variable::Value::String {
                                        text: s("Desktop Display"),
                                        source: ftd::TextSource::Caption,
                                    },
                                }),
                                conditions: vec![],
                                ..Default::default()
                            },
                        )])
                        .collect(),
                        ..Default::default()
                    },
                }],
                ..Default::default()
            }),
        );

        bag.insert(
            s("foo/bar#foo"),
            ftd::p2::Thing::Component(ftd::Component {
                root: "ftd#column".to_string(),
                full_name: s("foo/bar#foo"),
                properties: std::iter::IntoIterator::into_iter([
                    (
                        s("append-at"),
                        ftd::component::Property {
                            default: Some(ftd::PropertyValue::Value {
                                value: ftd::variable::Value::String {
                                    text: s("some-child"),
                                    source: ftd::TextSource::Header,
                                },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                    (
                        s("open"),
                        ftd::component::Property {
                            default: Some(ftd::PropertyValue::Value {
                                value: ftd::Value::Boolean { value: true },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                ])
                .collect(),
                instructions: vec![
                    ftd::component::Instruction::ChildComponent {
                        child: ftd::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "foo/bar#mobile-display".to_string(),
                            condition: Some(ftd::p2::Boolean::Equal {
                                left: ftd::PropertyValue::Reference {
                                    name: s("foo/bar#mobile"),
                                    kind: ftd::p2::Kind::Boolean {
                                        default: None,
                                        is_reference: false,
                                    },
                                },
                                right: ftd::PropertyValue::Value {
                                    value: ftd::variable::Value::Boolean { value: true },
                                },
                            }),
                            properties: std::iter::IntoIterator::into_iter([(
                                s("id"),
                                ftd::component::Property {
                                    default: Some(ftd::PropertyValue::Value {
                                        value: ftd::variable::Value::String {
                                            text: s("some-child"),
                                            source: ftd::TextSource::Header,
                                        },
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::component::Instruction::ChildComponent {
                        child: ftd::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "foo/bar#desktop-display".to_string(),
                            condition: Some(ftd::p2::Boolean::Equal {
                                left: ftd::PropertyValue::Reference {
                                    name: s("foo/bar#mobile"),
                                    kind: ftd::p2::Kind::Boolean {
                                        default: None,
                                        is_reference: false,
                                    },
                                },
                                right: ftd::PropertyValue::Value {
                                    value: ftd::variable::Value::Boolean { value: false },
                                },
                            }),
                            properties: std::iter::IntoIterator::into_iter([(
                                s("id"),
                                ftd::component::Property {
                                    default: Some(ftd::PropertyValue::Value {
                                        value: ftd::variable::Value::String {
                                            text: s("some-child"),
                                            source: ftd::TextSource::Header,
                                        },
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                ],
                ..Default::default()
            }),
        );

        bag.insert(
            s("foo/bar#mobile"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("mobile"),
                value: ftd::PropertyValue::Value {
                    value: ftd::variable::Value::Boolean { value: true },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            s("foo/bar#mobile-display"),
            ftd::p2::Thing::Component(ftd::Component {
                root: "ftd#column".to_string(),
                full_name: s("foo/bar#mobile-display"),
                arguments: std::iter::IntoIterator::into_iter([(
                    s("id"),
                    ftd::p2::Kind::optional(ftd::p2::Kind::string()),
                )])
                .collect(),
                properties: std::iter::IntoIterator::into_iter([(
                    s("id"),
                    ftd::component::Property {
                        default: Some(ftd::PropertyValue::Variable {
                            name: "id".to_string(),
                            kind: ftd::p2::Kind::Optional {
                                kind: Box::new(ftd::p2::Kind::string()),
                                is_reference: false,
                            },
                        }),
                        conditions: vec![],
                        ..Default::default()
                    },
                )])
                .collect(),
                instructions: vec![ftd::component::Instruction::ChildComponent {
                    child: ftd::component::ChildComponent {
                        is_recursive: false,
                        events: vec![],
                        root: "ftd#text".to_string(),
                        condition: None,
                        properties: std::iter::IntoIterator::into_iter([
                            (
                                s("id"),
                                ftd::component::Property {
                                    default: Some(ftd::PropertyValue::Value {
                                        value: ftd::variable::Value::String {
                                            text: s("mobile-display"),
                                            source: ftd::TextSource::Header,
                                        },
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            ),
                            (
                                s("text"),
                                ftd::component::Property {
                                    default: Some(ftd::PropertyValue::Value {
                                        value: ftd::variable::Value::String {
                                            text: s("Mobile Display"),
                                            source: ftd::TextSource::Caption,
                                        },
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            ),
                        ])
                        .collect(),
                        ..Default::default()
                    },
                }],
                ..Default::default()
            }),
        );

        bag.insert(
            s("foo/bar#id@1,0,0,0"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("id"),
                value: ftd::PropertyValue::Value {
                    value: ftd::variable::Value::String {
                        text: s("some-child"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            s("foo/bar#id@1,0,0,1"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("id"),
                value: ftd::PropertyValue::Value {
                    value: ftd::variable::Value::String {
                        text: s("some-child"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
            }),
        );

        p!(
            "
            -- ftd.column mobile-display:
            optional string id:
            id: $id

            --- ftd.text: Mobile Display
            id: mobile-display

            -- ftd.column desktop-display:
            optional string id:
            id: $id

            --- ftd.text: Desktop Display

            -- boolean mobile: true

            -- ftd.column foo:
            open: true
            append-at: some-child

            --- mobile-display:
            if: $mobile
            id: some-child

            --- desktop-display:
            if: not $mobile
            id: some-child

            -- ftd.text: Start Browser

            -- ftd.column:
            id: c1

            -- ftd.column:
            id: c2

            -- foo:
            id: foo-id

            -- ftd.text: hello

            -- ftd.text: hello1
            ",
            (bag, main),
        );
    }

    #[test]
    fn nested_open_container() {
        let mut external_children = super::default_column();
        external_children.container.children = vec![
            ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("hello"),
                line: true,
                ..Default::default()
            }),
            ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("hello again"),
                line: true,
                ..Default::default()
            }),
        ];

        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Column(ftd::Column {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![
                                ftd::Element::Column(ftd::Column {
                                    spacing: None,
                                    container: ftd::Container {
                                        children: vec![ftd::Element::Column(ftd::Column {
                                            spacing: None,
                                            container: ftd::Container {
                                                children: vec![],
                                                ..Default::default()
                                            },
                                            common: ftd::Common {
                                                data_id: Some(s("desktop-container")),
                                                ..Default::default()
                                            },
                                        })],
                                        external_children: Some((
                                            s("desktop-container"),
                                            vec![vec![0]],
                                            vec![],
                                        )),
                                        open: Some(true),
                                        append_at: Some(s("desktop-container")),
                                        ..Default::default()
                                    },
                                    common: ftd::Common {
                                        condition: Some(ftd::Condition {
                                            variable: s("foo/bar#is-mobile"),
                                            value: serde_json::Value::Bool(false),
                                        }),
                                        is_not_visible: true,
                                        data_id: Some(s("main-container")),
                                        ..Default::default()
                                    },
                                }),
                                ftd::Element::Column(ftd::Column {
                                    spacing: None,
                                    container: ftd::Container {
                                        children: vec![ftd::Element::Column(ftd::Column {
                                            spacing: None,
                                            common: ftd::Common {
                                                data_id: Some(s("mobile-container")),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        })],
                                        external_children: Some((
                                            s("mobile-container"),
                                            vec![vec![0]],
                                            vec![],
                                        )),
                                        open: Some(true),
                                        append_at: Some(s("mobile-container")),
                                        ..Default::default()
                                    },
                                    common: ftd::Common {
                                        condition: Some(ftd::Condition {
                                            variable: s("foo/bar#is-mobile"),
                                            value: serde_json::Value::Bool(true),
                                        }),
                                        data_id: Some(s("main-container")),
                                        ..Default::default()
                                    },
                                }),
                            ],
                            ..Default::default()
                        },
                        common: ftd::Common {
                            data_id: Some(s("start")),
                            ..Default::default()
                        },
                    })],
                    external_children: Some((
                        s("main-container"),
                        vec![vec![0, 0], vec![0, 1]],
                        vec![ftd::Element::Column(external_children)],
                    )),
                    open: Some(true),
                    append_at: Some(s("main-container")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.column desktop:
                open: true
                append-at: desktop-container

                --- ftd.column:
                id: desktop-container

                -- ftd.column mobile:
                open: true
                append-at: mobile-container

                --- ftd.column:
                id: mobile-container

                -- boolean is-mobile: true

                -- ftd.column page:
                open: true
                append-at: main-container

                --- ftd.column:
                id: start

                --- desktop:
                if: not $is-mobile
                id: main-container

                --- container: start

                --- mobile:
                if: $is-mobile
                id: main-container

                -- page:

                -- ftd.text: hello

                -- ftd.text: hello again
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn deep_open_container_call() {
        let mut external_children = super::default_column();
        external_children.container.children = vec![
            ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("hello"),
                line: true,
                ..Default::default()
            }),
            ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("hello again"),
                line: true,
                ..Default::default()
            }),
        ];

        let mut main = super::default_column();

        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![ftd::Element::Column(ftd::Column {
                                    spacing: None,
                                    common: ftd::Common {
                                        data_id: Some(s("foo")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            common: ftd::Common {
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#is-mobile"),
                                    value: serde_json::Value::Bool(false),
                                }),
                                is_not_visible: true,
                                data_id: Some(s("main-container")),
                                ..Default::default()
                            },
                        }),
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![ftd::Element::Column(ftd::Column {
                                    spacing: None,
                                    common: ftd::Common {
                                        data_id: Some(s("foo")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            common: ftd::Common {
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#is-mobile"),
                                    value: serde_json::Value::Bool(true),
                                }),
                                data_id: Some(s("main-container")),
                                ..Default::default()
                            },
                        }),
                    ],
                    external_children: Some((
                        s("foo"),
                        vec![vec![0, 0], vec![1, 0]],
                        vec![ftd::Element::Column(external_children)],
                    )),
                    open: Some(true),
                    append_at: Some(s("main-container.foo")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.column desktop:
                optional string id:
                id: $id

                --- ftd.column:
                id: foo

                -- ftd.column mobile:
                optional string id:
                id: $id

                --- ftd.column:
                id: foo

                -- boolean is-mobile: true

                -- ftd.column page:
                open: true
                append-at: main-container.foo

                --- desktop:
                if: not $is-mobile
                id: main-container

                --- mobile:
                if: $is-mobile
                id: main-container

                -- page:

                -- ftd.text: hello

                -- ftd.text: hello again
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn deep_nested_open_container_call() {
        let mut nested_external_children = super::default_column();
        nested_external_children.container.children = vec![
            ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("hello"),
                line: true,
                ..Default::default()
            }),
            ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("hello again"),
                line: true,
                ..Default::default()
            }),
        ];

        let mut external_children = super::default_column();
        external_children.container.children = vec![ftd::Element::Column(ftd::Column {
            spacing: None,
            container: ftd::Container {
                children: vec![ftd::Element::Row(ftd::Row {
                    spacing: None,
                    container: ftd::Container {
                        children: vec![ftd::Element::Column(ftd::Column {
                            spacing: None,
                            common: ftd::Common {
                                data_id: Some(s("foo")),
                                ..Default::default()
                            },
                            ..Default::default()
                        })],
                        ..Default::default()
                    },
                    common: ftd::Common {
                        data_id: Some(s("desktop-container")),
                        ..Default::default()
                    },
                })],
                external_children: Some((
                    s("desktop-container"),
                    vec![vec![0]],
                    vec![ftd::Element::Column(nested_external_children)],
                )),
                open: Some(true),
                append_at: Some(s("desktop-container")),
                ..Default::default()
            },
            ..Default::default()
        })];

        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![ftd::Element::Row(ftd::Row {
                                    spacing: None,
                                    container: ftd::Container {
                                        children: vec![ftd::Element::Column(ftd::Column {
                                            spacing: None,
                                            common: ftd::Common {
                                                data_id: Some(s("foo")),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        })],
                                        ..Default::default()
                                    },
                                    common: ftd::Common {
                                        data_id: Some(s("desktop-container")),
                                        ..Default::default()
                                    },
                                })],
                                external_children: Some((
                                    s("desktop-container"),
                                    vec![vec![0]],
                                    vec![],
                                )),
                                open: Some(true),
                                append_at: Some(s("desktop-container")),
                                ..Default::default()
                            },
                            common: ftd::Common {
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#is-mobile"),
                                    value: serde_json::Value::Bool(false),
                                }),
                                data_id: Some(s("main-container")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![ftd::Element::Row(ftd::Row {
                                    spacing: None,
                                    container: ftd::Container {
                                        children: vec![ftd::Element::Column(ftd::Column {
                                            spacing: None,
                                            common: ftd::Common {
                                                data_id: Some(s("foo")),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        })],
                                        ..Default::default()
                                    },
                                    common: ftd::Common {
                                        data_id: Some(s("mobile-container")),
                                        ..Default::default()
                                    },
                                })],
                                external_children: Some((
                                    s("mobile-container"),
                                    vec![vec![0]],
                                    vec![],
                                )),
                                open: Some(true),
                                append_at: Some(s("mobile-container")),
                                ..Default::default()
                            },
                            common: ftd::Common {
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#is-mobile"),
                                    value: serde_json::Value::Bool(true),
                                }),
                                is_not_visible: true,
                                data_id: Some(s("main-container")),
                                ..Default::default()
                            },
                        }),
                    ],
                    external_children: Some((
                        s("foo"),
                        vec![vec![0, 0, 0], vec![1, 0, 0]],
                        vec![ftd::Element::Column(external_children)],
                    )),
                    open: Some(true),
                    append_at: Some(s("main-container.foo")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.column ft_container:
                optional string id:
                id: $id

                -- ftd.column ft_container_mobile:
                optional string id:
                id: $id


                -- ftd.column desktop:
                open: true
                append-at: desktop-container
                optional string id:
                id: $id

                --- ftd.row:
                id: desktop-container

                --- ft_container:
                id: foo



                -- ftd.column mobile:
                open: true
                append-at: mobile-container
                optional string id:
                id: $id

                --- ftd.row:
                id: mobile-container

                --- ft_container_mobile:
                id: foo


                -- boolean is-mobile: false


                -- ftd.column page:
                open: true
                append-at: main-container.foo

                --- desktop:
                if: not $is-mobile
                id: main-container

                --- container: ftd.main

                --- mobile:
                if: $is-mobile
                id: main-container



                -- page:

                -- desktop:

                -- ftd.text: hello

                -- ftd.text: hello again

                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    #[ignore]
    fn invalid_deep_open_container() {
        let mut external_children = super::default_column();
        external_children.container.children = vec![
            ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("hello"),
                line: true,
                ..Default::default()
            }),
            ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("hello again"),
                line: true,
                ..Default::default()
            }),
        ];

        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Column(ftd::Column {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![
                                ftd::Element::Column(ftd::Column {
                                    spacing: None,
                                    container: ftd::Container {
                                        children: vec![ftd::Element::Column(ftd::Column {
                                            spacing: None,
                                            container: ftd::Container {
                                                children: vec![],
                                                ..Default::default()
                                            },
                                            common: ftd::Common {
                                                data_id: Some(s("main-container")),
                                                ..Default::default()
                                            },
                                        })],
                                        ..Default::default()
                                    },
                                    common: ftd::Common {
                                        condition: Some(ftd::Condition {
                                            variable: s("foo/bar#is-mobile"),
                                            value: serde_json::Value::Bool(false),
                                        }),
                                        is_not_visible: true,
                                        ..Default::default()
                                    },
                                }),
                                ftd::Element::Column(ftd::Column {
                                    spacing: None,
                                    container: ftd::Container {
                                        children: vec![ftd::Element::Column(ftd::Column {
                                            spacing: None,
                                            common: ftd::Common {
                                                data_id: Some(s("main-container")),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        })],
                                        ..Default::default()
                                    },
                                    common: ftd::Common {
                                        condition: Some(ftd::Condition {
                                            variable: s("foo/bar#is-mobile"),
                                            value: serde_json::Value::Bool(true),
                                        }),
                                        ..Default::default()
                                    },
                                }),
                            ],
                            ..Default::default()
                        },
                        common: ftd::Common {
                            data_id: Some(s("start")),
                            ..Default::default()
                        },
                    })],
                    external_children: Some((
                        s("main-container"),
                        vec![],
                        vec![ftd::Element::Column(external_children)],
                    )),
                    open: Some(true),
                    append_at: Some(s("main-container")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.column desktop:
                optional string id:
                id: $id

                --- ftd.column:
                id: main-container

                -- ftd.column mobile:
                optional string id:
                id: $id

                --- ftd.column:
                id: main-container

                -- boolean is-mobile: true

                -- ftd.column page:
                open: true
                append-at: main-container

                --- ftd.column:
                id: start

                --- desktop:
                if: not $is-mobile

                --- container: start

                --- mobile:
                if: $is-mobile

                -- page:

                -- ftd.text: hello

                -- ftd.text: hello again
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn open_container_id_1() {
        let mut main = self::default_column();
        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            common: ftd::Common {
                data_id: Some(s("r1")),
                id: Some(s("r1")),
                ..Default::default()
            },
            container: ftd::Container {
                open: Some(false),
                ..Default::default()
            },
        }));
        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                external_children: Default::default(),
                children: vec![ftd::Element::Markup(ftd::Markups {
                    text: ftd::markdown_line("hello"),
                    line: true,
                    ..Default::default()
                })],
                open: Some(true),
                ..Default::default()
            },
            common: ftd::Common {
                data_id: Some(s("r2")),
                id: Some(s("r2")),
                ..Default::default()
            },
        }));

        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                open: Some(false),
                ..Default::default()
            },
            common: ftd::Common {
                data_id: Some(s("r3")),
                id: Some(s("r3")),
                ..Default::default()
            },
        }));

        let mut bag = super::default_bag();

        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@0", -1, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@1", 0, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@2", -1, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@1,0", -1, &mut bag);

        insert_integer_by_root("foo/bar#CHILDREN-COUNT@0", 0, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT@1", 1, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT@2", 0, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT@1,0", 0, &mut bag);

        insert_integer_by_root("foo/bar#SIBLING-INDEX-0@0", 0, &mut bag);
        insert_integer_by_root("foo/bar#SIBLING-INDEX-0@1", 1, &mut bag);
        insert_integer_by_root("foo/bar#SIBLING-INDEX-0@2", 2, &mut bag);

        insert_integer_by_root("foo/bar#SIBLING-INDEX@0", 1, &mut bag);
        insert_integer_by_root("foo/bar#SIBLING-INDEX@1", 2, &mut bag);
        insert_integer_by_root("foo/bar#SIBLING-INDEX@2", 3, &mut bag);

        let (g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.row:
                id: r1
                open: false

                -- ftd.row:
                id: r2
                open: true

                --- ftd.text: hello

                -- ftd.row:
                id: r3
                open: false
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn submit() {
        let mut main = super::default_column();

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("hello"),
                line: true,
                common: ftd::Common {
                    submit: Some("https://httpbin.org/post?x=10".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.text: hello
                submit: https://httpbin.org/post?x=10
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        let mut bag = super::default_bag();

        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@0", -1, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT@0", 0, &mut bag);
        insert_integer_by_root("foo/bar#SIBLING-INDEX-0@0", 0, &mut bag);
        insert_integer_by_root("foo/bar#SIBLING-INDEX@0", 1, &mut bag);

        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn basic_loop_on_record_1() {
        let mut main = super::default_column();
        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("hello"),
                        line: true,
                        common: ftd::Common {
                            reference: Some(s("foo/bar#name@0")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("world"),
                        line: true,
                        common: ftd::Common {
                            reference: Some(s("foo/bar#body@0")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            ..Default::default()
        }));

        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("Arpita Jaiswal"),
                        line: true,
                        common: ftd::Common {
                            reference: Some(s("foo/bar#name@1")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("Arpita is developer at Fifthtry"),
                        line: true,
                        common: ftd::Common {
                            reference: Some(s("foo/bar#body@1")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            common: ftd::Common {
                reference: Some(s("foo/bar#people")),
                ..Default::default()
            },
        }));

        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("Amit Upadhyay"),
                        line: true,
                        common: ftd::Common {
                            reference: Some(s("foo/bar#name@2")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("Amit is CEO of FifthTry."),
                        line: true,
                        common: ftd::Common {
                            reference: Some(s("foo/bar#body@2")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            common: ftd::Common {
                reference: Some(s("foo/bar#people")),
                ..Default::default()
            },
        }));

        let mut bag = super::default_bag();

        bag.insert(
            "foo/bar#foo".to_string(),
            ftd::p2::Thing::Component(ftd::Component {
                root: "ftd#row".to_string(),
                full_name: s("foo/bar#foo"),
                arguments: std::iter::IntoIterator::into_iter([
                    (s("body"), ftd::p2::Kind::string()),
                    (s("name"), ftd::p2::Kind::caption()),
                ])
                .collect(),
                instructions: vec![
                    ftd::component::Instruction::ChildComponent {
                        child: ftd::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("text"),
                                ftd::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: "name".to_string(),
                                        kind: ftd::p2::Kind::caption_or_body(),
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::component::Instruction::ChildComponent {
                        child: ftd::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("text"),
                                ftd::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: "body".to_string(),
                                        kind: ftd::p2::Kind::caption_or_body(),
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                ],
                ..Default::default()
            }),
        );

        bag.insert(
            "foo/bar#get".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "get".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "world".to_string(),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#name".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "name".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "Arpita Jaiswal".to_string(),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#people".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "foo/bar#people".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::List {
                        data: vec![
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: "foo/bar#person".to_string(),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("bio"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::String {
                                                    text: "Arpita is developer at Fifthtry"
                                                        .to_string(),
                                                    source: ftd::TextSource::Body,
                                                },
                                            },
                                        ),
                                        (
                                            s("name"),
                                            ftd::PropertyValue::Reference {
                                                name: "foo/bar#name".to_string(),
                                                kind: ftd::p2::Kind::caption(),
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: "foo/bar#person".to_string(),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("bio"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::String {
                                                    text: "Amit is CEO of FifthTry.".to_string(),
                                                    source: ftd::TextSource::Body,
                                                },
                                            },
                                        ),
                                        (
                                            s("name"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::String {
                                                    text: "Amit Upadhyay".to_string(),
                                                    source: ftd::TextSource::Caption,
                                                },
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                        ],
                        kind: ftd::p2::Kind::Record {
                            name: "foo/bar#person".to_string(),
                            default: None,
                            is_reference: false,
                        },
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#$loop$@1".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "$loop$".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: "foo/bar#person".to_string(),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("bio"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "Arpita is developer at Fifthtry".to_string(),
                                        source: ftd::TextSource::Body,
                                    },
                                },
                            ),
                            (
                                s("name"),
                                ftd::PropertyValue::Reference {
                                    name: "foo/bar#name".to_string(),
                                    kind: ftd::p2::Kind::caption(),
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#$loop$@2".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "$loop$".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: "foo/bar#person".to_string(),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("bio"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "Amit is CEO of FifthTry.".to_string(),
                                        source: ftd::TextSource::Body,
                                    },
                                },
                            ),
                            (
                                s("name"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "Amit Upadhyay".to_string(),
                                        source: ftd::TextSource::Caption,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#body@0".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "body".to_string(),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#get"),
                    kind: ftd::p2::Kind::string(),
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#body@1".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "body".to_string(),
                value: ftd::PropertyValue::Variable {
                    name: s("foo/bar#$loop$@1.bio"),
                    kind: ftd::p2::Kind::body(),
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#body@2".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "body".to_string(),
                value: ftd::PropertyValue::Variable {
                    name: s("foo/bar#$loop$@2.bio"),
                    kind: ftd::p2::Kind::body(),
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#name@0".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "name".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("hello"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#name@1".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "name".to_string(),
                value: ftd::PropertyValue::Variable {
                    name: s("foo/bar#$loop$@1.name"),
                    kind: ftd::p2::Kind::caption(),
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#name@2".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "name".to_string(),
                value: ftd::PropertyValue::Variable {
                    name: s("foo/bar#$loop$@2.name"),
                    kind: ftd::p2::Kind::caption(),
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#person".to_string(),
            ftd::p2::Thing::Record(ftd::p2::Record {
                name: "foo/bar#person".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    (s("bio"), ftd::p2::Kind::body()),
                    (s("name"), ftd::p2::Kind::caption()),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![s("name"), s("bio")],
            }),
        );

        p!(
            "
            -- ftd.row foo:
            caption name:
            string body:

            --- ftd.text: $name

            --- ftd.text: $body

            -- record person:
            caption name:
            body bio:

            -- person list people:

            -- string name: Arpita Jaiswal

            -- people: $name

            Arpita is developer at Fifthtry

            -- people: Amit Upadhyay

            Amit is CEO of FifthTry.

            -- string get: world

            -- foo: hello
            body: $get

            -- foo: $obj.name
            $loop$: $people as $obj
            body: $obj.bio
            ",
            (bag, main),
        );
    }

    #[test]
    fn basic_loop_on_record_with_if_condition() {
        let mut main = super::default_column();
        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("Arpita Jaiswal"),
                        line: true,
                        common: ftd::Common {
                            reference: Some(s("foo/bar#name@0")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("Arpita is developer at Fifthtry"),
                        line: true,
                        common: ftd::Common {
                            reference: Some(s("foo/bar#body@0")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            common: ftd::Common {
                reference: Some(s("foo/bar#people")),
                condition: Some(ftd::Condition {
                    variable: s("foo/bar#$loop$@0.ceo"),
                    value: serde_json::Value::Bool(true),
                }),
                is_not_visible: true,
                ..Default::default()
            },
            ..Default::default()
        }));

        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("Amit Upadhyay"),
                        line: true,
                        common: ftd::Common {
                            reference: Some(s("foo/bar#name@1")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("Amit is CEO of FifthTry."),
                        line: true,
                        common: ftd::Common {
                            reference: Some(s("foo/bar#body@1")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            common: ftd::Common {
                condition: Some(ftd::Condition {
                    variable: s("foo/bar#$loop$@1.ceo"),
                    value: serde_json::Value::Bool(true),
                }),
                reference: Some(s("foo/bar#people")),
                ..Default::default()
            },
            ..Default::default()
        }));

        let mut bag = super::default_bag();

        bag.insert(
            "foo/bar#foo".to_string(),
            ftd::p2::Thing::Component(ftd::Component {
                root: "ftd#row".to_string(),
                full_name: s("foo/bar#foo"),
                arguments: std::iter::IntoIterator::into_iter([
                    (s("body"), ftd::p2::Kind::string()),
                    (s("name"), ftd::p2::Kind::caption()),
                ])
                .collect(),
                instructions: vec![
                    ftd::component::Instruction::ChildComponent {
                        child: ftd::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("text"),
                                ftd::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: "name".to_string(),
                                        kind: ftd::p2::Kind::caption_or_body(),
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::component::Instruction::ChildComponent {
                        child: ftd::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("text"),
                                ftd::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: "body".to_string(),
                                        kind: ftd::p2::Kind::caption_or_body(),
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                ],
                ..Default::default()
            }),
        );

        bag.insert(
            "foo/bar#people".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "foo/bar#people".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::List {
                        data: vec![
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: "foo/bar#person".to_string(),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("bio"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::String {
                                                    text: "Arpita is developer at Fifthtry"
                                                        .to_string(),
                                                    source: ftd::TextSource::Body,
                                                },
                                            },
                                        ),
                                        (
                                            s("ceo"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::Boolean { value: false },
                                            },
                                        ),
                                        (
                                            s("name"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::String {
                                                    text: "Arpita Jaiswal".to_string(),
                                                    source: ftd::TextSource::Caption,
                                                },
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: "foo/bar#person".to_string(),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("bio"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::String {
                                                    text: "Amit is CEO of FifthTry.".to_string(),
                                                    source: ftd::TextSource::Body,
                                                },
                                            },
                                        ),
                                        (
                                            s("ceo"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::Boolean { value: true },
                                            },
                                        ),
                                        (
                                            s("name"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::String {
                                                    text: "Amit Upadhyay".to_string(),
                                                    source: ftd::TextSource::Caption,
                                                },
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                        ],
                        kind: ftd::p2::Kind::Record {
                            name: "foo/bar#person".to_string(),
                            default: None,
                            is_reference: false,
                        },
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#person".to_string(),
            ftd::p2::Thing::Record(ftd::p2::Record {
                name: "foo/bar#person".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    (s("bio"), ftd::p2::Kind::body()),
                    (s("name"), ftd::p2::Kind::caption()),
                    (s("ceo"), ftd::p2::Kind::boolean()),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![s("name"), s("bio"), s("ceo")],
            }),
        );

        bag.insert(
            s("foo/bar#$loop$@0"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: "$loop$".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: "foo/bar#person".to_string(),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("bio"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "Arpita is developer at Fifthtry".to_string(),
                                        source: ftd::TextSource::Body,
                                    },
                                },
                            ),
                            (
                                s("ceo"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::Boolean { value: false },
                                },
                            ),
                            (
                                s("name"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "Arpita Jaiswal".to_string(),
                                        source: ftd::TextSource::Caption,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        bag.insert(
            s("foo/bar#$loop$@1"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: "$loop$".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: "foo/bar#person".to_string(),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("bio"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "Amit is CEO of FifthTry.".to_string(),
                                        source: ftd::TextSource::Body,
                                    },
                                },
                            ),
                            (
                                s("ceo"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::Boolean { value: true },
                                },
                            ),
                            (
                                s("name"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "Amit Upadhyay".to_string(),
                                        source: ftd::TextSource::Caption,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        bag.insert(
            s("foo/bar#body@0"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: "body".to_string(),
                value: ftd::PropertyValue::Variable {
                    name: "foo/bar#$loop$@0.bio".to_string(),
                    kind: ftd::p2::Kind::body(),
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        bag.insert(
            s("foo/bar#body@1"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: "body".to_string(),
                value: ftd::PropertyValue::Variable {
                    name: "foo/bar#$loop$@1.bio".to_string(),
                    kind: ftd::p2::Kind::body(),
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        bag.insert(
            s("foo/bar#name@0"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: "name".to_string(),
                value: ftd::PropertyValue::Variable {
                    name: "foo/bar#$loop$@0.name".to_string(),
                    kind: ftd::p2::Kind::caption(),
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        bag.insert(
            s("foo/bar#name@1"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: "name".to_string(),
                value: ftd::PropertyValue::Variable {
                    name: "foo/bar#$loop$@1.name".to_string(),
                    kind: ftd::p2::Kind::caption(),
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        p!(
            "
            -- ftd.row foo:
            caption name:
            string body:

            --- ftd.text: $name

            --- ftd.text: $body

            -- record person:
            caption name:
            body bio:
            boolean ceo:

            -- person list people:

            -- people: Arpita Jaiswal
            ceo: false

            Arpita is developer at Fifthtry

            -- people: Amit Upadhyay
            ceo: true

            Amit is CEO of FifthTry.

            -- foo: $obj.name
            $loop$: $people as $obj
            if: $obj.ceo
            body: $obj.bio
            ",
            (bag, main),
        );
    }

    #[test]
    fn basic_loop_on_string() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("Arpita"),
                line: true,
                common: ftd::Common {
                    reference: Some(s("foo/bar#people")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("Asit"),
                line: true,
                common: ftd::Common {
                    reference: Some(s("foo/bar#people")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("Sourabh"),
                line: true,
                common: ftd::Common {
                    reference: Some(s("foo/bar#people")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("$loop$"),
                line: true,
                common: ftd::Common {
                    reference: Some(s("foo/bar#people")),
                    is_dummy: true,
                    ..Default::default()
                },
                ..Default::default()
            }));

        let mut bag = super::default_bag();

        bag.insert(
            "foo/bar#$loop$@0".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "$loop$".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Arpita"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#$loop$@1".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "$loop$".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Asit"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#$loop$@2".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "$loop$".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Sourabh"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#$loop$@3".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "$loop$".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("$loop$"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
            }),
        );

        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@0", -1, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@1", -1, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@2", -1, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@3", -1, &mut bag);

        insert_integer_by_root("foo/bar#CHILDREN-COUNT@0", 0, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT@1", 0, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT@2", 0, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT@3", 0, &mut bag);

        bag.insert(
            "foo/bar#people".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "foo/bar#people".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::List {
                        data: vec![
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: "Arpita".to_string(),
                                    source: ftd::TextSource::Caption,
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: "Asit".to_string(),
                                    source: ftd::TextSource::Caption,
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: "Sourabh".to_string(),
                                    source: ftd::TextSource::Caption,
                                },
                            },
                        ],
                        kind: ftd::p2::Kind::string(),
                    },
                },
                conditions: vec![],
            }),
        );
        let (g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- string list people:

                -- people: Arpita

                -- people: Asit

                -- people: Sourabh

                -- ftd.text: $obj
                $loop$: $people as $obj
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn loop_inside_subsection() {
        let mut main = super::default_column();
        let mut col = ftd::Column {
            ..Default::default()
        };

        col.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("Arpita Jaiswal"),
                        line: true,
                        common: ftd::Common {
                            reference: Some(s("foo/bar#name@0,0")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("Arpita is developer at Fifthtry"),
                        line: true,
                        common: ftd::Common {
                            reference: Some(s("foo/bar#body@0,0")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            common: ftd::Common {
                reference: Some(s("foo/bar#people")),
                ..Default::default()
            },
        }));

        col.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("Amit Upadhyay"),
                        line: true,
                        common: ftd::Common {
                            reference: Some(s("foo/bar#name@0,1")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("Amit is CEO of FifthTry."),
                        line: true,
                        common: ftd::Common {
                            reference: Some(s("foo/bar#body@0,1")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            common: ftd::Common {
                reference: Some(s("foo/bar#people")),
                ..Default::default()
            },
        }));

        main.container.children.push(ftd::Element::Column(col));

        let mut bag = super::default_bag();

        bag.insert(
            "foo/bar#foo".to_string(),
            ftd::p2::Thing::Component(ftd::Component {
                root: "ftd.row".to_string(),
                full_name: s("foo/bar#foo"),
                arguments: std::iter::IntoIterator::into_iter([
                    (s("body"), ftd::p2::Kind::string()),
                    (s("name"), ftd::p2::Kind::caption()),
                ])
                .collect(),
                instructions: vec![
                    ftd::component::Instruction::ChildComponent {
                        child: ftd::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("text"),
                                ftd::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: "name".to_string(),
                                        kind: ftd::p2::Kind::caption_or_body(),
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::component::Instruction::ChildComponent {
                        child: ftd::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("text"),
                                ftd::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: "body".to_string(),
                                        kind: ftd::p2::Kind::caption_or_body(),
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                ],
                invocations: vec![
                    std::iter::IntoIterator::into_iter([
                        (
                            s("body"),
                            ftd::Value::String {
                                text: s("Arpita is developer at Fifthtry"),
                                source: ftd::TextSource::Body,
                            },
                        ),
                        (
                            s("name"),
                            ftd::Value::String {
                                text: s("Arpita Jaiswal"),
                                source: ftd::TextSource::Caption,
                            },
                        ),
                    ])
                    .collect(),
                    std::iter::IntoIterator::into_iter([
                        (
                            s("body"),
                            ftd::Value::String {
                                text: s("Amit is CEO of FifthTry."),
                                source: ftd::TextSource::Body,
                            },
                        ),
                        (
                            s("name"),
                            ftd::Value::String {
                                text: s("Amit Upadhyay"),
                                source: ftd::TextSource::Caption,
                            },
                        ),
                    ])
                    .collect(),
                ],
                ..Default::default()
            }),
        );

        bag.insert(
            "foo/bar#people".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "foo/bar#people".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::List {
                        data: vec![
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: "foo/bar#person".to_string(),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("bio"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::String {
                                                    text: "Arpita is developer at Fifthtry"
                                                        .to_string(),
                                                    source: ftd::TextSource::Body,
                                                },
                                            },
                                        ),
                                        (
                                            s("name"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::String {
                                                    text: "Arpita Jaiswal".to_string(),
                                                    source: ftd::TextSource::Caption,
                                                },
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: "foo/bar#person".to_string(),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("bio"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::String {
                                                    text: "Amit is CEO of FifthTry.".to_string(),
                                                    source: ftd::TextSource::Body,
                                                },
                                            },
                                        ),
                                        (
                                            s("name"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::String {
                                                    text: "Amit Upadhyay".to_string(),
                                                    source: ftd::TextSource::Caption,
                                                },
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                        ],
                        kind: ftd::p2::Kind::Record {
                            name: "foo/bar#person".to_string(),
                            default: None,
                            is_reference: true,
                        },
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#person".to_string(),
            ftd::p2::Thing::Record(ftd::p2::Record {
                name: "foo/bar#person".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    (s("bio"), ftd::p2::Kind::body()),
                    (s("name"), ftd::p2::Kind::caption()),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![s("name"), s("bio")],
            }),
        );

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.row foo:
                caption name:
                string body:

                --- ftd.text: $name

                --- ftd.text: $body

                -- record person:
                caption name:
                body bio:

                -- person list people:

                -- people: Arpita Jaiswal

                Arpita is developer at Fifthtry

                -- people: Amit Upadhyay

                Amit is CEO of FifthTry.

                -- ftd.column:

                --- foo: $obj.name
                $loop$: $people as $obj
                body: $obj.bio
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        // pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn basic_processor() {
        let mut main = super::default_column();

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("\"0.1.18\""),
                line: true,
                common: ftd::Common {
                    reference: Some(s("foo/bar#test")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let mut bag = super::default_bag();

        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@0", -1, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT@0", 0, &mut bag);
        insert_integer_by_root("foo/bar#SIBLING-INDEX-0@0", 0, &mut bag);
        insert_integer_by_root("foo/bar#SIBLING-INDEX@0", 1, &mut bag);

        bag.insert(
            "foo/bar#test".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "foo/bar#test".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "\"0.1.18\"".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
            }),
        );

        let (g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- string test:
                $processor$: read_version_from_cargo_toml

                -- ftd.text: $test
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn basic_processor_that_overwrites() {
        let mut main = super::default_column();

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("\"0.1.18\""),
                line: true,
                common: ftd::Common {
                    reference: Some(s("foo/bar#test")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let mut bag = super::default_bag();

        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@0", -1, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT@0", 0, &mut bag);
        insert_integer_by_root("foo/bar#SIBLING-INDEX-0@0", 0, &mut bag);
        insert_integer_by_root("foo/bar#SIBLING-INDEX@0", 1, &mut bag);

        bag.insert(
            "foo/bar#test".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "test".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "\"0.1.18\"".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
            }),
        );

        let (g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- string test: yo

                -- test:
                $processor$: read_version_from_cargo_toml

                -- ftd.text: $test
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn basic_processor_for_list() {
        let mut main = super::default_column();

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("\"ftd\""),
                line: true,
                common: ftd::Common {
                    reference: Some(s("foo/bar#test")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("\"0.1.18\""),
                line: true,
                common: ftd::Common {
                    reference: Some(s("foo/bar#test")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("["),
                line: true,
                common: ftd::Common {
                    reference: Some(s("foo/bar#test")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("\"2018\""),
                line: true,
                common: ftd::Common {
                    reference: Some(s("foo/bar#test")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("\"ftd: FifthTry Document Format\""),
                line: true,
                common: ftd::Common {
                    reference: Some(s("foo/bar#test")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("\"MIT\""),
                line: true,
                common: ftd::Common {
                    reference: Some(s("foo/bar#test")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("\"https://github.com/FifthTry/ftd\""),
                line: true,
                common: ftd::Common {
                    reference: Some(s("foo/bar#test")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("\"https://ftd.dev\""),
                line: true,
                common: ftd::Common {
                    reference: Some(s("foo/bar#test")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("$loop$"),
                line: true,
                common: ftd::Common {
                    reference: Some(s("foo/bar#test")),
                    is_dummy: true,
                    ..Default::default()
                },
                ..Default::default()
            }));

        let mut bag = super::default_bag();

        bag.insert(
            "foo/bar#test".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "foo/bar#test".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::List {
                        data: vec![
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: "\"ftd\"".to_string(),
                                    source: ftd::TextSource::Header,
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: "\"0.1.18\"".to_string(),
                                    source: ftd::TextSource::Header,
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: "[".to_string(),
                                    source: ftd::TextSource::Header,
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: "\"2018\"".to_string(),
                                    source: ftd::TextSource::Header,
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: "\"ftd: FifthTry Document Format\"".to_string(),
                                    source: ftd::TextSource::Header,
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: "\"MIT\"".to_string(),
                                    source: ftd::TextSource::Header,
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: "\"https://github.com/FifthTry/ftd\"".to_string(),
                                    source: ftd::TextSource::Header,
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: "\"https://ftd.dev\"".to_string(),
                                    source: ftd::TextSource::Header,
                                },
                            },
                        ],
                        kind: ftd::p2::Kind::string(),
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#$loop$@0".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "$loop$".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "\"ftd\"".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#$loop$@1".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "$loop$".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "\"0.1.18\"".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#$loop$@2".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "$loop$".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "[".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#$loop$@3".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "$loop$".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "\"2018\"".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#$loop$@4".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "$loop$".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "\"ftd: FifthTry Document Format\"".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#$loop$@5".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "$loop$".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "\"MIT\"".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#$loop$@6".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "$loop$".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "\"https://github.com/FifthTry/ftd\"".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#$loop$@7".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "$loop$".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "\"https://ftd.dev\"".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#$loop$@8".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "$loop$".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "$loop$".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
            }),
        );

        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@0", -1, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@1", -1, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@2", -1, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@3", -1, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@4", -1, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@5", -1, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@6", -1, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@7", -1, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@8", -1, &mut bag);

        insert_integer_by_root("foo/bar#CHILDREN-COUNT@0", 0, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT@1", 0, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT@2", 0, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT@3", 0, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT@4", 0, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT@5", 0, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT@6", 0, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT@7", 0, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT@8", 0, &mut bag);

        let (g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- string list test:
                $processor$: read_package_from_cargo_toml

                -- ftd.text: $obj
                $loop$: $test as $obj
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn processor_for_list_of_record() {
        let mut main = super::default_column();

        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("\"ftd\""),
                        line: true,
                        common: ftd::Common {
                            reference: Some(s("foo/bar#name@0")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("name"),
                        line: true,
                        common: ftd::Common {
                            reference: Some(s("foo/bar#body@0")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            common: ftd::Common {
                reference: Some(s("foo/bar#test")),
                ..Default::default()
            },
        }));

        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("\"0.1.18\""),
                        line: true,
                        common: ftd::Common {
                            reference: Some(s("foo/bar#name@1")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("version"),
                        line: true,
                        common: ftd::Common {
                            reference: Some(s("foo/bar#body@1")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            common: ftd::Common {
                reference: Some(s("foo/bar#test")),
                ..Default::default()
            },
        }));

        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("["),
                        line: true,
                        common: ftd::Common {
                            reference: Some(s("foo/bar#name@2")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("authors"),
                        line: true,
                        common: ftd::Common {
                            reference: Some(s("foo/bar#body@2")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            common: ftd::Common {
                reference: Some(s("foo/bar#test")),
                ..Default::default()
            },
        }));

        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("\"2018\""),
                        line: true,
                        common: ftd::Common {
                            reference: Some(s("foo/bar#name@3")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("edition"),
                        line: true,
                        common: ftd::Common {
                            reference: Some(s("foo/bar#body@3")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            common: ftd::Common {
                reference: Some(s("foo/bar#test")),
                ..Default::default()
            },
        }));

        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("\"ftd: FifthTry Document Format\""),
                        line: true,
                        common: ftd::Common {
                            reference: Some(s("foo/bar#name@4")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("description"),
                        line: true,
                        common: ftd::Common {
                            reference: Some(s("foo/bar#body@4")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            common: ftd::Common {
                reference: Some(s("foo/bar#test")),
                ..Default::default()
            },
        }));

        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("\"MIT\""),
                        line: true,
                        common: ftd::Common {
                            reference: Some(s("foo/bar#name@5")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("license"),
                        line: true,
                        common: ftd::Common {
                            reference: Some(s("foo/bar#body@5")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            common: ftd::Common {
                reference: Some(s("foo/bar#test")),
                ..Default::default()
            },
        }));

        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("\"https://github.com/FifthTry/ftd\""),
                        line: true,
                        common: ftd::Common {
                            reference: Some(s("foo/bar#name@6")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("repository"),
                        line: true,
                        common: ftd::Common {
                            reference: Some(s("foo/bar#body@6")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            common: ftd::Common {
                reference: Some(s("foo/bar#test")),
                ..Default::default()
            },
        }));

        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("\"https://ftd.dev\""),
                        line: true,
                        common: ftd::Common {
                            reference: Some(s("foo/bar#name@7")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("homepage"),
                        line: true,
                        common: ftd::Common {
                            reference: Some(s("foo/bar#body@7")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            common: ftd::Common {
                reference: Some(s("foo/bar#test")),
                ..Default::default()
            },
        }));

        /*let mut bag = super::default_bag();

        bag.insert(
            "foo/bar#data".to_string(),
            ftd::p2::Thing::Record(ftd::p2::Record {
                name: "foo/bar#data".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    (s("description"), ftd::p2::Kind::string()),
                    (s("title"), ftd::p2::Kind::string()),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![s("title"), s("description")],
            }),
        );

        bag.insert(
            "foo/bar#foo".to_string(),
            ftd::p2::Thing::Component(ftd::Component {
                root: "ftd.row".to_string(),
                full_name: "foo/bar#foo".to_string(),
                arguments: std::iter::IntoIterator::into_iter([
                    (s("body"), ftd::p2::Kind::string()),
                    (s("name"), ftd::p2::Kind::caption()),
                ])
                .collect(),
                instructions: vec![
                    ftd::component::Instruction::ChildComponent {
                        child: ftd::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("text"),
                                ftd::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: "name".to_string(),
                                        kind: ftd::p2::Kind::caption_or_body(),
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::component::Instruction::ChildComponent {
                        child: ftd::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("text"),
                                ftd::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: "body".to_string(),
                                        kind: ftd::p2::Kind::caption_or_body(),
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                ],
                ..Default::default()
            }),
        );

        bag.insert(
            "foo/bar#test".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "foo/bar#test".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::List {
                        data: vec![
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: "foo/bar#data".to_string(),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("description"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: "name".to_string(),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                        (
                                            s("title"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: "\"ftd\"".to_string(),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: "foo/bar#data".to_string(),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("description"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: "version".to_string(),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                        (
                                            s("title"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: "\"0.1.18\"".to_string(),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: "foo/bar#data".to_string(),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("description"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: "authors".to_string(),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                        (
                                            s("title"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: "[".to_string(),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: "foo/bar#data".to_string(),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("description"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: "edition".to_string(),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                        (
                                            s("title"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: "\"2018\"".to_string(),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: "foo/bar#data".to_string(),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("description"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: "description".to_string(),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                        (
                                            s("title"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: "\"ftd: FifthTry Document Format\""
                                                        .to_string(),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: "foo/bar#data".to_string(),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("description"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: "license".to_string(),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                        (
                                            s("title"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: "\"MIT\"".to_string(),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: "foo/bar#data".to_string(),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("description"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: "repository".to_string(),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                        (
                                            s("title"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: "\"https://github.com/FifthTry/ftd\""
                                                        .to_string(),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: "foo/bar#data".to_string(),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("description"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: "homepage".to_string(),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                        (
                                            s("title"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: "\"https://ftd.dev\"".to_string(),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                        ],
                        kind: ftd::p2::Kind::Record {
                            name: s("foo/bar#data"),
                            default: None,
                        },
                    },
                },
                conditions: vec![],
            }),
        );*/

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.row foo:
                caption name:
                string body:
    
                --- ftd.text: $name
    
                --- ftd.text: $body
    
                -- record data:
                string title:
                string description:
    
                -- data list test:
                $processor$: read_package_records_from_cargo_toml
    
                -- foo: $obj.title
                $loop$: $test as $obj
                body: $obj.description
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn loop_with_tree_structure() {
        let mut main = super::default_column();
        let col = ftd::Element::Column(ftd::Column {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("ab title"),
                        line: true,
                        common: ftd::Common {
                            reference: Some(s("foo/bar#toc@0.title")),
                            link: Some(s("ab link")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Column(ftd::Column {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![ftd::Element::Markup(ftd::Markups {
                                text: ftd::markdown_line("aa title"),
                                line: true,
                                common: ftd::Common {
                                    reference: Some(s("foo/bar#toc@0,1.title")),
                                    link: Some(s("aa link")),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })],
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Column(ftd::Column {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![ftd::Element::Markup(ftd::Markups {
                                text: ftd::markdown_line("aaa title"),
                                line: true,
                                common: ftd::Common {
                                    reference: Some(s("foo/bar#toc@0,2.title")),
                                    link: Some(s("aaa link")),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })],
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            common: ftd::Common {
                reference: Some(s("foo/bar#toc")),
                ..Default::default()
            },
            ..Default::default()
        });
        let col1 = ftd::Element::Column(ftd::Column {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("ab title"),
                        line: true,
                        common: ftd::Common {
                            reference: Some(s("foo/bar#toc@1,0.title")),
                            link: Some(s("ab link")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Column(ftd::Column {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![ftd::Element::Markup(ftd::Markups {
                                text: ftd::markdown_line("aa title"),
                                line: true,
                                common: ftd::Common {
                                    reference: Some(s("foo/bar#toc@1,0,1.title")),
                                    link: Some(s("aa link")),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })],
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Column(ftd::Column {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![ftd::Element::Markup(ftd::Markups {
                                text: ftd::markdown_line("aaa title"),
                                line: true,
                                common: ftd::Common {
                                    reference: Some(s("foo/bar#toc@1,0,2.title")),
                                    link: Some(s("aaa link")),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })],
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            common: ftd::Common {
                reference: Some(s("foo/bar#toc")),
                ..Default::default()
            },
            ..Default::default()
        });
        main.container.children.push(col.clone());
        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![col1],
                ..Default::default()
            },
            ..Default::default()
        }));

        let mut bag = super::default_bag();

        bag.insert(
            s("foo/bar#aa"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("foo/bar#aa"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::List {
                        data: vec![
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: s("foo/bar#toc-record"),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("children"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::List {
                                                    data: vec![],
                                                    kind: ftd::p2::Kind::Record {
                                                        name: s("foo/bar#toc-record"),
                                                        default: None,
                                                        is_reference: true,
                                                    },
                                                },
                                            },
                                        ),
                                        (
                                            s("link"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: s("aa link"),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                        (
                                            s("title"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: s("aa title"),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: s("foo/bar#toc-record"),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("children"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::List {
                                                    data: vec![],
                                                    kind: ftd::p2::Kind::Record {
                                                        name: s("foo/bar#toc-record"),
                                                        default: None,
                                                        is_reference: true,
                                                    },
                                                },
                                            },
                                        ),
                                        (
                                            s("link"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: s("aaa link"),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                        (
                                            s("title"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: s("aaa title"),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                        ],
                        kind: ftd::p2::Kind::Record {
                            name: s("foo/bar#toc-record"),
                            default: None,
                            is_reference: true,
                        },
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            s("foo/bar#toc"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("foo/bar#toc"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::List {
                        data: vec![ftd::PropertyValue::Value {
                            value: ftd::Value::Record {
                                name: s("foo/bar#toc-record"),
                                fields: std::iter::IntoIterator::into_iter([
                                    (
                                        s("children"),
                                        ftd::PropertyValue::Value {
                                            value: ftd::variable::Value::List {
                                                data: vec![
                                                    ftd::PropertyValue::Value {value: ftd::Value::Record {
                                                        name: s("foo/bar#toc-record"),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                s("children"),
                                                                ftd::PropertyValue::Value {
                                                                    value: ftd::variable::Value::List {
                                                                        data: vec![],
                                                                        kind: ftd::p2::Kind::Record {
                                                                            name: s("foo/bar#toc-record"),
                                                                            default: None,
                                                                            is_reference: true,
                                                                        },
                                                                    },
                                                                },
                                                            ),
                                                            (
                                                                s("link"),
                                                                ftd::PropertyValue::Value {
                                                                    value: ftd::variable::Value::String {
                                                                        text: s("aa link"),
                                                                        source: ftd::TextSource::Header,
                                                                    },
                                                                },
                                                            ),
                                                            (
                                                                s("title"),
                                                                ftd::PropertyValue::Value {
                                                                    value: ftd::variable::Value::String {
                                                                        text: s("aa title"),
                                                                        source: ftd::TextSource::Header,
                                                                    },
                                                                },
                                                            ),
                                                        ])
                                                        .collect(),
                                                    }},
                                                    ftd::PropertyValue::Value {value: ftd::Value::Record {
                                                        name: s("foo/bar#toc-record"),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                s("children"),
                                                                ftd::PropertyValue::Value {
                                                                    value: ftd::variable::Value::List {
                                                                        data: vec![],
                                                                        kind: ftd::p2::Kind::Record {
                                                                            name: s("foo/bar#toc-record"),
                                                                            default: None,
                                                                            is_reference: true,
                                                                        },
                                                                    },
                                                                },
                                                            ),
                                                            (
                                                                s("link"),
                                                                ftd::PropertyValue::Value {
                                                                    value: ftd::variable::Value::String {
                                                                        text: s("aaa link"),
                                                                        source: ftd::TextSource::Header,
                                                                    },
                                                                },
                                                            ),
                                                            (
                                                                s("title"),
                                                                ftd::PropertyValue::Value {
                                                                    value: ftd::variable::Value::String {
                                                                        text: s("aaa title"),
                                                                        source: ftd::TextSource::Header,
                                                                    },
                                                                },
                                                            ),
                                                        ])
                                                        .collect(),
                                                    }},
                                                ],
                                                kind: ftd::p2::Kind::Record {
                                                    name: s("foo/bar#toc-record"),
                                                    default: None,
                                                    is_reference: true,
                                                },
                                            },
                                        },
                                    ),
                                    (
                                        s("link"),
                                        ftd::PropertyValue::Value {
                                            value: ftd::variable::Value::String {
                                                text: s("ab link"),
                                                source: ftd::TextSource::Header,
                                            },
                                        },
                                    ),
                                    (
                                        s("title"),
                                        ftd::PropertyValue::Value {
                                            value: ftd::variable::Value::String {
                                                text: s("ab title"),
                                                source: ftd::TextSource::Header,
                                            },
                                        },
                                    ),
                                ])
                                .collect(),
                            },
                        }],
                        kind: ftd::p2::Kind::Record {
                            name: s("foo/bar#toc-record"),
                            default: None,
                            is_reference: true,
                        },
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            s("foo/bar#toc"),
            ftd::p2::Thing::Component(ftd::Component {
                root: "ftd.column".to_string(),
                full_name: "foo/bar#toc-item".to_string(),
                arguments: std::iter::IntoIterator::into_iter([(
                    s("toc"),
                    ftd::p2::Kind::Record {
                        name: "foo/bar#toc-record".to_string(),
                        default: None,
                        is_reference: true,
                    },
                )])
                .collect(),
                instructions: vec![
                    Instruction::ChildComponent {
                        child: ftd::ChildComponent {
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("link"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Variable {
                                            name: "toc.link".to_string(),
                                            kind: ftd::p2::Kind::Optional {
                                                kind: Box::new(ftd::p2::Kind::string()),
                                                is_reference: false,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("text"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Variable {
                                            name: "toc.title".to_string(),
                                            kind: ftd::p2::Kind::Optional {
                                                kind: Box::new(ftd::p2::Kind::caption_or_body()),
                                                is_reference: false,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    Instruction::RecursiveChildComponent {
                        child: ftd::ChildComponent {
                            is_recursive: true,
                            events: vec![],
                            root: "toc-item".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("$loop$"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Variable {
                                            name: "toc.children".to_string(),
                                            kind: ftd::p2::Kind::Record {
                                                name: s("foo/bar#toc-record"),
                                                default: None,
                                                is_reference: true,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("toc"),
                                    ftd::component::Property {
                                        default: Some(ftd::PropertyValue::Variable {
                                            name: "$loop$".to_string(),
                                            kind: ftd::p2::Kind::Record {
                                                name: s("foo/bar#toc-record"),
                                                default: None,
                                                is_reference: true,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                ],
                ..Default::default()
            }),
        );

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- record toc-record:
                string title:
                string link:
                toc-record list children:

                -- ftd.column toc-item:
                toc-record toc:

                --- ftd.text: $toc.title
                link: $toc.link

                --- toc-item:
                $loop$: $toc.children as $obj
                toc: $obj

                -- toc-record list aa:

                -- aa:
                title: aa title
                link: aa link

                -- aa:
                title: aaa title
                link: aaa link

                -- toc-record list toc:

                -- toc:
                title: ab title
                link: ab link
                children: $aa

                -- ftd.row foo:

                --- toc-item:
                $loop$: $toc as $obj
                toc: $obj

                -- toc-item:
                $loop$: $toc as $obj
                toc: $obj

                -- foo:
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        // pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn import_check() {
        let mut main = super::default_column();
        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![ftd::Element::Markup(ftd::Markups {
                    text: ftd::markdown_line("Hello World"),
                    line: true,
                    common: ftd::Common {
                        reference: Some(s("hello-world-variable#hello-world")),
                        ..Default::default()
                    },
                    ..Default::default()
                })],
                ..Default::default()
            },
            ..Default::default()
        }));

        let mut bag = super::default_bag();
        bag.insert(
            s("hello-world#foo"),
            ftd::p2::Thing::Component(ftd::Component {
                root: s("ftd#row"),
                full_name: s("hello-world#foo"),
                instructions: vec![ftd::Instruction::ChildComponent {
                    child: ftd::ChildComponent {
                        events: vec![],
                        root: s("ftd#text"),
                        condition: None,
                        properties: std::iter::IntoIterator::into_iter([(
                            s("text"),
                            ftd::component::Property {
                                default: Some(ftd::PropertyValue::Reference {
                                    name: "hello-world-variable#hello-world".to_string(),
                                    kind: ftd::p2::Kind::caption_or_body(),
                                }),
                                conditions: vec![],
                                ..Default::default()
                            },
                        )])
                        .collect(),
                        ..Default::default()
                    },
                }],
                invocations: vec![],
                ..Default::default()
            }),
        );
        bag.insert(
            s("hello-world-variable#hello-world"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("hello-world"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Hello World"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );

        p!(
            "
            -- import: hello-world as hw

            -- hw.foo:
            ",
            (bag, main),
        );
    }

    #[test]
    fn argument_with_default_value() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("hello world"),
                line: true,
                line_clamp: Some(10),
                common: ftd::Common {
                    reference: Some(s("foo/bar#name@0")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("hello"),
                line: true,
                line_clamp: Some(10),
                common: ftd::Common {
                    reference: Some(s("foo/bar#name@1")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("this is nice"),
                line: true,
                line_clamp: Some(20),
                common: ftd::Common {
                    reference: Some(s("foo/bar#name@2")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let mut bag = super::default_bag();

        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@0", -1, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@1", -1, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@2", -1, &mut bag);

        insert_integer_by_root("foo/bar#CHILDREN-COUNT@0", 0, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT@1", 0, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT@2", 0, &mut bag);

        insert_integer_by_root("foo/bar#SIBLING-INDEX-0@0", 0, &mut bag);
        insert_integer_by_root("foo/bar#SIBLING-INDEX-0@1", 1, &mut bag);
        insert_integer_by_root("foo/bar#SIBLING-INDEX-0@2", 2, &mut bag);

        insert_integer_by_root("foo/bar#SIBLING-INDEX@0", 1, &mut bag);
        insert_integer_by_root("foo/bar#SIBLING-INDEX@1", 2, &mut bag);
        insert_integer_by_root("foo/bar#SIBLING-INDEX@2", 3, &mut bag);

        bag.insert(
            s("foo/bar#foo"),
            ftd::p2::Thing::Component(ftd::Component {
                root: s("ftd#text"),
                full_name: s("foo/bar#foo"),
                arguments: std::iter::IntoIterator::into_iter([
                    (
                        s("name"),
                        ftd::p2::Kind::caption().set_default(Some(s("hello world"))),
                    ),
                    (
                        s("line-clamp"),
                        ftd::p2::Kind::Integer {
                            default: Some(s("10")),
                            is_reference: false,
                        },
                    ),
                ])
                .collect(),
                properties: std::iter::IntoIterator::into_iter([
                    (
                        s("line-clamp"),
                        ftd::component::Property {
                            default: Some(ftd::PropertyValue::Variable {
                                name: s("line-clamp"),
                                kind: ftd::p2::Kind::Optional {
                                    kind: Box::from(ftd::p2::Kind::Integer {
                                        default: Some(s("10")),
                                        is_reference: false,
                                    }),
                                    is_reference: false,
                                },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                    (
                        s("text"),
                        ftd::component::Property {
                            default: Some(ftd::PropertyValue::Variable {
                                name: s("name"),
                                kind: ftd::p2::Kind::caption_or_body()
                                    .set_default(Some(s("hello world"))),
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                ])
                .collect(),
                invocations: vec![
                    std::iter::IntoIterator::into_iter([
                        (
                            s("name"),
                            ftd::Value::String {
                                text: s("hello world"),
                                source: ftd::TextSource::Default,
                            },
                        ),
                        (s("line-clamp"), ftd::Value::Integer { value: 10 }),
                    ])
                    .collect(),
                    std::iter::IntoIterator::into_iter([
                        (
                            s("name"),
                            ftd::Value::String {
                                text: s("hello"),
                                source: ftd::TextSource::Caption,
                            },
                        ),
                        (s("line-clamp"), ftd::Value::Integer { value: 10 }),
                    ])
                    .collect(),
                    std::iter::IntoIterator::into_iter([
                        (
                            s("name"),
                            ftd::Value::String {
                                text: s("this is nice"),
                                source: ftd::TextSource::Caption,
                            },
                        ),
                        (s("line-clamp"), ftd::Value::Integer { value: 20 }),
                    ])
                    .collect(),
                ],
                line_number: 1,
                ..Default::default()
            }),
        );
        bag.insert(
            s("foo/bar#name@0"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("name"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("hello world"),
                        source: ftd::TextSource::Default,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#name@1"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("name"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("hello"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#name@2"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("name"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("this is nice"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#line-clamp@0"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("line-clamp"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 10 },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#line-clamp@1"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("line-clamp"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 10 },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#line-clamp@2"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("line-clamp"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 20 },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        let (g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.text foo:
                caption name: hello world
                integer line-clamp: 10
                text: $name
                line-clamp: $line-clamp

                -- foo:

                -- foo: hello

                -- foo: this is nice
                line-clamp: 20
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn record_with_default_value() {
        let mut bag = super::default_bag();

        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@0", -1, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT@0", 0, &mut bag);
        insert_integer_by_root("foo/bar#SIBLING-INDEX-0@0", 0, &mut bag);
        insert_integer_by_root("foo/bar#SIBLING-INDEX@0", 1, &mut bag);

        bag.insert(
            s("foo/bar#abrar"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("abrar"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("foo/bar#person"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("address"),
                                ftd::PropertyValue::Value {
                                    value: ftd::variable::Value::String {
                                        text: s("Bihar"),
                                        source: ftd::TextSource::Default,
                                    },
                                },
                            ),
                            (
                                s("age"),
                                ftd::PropertyValue::Reference {
                                    name: s("foo/bar#default-age"),
                                    kind: ftd::p2::Kind::Integer {
                                        default: Some(s("$foo/bar#default-age")),
                                        is_reference: false,
                                    },
                                },
                            ),
                            (
                                s("bio"),
                                ftd::PropertyValue::Value {
                                    value: ftd::variable::Value::String {
                                        text: s("Software developer working at fifthtry."),
                                        source: ftd::TextSource::Body,
                                    },
                                },
                            ),
                            (
                                s("name"),
                                ftd::PropertyValue::Reference {
                                    name: s("foo/bar#abrar-name"),
                                    kind: ftd::p2::Kind::caption(),
                                },
                            ),
                            (
                                s("size"),
                                ftd::PropertyValue::Value {
                                    value: ftd::variable::Value::Integer { value: 10 },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#abrar-name"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("abrar-name"),
                value: ftd::PropertyValue::Value {
                    value: ftd::variable::Value::String {
                        text: s("Abrar Khan"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#default-age"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("default-age"),
                value: ftd::PropertyValue::Value {
                    value: ftd::variable::Value::Integer { value: 20 },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#person"),
            ftd::p2::Thing::Record(ftd::p2::Record {
                name: s("foo/bar#person"),
                fields: std::iter::IntoIterator::into_iter([
                    (
                        s("address"),
                        ftd::p2::Kind::string().set_default(Some(s("Bihar"))),
                    ),
                    (
                        s("age"),
                        ftd::p2::Kind::Integer {
                            default: Some(s("$foo/bar#default-age")),
                            is_reference: false,
                        },
                    ),
                    (
                        s("bio"),
                        ftd::p2::Kind::body().set_default(Some(s("Some Bio"))),
                    ),
                    (s("name"), ftd::p2::Kind::caption()),
                    (
                        s("size"),
                        ftd::p2::Kind::Integer {
                            default: Some(s("10")),
                            is_reference: false,
                        },
                    ),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![s("name"), s("address"), s("bio"), s("age"), s("size")],
            }),
        );

        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("Software developer working at fifthtry."),
                line: true,
                line_clamp: Some(20),
                common: ftd::Common {
                    reference: Some(s("foo/bar#abrar.bio")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- integer default-age: 20

                -- record person:
                caption name:
                string address: Bihar
                body bio: Some Bio
                integer age: $default-age
                integer size: 10

                -- string abrar-name: Abrar Khan

                -- person abrar: $abrar-name

                Software developer working at fifthtry.

                -- ftd.text: $abrar.bio
                line-clamp: $abrar.age
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn default_with_reference() {
        let mut main = super::default_column();
        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![ftd::Element::Markup(ftd::Markups {
                    text: ftd::markdown_line("Arpita"),
                    line: true,
                    line_clamp: Some(10),
                    common: ftd::Common {
                        reference: Some(s("foo/bar#name@0")),
                        ..Default::default()
                    },
                    ..Default::default()
                })],
                ..Default::default()
            },
            ..Default::default()
        }));
        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![ftd::Element::Markup(ftd::Markups {
                    text: ftd::markdown_line("Amit Upadhyay"),
                    line: true,
                    line_clamp: Some(20),
                    common: ftd::Common {
                        reference: Some(s("foo/bar#name@1")),
                        ..Default::default()
                    },
                    ..Default::default()
                })],
                ..Default::default()
            },
            ..Default::default()
        }));

        let mut bag = super::default_bag();
        bag.insert(
            s("foo/bar#default-name"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("default-name"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Arpita"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#default-size"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("default-size"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 10 },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#foo"),
            ftd::p2::Thing::Component(ftd::Component {
                root: s("ftd#row"),
                full_name: s("foo/bar#foo"),
                arguments: std::iter::IntoIterator::into_iter([
                    (
                        s("name"),
                        ftd::p2::Kind::string().set_default(Some(s("$foo/bar#default-name"))),
                    ),
                    (
                        s("text-size"),
                        ftd::p2::Kind::Integer {
                            default: Some(s("$foo/bar#default-size")),
                            is_reference: false,
                        },
                    ),
                ])
                .collect(),
                instructions: vec![ftd::Instruction::ChildComponent {
                    child: ftd::ChildComponent {
                        events: vec![],
                        root: s("ftd#text"),
                        condition: None,
                        properties: std::iter::IntoIterator::into_iter([
                            (
                                s("line-clamp"),
                                ftd::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: s("text-size"),
                                        kind: ftd::p2::Kind::Optional {
                                            kind: Box::new(ftd::p2::Kind::Integer {
                                                default: Some(s("$foo/bar#default-size")),
                                                is_reference: false,
                                            }),
                                            is_reference: false,
                                        },
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            ),
                            (
                                s("text"),
                                ftd::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: s("name"),
                                        kind: ftd::p2::Kind::caption_or_body()
                                            .set_default(Some(s("$foo/bar#default-name"))),
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            ),
                        ])
                        .collect(),
                        ..Default::default()
                    },
                }],
                kernel: false,
                ..Default::default()
            }),
        );
        bag.insert(
            s("foo/bar#name@0"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("name"),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#default-name"),
                    kind: ftd::p2::Kind::string().set_default(Some(s("$foo/bar#default-name"))),
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#name@1"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("name"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Amit Upadhyay"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#text-size@0"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("text-size"),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#default-size"),
                    kind: ftd::p2::Kind::integer().set_default(Some(s("$foo/bar#default-size"))),
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#text-size@1"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("text-size"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 20 },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        p!(
            "
            -- string default-name: Arpita

            -- integer default-size: 10

            -- ftd.row foo:
            string name: $default-name
            integer text-size: $default-size

            --- ftd.text: $name
            line-clamp: $text-size

            -- foo:

            -- foo:
            name: Amit Upadhyay
            text-size: 20
            ",
            (bag, main),
        );
    }

    #[test]
    fn or_type_with_default_value() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("Amit Upadhyay"),
                line: true,
                common: ftd::Common {
                    reference: Some(s("foo/bar#amitu.name")),
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("1000"),
                line: true,
                common: ftd::Common {
                    reference: Some(s("foo/bar#amitu.phone")),
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("John Doe"),
                line: true,
                line_clamp: Some(50),
                common: ftd::Common {
                    reference: Some(s("foo/bar#acme.contact")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let mut bag = super::default_bag();

        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@0", -1, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@1", -1, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@2", -1, &mut bag);

        insert_integer_by_root("foo/bar#CHILDREN-COUNT@0", 0, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT@1", 0, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT@2", 0, &mut bag);

        insert_integer_by_root("foo/bar#SIBLING-INDEX-0@0", 0, &mut bag);
        insert_integer_by_root("foo/bar#SIBLING-INDEX-0@1", 1, &mut bag);
        insert_integer_by_root("foo/bar#SIBLING-INDEX-0@2", 2, &mut bag);

        insert_integer_by_root("foo/bar#SIBLING-INDEX@0", 1, &mut bag);
        insert_integer_by_root("foo/bar#SIBLING-INDEX@1", 2, &mut bag);
        insert_integer_by_root("foo/bar#SIBLING-INDEX@2", 3, &mut bag);

        bag.insert(
            s("foo/bar#acme"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("acme"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::OrType {
                        name: s("foo/bar#lead"),
                        variant: s("company"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("contact"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("John Doe"),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("fax"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("+1-234-567890"),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("name"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("Acme Inc."),
                                        source: ftd::TextSource::Caption,
                                    },
                                },
                            ),
                            (
                                s("no-of-employees"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::Integer { value: 50 },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#amitu"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("amitu"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::OrType {
                        name: s("foo/bar#lead"),
                        variant: s("individual"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("name"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("Amit Upadhyay"),
                                        source: ftd::TextSource::Caption,
                                    },
                                },
                            ),
                            (
                                s("phone"),
                                ftd::PropertyValue::Reference {
                                    name: s("foo/bar#default-phone"),
                                    kind: ftd::p2::Kind::string()
                                        .set_default(Some(s("$foo/bar#default-phone"))),
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#default-phone"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("default-phone"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("1000"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#lead"),
            ftd::p2::Thing::OrType(ftd::OrType {
                name: s("foo/bar#lead"),
                variants: vec![
                    ftd::p2::Record {
                        name: s("foo/bar#lead.individual"),
                        fields: std::iter::IntoIterator::into_iter([
                            (s("name"), ftd::p2::Kind::caption()),
                            (
                                s("phone"),
                                ftd::p2::Kind::string()
                                    .set_default(Some(s("$foo/bar#default-phone"))),
                            ),
                        ])
                        .collect(),
                        instances: Default::default(),
                        order: vec![s("name"), s("phone")],
                    },
                    ftd::p2::Record {
                        name: s("foo/bar#lead.company"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("contact"),
                                ftd::p2::Kind::string().set_default(Some(s("1001"))),
                            ),
                            (s("fax"), ftd::p2::Kind::string()),
                            (s("name"), ftd::p2::Kind::caption()),
                            (
                                s("no-of-employees"),
                                ftd::p2::Kind::integer().set_default(Some(s("50"))),
                            ),
                        ])
                        .collect(),
                        instances: Default::default(),
                        order: vec![s("name"), s("contact"), s("fax"), s("no-of-employees")],
                    },
                ],
            }),
        );

        let (g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- string default-phone: 1000

                -- or-type lead:

                --- individual:
                caption name:
                string phone: $default-phone

                --- company:
                caption name:
                string contact: 1001
                string fax:
                integer no-of-employees: 50

                -- lead.individual amitu: Amit Upadhyay

                -- lead.company acme: Acme Inc.
                contact: John Doe
                fax: +1-234-567890

                -- ftd.text: $amitu.name

                -- ftd.text: $amitu.phone

                -- ftd.text: $acme.contact
                line-clamp: $acme.no-of-employees

                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn default_id() {
        let mut main = super::default_column();

        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Column(ftd::Column {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![ftd::Element::Row(ftd::Row {
                                spacing: None,
                                container: ftd::Container {
                                    children: vec![ftd::Element::Column(ftd::Column {
                                        spacing: None,
                                        container: ftd::Container {
                                            children: vec![ftd::Element::Markup(ftd::Markups {
                                                text: ftd::markdown_line("hello"),
                                                line: true,
                                                ..Default::default()
                                            })],
                                            ..Default::default()
                                        },
                                        common: ftd::Common {
                                            data_id: Some(s("display-text-id")),
                                            ..Default::default()
                                        },
                                    })],
                                    ..Default::default()
                                },
                                ..Default::default()
                            })],
                            ..Default::default()
                        },
                        common: ftd::Common {
                            data_id: Some(s("inside-page-id")),
                            ..Default::default()
                        },
                    })],
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![ftd::Element::Row(ftd::Row {
                                    spacing: None,
                                    container: ftd::Container {
                                        children: vec![ftd::Element::Column(ftd::Column {
                                            spacing: None,
                                            container: ftd::Container {
                                                children: vec![ftd::Element::Markup(
                                                    ftd::Markups {
                                                        text: ftd::markdown_line("hello"),
                                                        line: true,
                                                        ..Default::default()
                                                    },
                                                )],
                                                ..Default::default()
                                            },
                                            common: ftd::Common {
                                                data_id: Some(s("display-text-id")),
                                                id: Some(s(
                                                    "page-id:inside-page-id:display-text-id",
                                                )),
                                                ..Default::default()
                                            },
                                        })],
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            common: ftd::Common {
                                data_id: Some(s("inside-page-id")),
                                id: Some(s("page-id:inside-page-id")),
                                ..Default::default()
                            },
                        }),
                        ftd::Element::Row(ftd::Row {
                            spacing: None,
                            common: ftd::Common {
                                data_id: Some(s("page-id-row")),
                                id: Some(s("page-id-row")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                common: ftd::Common {
                    data_id: Some(s("page-id")),
                    id: Some(s("page-id")),
                    ..Default::default()
                },
            }));

        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            ..Default::default()
        }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.column display-text:

                --- ftd.text: hello


                -- ftd.column inside-page:

                --- ftd.row:

                --- display-text:
                id: display-text-id


                -- ftd.column page:

                --- inside-page:
                id: inside-page-id

                -- page:

                -- page:
                id: page-id

                -- ftd.row:

                -- container: page-id

                -- ftd.row:
                id: page-id-row

                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    #[ignore]
    fn region_h1() {
        let mut main = super::default_column();

        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("Heading 31"),
                        line: true,
                        common: ftd::Common {
                            region: Some(ftd::Region::Title),
                            reference: Some(s("foo/bar#title@0")),
                            ..Default::default()
                        },
                        ..Default::default()
                    })],
                    ..Default::default()
                },
                common: ftd::Common {
                    region: Some(ftd::Region::H3),
                    id: Some(s("heading-31")),
                    ..Default::default()
                },
            }));

        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("Heading 11"),
                            line: true,
                            common: ftd::Common {
                                region: Some(ftd::Region::Title),
                                reference: Some(s("foo/bar#title@1")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![
                                    ftd::Element::Markup(ftd::Markups {
                                        text: ftd::markdown_line("Heading 21"),
                                        line: true,
                                        common: ftd::Common {
                                            region: Some(ftd::Region::Title),
                                            reference: Some(s("foo/bar#title@2")),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    }),
                                    ftd::Element::Column(ftd::Column {
                                        spacing: None,
                                        container: ftd::Container {
                                            children: vec![
                                                ftd::Element::Markup(ftd::Markups {
                                                    text: ftd::markdown_line("Heading 32"),
                                                    line: true,
                                                    common: ftd::Common {
                                                        region: Some(ftd::Region::Title),
                                                        reference: Some(s("foo/bar#title@3")),
                                                        ..Default::default()
                                                    },
                                                    ..Default::default()
                                                }),
                                                ftd::Element::Markup(ftd::Markups {
                                                    text: ftd::markdown_line("hello"),
                                                    line: true,
                                                    ..Default::default()
                                                }),
                                            ],
                                            ..Default::default()
                                        },
                                        common: ftd::Common {
                                            region: Some(ftd::Region::H3),
                                            id: Some(s("heading-32")),
                                            ..Default::default()
                                        },
                                    }),
                                ],
                                ..Default::default()
                            },
                            common: ftd::Common {
                                region: Some(ftd::Region::H2),
                                id: Some(s("heading-21")),
                                ..Default::default()
                            },
                        }),
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![ftd::Element::Markup(ftd::Markups {
                                    text: ftd::markdown_line("Heading 22"),
                                    line: true,
                                    common: ftd::Common {
                                        reference: Some(s("foo/bar#title@5")),
                                        region: Some(ftd::Region::Title),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            common: ftd::Common {
                                region: Some(ftd::Region::H2),
                                id: Some(s("heading-22")),
                                ..Default::default()
                            },
                        }),
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![ftd::Element::Markup(ftd::Markups {
                                    text: ftd::markdown_line("Heading 23"),
                                    line: true,
                                    common: ftd::Common {
                                        region: Some(ftd::Region::Title),
                                        reference: Some(s("foo/bar#title@6")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            common: ftd::Common {
                                region: Some(ftd::Region::H2),
                                id: Some(s("heading-23")),
                                ..Default::default()
                            },
                        }),
                    ],
                    ..Default::default()
                },
                common: ftd::Common {
                    region: Some(ftd::Region::H1),
                    id: Some(s("heading-11")),
                    ..Default::default()
                },
            }));

        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("Heading 12"),
                            line: true,
                            common: ftd::Common {
                                reference: Some(s("foo/bar#title@7")),
                                region: Some(ftd::Region::Title),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![ftd::Element::Markup(ftd::Markups {
                                    text: ftd::markdown_line("Heading 33"),
                                    line: true,
                                    common: ftd::Common {
                                        reference: Some(s("foo/bar#title@8")),
                                        region: Some(ftd::Region::Title),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            common: ftd::Common {
                                region: Some(ftd::Region::H3),
                                id: Some(s("heading-33")),
                                ..Default::default()
                            },
                        }),
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![ftd::Element::Markup(ftd::Markups {
                                    text: ftd::markdown_line("Heading 24"),
                                    line: true,
                                    common: ftd::Common {
                                        reference: Some(s("foo/bar#title@9")),
                                        region: Some(ftd::Region::Title),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            common: ftd::Common {
                                region: Some(ftd::Region::H2),
                                id: Some(s("heading-24")),
                                ..Default::default()
                            },
                        }),
                    ],
                    ..Default::default()
                },
                common: ftd::Common {
                    region: Some(ftd::Region::H1),
                    id: Some(s("heading-12")),
                    ..Default::default()
                },
            }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.column h1:
                region: h1
                caption title:

                --- ftd.text:
                text: $title
                caption title:
                region: title

                -- ftd.column h2:
                region: h2
                caption title:

                --- ftd.text:
                text: $title
                caption title:
                region: title

                -- ftd.column h3:
                region: h3
                caption title:

                --- ftd.text:
                text: $title
                caption title:
                region: title

                -- h3: Heading 31

                -- h1: Heading 11

                -- h2: Heading 21

                -- h3: Heading 32

                -- ftd.text: hello

                -- h2: Heading 22

                -- h2: Heading 23

                -- h1: Heading 12

                -- h3: Heading 33

                -- h2: Heading 24

                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn event_onclick() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("Mobile"),
                            line: true,
                            common: ftd::Common {
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#mobile"),
                                    value: serde_json::Value::Bool(true),
                                }),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("Desktop"),
                            line: true,
                            common: ftd::Common {
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#mobile"),
                                    value: serde_json::Value::Bool(false),
                                }),
                                is_not_visible: true,
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("Click Here!"),
                line: true,
                common: ftd::Common {
                    events: vec![ftd::Event {
                        name: s("onclick"),
                        action: ftd::Action {
                            action: s("toggle"),
                            target: s("foo/bar#mobile"),
                            parameters: Default::default(),
                        },
                    }],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- boolean mobile: true

                -- ftd.column foo:

                --- ftd.text: Mobile
                if: $mobile

                --- ftd.text: Desktop
                if: not $mobile

                -- foo:

                -- ftd.text: Click Here!
                $on-click$: toggle $mobile
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn event_toggle_with_local_variable() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("Hello"),
                line: true,
                common: ftd::Common {
                    reference: Some(s("foo/bar#name@0")),
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#open@0"),
                        value: serde_json::Value::Bool(true),
                    }),
                    events: vec![ftd::Event {
                        name: s("onclick"),
                        action: ftd::Action {
                            action: s("toggle"),
                            target: s("foo/bar#open@0"),
                            parameters: Default::default(),
                        },
                    }],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let mut bag = super::default_bag();

        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@0", -1, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT@0", 0, &mut bag);
        insert_integer_by_root("foo/bar#SIBLING-INDEX-0@0", 0, &mut bag);
        insert_integer_by_root("foo/bar#SIBLING-INDEX@0", 1, &mut bag);

        bag.insert(
            s("foo/bar#foo"),
            ftd::p2::Thing::Component(ftd::Component {
                root: "ftd#text".to_string(),
                full_name: "foo/bar#foo".to_string(),
                arguments: std::iter::IntoIterator::into_iter([
                    (s("name"), ftd::p2::Kind::caption()),
                    (
                        s("open"),
                        ftd::p2::Kind::boolean().set_default(Some(s("true"))),
                    ),
                ])
                .collect(),
                properties: std::iter::IntoIterator::into_iter([(
                    s("text"),
                    ftd::component::Property {
                        default: Some(ftd::PropertyValue::Variable {
                            name: s("name"),
                            kind: ftd::p2::Kind::String {
                                caption: true,
                                body: true,
                                default: None,
                                is_reference: false,
                            },
                        }),
                        ..Default::default()
                    },
                )])
                .collect(),
                instructions: vec![],
                events: vec![ftd::p2::Event {
                    name: ftd::p2::EventName::OnClick,
                    action: ftd::p2::Action {
                        action: ftd::p2::ActionKind::Toggle,
                        target: ftd::PropertyValue::Variable {
                            name: s("open"),
                            kind: ftd::p2::Kind::boolean().set_default(Some(s("true"))),
                        },
                        parameters: Default::default(),
                    },
                }],
                condition: Some(ftd::p2::Boolean::Equal {
                    left: ftd::PropertyValue::Variable {
                        name: s("open"),
                        kind: ftd::p2::Kind::boolean().set_default(Some(s("true"))),
                    },
                    right: ftd::PropertyValue::Value {
                        value: ftd::variable::Value::Boolean { value: true },
                    },
                }),
                kernel: false,
                invocations: vec![std::iter::IntoIterator::into_iter([
                    (
                        s("name"),
                        ftd::Value::String {
                            text: s("Hello"),
                            source: ftd::TextSource::Caption,
                        },
                    ),
                    (s("open"), ftd::Value::Boolean { value: true }),
                ])
                .collect()],
                line_number: 1,
                ..Default::default()
            }),
        );
        bag.insert(
            s("foo/bar#name@0"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("name"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Hello"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#open@0"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("open"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Boolean { value: true },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        let (g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.text foo:
                caption name:
                boolean open: true
                text: $name
                if: $open
                $on-click$: toggle $open

                -- foo: Hello
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
        pretty_assertions::assert_eq!(g_bag, bag);
    }

    #[test]
    fn event_toggle_with_local_variable_for_component() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("Click here"),
                            line: true,
                            common: ftd::Common {
                                events: vec![ftd::Event {
                                    name: s("onclick"),
                                    action: ftd::Action {
                                        action: s("toggle"),
                                        target: s("foo/bar#open@0"),
                                        parameters: Default::default(),
                                    },
                                }],
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("Open True"),
                            line: true,
                            common: ftd::Common {
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#open@0"),
                                    value: serde_json::Value::Bool(true),
                                }),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("Open False"),
                            line: true,
                            common: ftd::Common {
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#open@0"),
                                    value: serde_json::Value::Bool(false),
                                }),
                                is_not_visible: true,
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.column foo:
                boolean open: true

                --- ftd.text: Click here
                $on-click$: toggle $open

                --- ftd.text: Open True
                if: $open

                --- ftd.text: Open False
                if: not $open

                -- foo:
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn event_toggle_for_loop() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("ab title"),
                            line: true,
                            common: ftd::Common {
                                events: vec![ftd::Event {
                                    name: s("onclick"),
                                    action: ftd::Action {
                                        action: s("toggle"),
                                        target: s("foo/bar#open@0"),
                                        parameters: Default::default(),
                                    },
                                }],
                                reference: Some(s("foo/bar#toc@0.title")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![ftd::Element::Markup(ftd::Markups {
                                    text: ftd::markdown_line("aa title"),
                                    line: true,
                                    common: ftd::Common {
                                        events: vec![ftd::Event {
                                            name: s("onclick"),
                                            action: ftd::Action {
                                                action: s("toggle"),
                                                target: s("foo/bar#open@0,1"),
                                                parameters: Default::default(),
                                            },
                                        }],
                                        reference: Some(s("foo/bar#toc@0,1.title")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            common: ftd::Common {
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#open@0"),
                                    value: serde_json::Value::Bool(true),
                                }),
                                ..Default::default()
                            },
                        }),
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![ftd::Element::Markup(ftd::Markups {
                                    text: ftd::markdown_line("aaa title"),
                                    line: true,
                                    common: ftd::Common {
                                        events: vec![ftd::Event {
                                            name: s("onclick"),
                                            action: ftd::Action {
                                                action: s("toggle"),
                                                target: s("foo/bar#open@0,2"),
                                                parameters: Default::default(),
                                            },
                                        }],
                                        reference: Some(s("foo/bar#toc@0,2.title")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            common: ftd::Common {
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#open@0"),
                                    value: serde_json::Value::Bool(true),
                                }),
                                ..Default::default()
                            },
                        }),
                    ],
                    ..Default::default()
                },
                common: ftd::Common {
                    reference: Some(s("foo/bar#toc")),
                    ..Default::default()
                },
            }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- record toc-record:
                string title:
                toc-record list children:

                -- ftd.column toc-item:
                toc-record toc:
                boolean open: true

                --- ftd.text: $toc.title
                $on-click$: toggle $open

                --- toc-item:
                if: $open
                $loop$: $toc.children as $obj
                toc: $obj

                -- toc-record list aa:

                -- aa:
                title: aa title

                -- aa:
                title: aaa title

                -- toc-record list toc:

                -- toc:
                title: ab title
                children: $aa

                -- toc-item:
                $loop$: $toc as $obj
                toc: $obj
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn test_local_variable() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Column(ftd::Column {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![
                                ftd::Element::Column(ftd::Column {
                                    spacing: None,
                                    container: ftd::Container {
                                        children: vec![
                                            ftd::Element::Markup(ftd::Markups {
                                                text: ftd::markdown_line("Click here!"),
                                                line: true,
                                                common: ftd::Common {
                                                    events: vec![ftd::Event {
                                                        name: s("onclick"),
                                                        action: ftd::Action {
                                                            action: s("toggle"),
                                                            target: s("foo/bar#open@0"),
                                                            parameters: Default::default(),
                                                        },
                                                    }],
                                                    ..Default::default()
                                                },
                                                ..Default::default()
                                            }),
                                            ftd::Element::Markup(ftd::Markups {
                                                text: ftd::markdown_line("Hello"),
                                                line: true,
                                                ..Default::default()
                                            }),
                                        ],
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                                ftd::Element::Column(ftd::Column {
                                    spacing: None,
                                    container: ftd::Container {
                                        children: vec![ftd::Element::Markup(ftd::Markups {
                                            text: ftd::markdown_line("Hello Bar"),
                                            line: true,
                                            ..Default::default()
                                        })],
                                        ..Default::default()
                                    },
                                    common: ftd::Common {
                                        condition: Some(ftd::Condition {
                                            variable: s("foo/bar#open@0"),
                                            value: serde_json::Value::Bool(true),
                                        }),
                                        ..Default::default()
                                    },
                                }),
                            ],
                            ..Default::default()
                        },
                        common: ftd::Common {
                            data_id: Some(s("foo-id")),
                            ..Default::default()
                        },
                    })],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.column bar:
                boolean open-bar: true

                --- ftd.text: Hello Bar


                -- ftd.column foo:
                boolean open: true

                --- ftd.column:
                id: foo-id

                --- ftd.column:

                --- ftd.text: Click here!
                $on-click$: toggle $open

                --- ftd.text: Hello

                --- container: foo-id

                --- bar:
                if: $open


                -- foo:
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn if_on_var_integer() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Integer(ftd::Text {
                text: markdown_line("20"),
                common: ftd::Common {
                    reference: Some(s("foo/bar#bar")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- boolean foo: false

                -- integer bar: 10

                -- bar: 20
                if: not $foo

                -- ftd.integer:
                value: $bar

                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn if_on_var_text() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: markdown_line("other-foo says hello"),
                line: true,
                common: ftd::Common {
                    reference: Some(s("foo/bar#bar")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- boolean foo: false

                -- boolean other-foo: true

                -- string bar: hello

                -- bar: foo says hello
                if: not $foo

                -- bar: other-foo says hello
                if: $other-foo

                -- ftd.text: $bar

                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn cursor_pointer() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: markdown_line("hello"),
                line: true,
                common: ftd::Common {
                    cursor: Some(s("pointer")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.text: hello
                cursor: pointer

                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn comments() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("hello2"),
                line: true,
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("/hello3"),
                line: true,
                common: ftd::Common {
                    color: Some(ftd::Color {
                        light: ftd::ColorValue {
                            r: 255,
                            g: 0,
                            b: 0,
                            alpha: 1.0,
                        },
                        dark: ftd::ColorValue {
                            r: 255,
                            g: 0,
                            b: 0,
                            alpha: 1.0,
                        },
                        reference: Some(s("foo/bar#red")),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![ftd::Element::Markup(ftd::Markups {
                    text: ftd::markdown_line("hello5"),
                    line: true,
                    common: ftd::Common {
                        color: Some(ftd::Color {
                            light: ftd::ColorValue {
                                r: 0,
                                g: 128,
                                b: 0,
                                alpha: 1.0,
                            },
                            dark: ftd::ColorValue {
                                r: 0,
                                g: 128,
                                b: 0,
                                alpha: 1.0,
                            },
                            reference: Some(s("foo/bar#green")),
                        }),
                        ..Default::default()
                    },
                    ..Default::default()
                })],
                ..Default::default()
            },
            ..Default::default()
        }));

        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![ftd::Element::Markup(ftd::Markups {
                    text: ftd::markdown_line("/foo says hello"),
                    line: true,
                    ..Default::default()
                })],
                ..Default::default()
            },
            ..Default::default()
        }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                r"
                -- ftd.color red: red
                dark: red
                
                -- ftd.color green: green
                dark: green
                
                /-- ftd.text:
                cursor: pointer

                hello1

                -- ftd.text:
                /color: red

                hello2

                -- ftd.text:
                color: $red

                \/hello3

                -- ftd.row:

                /--- ftd.text: hello4

                --- ftd.text: hello5
                color: $green
                /padding-left: 20

                -- ftd.row foo:
                /color: red

                --- ftd.text:

                \/foo says hello

                /--- ftd.text: foo says hello again

                -- foo:

                /-- foo:
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn component_declaration_anywhere_2() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![
                                    ftd::Element::Markup(ftd::Markups {
                                        text: ftd::markdown_line("Bar says hello"),
                                        line: true,
                                        common: ftd::Common {
                                            reference: Some(s("foo/bar#name@0,0")),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    }),
                                    ftd::Element::Markup(ftd::Markups {
                                        text: ftd::markdown_line("Hello"),
                                        line: true,
                                        common: ftd::Common {
                                            reference: Some(s("foo/bar#greeting")),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    }),
                                ],
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("foo says hello"),
                            line: true,
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("Hello"),
                            line: true,
                            common: ftd::Common {
                                reference: Some(s("foo/bar#greeting")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- foo:

                -- ftd.column foo:

                --- bar: Bar says hello

                --- ftd.text: foo says hello

                --- ftd.text: $greeting

                -- string greeting: Hello

                -- ftd.column bar:
                caption name:

                --- ftd.text: $name

                --- ftd.text: $greeting
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn action_increment_decrement_condition_1() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Integer(ftd::Text {
                text: ftd::markdown_line("0"),
                common: ftd::Common {
                    reference: Some(s("foo/bar#count")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("Hello on 8"),
                line: true,
                common: ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#count"),
                        value: serde_json::Value::from(8),
                    }),
                    is_not_visible: true,
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("increment counter"),
                line: true,
                common: ftd::Common {
                    events: vec![ftd::Event {
                        name: s("onclick"),
                        action: ftd::Action {
                            action: s("increment"),
                            target: s("foo/bar#count"),
                            parameters: Default::default(),
                        },
                    }],
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("decrement counter"),
                line: true,
                common: ftd::Common {
                    events: vec![ftd::Event {
                        name: s("onclick"),
                        action: ftd::Action {
                            action: s("decrement"),
                            target: s("foo/bar#count"),
                            parameters: Default::default(),
                        },
                    }],
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("increment counter"),
                line: true,
                common: ftd::Common {
                    events: vec![ftd::Event {
                        name: s("onclick"),
                        action: ftd::Action {
                            action: s("increment"),
                            target: s("foo/bar#count"),
                            parameters: std::iter::IntoIterator::into_iter([(
                                s("by"),
                                vec![ftd::event::ParameterData {
                                    value: serde_json::Value::from(2),
                                    reference: None,
                                }],
                            )])
                            .collect(),
                        },
                    }],
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("increment counter by 2 clamp 2 10"),
                line: true,
                common: ftd::Common {
                    events: vec![ftd::Event {
                        name: s("onclick"),
                        action: ftd::Action {
                            action: s("increment"),
                            target: s("foo/bar#count"),
                            parameters: std::iter::IntoIterator::into_iter([
                                (
                                    s("by"),
                                    vec![ftd::event::ParameterData {
                                        value: serde_json::Value::from(2),
                                        reference: None,
                                    }],
                                ),
                                (
                                    s("clamp"),
                                    vec![
                                        ftd::event::ParameterData {
                                            value: serde_json::Value::from(2),
                                            reference: None,
                                        },
                                        ftd::event::ParameterData {
                                            value: serde_json::Value::from(10),
                                            reference: None,
                                        },
                                    ],
                                ),
                            ])
                            .collect(),
                        },
                    }],
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("decrement count clamp 2 10"),
                line: true,
                common: ftd::Common {
                    events: vec![ftd::Event {
                        name: s("onclick"),
                        action: ftd::Action {
                            action: s("decrement"),
                            target: s("foo/bar#count"),
                            parameters: std::iter::IntoIterator::into_iter([(
                                s("clamp"),
                                vec![
                                    ftd::event::ParameterData {
                                        value: serde_json::Value::from(2),
                                        reference: None,
                                    },
                                    ftd::event::ParameterData {
                                        value: serde_json::Value::from(10),
                                        reference: None,
                                    },
                                ],
                            )])
                            .collect(),
                        },
                    }],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- integer count: 0

                -- ftd.integer:
                value: $count

                -- ftd.text: Hello on 8
                if: $count == 8

                -- ftd.text: increment counter
                $on-click$: increment $count

                -- ftd.text: decrement counter
                $on-click$: decrement $count

                -- ftd.text: increment counter
                $on-click$: increment $count by 2

                -- ftd.text: increment counter by 2 clamp 2 10
                $on-click$: increment $count by 2 clamp 2 10

                -- ftd.text: decrement count clamp 2 10
                $on-click$: decrement $count clamp 2 10
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn action_increment_decrement_local_variable() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Integer(ftd::Text {
                            text: ftd::markdown_line("0"),
                            common: ftd::Common {
                                reference: Some(s("foo/bar#count@0")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("increment counter"),
                            line: true,
                            common: ftd::Common {
                                events: vec![ftd::Event {
                                    name: s("onclick"),
                                    action: ftd::Action {
                                        action: s("increment"),
                                        target: s("foo/bar#count@0"),
                                        parameters: std::iter::IntoIterator::into_iter([(
                                            s("by"),
                                            vec![ftd::event::ParameterData {
                                                value: serde_json::Value::from(3),
                                                reference: Some(s("foo/bar#by@0")),
                                            }],
                                        )])
                                        .collect(),
                                    },
                                }],
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("decrement counter"),
                            line: true,
                            common: ftd::Common {
                                events: vec![ftd::Event {
                                    name: s("onclick"),
                                    action: ftd::Action {
                                        action: s("decrement"),
                                        target: s("foo/bar#count@0"),
                                        parameters: std::iter::IntoIterator::into_iter([(
                                            s("by"),
                                            vec![ftd::event::ParameterData {
                                                value: serde_json::Value::from(2),
                                                reference: Some(s("foo/bar#decrement-by")),
                                            }],
                                        )])
                                        .collect(),
                                    },
                                }],
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- integer decrement-by: 2

                -- ftd.column foo:
                integer by: 4
                integer count: 0

                --- ftd.integer:
                value: $count

                --- ftd.text: increment counter
                $on-click$: increment $count by $by

                --- ftd.text: decrement counter
                $on-click$: decrement $count by $decrement-by

                -- foo:
                by: 3

                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn nested_component() {
        let mut main = super::default_column();
        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![ftd::Element::Markup(ftd::Markups {
                    text: ftd::markup_line("CTA says Hello"),
                    line: true,
                    common: ftd::Common {
                        reference: Some(s("foo/bar#cta@0")),
                        ..Default::default()
                    },
                    ..Default::default()
                })],
                ..Default::default()
            },
            ..Default::default()
        }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- secondary-button: CTA says Hello

                -- secondary-button-1 secondary-button:
                caption cta:
                cta: $cta


                -- ftd.row secondary-button-1:
                caption cta:

                --- ftd.text: $cta
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn action_increment_decrement_on_component() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Image(ftd::Image {
                src: i("https://www.liveabout.com/thmb/YCJmu1khSJo8kMYM090QCd9W78U=/1250x0/filters:no_upscale():max_bytes(150000):strip_icc():format(webp)/powerpuff_girls-56a00bc45f9b58eba4aea61d.jpg", Some(s("foo/bar#src@0"))),
                common: ftd::Common {
                    condition: Some(
                        ftd::Condition {
                            variable: s("foo/bar#count"),
                            value: serde_json::Value::from(0),
                        },
                    ),
                    is_not_visible: false,
                    events: vec![
                        ftd::Event {
                            name: s("onclick"),
                            action: ftd::Action {
                                action: s("increment"),
                                target: s("foo/bar#count"),
                                parameters: std::iter::IntoIterator::into_iter([(s("clamp"), vec![ftd::event::ParameterData {
                                    value: serde_json::Value::from(0),
                                    reference: None,
                                }, ftd::event::ParameterData {
                                    value: serde_json::Value::from(1),
                                    reference: None,
                                }])])
                                    .collect(),
                            },
                        },
                    ],
                    reference: Some(s("foo/bar#src@0")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Image(ftd::Image {
                src: i(
                    "https://upload.wikimedia.org/wikipedia/en/d/d4/Mickey_Mouse.png",
                    Some(s("foo/bar#src@1")),
                ),
                common: ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#count"),
                        value: serde_json::Value::from(1),
                    }),
                    is_not_visible: true,
                    events: vec![ftd::Event {
                        name: s("onclick"),
                        action: ftd::Action {
                            action: s("increment"),
                            target: s("foo/bar#count"),
                            parameters: std::iter::IntoIterator::into_iter([(
                                s("clamp"),
                                vec![
                                    ftd::event::ParameterData {
                                        value: serde_json::Value::from(0),
                                        reference: None,
                                    },
                                    ftd::event::ParameterData {
                                        value: serde_json::Value::from(1),
                                        reference: None,
                                    },
                                ],
                            )])
                            .collect(),
                        },
                    }],
                    reference: Some(s("foo/bar#src@1")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- integer count: 0

                -- ftd.image-src src0: 
                light: https://www.liveabout.com/thmb/YCJmu1khSJo8kMYM090QCd9W78U=/1250x0/filters:no_upscale():max_bytes(150000):strip_icc():format(webp)/powerpuff_girls-56a00bc45f9b58eba4aea61d.jpg
                dark: https://www.liveabout.com/thmb/YCJmu1khSJo8kMYM090QCd9W78U=/1250x0/filters:no_upscale():max_bytes(150000):strip_icc():format(webp)/powerpuff_girls-56a00bc45f9b58eba4aea61d.jpg

                -- ftd.image-src src1: 
                light: https://upload.wikimedia.org/wikipedia/en/d/d4/Mickey_Mouse.png
                dark: https://upload.wikimedia.org/wikipedia/en/d/d4/Mickey_Mouse.png

                -- ftd.image slide:
                ftd.image-src src:
                integer idx:
                src: $src
                if: $count == $idx
                $on-click$: increment $count clamp 0 1

                -- slide:
                src: $src0
                idx: 0

                -- slide:
                src: $src1
                idx: 1
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn loop_on_list_string() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("Arpita"),
                            line: true,
                            common: ftd::Common {
                                reference: Some(s("foo/bar#$loop$@0,0")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("Ayushi"),
                            line: true,
                            common: ftd::Common {
                                reference: Some(s("foo/bar#$loop$@0,1")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("AmitU"),
                            line: true,
                            common: ftd::Common {
                                reference: Some(s("foo/bar#$loop$@0,2")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.column foo:
                string list bar:

                --- ftd.text: $obj
                $loop$: $bar as $obj

                -- string list names:

                -- names: Arpita

                -- names: Ayushi

                -- names: AmitU

                -- foo:
                bar: $names
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn open_container_with_parent_id() {
        let mut main = super::default_column();
        let beverage_external_children = vec![ftd::Element::Column(ftd::Column {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Column(ftd::Column {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![
                                ftd::Element::Markup(ftd::Markups {
                                    text: ftd::markdown_line("Water"),
                                    line: true,
                                    common: ftd::Common {
                                        events: vec![ftd::Event {
                                            name: s("onclick"),
                                            action: ftd::Action {
                                                action: s("toggle"),
                                                target: s("foo/bar#visible@0,0,2"),
                                                ..Default::default()
                                            },
                                        }],
                                        reference: Some(s("foo/bar#name@0,0,2")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                                ftd::Element::Column(ftd::Column {
                                    spacing: None,
                                    common: ftd::Common {
                                        condition: Some(ftd::Condition {
                                            variable: s("foo/bar#visible@0,0,2"),
                                            value: serde_json::Value::Bool(true),
                                        }),
                                        data_id: Some(s("some-child")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                            ],
                            external_children: Some((s("some-child"), vec![vec![1]], vec![])),
                            open: Some(true),
                            append_at: Some(s("some-child")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Column(ftd::Column {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![
                                ftd::Element::Markup(ftd::Markups {
                                    text: ftd::markdown_line("Juice"),
                                    line: true,
                                    common: ftd::Common {
                                        events: vec![ftd::Event {
                                            name: s("onclick"),
                                            action: ftd::Action {
                                                action: s("toggle"),
                                                target: s("foo/bar#visible@0,0,3"),
                                                ..Default::default()
                                            },
                                        }],
                                        reference: Some(s("foo/bar#name@0,0,3")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                                ftd::Element::Column(ftd::Column {
                                    spacing: None,
                                    common: ftd::Common {
                                        condition: Some(ftd::Condition {
                                            variable: s("foo/bar#visible@0,0,3"),
                                            value: serde_json::Value::Bool(true),
                                        }),
                                        data_id: Some(s("some-child")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                            ],
                            external_children: Some((
                                s("some-child"),
                                vec![vec![1]],
                                vec![ftd::Element::Column(ftd::Column {
                                    spacing: None,
                                    container: ftd::Container {
                                        children: vec![ftd::Element::Column(ftd::Column {
                                            spacing: None,
                                            container: ftd::Container {
                                                children: vec![
                                                    ftd::Element::Markup(ftd::Markups {
                                                        text: ftd::markdown_line("Mango Juice"),
                                                        line: true,
                                                        common: ftd::Common {
                                                            events: vec![ftd::Event {
                                                                name: s("onclick"),
                                                                action: ftd::Action {
                                                                    action: s("toggle"),
                                                                    target: s(
                                                                        "foo/bar#visible@0,0,1,2",
                                                                    ),
                                                                    ..Default::default()
                                                                },
                                                            }],
                                                            reference: Some(s(
                                                                "foo/bar#name@0,0,1,2",
                                                            )),
                                                            ..Default::default()
                                                        },
                                                        ..Default::default()
                                                    }),
                                                    ftd::Element::Column(ftd::Column {
                                                        spacing: None,
                                                        common: ftd::Common {
                                                            condition: Some(ftd::Condition {
                                                                variable: s(
                                                                    "foo/bar#visible@0,0,1,2",
                                                                ),
                                                                value: serde_json::Value::Bool(
                                                                    true,
                                                                ),
                                                            }),
                                                            data_id: Some(s("some-child")),
                                                            ..Default::default()
                                                        },
                                                        ..Default::default()
                                                    }),
                                                ],
                                                external_children: Some((
                                                    s("some-child"),
                                                    vec![vec![1]],
                                                    vec![],
                                                )),
                                                open: Some(true),
                                                append_at: Some(s("some-child")),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        })],
                                        ..Default::default()
                                    },
                                    common: ftd::Common {
                                        width: Some(ftd::Length::Fill),
                                        height: Some(ftd::Length::Fill),
                                        position: Some(ftd::Position::Center),
                                        ..Default::default()
                                    },
                                })],
                            )),
                            open: Some(true),
                            append_at: Some(s("some-child")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            common: ftd::Common {
                width: Some(ftd::Length::Fill),
                height: Some(ftd::Length::Fill),
                position: Some(ftd::Position::Center),
                ..Default::default()
            },
        })];

        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Column(ftd::Column {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![
                                ftd::Element::Markup(ftd::Markups {
                                    text: ftd::markdown_line("Beverage"),
                                    line: true,
                                    common: ftd::Common {
                                        events: vec![ftd::Event {
                                            name: s("onclick"),
                                            action: ftd::Action {
                                                action: s("toggle"),
                                                target: s("foo/bar#visible@0,0"),
                                                ..Default::default()
                                            },
                                        }],
                                        reference: Some(s("foo/bar#name@0,0")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                                ftd::Element::Column(ftd::Column {
                                    spacing: None,
                                    common: ftd::Common {
                                        condition: Some(ftd::Condition {
                                            variable: s("foo/bar#visible@0,0"),
                                            value: serde_json::Value::Bool(true),
                                        }),
                                        data_id: Some(s("some-child")),
                                        id: Some(s("beverage:some-child")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                            ],
                            external_children: Some((
                                s("some-child"),
                                vec![vec![1]],
                                beverage_external_children,
                            )),
                            open: Some(true),
                            append_at: Some(s("some-child")),
                            ..Default::default()
                        },
                        common: ftd::Common {
                            data_id: Some(s("beverage")),
                            id: Some(s("beverage")),
                            ..Default::default()
                        },
                    })],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
            -- ftd.column display-item1:
            string name:
            open: true
            append-at: some-child
            boolean visible: true

            --- ftd.text: $name
            $on-click$: toggle $visible

            --- ftd.column:
            if: $visible
            id: some-child

            -- ftd.column:

            -- display-item1:
            name: Beverage
            id: beverage


            -- display-item1:
            name: Water


            -- container: beverage


            -- display-item1:
            name: Juice


            -- display-item1:
            name: Mango Juice
            "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn text_check() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("$hello"),
                            line: true,
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("hello"),
                            line: true,
                            common: ftd::Common {
                                reference: Some(s("foo/bar#hello2@0")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("hello"),
                            line: true,
                            common: ftd::Common {
                                reference: Some(s("foo/bar#hello")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("hello"),
                            line: true,
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                r"
                -- string hello: hello

                -- ftd.column foo:
                string hello2:

                --- ftd.text: \$hello

                --- ftd.text: $hello2

                --- ftd.text: $hello

                --- ftd.text: hello

                -- foo:
                hello2: $hello
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn caption() {
        let mut main = super::default_column();

        main.container
            .children
            .push(ftd::Element::Integer(ftd::Text {
                text: ftd::markdown_line("32"),
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Boolean(ftd::Text {
                text: ftd::markdown_line("true"),
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Decimal(ftd::Text {
                text: ftd::markdown_line("0.06"),
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.integer: 32

                -- ftd.boolean: true

                -- ftd.decimal: 0.06
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn heading_id() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("Heading 00"),
                            line: true,
                            common: ftd::Common {
                                region: Some(ftd::Region::Title),
                                reference: Some(s("foo/bar#title@0")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("Heading 00 body"),
                            line: true,
                            common: ftd::Common {
                                id: Some(s("one:markdown-id")),
                                data_id: Some(s("markdown-id")),
                                reference: Some(s("foo/bar#body@0,1")),
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#body@0"),
                                    value: serde_json::Value::String(s("$IsNotNull$")),
                                }),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                common: ftd::Common {
                    region: Some(ftd::Region::H0),
                    id: Some(s("one")),
                    data_id: Some(s("one")),
                    ..Default::default()
                },
            }));

        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("Heading 01"),
                            line: true,
                            common: ftd::Common {
                                region: Some(ftd::Region::Title),
                                reference: Some(s("foo/bar#title@1")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("Heading 01 body"),
                            line: true,
                            common: ftd::Common {
                                data_id: Some(s("markdown-id")),
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#body@1"),
                                    value: serde_json::Value::String(s("$IsNotNull$")),
                                }),
                                reference: Some(s("foo/bar#body@1,1")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                common: ftd::Common {
                    region: Some(ftd::Region::H0),
                    id: Some(s("heading-01")),
                    ..Default::default()
                },
            }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- h0: Heading 00
                id: one

                Heading 00 body

                -- h0: Heading 01

                Heading 01 body

                -- ftd.column h0:
                caption title:
                optional body body:
                region: h0

                --- ftd.text:
                text: $title
                region: title

                --- markdown:
                if: $body is not null
                body: $body
                id: markdown-id

                -- ftd.text markdown:
                body body:
                text: $body
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn new_id() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("hello"),
                        line: true,
                        common: ftd::Common {
                            data_id: Some(s("hello")),
                            ..Default::default()
                        },
                        ..Default::default()
                    })],
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("hello"),
                        line: true,
                        common: ftd::Common {
                            data_id: Some(s("hello")),
                            id: Some(s("asd:hello")),
                            ..Default::default()
                        },
                        ..Default::default()
                    })],
                    ..Default::default()
                },
                common: ftd::Common {
                    data_id: Some(s("asd")),
                    id: Some(s("asd")),
                    ..Default::default()
                },
            }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
            --  ftd.column foo:

            --- ftd.text: hello
            id: hello

            -- foo:

            -- foo:
            id: asd
            "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn list_is_empty_check() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("Hello people"),
                line: true,
                ..Default::default()
            }));

        main.container.children.push(ftd::Element::Null);

        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Null,
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("Hello empty list"),
                            line: true,
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("Hello list"),
                            line: true,
                            ..Default::default()
                        }),
                        ftd::Element::Null,
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));
        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- string list people:

                -- people: Ayushi

                -- people: Arpita

                -- ftd.text: Hello people
                if: $people is not empty

                -- ftd.text: Hello nobody
                if: $people is empty


                -- string list empty-list:


                -- ftd.column foo:
                string list string-list:

                --- ftd.text: Hello list
                if: $string-list is not empty

                --- ftd.text: Hello empty list
                if: $string-list is empty

                -- foo:
                string-list: $empty-list

                -- foo:
                string-list: $people
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn parent_with_unsatisfied_condition() {
        let mut main = super::default_column();
        main.container.children.push(ftd::Element::Null);
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("Hello"),
                        line: true,
                        ..Default::default()
                    })],
                    ..Default::default()
                },
                common: ftd::Common {
                    is_not_visible: true,
                    ..Default::default()
                },
            }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- string list empty-list:

                -- ftd.column:
                if: $empty-list is not empty

                --- ftd.text: Hello

                -- foo:

                -- ftd.column foo:
                if: $empty-list is not empty

                --- ftd.text: Hello
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn open_container_id_with_children() {
        let mut external_children = super::default_column();
        external_children
            .container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("hello"),
                line: true,
                ..Default::default()
            }));
        external_children
            .container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("world"),
                line: true,
                ..Default::default()
            }));

        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Column(ftd::Column {
                        spacing: None,
                        common: ftd::Common {
                            id: Some(s("foo-id:some-id")),
                            data_id: Some(s("some-id")),
                            ..Default::default()
                        },
                        ..Default::default()
                    })],
                    external_children: Some((
                        s("some-id"),
                        vec![vec![0]],
                        vec![ftd::Element::Column(external_children)],
                    )),
                    open: Some(true),
                    append_at: Some(s("some-id")),
                    ..Default::default()
                },
                common: ftd::Common {
                    id: Some(s("foo-id")),
                    data_id: Some(s("foo-id")),
                    ..Default::default()
                },
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("Outside"),
                line: true,
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- foo:
                id: foo-id

                --- ftd.text: hello

                --- ftd.text: world

                -- ftd.text: Outside


                -- ftd.column foo:
                open: true
                append-at: some-id

                --- ftd.column:
                id: some-id
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn loop_record_list() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![ftd::Element::Markup(ftd::Markups {
                                    text: ftd::markdown_line("commit message 1"),
                                    line: true,
                                    common: ftd::Common {
                                        reference: Some(s("foo/bar#commit@0,0.message")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![ftd::Element::Markup(ftd::Markups {
                                    text: ftd::markdown_line("commit message 2"),
                                    line: true,
                                    common: ftd::Common {
                                        reference: Some(s("foo/bar#commit@0,1.message")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![ftd::Element::Markup(ftd::Markups {
                                    text: ftd::markdown_line("file filename 1"),
                                    line: true,
                                    common: ftd::Common {
                                        reference: Some(s("foo/bar#file@0,2.filename")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![ftd::Element::Markup(ftd::Markups {
                                    text: ftd::markdown_line("file filename 2"),
                                    line: true,
                                    common: ftd::Common {
                                        reference: Some(s("foo/bar#file@0,3.filename")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- record commit:
                string message:

                -- record file:
                string filename:

                -- record changes:
                commit list commits:
                file list files:


                -- commit list commit-list:

                -- commit-list:
                message: commit message 1

                -- commit-list:
                message: commit message 2


                -- file list file-list:

                -- file-list:
                filename: file filename 1

                -- file-list:
                filename: file filename 2


                -- changes rec-changes:
                commits: $commit-list
                files: $file-list

                -- display:
                changes: $rec-changes


                -- ftd.column display:
                changes changes:

                --- display-commit:
                $loop$: $changes.commits as $obj
                commit: $obj

                --- display-file:
                $loop$: $changes.files as $obj
                file: $obj


                -- ftd.column display-commit:
                commit commit:

                --- ftd.text: $commit.message


                -- ftd.column display-file:
                file file:

                --- ftd.text: $file.filename
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn scene_children_with_default_position() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Scene(ftd::Scene {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("Hello"),
                        line: true,
                        common: ftd::Common {
                            top: Some(0),
                            left: Some(0),
                            ..Default::default()
                        },
                        ..Default::default()
                    }), ftd::Element::Markup(ftd::Markups {
                        text: ftd::markdown_line("World"),
                        line: true,
                        common: ftd::Common {
                            top: Some(10),
                            right: Some(30),
                            scale: Some(1.5),
                            scale_x: Some(-1.0),
                            scale_y: Some(-1.0),
                            rotate: Some(45),
                            position: Some(ftd::Position::Center),
                            ..Default::default()
                        },
                        ..Default::default()
                    })],
                    ..Default::default()
                },
                common: ftd::Common {
                    width: Some(
                        ftd::Length::Px {
                            value: 1000,
                        },
                    ),
                    background_image: Some(
                        i("https://image.shutterstock.com/z/stock-&lt;!&ndash;&ndash;&gt;vector-vector-illustration-of-a-beautiful-summer-landscape-143054302.jpg", Some(s("foo/bar#bg-src"))),
                    ),
                    ..Default::default()
                }
            }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.image-src bg-src: https://image.shutterstock.com/z/stock-&lt;!&ndash;&ndash;&gt;vector-vector-illustration-of-a-beautiful-summer-landscape-143054302.jpg
                dark: https://image.shutterstock.com/z/stock-&lt;!&ndash;&ndash;&gt;vector-vector-illustration-of-a-beautiful-summer-landscape-143054302.jpg

                -- ftd.scene:
                background-image: $bg-src
                width: 1000

                --- ftd.text: Hello

                --- foo:
                top: 10
                right: 30
                align: center
                scale: 1.5
                rotate: 45
                scale-x: -1
                scale-y: -1

                -- ftd.text foo:
                text: World
                "
            ),
            &ftd::p2::TestLibrary {},
        )
            .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn event_set() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("Start..."),
                line: true,
                common: ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#current"),
                        value: serde_json::Value::String(s("some value")),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("some value"),
                line: true,
                common: ftd::Common {
                    reference: Some(s("foo/bar#current")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("change message"),
                line: true,
                common: ftd::Common {
                    events: vec![ftd::Event {
                        name: s("onclick"),
                        action: ftd::Action {
                            action: s("set-value"),
                            target: s("foo/bar#current"),
                            parameters: std::iter::IntoIterator::into_iter([(
                                s("value"),
                                vec![
                                    ftd::event::ParameterData {
                                        value: serde_json::Value::String(s("hello world")),
                                        reference: None,
                                    },
                                    ftd::event::ParameterData {
                                        value: serde_json::Value::String(s("string")),
                                        reference: None,
                                    },
                                ],
                            )])
                            .collect(),
                        },
                    }],
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("change message again"),
                line: true,
                common: ftd::Common {
                    events: vec![ftd::Event {
                        name: s("onclick"),
                        action: ftd::Action {
                            action: s("set-value"),
                            target: s("foo/bar#current"),
                            parameters: std::iter::IntoIterator::into_iter([(
                                s("value"),
                                vec![
                                    ftd::event::ParameterData {
                                        value: serde_json::Value::String(s("good bye")),
                                        reference: Some(s("foo/bar#msg")),
                                    },
                                    ftd::event::ParameterData {
                                        value: serde_json::Value::String(s("string")),
                                        reference: None,
                                    },
                                ],
                            )])
                            .collect(),
                        },
                    }],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- string current: some value

                -- ftd.text: Start...
                if: $current == some value

                -- ftd.text: $current

                -- ftd.text: change message
                $on-click$: $current = hello world

                -- string msg: good bye

                -- ftd.text: change message again
                $on-click$: $current = $msg
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn absolute_positioning() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("hello world"),
                line: true,
                common: ftd::Common {
                    anchor: Some(ftd::Anchor::Parent),
                    right: Some(0),
                    top: Some(100),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.text: hello world
                anchor: parent
                right: 0
                top: 100
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn inherit_check() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("hello"),
                line: true,
                line_clamp: Some(50),
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("hello"),
                line: true,
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.text foo: hello
                inherit line-clamp:

                -- foo:
                line-clamp: 50

                -- foo:
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn inner_container_check() {
        let mut main = super::default_column();
        let col = ftd::Element::Column(ftd::Column {
            spacing: None,
            container: ftd::Container {
                children: vec![ftd::Element::Column(ftd::Column {
                    spacing: None,
                    container: ftd::Container {
                        children: vec![
                            ftd::Element::Image(ftd::Image {
                                src: i(
                                    "https://www.nilinswap.com/static/img/dp.jpeg",
                                    Some(s("foo/bar#src0")),
                                ),
                                common: ftd::Common {
                                    reference: Some(s("foo/bar#src0")),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }),
                            ftd::Element::Markup(ftd::Markups {
                                text: ftd::markdown_line("Swapnil Sharma"),
                                line: true,
                                ..Default::default()
                            }),
                        ],
                        ..Default::default()
                    },
                    ..Default::default()
                })],
                ..Default::default()
            },
            ..Default::default()
        });
        main.container.children.push(col.clone());
        main.container.children.push(col);

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.image-src src0: 
                light: https://www.nilinswap.com/static/img/dp.jpeg
                dark: https://www.nilinswap.com/static/img/dp.jpeg

                -- ftd.column:

                --- ftd.column:

                --- ftd.image:
                src: $src0

                --- ftd.text: Swapnil Sharma


                -- ftd.column foo:

                --- ftd.column:

                --- ftd.image:
                src: $src0

                --- ftd.text: Swapnil Sharma

                -- foo:
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn mouse_in() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("Hello World"),
                line: true,
                common: ftd::Common {
                    conditional_attribute: std::iter::IntoIterator::into_iter([(
                        s("color"),
                        ftd::ConditionalAttribute {
                            attribute_type: ftd::AttributeType::Style,
                            conditions_with_value: vec![(
                                ftd::Condition {
                                    variable: s("foo/bar#MOUSE-IN@0"),
                                    value: serde_json::Value::Bool(true),
                                },
                                ftd::ConditionalValue {
                                    value: serde_json::from_str("{\"$kind$\":\"light\",\"dark\":\"rgba(255,0,0,1)\",\"light\":\"rgba(255,0,0,1)\"}").unwrap(),
                                    important: false,
                                    reference: Some(s("foo/bar#red")),
                                },
                            )],
                            default: None,
                        },
                    )])
                    .collect(),
                    events: vec![
                        ftd::Event {
                            name: s("onmouseenter"),
                            action: ftd::Action {
                                action: s("set-value"),
                                target: s("foo/bar#MOUSE-IN@0"),
                                parameters: std::iter::IntoIterator::into_iter([(
                                    s("value"),
                                    vec![
                                        ftd::event::ParameterData {
                                            value: serde_json::Value::from(true),
                                            reference: None,
                                        },
                                        ftd::event::ParameterData {
                                            value: serde_json::Value::String(s("boolean")),
                                            reference: None,
                                        },
                                    ],
                                )])
                                .collect(),
                            },
                        },
                        ftd::Event {
                            name: s("onmouseleave"),
                            action: ftd::Action {
                                action: s("set-value"),
                                target: s("foo/bar#MOUSE-IN@0"),
                                parameters: std::iter::IntoIterator::into_iter([(
                                    s("value"),
                                    vec![
                                        ftd::event::ParameterData {
                                            value: serde_json::Value::from(false),
                                            reference: None,
                                        },
                                        ftd::event::ParameterData {
                                            value: serde_json::Value::String(s("boolean")),
                                            reference: None,
                                        },
                                    ],
                                )])
                                .collect(),
                            },
                        },
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.color red: red
                dark: red

                -- ftd.text foo:
                text: Hello World
                color if $MOUSE-IN: $red

                -- foo:
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn event_stop_propagation() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("Hello"),
                            line: true,
                            common: ftd::Common {
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#open@0"),
                                    value: serde_json::Value::Bool(true),
                                }),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![ftd::Element::Markup(ftd::Markups {
                                    text: ftd::markdown_line("Hello Again"),
                                    line: true,
                                    common: ftd::Common {
                                        condition: Some(ftd::Condition {
                                            variable: s("foo/bar#open@0,1"),
                                            value: serde_json::Value::Bool(true),
                                        }),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            common: ftd::Common {
                                events: vec![
                                    ftd::Event {
                                        name: s("onclick"),
                                        action: ftd::Action {
                                            action: s("toggle"),
                                            target: s("foo/bar#open@0,1"),
                                            parameters: Default::default(),
                                        },
                                    },
                                    ftd::Event {
                                        name: s("onclick"),
                                        action: ftd::Action {
                                            action: s("stop-propagation"),
                                            target: s(""),
                                            parameters: Default::default(),
                                        },
                                    },
                                ],
                                ..Default::default()
                            },
                        }),
                    ],
                    ..Default::default()
                },
                common: ftd::Common {
                    events: vec![ftd::Event {
                        name: s("onclick"),
                        action: ftd::Action {
                            action: s("toggle"),
                            target: s("foo/bar#open@0"),
                            parameters: Default::default(),
                        },
                    }],
                    ..Default::default()
                },
            }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- foo:

                -- ftd.column foo:
                boolean open: true
                $on-click$: toggle $open

                --- ftd.text: Hello
                if: $open

                --- bar:


                -- ftd.column bar:
                boolean open: true
                $on-click$: toggle $open
                $on-click$: stop-propagation

                --- ftd.text: Hello Again
                if: $open

                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn new_syntax() {
        let mut main = super::default_column();
        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![ftd::Element::Integer(ftd::Text {
                    text: ftd::markdown_line("20"),
                    common: ftd::Common {
                        conditional_attribute: std::iter::IntoIterator::into_iter([(
                            s("color"),
                            ftd::ConditionalAttribute {
                                attribute_type: ftd::AttributeType::Style,
                                conditions_with_value: vec![
                                    (
                                        ftd::Condition {
                                            variable: s("foo/bar#b@0"),
                                            value: serde_json::Value::Bool(true),
                                        },
                                        ftd::ConditionalValue {
                                            value: serde_json::from_str("{\"$kind$\":\"light\",\"dark\":\"rgba(0,0,0,1)\",\"light\":\"rgba(0,0,0,1)\"}").unwrap(),
                                            important: false,
                                            reference: Some(s("foo/bar#black")),
                                        },
                                    ),
                                    (
                                        ftd::Condition {
                                            variable: s("foo/bar#a@0"),
                                            value: serde_json::Value::from(30),
                                        },
                                        ftd::ConditionalValue {
                                            value: serde_json::from_str("{\"$kind$\":\"light\",\"dark\":\"rgba(255,0,0,1)\",\"light\":\"rgba(255,0,0,1)\"}").unwrap(),
                                            important: false,
                                            reference: Some(s("foo/bar#red")),
                                        },
                                    ),
                                ],
                                default: None,
                            },
                        )])
                        .collect(),
                        reference: Some(s("foo/bar#a@0")),
                        ..Default::default()
                    },
                    ..Default::default()
                })],
                ..Default::default()
            },
            common: ftd::Common {
                events: vec![
                    ftd::Event {
                        name: s("onclick"),
                        action: ftd::Action {
                            action: s("toggle"),
                            target: s("foo/bar#b@0"),
                            parameters: Default::default(),
                        },
                    },
                    ftd::Event {
                        name: s("onclick"),
                        action: ftd::Action {
                            action: s("increment"),
                            target: s("foo/bar#a@0"),
                            parameters: std::iter::IntoIterator::into_iter([(
                                s("by"),
                                vec![ftd::event::ParameterData {
                                    value: serde_json::Value::from(2),
                                    reference: None,
                                }],
                            )])
                            .collect(),
                        },
                    },
                ],
                ..Default::default()
            },
        }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.color black: black
                dark: black

                -- ftd.color red: red
                dark: red

                -- ftd.row foo:
                integer a:
                boolean b: false
                $on-click$: toggle $b
                $on-click$: increment $a by 2

                --- ftd.integer:
                value: $a
                color if $b: $black
                color if $a == 30: $red

                -- foo:
                a: 20
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn condition_check() {
        let mut main = super::default_column();
        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![ftd::Element::Column(ftd::Column {
                    spacing: None,
                    container: ftd::Container {
                        children: vec![ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("Hello"),
                            line: true,
                            common: ftd::Common {
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#b@0,0"),
                                    value: serde_json::Value::Bool(true),
                                }),
                                is_not_visible: true,
                                ..Default::default()
                            },
                            ..Default::default()
                        })],
                        ..Default::default()
                    },
                    common: ftd::Common {
                        condition: Some(ftd::Condition {
                            variable: s("foo/bar#b@0"),
                            value: serde_json::Value::Bool(true),
                        }),
                        ..Default::default()
                    },
                })],
                ..Default::default()
            },
            ..Default::default()
        }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- boolean present: true

                -- ftd.column bar:
                boolean a: true
                if: $a
                boolean b: false

                --- ftd.text: Hello
                if: $b

                -- ftd.row foo:
                boolean b: true

                --- bar:
                if: $b

                -- foo:
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn external_variable() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Integer(ftd::Text {
                            text: ftd::markdown_line("20"),
                            common: ftd::Common {
                                conditional_attribute: std::iter::IntoIterator::into_iter([(
                                    s("color"),
                                    ftd::ConditionalAttribute {
                                        attribute_type: ftd::AttributeType::Style,
                                        conditions_with_value: vec![(
                                            ftd::Condition {
                                                variable: s("foo/bar#b@0"),
                                                value: serde_json::Value::Bool(true),
                                            },
                                            ftd::ConditionalValue {
                                                value: serde_json::from_str("{\"$kind$\":\"light\",\"dark\":\"rgba(0,0,0,1)\",\"light\":\"rgba(0,0,0,1)\"}").unwrap(),
                                                important: false,
                                                reference: Some(s("foo/bar#black")),
                                            },
                                        )],
                                        default: None,
                                    },
                                )])
                                .collect(),
                                reference: Some(s("foo/bar#a@0")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("whatever"),
                            line: true,
                            common: ftd::Common {
                                reference: Some(s("foo/bar#some-text@0")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                common: ftd::Common {
                    events: vec![
                        ftd::Event {
                            name: s("onclick"),
                            action: ftd::Action {
                                action: s("toggle"),
                                target: s("foo/bar#b@0"),
                                parameters: Default::default(),
                            },
                        },
                        ftd::Event {
                            name: s("onclick"),
                            action: ftd::Action {
                                action: s("increment"),
                                target: s("foo/bar#a@0"),
                                parameters: Default::default(),
                            },
                        },
                        ftd::Event {
                            name: s("onclick"),
                            action: ftd::Action {
                                action: s("set-value"),
                                target: s("foo/bar#some-text@0"),
                                parameters: std::iter::IntoIterator::into_iter([(
                                    "value".to_string(),
                                    vec![
                                        ftd::event::ParameterData {
                                            value: serde_json::Value::String(s("hello")),
                                            reference: Some(s("foo/bar#current")),
                                        },
                                        ftd::event::ParameterData {
                                            value: serde_json::Value::String(s("string")),
                                            reference: None,
                                        },
                                    ],
                                )])
                                .collect(),
                            },
                        },
                    ],
                    ..Default::default()
                },
            }));

        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![ftd::Element::Markup(ftd::Markups {
                    text: ftd::markdown_line("hello"),
                    line: true,
                    common: ftd::Common {
                        conditional_attribute: std::iter::IntoIterator::into_iter([(
                            s("color"),
                            ftd::ConditionalAttribute {
                                attribute_type: ftd::AttributeType::Style,
                                conditions_with_value: vec![(
                                    ftd::Condition {
                                        variable: s("foo/bar#foo@1"),
                                        value: serde_json::Value::Bool(true),
                                    },
                                    ftd::ConditionalValue {
                                        value: serde_json::from_str("{\"$kind$\":\"light\",\"dark\":\"rgba(255,0,0,1)\",\"light\":\"rgba(255,0,0,1)\"}").unwrap(),
                                        important: false,
                                        reference: Some(s("foo/bar#red")),
                                    },
                                )],
                                default: None,
                            },
                        )])
                        .collect(),
                        ..Default::default()
                    },
                    ..Default::default()
                })],
                ..Default::default()
            },
            common: ftd::Common {
                events: vec![ftd::Event {
                    name: s("onclick"),
                    action: ftd::Action {
                        action: s("toggle"),
                        target: s("foo/bar#foo@1"),
                        parameters: Default::default(),
                    },
                }],
                ..Default::default()
            },
        }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.color black: black
                dark: black

                -- ftd.color red: red
                dark: red

                -- ftd.column foo:
                integer a:
                boolean b: false
                $on-click$: toggle $b
                $on-click$: increment $a

                --- ftd.integer:
                value: $a
                color if $b: $black

                -- string current: hello

                -- foo:
                a: 20
                string some-text: whatever
                $on-click$: $some-text = $current

                --- ftd.text: $some-text

                -- ftd.row:
                boolean foo: false
                $on-click$: toggle $foo

                --- ftd.text: hello
                color if $foo: $red
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn new_var_syntax() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("hello"),
                line: true,
                line_clamp: Some(30),
                common: ftd::Common {
                    conditional_attribute: std::iter::IntoIterator::into_iter([(
                        s("color"),
                        ftd::ConditionalAttribute {
                            attribute_type: ftd::AttributeType::Style,
                            conditions_with_value: vec![(
                                ftd::Condition {
                                    variable: s("foo/bar#t@0"),
                                    value: serde_json::Value::Bool(true),
                                },
                                ftd::ConditionalValue {
                                    value: serde_json::from_str("{\"$kind$\":\"light\",\"dark\":\"rgba(255,0,0,1)\",\"light\":\"rgba(255,0,0,1)\"}").unwrap(),
                                    important: false,
                                    reference: Some(s("foo/bar#red")),
                                },
                            )],
                            default: None,
                        },
                    )])
                    .collect(),
                    reference: Some(s("foo/bar#bar")),
                    color: Some(ftd::Color {
                        light: ftd::ColorValue {
                            r: 255,
                            g: 0,
                            b: 0,
                            alpha: 1.0,
                        },
                        dark: ftd::ColorValue {
                            r: 255,
                            g: 0,
                            b: 0,
                            alpha: 1.0,
                        },
                        reference: Some(s("foo/bar#red")),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("hello"),
                            line: true,
                            common: ftd::Common {
                                reference: Some(s("foo/bar#ff@1")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Integer(ftd::Text {
                            text: ftd::markdown_line("20"),
                            common: ftd::Common {
                                reference: Some(s("foo/bar#i@1")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.color red: red
                dark: red

                -- ftd.column col:
                integer i:
                string ff: hello

                --- ftd.text: $ff

                --- ftd.integer: $i

                -- integer foo: 20

                -- foo: 30

                -- string bar: hello

                -- ftd.text: $bar
                boolean t: true
                string f: hello
                line-clamp: $foo
                color if $t: $red

                -- col:
                i: 20
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn text_block() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::TextBlock(ftd::TextBlock {
                text: ftd::markdown_line("hello"),
                line: true,
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::TextBlock(ftd::TextBlock {
                text: ftd::markdown_line("hello"),
                line: true,
                ..Default::default()
            }));

        main.container.children.push(ftd::Element::Code(ftd::Code {
            text: ftd::code_with_theme(
                "This is text",
                "txt",
                ftd::render::DEFAULT_THEME,
                "foo/bar",
            )
            .unwrap(),
            ..Default::default()
        }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.text-block: hello

                -- ftd.text-block b: hello

                -- b:

                -- ftd.code:

                This is text
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn variable_component() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("amitu"),
                            line: true,
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markdown_line("hello"),
                            line: true,
                            common: ftd::Common {
                                color: Some(ftd::Color {
                                    light: ftd::ColorValue {
                                        r: 255,
                                        g: 0,
                                        b: 0,
                                        alpha: 1.0,
                                    },
                                    dark: ftd::ColorValue {
                                        r: 255,
                                        g: 0,
                                        b: 0,
                                        alpha: 1.0,
                                    },
                                    reference: Some(s("foo/bar#red")),
                                }),
                                ..Default::default()
                            },
                            line_clamp: Some(10),
                            ..Default::default()
                        }),
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![
                                    ftd::Element::Markup(ftd::Markups {
                                        text: ftd::markdown_line("hello again"),
                                        line: true,
                                        common: ftd::Common {
                                            reference: Some(s("foo/bar#msg@0,2")),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    }),
                                    ftd::Element::Markup(ftd::Markups {
                                        text: ftd::markdown_line("hello world!"),
                                        line: true,
                                        common: ftd::Common {
                                            reference: Some(s("foo/bar#other-msg@0,2")),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    }),
                                    ftd::Element::Markup(ftd::Markups {
                                        text: ftd::markdown_line("hello"),
                                        line: true,
                                        common: ftd::Common {
                                            color: Some(ftd::Color {
                                                light: ftd::ColorValue {
                                                    r: 255,
                                                    g: 0,
                                                    b: 0,
                                                    alpha: 1.0,
                                                },
                                                dark: ftd::ColorValue {
                                                    r: 255,
                                                    g: 0,
                                                    b: 0,
                                                    alpha: 1.0,
                                                },
                                                reference: Some(s("foo/bar#red")),
                                            }),
                                            ..Default::default()
                                        },
                                        line_clamp: Some(20),
                                        ..Default::default()
                                    }),
                                    ftd::Element::Markup(ftd::Markups {
                                        text: ftd::markdown_line("hello amitu!"),
                                        line: true,
                                        common: ftd::Common {
                                            color: Some(ftd::Color {
                                                light: ftd::ColorValue {
                                                    r: 255,
                                                    g: 0,
                                                    b: 0,
                                                    alpha: 1.0,
                                                },
                                                dark: ftd::ColorValue {
                                                    r: 255,
                                                    g: 0,
                                                    b: 0,
                                                    alpha: 1.0,
                                                },
                                                reference: Some(s("foo/bar#red")),
                                            }),
                                            ..Default::default()
                                        },
                                        line_clamp: Some(10),
                                        ..Default::default()
                                    }),
                                ],
                                ..Default::default()
                            },
                            common: ftd::Common {
                                ..Default::default()
                            },
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.color red: red
                dark: red

                -- ftd.text foo: hello
                integer line-clamp: 10
                color: $red
                line-clamp: $line-clamp

                -- ftd.column moo: 
                caption msg: world
                string other-msg: world again
                ftd.ui t:
                ftd.ui k:
                
                --- ftd.text: $msg

                --- ftd.text: $other-msg

                --- t:

                --- k:

                -- ftd.column bar:
                ftd.ui t: foo:
                > line-clamp: 30
                ftd.ui g:

                --- ftd.text: amitu

                --- t:

                --- g:

                -- bar:
                g: moo: hello again
                > other-msg: hello world!
                > t: foo:
                >> line-clamp: 20
                > k: ftd.text: hello amitu!
                >> color: $red
                >> line-clamp: 10
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn optional_global_variable() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("hello"),
                line: true,
                common: ftd::Common {
                    reference: Some(s("foo/bar#active")),
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#active"),
                        value: serde_json::Value::String(s("$IsNotNull$")),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("Not Active"),
                line: true,
                common: ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#active"),
                        value: serde_json::Value::String(s("$IsNull$")),
                    }),
                    is_not_visible: true,
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line(""),
                line: true,
                common: ftd::Common {
                    reference: Some(s("foo/bar#flags")),
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#flags"),
                        value: serde_json::Value::String(s("$IsNotNull$")),
                    }),
                    is_not_visible: true,
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("No Flag Available"),
                line: true,
                common: ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#flags"),
                        value: serde_json::Value::String(s("$IsNull$")),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- optional string active:

                -- active: hello
                
                -- ftd.text: $active
                if: $active is not null

                -- ftd.text: Not Active
                if: $active is null

                -- optional string flags:
                
                -- ftd.text: $flags
                if: $flags is not null

                -- ftd.text: No Flag Available
                if: $flags is null
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn object() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                container: ftd::Container {
                    children: vec![ftd::Element::Markup(ftd::Markups {
                        text: ftd::markup_line("Data"),
                        line: true,
                        ..Default::default()
                    })],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let mut bag = super::default_bag();
        bag.insert(
            s("foo/bar#aa"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("aa"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Madhav"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#foo"),
            ftd::p2::Thing::Component(ftd::Component {
                root: s("ftd#column"),
                full_name: s("foo/bar#foo"),
                arguments: std::iter::IntoIterator::into_iter([(s("o"), ftd::p2::Kind::object())])
                    .collect(),
                instructions: vec![ftd::Instruction::ChildComponent {
                    child: ftd::ChildComponent {
                        root: s("ftd#text"),
                        properties: std::iter::IntoIterator::into_iter([(
                            s("text"),
                            ftd::component::Property {
                                default: Some(ftd::PropertyValue::Value {
                                    value: ftd::variable::Value::String {
                                        text: s("Data"),
                                        source: ftd::TextSource::Caption,
                                    },
                                }),
                                conditions: vec![],
                                nested_properties: Default::default(),
                            },
                        )])
                        .collect(),
                        ..Default::default()
                    },
                }],
                ..Default::default()
            }),
        );
        bag.insert(
            s("foo/bar#obj"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("obj"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Object {
                        values: std::iter::IntoIterator::into_iter([
                            (
                                s("a"),
                                ftd::PropertyValue::Reference {
                                    name: s("foo/bar#aa"),
                                    kind: ftd::p2::Kind::String {
                                        caption: true,
                                        body: false,
                                        default: None,
                                        is_reference: false,
                                    },
                                },
                            ),
                            (
                                s("b"),
                                ftd::PropertyValue::Value {
                                    value: ftd::variable::Value::String {
                                        text: s("bb"),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#o@0"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("o"),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#obj"),
                    kind: ftd::p2::Kind::object(),
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        p!(
            "
            -- string aa: Madhav

            -- object obj:
            a: $aa
            b: bb

            -- ftd.column foo:
            object o:
        
            --- ftd.text: Data

            -- foo:
            o: $obj
            ",
            (bag, main),
        );
    }

    #[test]
    fn event_change() {
        let mut main = super::default_column();
        let mut bag = super::default_bag();
        bag.insert(
            s("foo/bar#input-data"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("input-data"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Nothing"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            s("foo/bar#obj"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("obj"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Object {
                        values: std::iter::IntoIterator::into_iter([
                            (
                                s("function"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("some-function"),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("value"),
                                ftd::PropertyValue::Reference {
                                    name: s("foo/bar#input-data"),
                                    kind: ftd::p2::Kind::String {
                                        caption: true,
                                        body: false,
                                        default: None,
                                        is_reference: false,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
            }),
        );

        main.container
            .children
            .push(ftd::Element::Input(ftd::Input {
                common: ftd::Common {
                    events: vec![
                        ftd::Event {
                            name: s("onchange"),
                            action: ftd::Action {
                                action: s("set-value"),
                                target: s("foo/bar#input-data"),
                                parameters: std::iter::IntoIterator::into_iter([(
                                    s("value"),
                                    vec![
                                        ftd::event::ParameterData {
                                            value: serde_json::Value::String(s("$VALUE")),
                                            reference: None,
                                        },
                                        ftd::event::ParameterData {
                                            value: serde_json::Value::String(s("string")),
                                            reference: None,
                                        },
                                    ],
                                )])
                                .collect(),
                            },
                        },
                        ftd::Event {
                            name: s("onchange"),
                            action: ftd::Action {
                                action: s("message-host"),
                                target: s("$obj"),
                                parameters: std::iter::IntoIterator::into_iter([(
                                    "data".to_string(),
                                    vec![ftd::event::ParameterData {
                                    value: serde_json::from_str(
                                        "{\"function\":\"some-function\",\"value\":\"Nothing\"}",
                                    )
                                    .unwrap(),
                                    reference: Some(s("{\"value\":\"foo/bar#input-data\"}")),
                                }],
                                )])
                                .collect(),
                            },
                        },
                    ],
                    ..Default::default()
                },
                placeholder: None,
                ..Default::default()
            }));

        p!(
            "
            -- string input-data: Nothing

            -- object obj:
            function: some-function
            value: $input-data

            -- ftd.input:
            $on-change$: $input-data=$VALUE
            $on-change$: message-host $obj
            ",
            (bag, main),
        );
    }

    #[test]
    fn component_processor() {
        let mut main = super::default_column();

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markup_line("Hello from text-component processor"),
                line: true,
                line_clamp: Some(40),
                ..Default::default()
            }));

        p!(
            "
            -- ftd.text: hello
            $processor$: text-component-processor
            ",
            (super::default_bag(), main),
        );
    }

    #[test]
    fn global_variable_pass_as_reference() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("Arpita"),
                line: true,
                common: ftd::Common {
                    reference: Some(s("foo/bar#bar")),
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd::Element::Integer(ftd::Text {
                text: ftd::markdown_line("1"),
                common: ftd::Common {
                    reference: Some(s("foo/bar#ibar")),
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("Arpita"),
                line: true,
                common: ftd::Common {
                    reference: Some(s("foo/bar#lfoo")),
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("Arpita"),
                line: true,
                common: ftd::Common {
                    reference: Some(s("foo/bar#lfoo")),
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("Ayushi"),
                line: true,
                common: ftd::Common {
                    reference: Some(s("foo/bar#lfoo")),
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("$loop$"),
                line: true,
                common: ftd::Common {
                    is_dummy: true,
                    reference: Some(s("foo/bar#lfoo")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("Arpita"),
                line: true,
                common: ftd::Common {
                    reference: Some(s("foo/bar#lbar")),
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("Arpita"),
                line: true,
                common: ftd::Common {
                    reference: Some(s("foo/bar#lbar")),
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("Ayushi"),
                line: true,
                common: ftd::Common {
                    reference: Some(s("foo/bar#lbar")),
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("$loop$"),
                line: true,
                common: ftd::Common {
                    is_dummy: true,
                    reference: Some(s("foo/bar#lbar")),
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markdown_line("Arpita"),
                line: true,
                common: ftd::Common {
                    reference: Some(s("foo/bar#arpita.name")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let mut bag = super::default_bag();
        bag.insert(
            s("foo/bar#arpita"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("arpita"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("foo/bar#person"),
                        fields: std::iter::IntoIterator::into_iter([(
                            s("name"),
                            ftd::PropertyValue::Reference {
                                name: s("foo/bar#bar"),
                                kind: ftd::p2::Kind::String {
                                    caption: true,
                                    body: false,
                                    default: None,
                                    is_reference: false,
                                },
                            },
                        )])
                        .collect(),
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            s("foo/bar#person"),
            ftd::p2::Thing::Record(ftd::p2::Record {
                name: s("foo/bar#person"),
                fields: std::iter::IntoIterator::into_iter([(s("name"), ftd::p2::Kind::caption())])
                    .collect(),
                instances: Default::default(),
                order: vec![s("name")],
            }),
        );

        bag.insert(
            s("foo/bar#bar"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("bar"),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#foo"),
                    kind: ftd::p2::Kind::String {
                        caption: true,
                        body: false,
                        default: None,
                        is_reference: false,
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            s("foo/bar#foo"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("foo"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Arpita"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            s("foo/bar#ibar"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("ibar"),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#ifoo"),
                    kind: ftd::p2::Kind::Integer {
                        default: None,
                        is_reference: false,
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            s("foo/bar#ifoo"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("ifoo"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 1 },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            s("foo/bar#lbar"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("foo/bar#lbar"),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#lfoo"),
                    kind: ftd::p2::Kind::List {
                        kind: Box::new(ftd::p2::Kind::String {
                            caption: false,
                            body: false,
                            default: None,
                            is_reference: false,
                        }),
                        default: None,
                        is_reference: false,
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            s("foo/bar#lfoo"),
            ftd::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("foo/bar#lfoo"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::List {
                        data: vec![
                            ftd::PropertyValue::Reference {
                                name: s("foo/bar#foo"),
                                kind: ftd::p2::Kind::String {
                                    caption: true,
                                    body: false,
                                    default: None,
                                    is_reference: false,
                                },
                            },
                            ftd::PropertyValue::Reference {
                                name: s("foo/bar#bar"),
                                kind: ftd::p2::Kind::String {
                                    caption: true,
                                    body: false,
                                    default: None,
                                    is_reference: false,
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: s("Ayushi"),
                                    source: ftd::TextSource::Caption,
                                },
                            },
                        ],
                        kind: ftd::p2::Kind::String {
                            caption: false,
                            body: false,
                            default: None,
                            is_reference: false,
                        },
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            s("foo/bar#$loop$@2"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("$loop$"),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#foo"),
                    kind: ftd::p2::Kind::caption(),
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#$loop$@3"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("$loop$"),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#bar"),
                    kind: ftd::p2::Kind::caption(),
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#$loop$@4"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("$loop$"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Ayushi"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#$loop$@5"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("$loop$"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("$loop$"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        bag.insert(
            s("foo/bar#$loop$@6"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("$loop$"),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#foo"),
                    kind: ftd::p2::Kind::caption(),
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#$loop$@7"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("$loop$"),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#bar"),
                    kind: ftd::p2::Kind::caption(),
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#$loop$@8"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("$loop$"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Ayushi"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#$loop$@9"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("$loop$"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("$loop$"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@0", -1, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@1", -1, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@2", -1, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@3", -1, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@4", -1, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@5", -1, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@6", -1, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@7", -1, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@8", -1, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@9", -1, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@10", -1, &mut bag);

        insert_integer_by_root("foo/bar#CHILDREN-COUNT@0", 0, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT@1", 0, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT@2", 0, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT@3", 0, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT@4", 0, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT@5", 0, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT@6", 0, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT@7", 0, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT@8", 0, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT@9", 0, &mut bag);
        insert_integer_by_root("foo/bar#CHILDREN-COUNT@10", 0, &mut bag);

        insert_integer_by_root("foo/bar#SIBLING-INDEX-0@0", 0, &mut bag);
        insert_integer_by_root("foo/bar#SIBLING-INDEX-0@1", 1, &mut bag);
        insert_integer_by_root("foo/bar#SIBLING-INDEX-0@10", 10, &mut bag);

        insert_integer_by_root("foo/bar#SIBLING-INDEX@0", 1, &mut bag);
        insert_integer_by_root("foo/bar#SIBLING-INDEX@1", 2, &mut bag);
        insert_integer_by_root("foo/bar#SIBLING-INDEX@10", 11, &mut bag);

        let (g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- string foo: Arpita

                -- string bar: $foo

                -- integer ifoo: 1

                -- integer ibar: $ifoo

                -- string list lfoo:

                -- lfoo: $foo

                -- lfoo: $bar

                -- lfoo: Ayushi

                -- string list lbar: $lfoo

                -- record person:
                caption name:

                -- person arpita: $bar

                -- ftd.text: $bar

                -- ftd.integer: $ibar

                -- ftd.text: $obj
                $loop$: $lfoo as $obj

                -- ftd.text: $obj
                $loop$: $lbar as $obj

                -- ftd.text: $arpita.name
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
        pretty_assertions::assert_eq!(g_bag, bag);
    }

    #[test]
    fn locals_as_ref() {
        let mut bag = super::default_bag();
        bag.insert(
            s("foo/bar#active"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("active"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Boolean { value: true },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#active@0"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("active"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Boolean { value: false },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#active@1"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("active"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Boolean { value: false },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#bar"),
            ftd::p2::Thing::Component(ftd::Component {
                root: s("ftd#column"),
                full_name: s("foo/bar#bar"),
                arguments: std::iter::IntoIterator::into_iter([
                    (
                        s("active"),
                        ftd::p2::Kind::boolean().set_default(Some(s("false"))),
                    ),
                    (
                        s("bio"),
                        ftd::p2::Kind::string().set_default(Some(s("$subtitle"))),
                    ),
                    (
                        s("subtitle"),
                        ftd::p2::Kind::string().set_default(Some(s("$foo/bar#foo"))),
                    ),
                    (s("title"), ftd::p2::Kind::string()),
                    (s("w"), ftd::p2::Kind::integer()),
                ])
                .collect(),
                locals: Default::default(),
                properties: std::iter::IntoIterator::into_iter([
                    (
                        s("border-width"),
                        ftd::component::Property {
                            default: Some(ftd::PropertyValue::Variable {
                                name: s("w"),
                                kind: ftd::p2::Kind::Optional {
                                    kind: Box::new(ftd::p2::Kind::integer()),
                                    is_reference: false,
                                },
                            }),
                            conditions: vec![],
                            nested_properties: Default::default(),
                        },
                    ),
                    (
                        s("color"),
                        ftd::component::Property {
                            default: Some(ftd::PropertyValue::Reference {
                                name: s("foo/bar#green"),
                                kind: ftd::p2::Kind::Optional {
                                    kind: Box::new(ftd::p2::Kind::Record {
                                        name: s("ftd#color"),
                                        default: None,
                                        is_reference: false,
                                    }),
                                    is_reference: false,
                                },
                            }),
                            conditions: vec![],
                            nested_properties: Default::default(),
                        },
                    ),
                ])
                .collect(),
                instructions: vec![
                    ftd::Instruction::ChildComponent {
                        child: ftd::ChildComponent {
                            root: s("ftd#text"),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("text"),
                                ftd::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: s("title"),
                                        kind: ftd::p2::Kind::caption_or_body(),
                                    }),
                                    conditions: vec![],
                                    nested_properties: Default::default(),
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::Instruction::ChildComponent {
                        child: ftd::ChildComponent {
                            root: s("ftd#text"),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("text"),
                                ftd::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: s("subtitle"),
                                        kind: ftd::p2::Kind::caption_or_body()
                                            .set_default(Some(s("$foo/bar#foo"))),
                                    }),
                                    conditions: vec![],
                                    nested_properties: Default::default(),
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::Instruction::ChildComponent {
                        child: ftd::ChildComponent {
                            root: s("ftd#text"),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("text"),
                                ftd::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: s("bio"),
                                        kind: ftd::p2::Kind::caption_or_body()
                                            .set_default(Some(s("$subtitle"))),
                                    }),
                                    conditions: vec![],
                                    nested_properties: Default::default(),
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::Instruction::ChildComponent {
                        child: ftd::ChildComponent {
                            root: s("ftd#boolean"),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("value"),
                                ftd::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: s("active"),
                                        kind: ftd::p2::Kind::boolean()
                                            .set_default(Some(s("false"))),
                                    }),
                                    conditions: vec![],
                                    nested_properties: Default::default(),
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                ],
                events: vec![],
                condition: None,
                kernel: false,
                invocations: vec![],
                line_number: 0,
            }),
        );
        bag.insert(
            s("foo/bar#bar1"),
            ftd::p2::Thing::Component(ftd::Component {
                root: s("ftd#column"),
                full_name: s("foo/bar#bar1"),
                arguments: Default::default(),
                locals: Default::default(),
                properties: Default::default(),
                instructions: vec![],
                events: vec![],
                condition: None,
                kernel: false,
                invocations: vec![],
                line_number: 0,
            }),
        );
        bag.insert(
            s("foo/bar#bio@0"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("bio"),
                value: ftd::PropertyValue::Variable {
                    name: s("foo/bar#subtitle@0"),
                    kind: ftd::p2::Kind::string().set_default(Some(s("$foo/bar#foo"))),
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#bio@1"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("bio"),
                value: ftd::PropertyValue::Variable {
                    name: s("foo/bar#subtitle@1"),
                    kind: ftd::p2::Kind::string().set_default(Some(s("$foo/bar#foo"))),
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#foo"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("foo"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Foo"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#foo"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("foo"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Foo"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#gg@0"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("gg"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 1 },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#subtitle@0"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("subtitle"),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#foo"),
                    kind: ftd::p2::Kind::string().set_default(Some(s("$foo/bar#foo"))),
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#subtitle@1"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("subtitle"),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#foo"),
                    kind: ftd::p2::Kind::string().set_default(Some(s("$foo/bar#foo"))),
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#title@0"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("title"),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#foo"),
                    kind: ftd::p2::Kind::string(),
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#title@1"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("title"),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#foo"),
                    kind: ftd::p2::Kind::string(),
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#w@0"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("w"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 2 },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#w@1"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("w"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 1 },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#green"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("green"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("ftd#color"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("dark"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("green"),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("light"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("green"),
                                        source: ftd::TextSource::Caption,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markup_line("Foo"),
                            line: true,
                            common: ftd::Common {
                                reference: Some(s("foo/bar#title@0")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markup_line("Foo"),
                            line: true,
                            common: ftd::Common {
                                reference: Some(s("foo/bar#subtitle@0")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markup_line("Foo"),
                            line: true,
                            common: ftd::Common {
                                reference: Some(s("foo/bar#bio@0")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Boolean(ftd::Text {
                            text: ftd::markup_line("false"),
                            common: ftd::Common {
                                reference: Some(s("foo/bar#active@0")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Integer(ftd::Text {
                            text: ftd::markup_line("1"),
                            common: ftd::Common {
                                reference: Some(s("foo/bar#gg@0")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ],
                    external_children: None,
                    wrap: false,
                    ..Default::default()
                },
                spacing: None,
                common: ftd::Common {
                    color: Some(ftd::Color {
                        light: ftd::ColorValue {
                            r: 0,
                            g: 128,
                            b: 0,
                            alpha: 1.0,
                        },
                        dark: ftd::ColorValue {
                            r: 0,
                            g: 128,
                            b: 0,
                            alpha: 1.0,
                        },
                        reference: Some(s("foo/bar#green")),
                    }),
                    id: Some(s("bar-id")),
                    data_id: Some(s("bar-id")),
                    border_width: 2,
                    scale: Some(1.2),
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#active"),
                        value: serde_json::Value::Bool(true),
                    }),
                    ..Default::default()
                },
            }));

        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markup_line("Foo"),
                            line: true,
                            common: ftd::Common {
                                reference: Some(s("foo/bar#title@1")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markup_line("Foo"),
                            line: true,
                            common: ftd::Common {
                                reference: Some(s("foo/bar#subtitle@1")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::markup_line("Foo"),
                            line: true,
                            common: ftd::Common {
                                reference: Some(s("foo/bar#bio@1")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Boolean(ftd::Text {
                            text: ftd::markup_line("false"),
                            common: ftd::Common {
                                reference: Some(s("foo/bar#active@1")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ],
                    external_children: None,
                    wrap: false,
                    ..Default::default()
                },
                spacing: None,
                common: ftd::Common {
                    color: Some(ftd::Color {
                        light: ftd::ColorValue {
                            r: 0,
                            g: 128,
                            b: 0,
                            alpha: 1.0,
                        },
                        dark: ftd::ColorValue {
                            r: 0,
                            g: 128,
                            b: 0,
                            alpha: 1.0,
                        },
                        reference: Some(s("foo/bar#green")),
                    }),
                    border_width: 1,
                    ..Default::default()
                },
            }));

        p!(
            "
            -- string foo: Foo

            -- boolean active: true

            -- ftd.color green: green
            dark: green

            -- bar:
            if: $active
            id: bar-id
            scale: 1.2
            title: $foo
            w: 2
            integer gg: 1

            --- ftd.integer: $gg

            -- bar:
            title: $foo
            w: 1


            -- ftd.column bar1:
            ftd.ui

            -- ftd.column bar:
            string title:
            boolean active: false
            string subtitle: $foo
            string bio: $subtitle
            integer w:
            color: $green
            border-width: $w
            
            --- ftd.text: $title

            --- ftd.text: $subtitle

            --- ftd.text: $bio
            
            --- ftd.boolean: $active 
            ",
            (bag, main),
        );
    }

    #[test]
    fn optional_string_compare() {
        let mut bag = super::default_bag();
        bag.insert(
            s("foo/bar#bar"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("bar"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Optional {
                        data: Box::new(Some(ftd::Value::String {
                            text: "Something".to_string(),
                            source: ftd::TextSource::Caption,
                        })),
                        kind: ftd::p2::Kind::caption(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markup_line("Something"),
                common: ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#bar"),
                        value: serde_json::Value::String(s("Something")),
                    }),
                    reference: Some(s("foo/bar#bar")),
                    ..Default::default()
                },
                line: true,
                ..Default::default()
            }));

        p!(
            "
            -- optional string bar:

            -- bar: Something
            
            -- ftd.text: $bar
            if: $bar == Something
            ",
            (bag, main),
        );
    }

    #[test]
    fn hex_color_code() {
        let mut bag = super::default_bag();

        bag.insert(
            s("foo/bar#hex-color"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("hex-color"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: "ftd#color".to_string(),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                "light".to_string(),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "#2cc9b51a".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                "dark".to_string(),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "#2cc9b51a".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        let mut main = super::default_column();

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::markup_line("Hello"),
                line: true,
                common: ftd::Common {
                    color: Some(ftd::Color {
                        light: ftd::ColorValue {
                            r: 44,
                            g: 201,
                            b: 181,
                            alpha: 0.1,
                        },
                        dark: ftd::ColorValue {
                            r: 44,
                            g: 201,
                            b: 181,
                            alpha: 0.1,
                        },
                        reference: Some(s("foo/bar#hex-color")),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }));

        p!(
            "
            -- ftd.color hex-color:
            light: #2cc9b51a
            dark: #2cc9b51a

            -- ftd.text: Hello
            color: $hex-color
            ",
            (bag, main),
        );
    }

    #[test]
    fn special_variables() {
        let mut bag = super::default_bag();

        bag.insert(
            "foo/bar#current@0".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("current"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 1 },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        bag.insert(
            "foo/bar#presentation".to_string(),
            ftd::p2::Thing::Component(ftd::Component {
                root: s("ftd#column"),
                full_name: s("foo/bar#presentation"),
                arguments: std::iter::IntoIterator::into_iter([(
                    "current".to_string(),
                    ftd::p2::Kind::integer().set_default(Some(s("1"))),
                )])
                .collect(),
                properties: std::iter::IntoIterator::into_iter([
                    (
                        "append-at".to_string(),
                        ftd::component::Property {
                            default: Some(ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: s("col-id"),
                                    source: ftd::TextSource::Header,
                                },
                            }),
                            ..Default::default()
                        },
                    ),
                    (
                        "open".to_string(),
                        ftd::component::Property {
                            default: Some(ftd::PropertyValue::Value {
                                value: ftd::Value::Boolean { value: true },
                            }),
                            ..Default::default()
                        },
                    ),
                ])
                .collect(),
                instructions: vec![ftd::Instruction::ChildComponent {
                    child: ftd::ChildComponent {
                        root: s("ftd#column"),
                        properties: std::iter::IntoIterator::into_iter([(
                            "id".to_string(),
                            ftd::component::Property {
                                default: Some(ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("col-id"),
                                        source: ftd::TextSource::Header,
                                    },
                                }),
                                ..Default::default()
                            },
                        )])
                        .collect(),
                        ..Default::default()
                    },
                }],
                ..Default::default()
            }),
        );

        bag.insert(
            "foo/bar#slide".to_string(),
            ftd::p2::Thing::Component(ftd::Component {
                root: s("ftd#text"),
                full_name: s("foo/bar#slide"),
                arguments: std::iter::IntoIterator::into_iter([(
                    s("title"),
                    ftd::p2::Kind::caption(),
                )])
                .collect(),
                properties: std::iter::IntoIterator::into_iter([(
                    s("text"),
                    ftd::component::Property {
                        default: Some(ftd::PropertyValue::Variable {
                            name: s("title"),
                            kind: ftd::p2::Kind::caption_or_body(),
                        }),
                        ..Default::default()
                    },
                )])
                .collect(),
                events: vec![ftd::p2::Event {
                    name: ftd::p2::EventName::OnClick,
                    action: ftd::p2::Action {
                        action: ftd::p2::ActionKind::Increment,
                        target: ftd::PropertyValue::Variable {
                            name: s("PARENT.current"),
                            kind: ftd::p2::Kind::integer(),
                        },
                        parameters: std::iter::IntoIterator::into_iter([(
                            s("clamp"),
                            vec![
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::Integer { value: 1 },
                                },
                                ftd::PropertyValue::Variable {
                                    name: s("PARENT.CHILDREN-COUNT"),
                                    kind: ftd::p2::Kind::integer().set_default(Some(s("0"))),
                                },
                            ],
                        )])
                        .collect(),
                    },
                }],
                condition: Some(ftd::p2::Boolean::Equal {
                    left: ftd::PropertyValue::Variable {
                        name: s("PARENT.current"),
                        kind: ftd::p2::Kind::Element,
                    },
                    right: ftd::PropertyValue::Variable {
                        name: s("SIBLING-INDEX"),
                        kind: ftd::p2::Kind::Element,
                    },
                }),
                ..Default::default()
            }),
        );

        bag.insert(
            "foo/bar#title@0,0".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("title"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("First"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        bag.insert(
            "foo/bar#title@0,1".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("title"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Second"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        let mut main = super::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                container: ftd::Container {
                    children: vec![ftd::Element::Column(ftd::Column {
                        common: ftd::Common {
                            data_id: Some(s("col-id")),
                            ..Default::default()
                        },
                        ..Default::default()
                    })],
                    external_children: Some((
                        s("col-id"),
                        vec![vec![0]],
                        vec![ftd::Element::Column(ftd::Column {
                            container: ftd::Container {
                                children: vec![
                                    ftd::Element::Markup(ftd::Markups {
                                        text: ftd::markdown_line("First"),
                                        common: ftd::Common {
                                            condition: Some(ftd::Condition {
                                                variable: s("foo/bar#current@0"),
                                                value: serde_json::Value::from(1),
                                            }),
                                            events: vec![ftd::Event {
                                                name: s("onclick"),
                                                action: ftd::Action {
                                                    action: s("increment"),
                                                    target: s("foo/bar#current@0"),
                                                    parameters: std::iter::IntoIterator::into_iter(
                                                        [(
                                                            "clamp".to_string(),
                                                            vec![
                                                                ftd::event::ParameterData {
                                                                    value: serde_json::json!(1),
                                                                    reference: None,
                                                                },
                                                                ftd::event::ParameterData {
                                                                    value: serde_json::json!(2),
                                                                    reference: None,
                                                                },
                                                            ],
                                                        )],
                                                    )
                                                    .collect(),
                                                },
                                            }],
                                            reference: Some(s("foo/bar#title@0,0")),
                                            ..Default::default()
                                        },
                                        line: true,
                                        ..Default::default()
                                    }),
                                    ftd::Element::Markup(ftd::Markups {
                                        text: ftd::markdown_line("Second"),
                                        line: true,
                                        common: ftd::Common {
                                            condition: Some(ftd::Condition {
                                                variable: s("foo/bar#current@0"),
                                                value: serde_json::json!(2),
                                            }),
                                            is_not_visible: true,
                                            events: vec![ftd::Event {
                                                name: s("onclick"),
                                                action: ftd::Action {
                                                    action: s("increment"),
                                                    target: s("foo/bar#current@0"),
                                                    parameters: std::iter::IntoIterator::into_iter(
                                                        [(
                                                            "clamp".to_string(),
                                                            vec![
                                                                ftd::event::ParameterData {
                                                                    value: serde_json::json!(1),
                                                                    reference: None,
                                                                },
                                                                ftd::event::ParameterData {
                                                                    value: serde_json::json!(2),
                                                                    reference: None,
                                                                },
                                                            ],
                                                        )],
                                                    )
                                                    .collect(),
                                                },
                                            }],
                                            reference: Some(s("foo/bar#title@0,1")),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    }),
                                ],
                                ..Default::default()
                            },
                            common: ftd::Common {
                                width: Some(ftd::Length::Fill),
                                height: Some(ftd::Length::Fill),
                                position: Some(ftd::Position::Center),
                                ..Default::default()
                            },
                            ..Default::default()
                        })],
                    )),
                    open: Some(true),
                    append_at: Some(s("col-id")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        p!(
            "
            -- presentation:
            
            --- slide: First
            
            --- slide: Second


            -- ftd.column presentation:
            open: true
            append-at: col-id
            integer current: 1
            
            --- ftd.column:
            id: col-id
            
            
            -- ftd.text slide: $title
            caption title:
            if: $PARENT.current == $SIBLING-INDEX
            $on-click$: increment $PARENT.current clamp 1 $PARENT.CHILDREN-COUNT

            ",
            (bag, main),
        );
    }

    /*#[test]
    fn optional_condition_on_record() {
        let (_g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- record person-data:
                caption name:
                integer age:

                -- person-data person1: Madhav
                age: 10

                -- optional person-data person:

                -- ftd.text: $person.name
                if: $person is not null
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
    }*/

    /*#[test]
    fn loop_with_tree_structure_1() {
        let (g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- record toc-record:
                title: string
                link: string
                children: list toc-record

                -- component toc-item:
                component: ftd.column
                toc-record $toc:
                padding-left: 10

                --- ftd.text: ref $toc.title
                link: ref $toc.link

                --- toc-item:
                $loop$: $toc.children as $obj
                toc: $obj


                -- toc-record list toc:

                -- toc:
                title: ref ab.title
                link: ref ab.link
                children: ref ab.children

                -- toc-record ab:
                title: ab title
                link: ab link

                -- ab.children first_ab
                title: aa title
                link: aa link

                --- children:
                title:

                -- ab.children:
                title: aaa title
                link: aaa link



                -- toc-item:
                $loop$: toc as $obj
                toc: $obj
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        // pretty_assertions::assert_eq!(g_bag, bag);
        // pretty_assertions::assert_eq!(g_col, main);
        // --- toc-item:
        //                 $loop$: $toc.children as $t
        //                 toc: $t
    }

    #[test]
    fn loop_with_tree_structure_2() {
        let (g_bag, g_col) = ftd::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- record toc-record:
                title: string
                link: string
                children: list toc-record

                -- component toc-item:
                component: ftd.column
                toc-record $toc:
                padding-left: 10

                --- ftd.text: ref $toc.title
                link: ref $toc.link

                --- toc-item:
                $loop$: $toc.children as $obj
                toc: $obj


                -- toc-record list toc:
                $processor$: ft.toc

                - fifthtry/ftd/p1
                  `ftd::p1`: A JSON/YML Replacement
                - fifthtry/ftd/language
                  FTD Language
                  - fifthtry/ftd/p1-grammar
                    `ftd::p1` grammar




                -- toc-item:
                $loop$: $toc as $obj
                toc: $obj
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        // pretty_assertions::assert_eq!(g_bag, bag);
        // pretty_assertions::assert_eq!(g_col, main);
        // --- toc-item:
        //                 $loop$: $toc.children as $t
        //                 toc: $t
    }*/
}
