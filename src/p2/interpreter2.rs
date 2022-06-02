#[derive(Debug, Default)]
pub struct InterpreterState {
    pub(crate) bag: std::collections::BTreeMap<String, ftd::p2::Thing>,
    pub(crate) document_stack: Vec<ParsedDocument>,
}

impl InterpreterState {
    fn new() -> InterpreterState {
        InterpreterState {
            bag: ftd::p2::interpreter::default_bag(),
            ..Default::default()
        }
    }

    fn tdoc<'a>(
        &'a self,
        local_variables: &'a mut std::collections::BTreeMap<String, ftd::p2::Thing>,
    ) -> ftd::p2::TDoc<'a> {
        let l = self.document_stack.len() - 1;
        ftd::p2::TDoc {
            name: &self.document_stack[l].name,
            aliases: &self.document_stack[l].doc_aliases,
            bag: &self.bag,
            local_variables,
        }
    }

    fn continue_(mut self) -> ftd::p1::Result<Interpreter> {
        if self.document_stack.is_empty() {
            panic!()
        }

        let l = self.document_stack.len() - 1; // Get the top of the stack

        if self.document_stack[l].processing_imports {
            let (state, module) = self.process_imports()?;
            if let Some(module) = module {
                return Ok(Interpreter::StuckOnImport { state, module });
            }
            self = state;
            self.document_stack[l].done_processing_imports();
            self.document_stack[l].reorder(&self.bag)?;
        }
        let parsed_document = &mut self.document_stack[l];

        let mut instructions: Vec<ftd::Instruction> = Default::default();
        while let Some(p1) = parsed_document.sections.pop() {
            if p1.is_commented {
                continue;
            }

            // while this is a specific to entire document, we are still creating it in a loop
            // because otherwise the self.interpret() call won't compile.

            let doc = ftd::p2::TDoc {
                name: &parsed_document.name,
                aliases: &parsed_document.doc_aliases,
                bag: &self.bag,
                local_variables: &mut Default::default(),
            };

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
                instructions.push(ftd::Instruction::ChangeContainer {
                    name: doc.resolve_name_with_instruction(
                        p1.line_number,
                        p1.caption(p1.line_number, doc.name)?.as_str(),
                        &instructions,
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
                            instructions.push(ftd::Instruction::RecursiveChildComponent {
                                child: ftd::component::recursive_child_component(
                                    loop_data,
                                    &section_to_subsection,
                                    &doc,
                                    &Default::default(),
                                    None,
                                )?,
                            });
                        } else {
                            let parent = ftd::ChildComponent::from_p1(
                                p1.line_number,
                                p1.name.as_str(),
                                &p1.header,
                                &p1.caption,
                                &p1.body_without_comment(),
                                &doc,
                                &Default::default(),
                            )?;

                            let mut children = vec![];

                            for sub in p1.sub_sections.0.iter() {
                                if sub.is_commented {
                                    continue;
                                }
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
                                            &sub.body_without_comment(),
                                            &doc,
                                            &parent.arguments,
                                        )?
                                    };
                                    children.push(child);
                                }
                            }

                            instructions.push(ftd::Instruction::Component { children, parent })
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

        Ok(Interpreter::Done {
            state: self,
            instructions,
        })
    }

    fn process_imports(mut self) -> ftd::p1::Result<(Self, Option<String>)> {
        let last = self.document_stack.len() - 1;
        let top: &mut ParsedDocument = &mut self.document_stack[last];

        let mut iteration_index = 0;
        while iteration_index < top.sections.len() {
            if top.sections[iteration_index].is_commented
                || top.sections[iteration_index].name != "import"
            {
                iteration_index += 1;
                continue;
            }
            let (library_name, alias) = ftd::p2::utils::parse_import(
                &top.sections[iteration_index].caption,
                top.name.as_str(),
                top.sections[iteration_index].line_number,
            )?;

            top.doc_aliases.insert(alias, library_name.clone());

            if self.bag.contains_key(library_name.as_str()) {
                iteration_index += 1;
                continue;
            }

            top.sections.remove(iteration_index);
            return Ok((self, Some(library_name)));
        }

        Ok((self, None))
    }

    pub fn continue_after_import(mut self, id: &str, source: &str) -> ftd::p1::Result<Interpreter> {
        self.document_stack.push(ParsedDocument::parse(id, source)?);
        self.continue_()
        // interpret then
        // handle top / start_from
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
        lib: &dyn ftd::p2::Library,
    ) -> ftd::p1::Result<Interpreter> {
        let l = self.document_stack.len() - 1;
        let parsed_document = &mut self.document_stack[l];
        let doc = ftd::p2::TDoc {
            name: &parsed_document.name,
            aliases: &parsed_document.doc_aliases,
            bag: &self.bag,
            local_variables: &mut Default::default(),
        };
        let value = lib.process(p1, &doc)?;

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
}

impl ParsedDocument {
    fn parse(id: &str, source: &str) -> ftd::p1::Result<ParsedDocument> {
        Ok(ParsedDocument {
            name: id.to_string(),
            sections: ftd::p1::parse(source, id)?,
            processing_imports: true,
            doc_aliases: ftd::p2::interpreter::default_aliases(),
            var_types: Default::default(),
        })
    }

    fn done_processing_imports(&mut self) {
        self.processing_imports = false;
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
            },
        )?;
        new_p1.reverse();
        self.sections = new_p1;
        self.var_types = var_types;
        Ok(())
    }
}

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
    Done {
        state: InterpreterState,
        instructions: Vec<ftd::Instruction>,
    },
}

pub fn interpret(id: &str, source: &str) -> ftd::p1::Result<Interpreter> {
    let mut s = InterpreterState::new();
    s.document_stack.push(ParsedDocument::parse(id, source)?);
    s.continue_()
}

pub fn interpret_helper(
    name: &str,
    source: &str,
    lib: &dyn ftd::p2::Library,
) -> ftd::p1::Result<ftd::RT> {
    let mut s = interpret(name, source)?;
    let instructions: Vec<ftd::Instruction>;
    let state;
    loop {
        match s {
            Interpreter::Done {
                instructions: i,
                state: st,
            } => {
                if st.document_stack.len() > 1 {
                    s = st.continue_after_pop()?;
                } else {
                    instructions = i;
                    state = st;
                    break;
                }
            }
            Interpreter::StuckOnProcessor { state, section } => {
                s = state.continue_after_processor(&section, lib)?;
            }
            Interpreter::StuckOnImport { module, state: st } => {
                let mut bt: std::collections::BTreeMap<String, ftd::p2::Thing> =
                    std::collections::BTreeMap::new();
                let source = lib.get_with_result(module.as_str(), &st.tdoc(&mut bt))?;
                s = st.continue_after_import(module.as_str(), source.as_str())?;
            }
        }
    }

    Ok(ftd::RT::from(
        name,
        state.document_stack[0].clone().doc_aliases,
        state.bag,
        instructions,
    ))
}

#[cfg(test)]
mod test {
    use ftd::test::*;

    #[test]
    fn basic_1() {
        let mut bag = ftd::p2::interpreter::default_bag();
        bag.insert(
            "foo/bar#foo".to_string(),
            ftd::p2::Thing::Component(ftd::Component {
                root: "ftd#text".to_string(),
                full_name: s("foo/bar#foo"),
                properties: std::array::IntoIter::new([(
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
            (bag, ftd::p2::interpreter::default_column()),
        );
    }
}
