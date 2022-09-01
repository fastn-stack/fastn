#[derive(Debug, Default)]
pub struct InterpreterState {
    pub id: String,
    pub bag: ftd::Map<ftd::interpreter::Thing>,
    pub document_stack: Vec<ParsedDocument>,
    pub parsed_libs: ftd::Map<Vec<String>>,
}

impl InterpreterState {
    fn new(id: String) -> InterpreterState {
        InterpreterState {
            id,
            bag: ftd::interpreter::default_bag(),
            ..Default::default()
        }
    }

    fn continue_(mut self) -> ftd::p11::Result<Interpreter> {
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
            let doc = ftd::interpreter::TDoc {
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

            let var_data = ftd::interpreter::variable::VariableData::get_name_kind(
                &p1.name,
                &doc,
                p1.line_number,
                &parsed_document.var_types,
            );

            let mut thing = vec![];

            if p1.name.starts_with("record ") {
                // declare a record
                let d = ftd::interpreter::Record::from_p1(
                    p1.name.as_str(),
                    p1.headers.as_slice(),
                    &doc,
                    p1.line_number,
                )?;
                let name = doc.resolve_name(p1.line_number, &d.name.to_string())?;
                if self.bag.contains_key(name.as_str()) {
                    return ftd::interpreter::utils::e2(
                        format!("{} is already declared", d.name),
                        doc.name,
                        p1.line_number,
                    );
                }
                thing.push((name, ftd::interpreter::Thing::Record(d)));
            } else if p1.name.starts_with("or-type ") {
                // declare a record
                let d = ftd::interpreter::OrType::from_p1(&p1, &doc)?;
                let name = doc.resolve_name(p1.line_number, &d.name.to_string())?;
                if self.bag.contains_key(name.as_str()) {
                    return ftd::interpreter::utils::e2(
                        format!("{} is already declared", d.name),
                        doc.name,
                        p1.line_number,
                    );
                }
                thing.push((name, ftd::interpreter::Thing::OrType(d)));
            } else if p1.name.starts_with("map ") {
                let d = ftd::interpreter::Variable::map_from_p1(&p1, &doc)?;
                let name = doc.resolve_name(p1.line_number, &d.name.to_string())?;
                if self.bag.contains_key(name.as_str()) {
                    return ftd::p2::utils::e2(
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
                    return ftd::p2::utils::e2(
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
                    return ftd::p2::utils::e2(
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
                    return ftd::p2::utils::e2(
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
                        return ftd::p2::utils::e2(
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
                        return ftd::p2::utils::e2(
                            format!("'{}' is an or-type", p1.name.as_str()),
                            doc.name,
                            p1.line_number,
                        );
                    }
                    ftd::p2::Thing::OrTypeWithVariant { .. } => {
                        // do we allow initialization of a record by name? nopes
                        return ftd::p2::utils::e2(
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
}

#[derive(Debug, Clone)]
pub struct ParsedDocument {
    name: String,
    sections: Vec<ftd::p11::Section>,
    processing_imports: bool,
    doc_aliases: ftd::Map<String>,
    var_types: Vec<String>,
    foreign_variable_prefix: Vec<String>,
    instructions: Vec<ftd::interpreter::Instruction>,
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
        section: ftd::p11::Section,
    },
    StuckOnForeignVariable {
        variable: String,
        state: InterpreterState,
    },
    Done {
        document: ftd::interpreter::Document,
    },
}

impl ParsedDocument {
    fn parse(id: &str, source: &str) -> ftd::p1::Result<ParsedDocument> {
        Ok(ParsedDocument {
            name: id.to_string(),
            sections: ftd::p11::parse(source, id)?,
            processing_imports: true,
            doc_aliases: ftd::p2::interpreter::default_aliases(),
            var_types: Default::default(),
            foreign_variable_prefix: vec![],
            instructions: vec![],
        })
    }
}

pub fn interpret(id: &str, source: &str) -> ftd::p1::Result<Interpreter> {
    let mut s = InterpreterState::new(id.to_string());
    s.document_stack.push(ParsedDocument::parse(id, source)?);
    s.continue_()
}
