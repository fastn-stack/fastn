#[derive(Debug, Default)]
pub struct InterpreterState {
    pub id: String,
    pub package_name: Option<String>,
    pub bag: ftd::Map<ftd::ftd2021::p2::Thing>,
    pub document_stack: Vec<ParsedDocument>,
    pub parsed_libs: ftd::Map<Vec<String>>,
}

impl InterpreterState {
    fn new(id: String, package_name: Option<String>) -> InterpreterState {
        InterpreterState {
            id,
            package_name,
            bag: ftd::ftd2021::p2::interpreter::default_bag(),
            ..Default::default()
        }
    }

    pub fn tdoc<'a>(
        &'a self,
        local_variables: &'a mut ftd::Map<ftd::ftd2021::p2::Thing>,
        referenced_local_variables: &'a mut ftd::Map<String>,
    ) -> ftd::ftd2021::p2::TDoc<'a> {
        let l = self.document_stack.len() - 1;
        ftd::ftd2021::p2::TDoc {
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

    fn continue_(mut self) -> ftd::ftd2021::p1::Result<Interpreter> {
        if self.document_stack.is_empty() {
            panic!()
        }

        let l = self.document_stack.len() - 1; // Get the top of the stack

        // Removing commented parts from the parsed document
        // Process this only once per parsed document no need to overdo it
        if self.document_stack[l].processing_comments {
            self.document_stack[l].ignore_comments();
            self.document_stack[l].done_processing_comments();
        }
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

            let doc = ftd::ftd2021::p2::TDoc {
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

            // resolve for links before popping out the section
            if !p1.is_processed_for_links {
                let replace_blocks =
                    Self::resolve_global_ids(p1, &doc, &parsed_document.var_types)?;

                if !replace_blocks.is_empty() {
                    return Ok(Interpreter::CheckID {
                        replace_blocks,
                        state: self,
                    });
                }
                p1.done_processing_links();
            }

            // Once the foreign_variables are resolved for the section, then pop and evaluate it.
            // This ensures that a section is evaluated once only.
            let p1 = parsed_document.sections.pop().unwrap();

            // while this is a specific to entire document, we are still creating it in a loop
            // because otherwise the self.interpret() call won't compile.

            let var_data = ftd::ftd2021::variable::VariableData::get_name_kind(
                &p1.name,
                &doc,
                p1.line_number,
                &parsed_document.var_types,
            );

            let mut thing = vec![];

            if p1.name.starts_with("record ") {
                // declare a record
                let d = ftd::ftd2021::p2::Record::from_p1(
                    p1.name.as_str(),
                    &p1.header,
                    &doc,
                    p1.line_number,
                )?;
                let name = doc.resolve_name(p1.line_number, &d.name.to_string())?;
                if self.bag.contains_key(name.as_str()) {
                    return ftd::ftd2021::p2::utils::e2(
                        format!("{} is already declared", d.name),
                        doc.name,
                        p1.line_number,
                    );
                }
                thing.push((name, ftd::ftd2021::p2::Thing::Record(d)));
            } else if p1.name.starts_with("or-type ") {
                // declare a record
                let d = ftd::ftd2021::OrType::from_p1(&p1, &doc)?;
                let name = doc.resolve_name(p1.line_number, &d.name.to_string())?;
                if self.bag.contains_key(name.as_str()) {
                    return ftd::ftd2021::p2::utils::e2(
                        format!("{} is already declared", d.name),
                        doc.name,
                        p1.line_number,
                    );
                }
                thing.push((name, ftd::ftd2021::p2::Thing::OrType(d)));
            } else if p1.name.starts_with("map ") {
                let d = ftd::Variable::map_from_p1(&p1, &doc)?;
                let name = doc.resolve_name(p1.line_number, &d.name.to_string())?;
                if self.bag.contains_key(name.as_str()) {
                    return ftd::ftd2021::p2::utils::e2(
                        format!("{} is already declared", d.name),
                        doc.name,
                        p1.line_number,
                    );
                }
                thing.push((name, ftd::ftd2021::p2::Thing::Variable(d)));
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
            } else if let Ok(ftd::ftd2021::variable::VariableData {
                type_: ftd::ftd2021::variable::Type::Component,
                ..
            }) = var_data
            {
                // declare a function
                let d = ftd::Component::from_p1(&p1, &doc)?;
                let name = doc.resolve_name(p1.line_number, &d.full_name.to_string())?;
                if self.bag.contains_key(name.as_str()) {
                    return ftd::ftd2021::p2::utils::e2(
                        format!("{} is already declared", d.full_name),
                        doc.name,
                        p1.line_number,
                    );
                }
                thing.push((name, ftd::ftd2021::p2::Thing::Component(d)));
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
                    return ftd::ftd2021::p2::utils::e2(
                        format!("{} is already declared", d.name),
                        doc.name,
                        p1.line_number,
                    );
                }
                thing.push((name, ftd::ftd2021::p2::Thing::Variable(d)));
            } else if let ftd::ftd2021::p2::Thing::Variable(mut v) =
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
                let (doc_name, remaining) = ftd::ftd2021::p2::utils::get_doc_name_and_remaining(
                    doc.resolve_name(p1.line_number, p1.name.as_str())?.as_str(),
                )?;
                if remaining.is_some()
                    && p1
                        .header
                        .str_optional(doc.name, p1.line_number, "if")?
                        .is_some()
                {
                    return ftd::ftd2021::p2::utils::e2(
                        "Currently not supporting `if` for field value update.",
                        doc.name,
                        p1.line_number,
                    );
                }
                if let Some(expr) = p1.header.str_optional(doc.name, p1.line_number, "if")? {
                    let val = v.get_value(&p1, &doc)?;
                    v.conditions.push((
                        ftd::ftd2021::p2::Boolean::from_expression(
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
                    ftd::ftd2021::p2::Thing::Variable(doc.set_value(
                        p1.line_number,
                        p1.name.as_str(),
                        v,
                    )?),
                ));
            } else {
                // cloning because https://github.com/rust-lang/rust/issues/59159
                match (doc.get_thing(p1.line_number, p1.name.as_str())?).clone() {
                    ftd::ftd2021::p2::Thing::Variable(_) => {
                        return ftd::ftd2021::p2::utils::e2(
                            format!("variable should have prefix $, found: `{}`", p1.name),
                            doc.name,
                            p1.line_number,
                        );
                    }
                    ftd::ftd2021::p2::Thing::Component(c) => {
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
                            let section_to_subsection = ftd::ftd2021::p1::SubSection {
                                name: p1.name.to_string(),
                                caption: p1.caption.to_owned(),
                                header: p1.header.to_owned(),
                                body: p1.body.to_owned(),
                                is_commented: p1.is_commented,
                                line_number: p1.line_number,
                            };
                            parsed_document.instructions.push(
                                ftd::Instruction::RecursiveChildComponent {
                                    child: ftd::ftd2021::component::recursive_child_component(
                                        loop_data,
                                        &section_to_subsection,
                                        &doc,
                                        &Default::default(),
                                        None,
                                    )?,
                                },
                            );
                        } else {
                            let mut parent = ftd::ChildComponent::from_p1(
                                p1.line_number,
                                p1.name.as_str(),
                                &p1.header,
                                &p1.caption,
                                &p1.body,
                                &doc,
                                &Default::default(),
                            )?;

                            ftd::ftd2021::InterpreterState::evaluate_component_for_headings(
                                &mut parsed_document.page_headings,
                                &c,
                                &mut parent,
                                &doc,
                                &self.package_name,
                            )?;

                            let mut children = vec![];

                            for sub in p1.sub_sections.0.iter() {
                                if let Ok(loop_data) =
                                    sub.header.str(doc.name, p1.line_number, "$loop$")
                                {
                                    children.push(
                                        ftd::ftd2021::component::recursive_child_component(
                                            loop_data,
                                            sub,
                                            &doc,
                                            &parent.arguments,
                                            None,
                                        )?,
                                    );
                                } else {
                                    let root_name =
                                        ftd::ftd2021::p2::utils::get_root_component_name(
                                            &doc,
                                            parent.root.as_str(),
                                            sub.line_number,
                                        )?;
                                    let child = if root_name.eq("ftd#text") {
                                        ftd::ftd2021::p2::utils::get_markup_child(
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
                    ftd::ftd2021::p2::Thing::Record(mut r) => {
                        r.add_instance(&p1, &doc)?;
                        thing.push((
                            doc.resolve_name(p1.line_number, &p1.name)?,
                            ftd::ftd2021::p2::Thing::Record(r),
                        ));
                    }
                    ftd::ftd2021::p2::Thing::OrType(_r) => {
                        // do we allow initialization of a record by name? nopes
                        return ftd::ftd2021::p2::utils::e2(
                            format!("'{}' is an or-type", p1.name.as_str()),
                            doc.name,
                            p1.line_number,
                        );
                    }
                    ftd::ftd2021::p2::Thing::OrTypeWithVariant { .. } => {
                        // do we allow initialization of a record by name? nopes
                        return ftd::ftd2021::p2::utils::e2(
                            format!("'{}' is an or-type variant", p1.name.as_str(),),
                            doc.name,
                            p1.line_number,
                        );
                    }
                };
            }
            self.bag.extend(thing);
        }

        if parsed_document.process_lazy_processors {
            // process lazy processors and add those to the bag
            // after interpreting the entire document

            let doc = ftd::ftd2021::p2::TDoc {
                name: &parsed_document.name,
                aliases: &parsed_document.doc_aliases,
                bag: &self.bag,
                local_variables: &mut Default::default(),
                referenced_local_variables: &mut Default::default(),
            };

            while let Some(section) = parsed_document.lazy_processor_sections.pop() {
                // currently only page-headings is a lazy processor
                if ftd::ExampleLibrary::is_lazy_processor(&section, &doc)? {
                    let mut final_list: Vec<ftd::PageHeadingItemCompat> = vec![];
                    ftd::ftd2021::InterpreterState::from_page_heading_list_to_compat(
                        &parsed_document.page_headings,
                        &mut final_list,
                    );
                    let value = doc.from_json(&final_list, &section)?;
                    return self.continue_after_processor(&section, value);
                }
            }
            parsed_document.process_lazy_processors = false;
        }

        if self.document_stack.len() > 1 {
            return self.continue_after_pop();
        }

        let mut rt = ftd::ftd2021::RT::from(
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

        let d = ftd::ftd2021::p2::document::Document {
            main,
            name: rt.name,
            data: rt.bag.clone(),
            aliases: rt.aliases,
            instructions: rt.instructions,
        };

        Ok(Interpreter::Done { document: d })
    }

    // projects the condensed page-heading list into a
    // PageHeadingItemCompat list identical to the
    // record of fpm.toc-item
    pub fn from_page_heading_list_to_compat(
        page_headings: &Vec<ftd::PageHeadingItem>,
        target_compat_list: &mut Vec<ftd::PageHeadingItemCompat>,
    ) {
        fn make_compat_item(
            title: &Option<String>,
            url: &Option<String>,
            number: &Option<String>,
        ) -> ftd::PageHeadingItemCompat {
            ftd::PageHeadingItemCompat {
                url: url.clone(),
                number: number.clone(),
                title: title.clone(),
                path: None,
                is_heading: true,
                font_icon: None,
                is_disabled: false,
                is_active: false,
                is_open: false,
                image_src: None,
                document: None,
                children: vec![],
            }
        }

        for heading_item in page_headings {
            let mut start_compat_node =
                make_compat_item(&heading_item.title, &heading_item.url, &heading_item.number);
            ftd::ftd2021::InterpreterState::from_page_heading_list_to_compat(
                &heading_item.children,
                &mut start_compat_node.children,
            );
            target_compat_list.push(start_compat_node);
        }
    }

    fn evaluate_component_for_headings(
        page_headings: &mut Vec<ftd::PageHeadingItem>,
        parent: &ftd::Component,
        child: &mut ftd::ChildComponent,
        doc: &ftd::ftd2021::p2::TDoc,
        package_name: &Option<String>,
    ) -> ftd::ftd2021::p1::Result<()> {
        // todo: work on all these cases
        // Case 2: Container component (with defined id)
        //      id = use user defined id for linking else the auto-generated one
        //      Case 2.1: Container with region
        //          Case 2.1.1: Containing markdown component with region title
        //                      - Fetch the title from this component
        //                        (if found otherwise proceed to 2.1.2)
        //          Case 2.1.2: Not containing markdown component with region title
        //                      - Fetch the title from the first markdown component
        //                        (if found otherwise no heading)
        //      Case 2.2: Container without region
        //          Case 2.2.1: Containing markdown component
        //                      - Fetch the title from this component
        //                        (if found otherwise no heading)

        if ftd::ftd2021::p2::utils::is_container_component(
            doc,
            &parent.full_name,
            parent.line_number,
        )? {
            // Not sure if this needs to be handled,
            // ignoring this for now
            if parent.kernel {
                return Ok(());
            }

            let component_id = child.properties.get("id").and_then(|id_property| {
                id_property
                    .resolve_default_value_string(doc, child.line_number)
                    .ok()
            });

            // prioritize finding ftd.text with region title
            let (container_instructions, region_property) =
                find_container_instructions_with_region(parent, doc)?;
            if let Some(region) = region_property {
                let region_value = region.resolve_default_value_string(doc, parent.line_number)?;
                if is_valid_heading_region(region_value.as_str()) {
                    for instruction in container_instructions.iter() {
                        let header_and_title =
                            extract_title_if_markdown_component(instruction, parent, child, doc);

                        let heading_number = header_and_title
                            .and_then(|(header, title)| {
                                if let (Some(actual_title), Some(actual_header)) = (title, header) {
                                    return Some((actual_title, actual_header));
                                }
                                None
                            })
                            .and_then(|(title, _header)| {
                                let mut new_item = create_page_heading_item_with_region(
                                    &component_id,
                                    title.as_str(),
                                    doc.name,
                                    region_value.clone(),
                                    package_name,
                                )
                                .ok()?;
                                let assigned_number = insert_page_heading_in_tree(
                                    page_headings,
                                    &mut new_item,
                                    &None,
                                    doc.name,
                                    package_name,
                                )
                                .ok()?;
                                Some(assigned_number)
                            });

                        // adjust numbering of the title in the header component
                        if let Some(number) = heading_number {
                            adjust_heading_number_in_component(child, number.as_str());
                            break;
                        }
                    }
                }
            }
        }

        return Ok(());

        fn is_valid_heading_region(region: &str) -> bool {
            matches!(region, "h0" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6")
        }

        fn extract_title_if_markdown_component(
            instruction: &ftd::Instruction,
            parent: &ftd::Component,
            child: &ftd::ChildComponent,
            doc: &ftd::ftd2021::p2::TDoc,
        ) -> Option<(Option<String>, Option<String>)> {
            if let ftd::Instruction::ChildComponent { child: cc } = instruction
                && cc.root.eq("ftd#text")
            {
                let text_region = cc
                    .properties
                    .get("region")
                    .and_then(|text_region_property| {
                        text_region_property
                            .resolve_default_value_string(doc, cc.line_number)
                            .ok()
                    });
                if let Some(text_region) = text_region
                    && text_region.eq("title")
                {
                    let header_and_title = cc
                        .properties
                        .get("text")
                        .and_then(|text_property| text_property.default.as_ref())
                        .and_then(|text_property_value| {
                            resolve_title_header_from_container(
                                text_property_value,
                                parent,
                                &child.properties,
                                doc,
                            )
                            .ok()
                        });

                    return header_and_title;
                }
            }
            None
        }

        /// finds the container instructions along with its region
        fn find_container_instructions_with_region(
            root_component: &ftd::Component,
            doc: &ftd::ftd2021::p2::TDoc,
        ) -> ftd::ftd2021::p1::Result<(
            Vec<ftd::Instruction>,
            Option<ftd::ftd2021::component::Property>,
        )> {
            if matches!(root_component.root.as_str(), "ftd#row" | "ftd#column") {
                let region = root_component.properties.get("region");
                return Ok((root_component.instructions.clone(), region.cloned()));
            }

            if root_component.kernel {
                return Ok((vec![], None));
            }

            let parent =
                doc.get_component(root_component.line_number, root_component.root.as_str())?;

            find_container_instructions_with_region(&parent, doc)
        }

        pub fn resolve_property_value(
            property_value: &ftd::PropertyValue,
        ) -> ftd::ftd2021::p1::Result<Option<String>> {
            match property_value {
                ftd::PropertyValue::Value { value } => Ok(value.to_string()),
                ftd::PropertyValue::Variable { name, .. } => Ok(Some(format!("${name}"))),
                ftd::PropertyValue::Reference { name, .. } => Ok(Some(format!("${name}"))),
            }
        }

        fn adjust_heading_number_in_component(child: &mut ftd::ChildComponent, number: &str) {
            if !child.properties.contains_key("heading-number") {
                let number_property = ftd::ftd2021::component::Property {
                    default: Some(ftd::PropertyValue::Value {
                        value: (ftd::Value::List {
                            data: {
                                let mut property_values: Vec<ftd::PropertyValue> = vec![];
                                for num in number.split('.') {
                                    property_values.push(ftd::PropertyValue::Value {
                                        value: ftd::Value::String {
                                            text: num.to_string(),
                                            source: ftd::TextSource::Default,
                                        },
                                    })
                                }
                                property_values
                            },
                            kind: ftd::ftd2021::p2::Kind::string(),
                        }),
                    }),
                    ..Default::default()
                };
                child
                    .properties
                    .insert("heading-number".to_string(), number_property);
            }
        }

        fn resolve_title_header_from_container(
            text_property_value: &ftd::PropertyValue,
            current_component: &ftd::Component,
            properties: &ftd::Map<ftd::ftd2021::component::Property>,
            doc: &ftd::ftd2021::p2::TDoc,
        ) -> ftd::ftd2021::p1::Result<(Option<String>, Option<String>)> {
            if matches!(
                current_component.full_name.as_str(),
                "ftd#row" | "ftd#column"
            ) {
                return Ok((None, resolve_property_value(text_property_value)?));
            }

            if current_component.kernel {
                return Ok((None, None));
            }

            let root_component = doc.get_component(
                current_component.line_number,
                current_component.root.as_str(),
            )?;

            let (_, partial_resolved_title) = resolve_title_header_from_container(
                text_property_value,
                &root_component,
                &root_component.properties,
                doc,
            )?;

            if let Some(partial) = partial_resolved_title {
                let property_value = partial
                    .strip_prefix('$')
                    .and_then(|stripped_partial_header| properties.get(stripped_partial_header))
                    .and_then(|property| property.default.as_ref());

                if let Some(value) = property_value {
                    return Ok((
                        Some(partial.trim_start_matches('$').to_string()),
                        resolve_property_value(value)?,
                    ));
                }

                return Ok((None, Some(partial)));
            }
            Ok((None, None))
        }

        fn make_url(
            doc_name: &str,
            id: &Option<String>,
            package_name: &Option<String>,
        ) -> Option<String> {
            fn trim_package_from_url(url: String, package_name: &Option<String>) -> String {
                let trimmed_package = package_name.as_ref().map(|actual_package| {
                    url.trim_start_matches('/')
                        .trim_start_matches(actual_package.as_str())
                });
                if let Some(res) = trimmed_package {
                    return res.to_string();
                }
                url.trim_start_matches('/').to_string()
            }

            // remove package name from the url and keep the rest
            if let Some(actual_id) = id {
                let document_id = ftd::ftd2021::p2::utils::convert_to_document_id(doc_name);
                let original_url = format!("{}#{}", document_id, slug::slugify(actual_id));
                let url = trim_package_from_url(original_url, package_name);

                return Some(url);
            }
            None
        }

        // creates a new page-heading item along with its region
        fn create_page_heading_item_with_region(
            id: &Option<String>,
            title: &str,
            doc_name: &str,
            region: String,
            package_name: &Option<String>,
        ) -> ftd::ftd2021::p1::Result<ftd::PageHeadingItem> {
            let processed_url = make_url(doc_name, id, package_name);
            let ph = ftd::PageHeadingItem {
                title: Some(title.to_string()),
                url: processed_url,
                region: ftd::Region::from(Some(region), doc_name)?,
                number: None,
                children: vec![],
            };
            Ok(ph)
        }

        fn insert_page_heading_in_tree(
            tree_nodes: &mut Vec<ftd::PageHeadingItem>,
            new_heading: &mut ftd::PageHeadingItem,
            heading_number: &Option<String>,
            doc_name: &str,
            package_name: &Option<String>,
        ) -> ftd::ftd2021::p1::Result<String> {
            fn assign_auto_slug_id(
                heading: &mut ftd::PageHeadingItem,
                _assigned_number: &str,
                doc_name: &str,
                package_name: &Option<String>,
            ) {
                if let Some(title) = &heading.title {
                    // let auto_component_id = Some(slug::slugify(format!("{} {}", assigned_number, title)));
                    let auto_component_id = Some(title.clone());
                    heading.url = make_url(doc_name, &auto_component_id, package_name);
                }
            }

            fn get_depth_number(current_depth_index: usize, number: &Option<String>) -> String {
                if let Some(number) = number {
                    return format!("{number}.{current_depth_index}");
                }
                format!("{current_depth_index}")
            }

            let current_depth_nodes = tree_nodes.len();
            let new_heading_number = get_depth_number(current_depth_nodes + 1, heading_number);

            if tree_nodes.is_empty() {
                new_heading.number = Some(new_heading_number.clone());
                if new_heading.url.is_none() {
                    assign_auto_slug_id(
                        new_heading,
                        new_heading_number.as_str(),
                        doc_name,
                        package_name,
                    );
                }
                tree_nodes.push(new_heading.to_owned());
                return Ok(new_heading_number);
            }

            if let Some(last_heading) = tree_nodes.last_mut()
                && let (Some(current_heading_region), Some(last_heading_region)) =
                    (&new_heading.region, &last_heading.region)
            {
                let last_heading_priority = last_heading_region.heading_priority_value(doc_name)?;
                let current_heading_priority =
                    current_heading_region.heading_priority_value(doc_name)?;

                if current_heading_priority < last_heading_priority {
                    return insert_page_heading_in_tree(
                        &mut last_heading.children,
                        new_heading,
                        &last_heading.number,
                        doc_name,
                        package_name,
                    );
                }
            }

            new_heading.number = Some(new_heading_number.clone());
            if new_heading.url.is_none() {
                assign_auto_slug_id(
                    new_heading,
                    new_heading_number.as_str(),
                    doc_name,
                    package_name,
                );
            }
            tree_nodes.push(new_heading.to_owned());
            Ok(new_heading_number)
        }
    }

    fn resolve_foreign_variable_name(name: &str) -> String {
        name.replace('.', "-")
    }

    fn resolve_foreign_variable(
        section: &mut ftd::ftd2021::p1::Section,
        foreign_variables: &[String],
        doc: &ftd::ftd2021::p2::TDoc,
    ) -> ftd::ftd2021::p1::Result<Option<String>> {
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
            header: &mut ftd::ftd2021::p1::Header,
            body: &mut Option<(usize, String)>,
            line_number: usize,
            foreign_variables: &[String],
            doc: &ftd::ftd2021::p2::TDoc,
        ) -> ftd::ftd2021::p1::Result<Option<String>> {
            if let Some(caption) = caption
                && let Some(cap) =
                    process_foreign_variables(caption, foreign_variables, doc, line_number)?
            {
                return Ok(Some(cap));
            }

            for (line_number, _, header) in header.0.iter_mut() {
                if let Some(h) =
                    process_foreign_variables(header, foreign_variables, doc, *line_number)?
                {
                    return Ok(Some(h));
                }
            }

            if let Some((line_number, body)) = body
                && let Some(b) =
                    process_foreign_variables(body, foreign_variables, doc, *line_number)?
            {
                return Ok(Some(b));
            }

            Ok(None)
        }

        fn process_foreign_variables(
            value: &mut String,
            foreign_variables: &[String],
            doc: &ftd::ftd2021::p2::TDoc,
            line_number: usize,
        ) -> ftd::ftd2021::p1::Result<Option<String>> {
            if value.contains('#') {
                return Ok(None);
            }
            if let Some(val) = value.clone().strip_prefix('$')
                && is_foreign_variable(val, foreign_variables, doc, line_number)?
            {
                let val = doc.resolve_name(line_number, val)?;
                *value = ftd::ftd2021::InterpreterState::resolve_foreign_variable_name(
                    format!("${}", val.as_str()).as_str(),
                );
                return Ok(Some(val));
            }
            Ok(None)
        }

        fn is_foreign_variable(
            variable: &str,
            foreign_variables: &[String],
            doc: &ftd::ftd2021::p2::TDoc,
            line_number: usize,
        ) -> ftd::ftd2021::p1::Result<bool> {
            let var_name = doc.resolve_name(line_number, variable)?;

            if foreign_variables.iter().any(|v| var_name.starts_with(v)) {
                return Ok(true);
            }
            Ok(false)
        }
    }

    // TODO: Need to avoid double usage of regex while resolving and replacing for links

    /// returns a vector of replace blocks where link replacement or escaped links needs to be resolved
    /// along with the target textSource where these changes needs to happen
    ///
    /// text-source includes caption, header, body of the section
    #[allow(clippy::type_complexity)]
    fn resolve_global_ids(
        section: &mut ftd::ftd2021::p1::Section,
        doc: &ftd::ftd2021::p2::TDoc,
        var_types: &[String],
    ) -> ftd::ftd2021::p1::Result<Vec<ftd::ReplaceLinkBlock<std::collections::HashSet<String>>>>
    {
        // will contain all replace blocks for section and its sub_sections
        // where link replacement or escape links need to be resolved
        let mut replace_blocks: Vec<ftd::ReplaceLinkBlock<std::collections::HashSet<String>>> =
            vec![];

        if ftd::ftd2021::p2::utils::is_section_subsection_component(
            section.name.as_str(),
            doc,
            var_types,
            section.line_number,
        )? && let ftd::ftd2021::p2::Thing::Component(c) =
            doc.get_thing(section.line_number, section.name.as_str())?
            && ftd::ftd2021::p2::utils::is_markdown_component(
                doc,
                c.full_name.as_str(),
                section.line_number,
            )?
        {
            replace_blocks.extend(resolve_id_from_all_sources(
                &section.caption,
                &section.header,
                &section.body,
                section.line_number,
                true,
                0,
            ));
        }

        for (subsection_index, subsection) in itertools::enumerate(section.sub_sections.0.iter()) {
            if ftd::ftd2021::p2::utils::is_section_subsection_component(
                subsection.name.as_str(),
                doc,
                var_types,
                subsection.line_number,
            )? && let ftd::ftd2021::p2::Thing::Component(c) =
                doc.get_thing(subsection.line_number, subsection.name.as_str())?
                && ftd::ftd2021::p2::utils::is_markdown_component(
                    doc,
                    c.full_name.as_str(),
                    subsection.line_number,
                )?
            {
                replace_blocks.extend(resolve_id_from_all_sources(
                    &subsection.caption,
                    &subsection.header,
                    &subsection.body,
                    subsection.line_number,
                    false,
                    subsection_index,
                ));
            }
        }

        return Ok(replace_blocks);

        /// returns a vector of replace blocks for every text-source where link replacement or
        /// escaped links resolution needs to happen for a single section/ subsection
        fn resolve_id_from_all_sources(
            caption: &Option<String>,
            header: &ftd::ftd2021::p1::Header,
            body: &Option<(usize, String)>,
            line_number: usize,
            is_from_section: bool,
            index: usize,
        ) -> Vec<ftd::ReplaceLinkBlock<std::collections::HashSet<String>>> {
            let mut replace_blocks: Vec<ftd::ReplaceLinkBlock<std::collections::HashSet<String>>> =
                vec![];

            if let Some(caption) = caption {
                let (captured_ids, process_for_escaped_links) = find_referenced_links(caption);
                if !captured_ids.is_empty() || process_for_escaped_links {
                    replace_blocks.push((
                        captured_ids,
                        (ftd::TextSource::Caption, (is_from_section, index)),
                        line_number,
                    ));
                }
            }

            for (ln, _, header) in header.0.iter() {
                let (captured_ids, process_for_escaped_links) = find_referenced_links(header);
                if !captured_ids.is_empty() || process_for_escaped_links {
                    replace_blocks.push((
                        captured_ids,
                        (ftd::TextSource::Header, (is_from_section, index)),
                        *ln,
                    ));
                }
            }

            if let Some((ln, body)) = body {
                let (captured_ids, process_for_escaped_links) = find_referenced_links(body);
                if !captured_ids.is_empty() || process_for_escaped_links {
                    replace_blocks.push((
                        captured_ids,
                        (ftd::TextSource::Body, (is_from_section, index)),
                        *ln,
                    ));
                }
            }

            replace_blocks
        }

        /// returns (captured_ids, process_for_escaped_links) for the given text
        ///
        /// captured_ids = set of captured ids associated with the links present in the given text
        /// which needs to be resolved
        ///
        /// process_for_escaped_links = boolean if escaped links needs to be resolved
        fn find_referenced_links(value: &str) -> (std::collections::HashSet<String>, bool) {
            let mut process_for_escaped_links = false;
            let mut captured_ids: std::collections::HashSet<String> =
                std::collections::HashSet::new();

            // Character Prefix Group <prefix>
            // Referred Id Capture Group <id_or_text>
            // <type1> group and <ahead> group for any possible link
            for capture in ftd::regex::S.captures_iter(value) {
                let prefix = ftd::regex::capture_group_by_name(&capture, "prefix");

                // check if link is escaped ignore if true
                if !prefix.is_empty() && prefix.eq(r"\") {
                    process_for_escaped_links = true;
                    continue;
                }

                let type1 = ftd::regex::capture_group_by_name(&capture, "type1").trim();
                match type1.is_empty() {
                    true => {
                        // Type 2 syntax: [<id>]
                        // id = <id_or_text> group = <id>
                        // Linked text = id

                        let ahead = ftd::regex::capture_group_by_name(&capture, "ahead");
                        // ignore if a resolved link already exists
                        if !ahead.is_empty() && ftd::regex::URL.is_match(ahead) {
                            continue;
                        }

                        let captured_id =
                            ftd::regex::capture_group_by_name(&capture, "id_or_text").trim();

                        // In case user doesn't provide any id
                        match captured_id.is_empty() {
                            true => continue,
                            false => {
                                // In case user uses [] as checkboxes instead of links ignore them
                                // - [ ] item1
                                // - [x] item2
                                if matches!(captured_id, "x" | "X") {
                                    continue;
                                }
                            }
                        }

                        captured_ids.insert(captured_id.to_string());
                    }
                    false => {
                        // Type 1 syntax [<link_text>](<type1><id>)
                        // Linked text = <id_or_text>
                        // id = <id>
                        let captured_id = ftd::regex::capture_group_by_name(&capture, "id").trim();

                        // In case user doesn't provide any id
                        if captured_id.is_empty() {
                            continue;
                        }

                        captured_ids.insert(captured_id.to_string());
                    }
                }
            }

            (captured_ids, process_for_escaped_links)
        }
    }

    fn process_imports(
        top: &mut ParsedDocument,
        bag: &ftd::Map<ftd::ftd2021::p2::Thing>,
    ) -> ftd::ftd2021::p1::Result<Option<String>> {
        let mut iteration_index = 0;
        while iteration_index < top.sections.len() {
            if top.sections[iteration_index].name != "import" {
                iteration_index += 1;
                continue;
            }
            let (library_name, alias) = ftd::ftd2021::p2::utils::parse_import(
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

    /// store the section which needs to be resolved after interpretation
    /// and continue for the next section
    pub fn continue_after_storing_section(
        mut self,
        section: &ftd::ftd2021::p1::Section,
    ) -> ftd::ftd2021::p1::Result<Interpreter> {
        fn add_dummy_variable(
            parsed_document: &mut ParsedDocument,
            p1: &ftd::ftd2021::p1::Section,
            bag: &mut ftd::Map<ftd::ftd2021::p2::Thing>,
        ) -> ftd::ftd2021::p1::Result<()> {
            let doc = ftd::ftd2021::p2::TDoc {
                name: &parsed_document.name,
                aliases: &parsed_document.doc_aliases,
                bag,
                local_variables: &mut Default::default(),
                referenced_local_variables: &mut Default::default(),
            };

            let var_data = ftd::ftd2021::variable::VariableData::get_name_kind(
                &p1.name,
                &doc,
                p1.line_number,
                &parsed_document.var_types,
            );

            if let Ok(ftd::ftd2021::variable::VariableData {
                type_: ftd::ftd2021::variable::Type::Variable,
                name,
                ..
            }) = var_data
            {
                let name = doc.resolve_name(p1.line_number, &name)?;
                let ph: Vec<ftd::PageHeadingItem> = vec![];
                let dummy_value = doc.from_json(&ph, p1)?;
                let variable = ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                    name: name.clone(),
                    value: ftd::PropertyValue::Value { value: dummy_value },
                    conditions: vec![],
                    flags: ftd::ftd2021::variable::VariableFlags::from_p1(
                        &p1.header,
                        doc.name,
                        p1.line_number,
                    )?,
                });
                bag.insert(name, variable);
            }
            Ok(())
        }

        // Store the section which needs to be processed after interpreting
        // Where this section should be stored ? (could be a thought to consider)
        // For now it's kept under the parsed document
        if let Some(current_processing_document) = self.document_stack.last_mut() {
            current_processing_document
                .lazy_processor_sections
                .push(section.to_owned());
            current_processing_document.process_lazy_processors = true;

            // insert a placeholder (dummy) variable so as to ensure there exists a variable
            // with the same name if this is used by some other section in the same document
            // and doesnt throw any error
            add_dummy_variable(current_processing_document, section, &mut self.bag)?;
        }

        // Store first then go ahead
        self.continue_()
    }

    pub fn continue_after_checking_id(
        mut self,
        replace_blocks: Vec<ftd::ReplaceLinkBlock<std::collections::HashMap<String, String>>>,
    ) -> ftd::ftd2021::p1::Result<Interpreter> {
        // Checking in the last section from the topmost document in the document stack
        // which isn't popped out yet and replace links based on the captured id set
        // it received from the current processing section
        // NOTE: need to run regex again to find link syntax
        // match for the given id and replace it with the url received
        if let Some(current_processing_document) = self.document_stack.last_mut()
            && let Some(current_processing_section) =
                current_processing_document.get_last_mut_section()
        {
            for (id_map, source, ln) in replace_blocks.iter() {
                let is_from_section = source.1.0;
                let target_text_source = &source.0;

                match is_from_section {
                    true => match target_text_source {
                        ftd::TextSource::Caption => {
                            if let Some(ref mut cap) = current_processing_section.caption {
                                replace_all_links(cap, id_map, self.id.clone(), *ln)?;
                            }
                        }
                        ftd::TextSource::Header => {
                            for (_, _, v) in current_processing_section.header.0.iter_mut() {
                                replace_all_links(v, id_map, self.id.clone(), *ln)?;
                            }
                        }
                        ftd::TextSource::Body => {
                            if let Some(ref mut body) = current_processing_section.body {
                                replace_all_links(&mut body.1, id_map, self.id.clone(), *ln)?;
                            }
                        }
                        _ => {
                            unimplemented!()
                        }
                    },
                    false => {
                        let target_subsection_index = source.1.1;
                        let subsections = &mut current_processing_section.sub_sections.0;

                        let current_processing_subsection = subsections
                                .get_mut(target_subsection_index)
                                .ok_or_else(|| ftd::ftd2021::p1::Error::UnknownData {
                                    message: format!(
                                        "No subsection present at index {target_subsection_index} in sub_sections"
                                    ),
                                    doc_id: self.id.clone(),
                                    line_number: *ln,
                                })?;

                        match target_text_source {
                            ftd::TextSource::Caption => {
                                if let Some(ref mut cap) = current_processing_subsection.caption {
                                    replace_all_links(cap, id_map, self.id.clone(), *ln)?;
                                }
                            }
                            ftd::TextSource::Header => {
                                for (_, _, v) in current_processing_subsection.header.0.iter_mut() {
                                    replace_all_links(v, id_map, self.id.clone(), *ln)?;
                                }
                            }
                            ftd::TextSource::Body => {
                                if let Some(ref mut body) = current_processing_subsection.body {
                                    replace_all_links(&mut body.1, id_map, self.id.clone(), *ln)?;
                                }
                            }
                            _ => {
                                unimplemented!()
                            }
                        }
                    }
                }
            }
            current_processing_section.done_processing_links();
        }

        return self.continue_();

        /// replaces all links present in any textsource from a single section and its sub_sections,
        /// also resolves any escaped links if present.
        /// returns true if any replacement took place else false
        fn replace_all_links(
            value: &mut String,
            id_map: &std::collections::HashMap<String, String>,
            doc_id: String,
            line_number: usize,
        ) -> ftd::ftd2021::p1::Result<bool> {
            let mut is_replaced = false;
            let mut matches_with_replacements: Vec<(String, usize, usize)> = vec![];
            // Character Prefix Group <prefix>
            // Referred Id Capture Group <id_or_text>
            // <type1> Group and <id> Group for id from type 1 syntax";
            for capture in ftd::regex::S.captures_iter(value.as_ref()) {
                let prefix = ftd::regex::capture_group_by_name(&capture, "prefix");
                let type1 = ftd::regex::capture_group_by_name(&capture, "type1");

                match type1.trim().is_empty() {
                    true => {
                        // Type 2 syntax: [<id>]
                        // Match <prefix>?[<id_or_text>](<ahead>)?
                        // id = <id_or_text> group = <id>
                        // Linked text = id

                        let matched_pattern = ftd::regex::capture_group_by_index(&capture, 0);
                        let match_length = matched_pattern.len();
                        let match_start_index = capture.get(0).unwrap().start();

                        let ahead = ftd::regex::capture_group_by_name(&capture, "ahead");
                        let linked_text = ftd::regex::capture_group_by_name(&capture, "id_or_text");
                        let captured_id = linked_text.trim();

                        // In case, the link is escaped, ignore it
                        if !prefix.is_empty() && prefix.eq(r"\") {
                            let match_without_prefix = match ahead.is_empty() {
                                true => format!("[{linked_text}]"),
                                false => format!("[{linked_text}]({ahead})"),
                            };
                            matches_with_replacements.push((
                                match_without_prefix,
                                match_start_index,
                                match_length,
                            ));
                            continue;
                        }

                        // in case resolved link already exists
                        if !ahead.is_empty() && ftd::regex::URL.is_match(ahead) {
                            continue;
                        }

                        // In case the user doesn't provide any id
                        // like this - [ ] consider this as an empty checkbox
                        match captured_id.is_empty() {
                            true => continue,
                            false => {
                                // In case user uses [x] or [X] as checkboxes instead of links ignore them
                                if matches!(captured_id, "x" | "X") {
                                    continue;
                                }
                            }
                        }

                        let link = id_map.get(captured_id).ok_or_else(|| {
                            ftd::ftd2021::p1::Error::NotFound {
                                doc_id: doc_id.clone(),
                                line_number,
                                key: format!(
                                    "{captured_id} not found in id_map while replacing for links"
                                ),
                            }
                        })?;

                        let mut replacement = format!("[{linked_text}]({link})");
                        if !prefix.is_empty() {
                            replacement = format!("{prefix}{replacement}");
                        }

                        matches_with_replacements.push((
                            replacement,
                            match_start_index,
                            match_length,
                        ));
                    }
                    false => {
                        // Type 1 syntax: [<link-text>](id: <id>)
                        // Match <prefix>?[<id_or_text>](<type1>)
                        // Linked text = <id_or_text> = <link_text>
                        // id = <id_or_ahead>

                        let captured_id = ftd::regex::capture_group_by_name(&capture, "id").trim();
                        let linked_text = ftd::regex::capture_group_by_name(&capture, "id_or_text");

                        let matched_pattern = ftd::regex::capture_group_by_index(&capture, 0);
                        let match_start_index = capture.get(0).unwrap().start();
                        let match_length = matched_pattern.len();

                        // In case, the link is escaped, ignore it
                        if !prefix.is_empty() && prefix.eq(r"\") {
                            let match_without_prefix = format!("[{linked_text}]({type1})");
                            matches_with_replacements.push((
                                match_without_prefix,
                                match_start_index,
                                match_length,
                            ));
                            continue;
                        }

                        // In case the user doesn't provide any id
                        if captured_id.is_empty() {
                            continue;
                        }

                        let link = id_map.get(captured_id).ok_or_else(|| {
                            ftd::ftd2021::p1::Error::NotFound {
                                doc_id: doc_id.clone(),
                                line_number,
                                key: format!(
                                    "{captured_id} not found in id_map while replacing for links"
                                ),
                            }
                        })?;

                        let mut replacement = format!("[{linked_text}]({link})");
                        if !prefix.is_empty() {
                            replacement = format!("{prefix}{replacement}");
                        }

                        matches_with_replacements.push((
                            replacement,
                            match_start_index,
                            match_length,
                        ));
                    }
                }
            }

            // replace all link syntax with actual links, also fix the escaped links
            while let Some((replacement, match_start_index, match_length)) =
                matches_with_replacements.pop()
            {
                *value = format!(
                    "{}{}{}",
                    &value[..match_start_index],
                    replacement,
                    &value[match_start_index + match_length..]
                );
                is_replaced = true;
            }

            Ok(is_replaced)
        }
    }

    pub fn continue_after_import(
        mut self,
        id: &str,
        source: &str,
    ) -> ftd::ftd2021::p1::Result<Interpreter> {
        self.document_stack.push(ParsedDocument::parse(id, source)?);
        self.continue_()
        // interpret then
        // handle top / start_from
    }

    pub fn continue_after_variable(
        mut self,
        variable: &str,
        value: ftd::Value,
    ) -> ftd::ftd2021::p1::Result<Interpreter> {
        let l = self.document_stack.len() - 1;
        let doc = ftd::ftd2021::p2::TDoc {
            name: &self.document_stack[l].name,
            aliases: &self.document_stack[l].doc_aliases,
            bag: &self.bag,
            local_variables: &mut Default::default(),
            referenced_local_variables: &mut Default::default(),
        };
        let var_name = ftd::ftd2021::InterpreterState::resolve_foreign_variable_name(
            doc.resolve_name(0, variable)?.as_str(),
        );
        self.bag.insert(
            var_name.clone(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: var_name,
                value: ftd::PropertyValue::Value { value },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        self.continue_()
    }

    pub fn continue_after_pop(mut self) -> ftd::ftd2021::p1::Result<Interpreter> {
        self.document_stack.pop();
        self.continue_()
        // interpret then
        // handle top / start_from
    }

    pub fn continue_after_processor(
        mut self,
        p1: &ftd::ftd2021::p1::Section,
        value: ftd::Value,
    ) -> ftd::ftd2021::p1::Result<Interpreter> {
        let l = self.document_stack.len() - 1;
        let parsed_document = &mut self.document_stack[l];

        let doc = ftd::ftd2021::p2::TDoc {
            name: &parsed_document.name,
            aliases: &parsed_document.doc_aliases,
            bag: &self.bag,
            local_variables: &mut Default::default(),
            referenced_local_variables: &mut Default::default(),
        };

        let var_data = ftd::ftd2021::variable::VariableData::get_name_kind(
            &p1.name,
            &doc,
            p1.line_number,
            &parsed_document.var_types,
        );

        if let Ok(ftd::ftd2021::variable::VariableData {
            type_: ftd::ftd2021::variable::Type::Variable,
            name,
            ..
        }) = var_data
        {
            let name = doc.resolve_name(p1.line_number, &name)?;
            let variable = ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: name.clone(),
                value: ftd::PropertyValue::Value { value },
                conditions: vec![],
                flags: ftd::ftd2021::variable::VariableFlags::from_p1(
                    &p1.header,
                    doc.name,
                    p1.line_number,
                )?,
            });
            self.bag.insert(name, variable);
            return self.continue_();
        }

        match doc.get_thing(p1.line_number, p1.name.as_str())? {
            ftd::ftd2021::p2::Thing::Variable(mut v) => {
                // for case: 2
                let doc_name = ftd::ftd2021::p2::utils::get_doc_name_and_remaining(
                    doc.resolve_name(p1.line_number, p1.name.as_str())?.as_str(),
                )?
                .0;
                v.value = ftd::PropertyValue::Value { value };
                let key = doc.resolve_name(p1.line_number, doc_name.as_str())?;
                let variable = ftd::ftd2021::p2::Thing::Variable(doc.set_value(
                    p1.line_number,
                    p1.name.as_str(),
                    v,
                )?);
                self.bag.insert(key, variable);
            }
            ftd::ftd2021::p2::Thing::Component(_) => {
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

    pub(crate) fn p1_from_processor(p1: &mut ftd::ftd2021::p1::Section, value: ftd::Value) {
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
    sections: Vec<ftd::ftd2021::p1::Section>,
    processing_imports: bool,
    processing_comments: bool,
    process_lazy_processors: bool,
    doc_aliases: ftd::Map<String>,
    var_types: Vec<String>,
    foreign_variable_prefix: Vec<String>,
    instructions: Vec<ftd::Instruction>,
    /// sections stored which needs to be processed
    /// after interpretation of the current document is over
    lazy_processor_sections: Vec<ftd::ftd2021::p1::Section>,
    /// page headings of the current document will be stored in this list
    page_headings: Vec<ftd::PageHeadingItem>,
}

impl ParsedDocument {
    fn parse(id: &str, source: &str) -> ftd::ftd2021::p1::Result<ParsedDocument> {
        Ok(ParsedDocument {
            name: id.to_string(),
            sections: ftd::ftd2021::p1::parse(source, id)?,
            processing_imports: true,
            processing_comments: true,
            process_lazy_processors: false,
            doc_aliases: ftd::ftd2021::p2::interpreter::default_aliases(),
            var_types: Default::default(),
            foreign_variable_prefix: vec![],
            instructions: vec![],
            lazy_processor_sections: vec![],
            page_headings: vec![],
        })
    }

    fn done_processing_imports(&mut self) {
        self.processing_imports = false;
    }

    fn done_processing_comments(&mut self) {
        self.processing_comments = false;
    }

    pub fn get_last_section(&self) -> Option<&ftd::ftd2021::p1::Section> {
        self.sections.last()
    }

    pub fn get_last_mut_section(&mut self) -> Option<&mut ftd::ftd2021::p1::Section> {
        self.sections.last_mut()
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
            .filter(|s| !s.is_commented)
            .map(|s| s.remove_comments())
            .collect::<Vec<ftd::ftd2021::p1::Section>>();
    }

    fn reorder(&mut self, bag: &ftd::Map<ftd::ftd2021::p2::Thing>) -> ftd::ftd2021::p1::Result<()> {
        let (mut new_p1, var_types) = ftd::ftd2021::p2::utils::reorder(
            &self.sections,
            &ftd::ftd2021::p2::TDoc {
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
        // TODO: this should contain the name of processor as well
        state: InterpreterState,
        section: ftd::ftd2021::p1::Section,
    },
    StuckOnForeignVariable {
        variable: String,
        state: InterpreterState,
    },
    CheckID {
        replace_blocks: Vec<ftd::ReplaceLinkBlock<std::collections::HashSet<String>>>,
        state: InterpreterState,
    },
    Done {
        document: ftd::ftd2021::p2::Document,
    },
}

pub fn interpret(
    id: &str,
    source: &str,
    package_name: &Option<String>,
) -> ftd::ftd2021::p1::Result<Interpreter> {
    let mut s = InterpreterState::new(id.to_string(), package_name.clone());
    s.document_stack.push(ParsedDocument::parse(id, source)?);
    s.continue_()
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Thing {
    Component(ftd::Component),
    Variable(ftd::Variable),
    Record(ftd::ftd2021::p2::Record),
    OrType(ftd::ftd2021::OrType),
    OrTypeWithVariant {
        e: ftd::ftd2021::OrType,
        variant: String,
    },
    // Library -> Name of library successfully parsed
}

pub fn default_bag() -> ftd::Map<ftd::ftd2021::p2::Thing> {
    let record = |n: &str, r: &str| (n.to_string(), ftd::ftd2021::p2::Kind::record(r));
    let color = |n: &str| record(n, "ftd#color");
    std::iter::IntoIterator::into_iter([
        (
            "ftd#row".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::ftd2021::p2::element::row_function()),
        ),
        (
            "ftd#column".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::ftd2021::p2::element::column_function()),
        ),
        (
            "ftd#text-block".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::ftd2021::p2::element::text_function()),
        ),
        (
            "ftd#code".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::ftd2021::p2::element::code_function()),
        ),
        (
            "ftd#image".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::ftd2021::p2::element::image_function()),
        ),
        (
            "ftd#iframe".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::ftd2021::p2::element::iframe_function()),
        ),
        (
            "ftd#integer".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::ftd2021::p2::element::integer_function()),
        ),
        (
            "ftd#decimal".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::ftd2021::p2::element::decimal_function()),
        ),
        (
            "ftd#boolean".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::ftd2021::p2::element::boolean_function()),
        ),
        (
            "ftd#scene".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::ftd2021::p2::element::scene_function()),
        ),
        (
            "ftd#grid".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::ftd2021::p2::element::grid_function()),
        ),
        (
            "ftd#text".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::ftd2021::p2::element::markup_function()),
        ),
        (
            "ftd#input".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::ftd2021::p2::element::input_function()),
        ),
        (
            "ftd#null".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::ftd2021::p2::element::null()),
        ),
        (
            "ftd#dark-mode".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
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
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
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
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
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
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
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
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
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
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
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
            "ftd#markdown-color-data".to_string(),
            ftd::ftd2021::p2::Thing::Record(ftd::ftd2021::p2::Record {
                name: "ftd#markdown-color-data".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    (
                        "link".to_string(),
                        ftd::ftd2021::p2::Kind::record("ftd#color"),
                    ),
                    (
                        "code".to_string(),
                        ftd::ftd2021::p2::Kind::record("ftd#color"),
                    ),
                    (
                        "link-code".to_string(),
                        ftd::ftd2021::p2::Kind::record("ftd#color"),
                    ),
                    (
                        "link-visited".to_string(),
                        ftd::ftd2021::p2::Kind::record("ftd#color"),
                    ),
                    (
                        "link-visited-code".to_string(),
                        ftd::ftd2021::p2::Kind::record("ftd#color"),
                    ),
                    (
                        "ul-ol-li-before".to_string(),
                        ftd::ftd2021::p2::Kind::record("ftd#color"),
                    ),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![
                    "link".to_string(),
                    "code".to_string(),
                    "link-code".to_string(),
                    "link-visited".to_string(),
                    "link-visited-code".to_string(),
                    "ul-ol-li-before".to_string(),
                ],
            }),
        ),
        ("ftd#markdown-color".to_string(), markdown::color()),
        (
            "ftd#markdown-background-color-data".to_string(),
            ftd::ftd2021::p2::Thing::Record(ftd::ftd2021::p2::Record {
                name: "ftd#markdown-background-color-data".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    (
                        "link".to_string(),
                        ftd::ftd2021::p2::Kind::record("ftd#color"),
                    ),
                    (
                        "code".to_string(),
                        ftd::ftd2021::p2::Kind::record("ftd#color"),
                    ),
                    (
                        "link-code".to_string(),
                        ftd::ftd2021::p2::Kind::record("ftd#color"),
                    ),
                    (
                        "link-visited".to_string(),
                        ftd::ftd2021::p2::Kind::record("ftd#color"),
                    ),
                    (
                        "link-visited-code".to_string(),
                        ftd::ftd2021::p2::Kind::record("ftd#color"),
                    ),
                    (
                        "ul-ol-li-before".to_string(),
                        ftd::ftd2021::p2::Kind::record("ftd#color"),
                    ),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![
                    "link".to_string(),
                    "code".to_string(),
                    "link-code".to_string(),
                    "link-visited".to_string(),
                    "link-visited-code".to_string(),
                    "ul-ol-li-before".to_string(),
                ],
            }),
        ),
        (
            "ftd#markdown-background-color".to_string(),
            markdown::background_color(),
        ),
        (
            "ftd#image-src".to_string(),
            ftd::ftd2021::p2::Thing::Record(ftd::ftd2021::p2::Record {
                name: "ftd#image-src".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    ("light".to_string(), ftd::ftd2021::p2::Kind::caption()),
                    ("dark".to_string(), ftd::ftd2021::p2::Kind::string()),
                ])
                .collect(),
                instances: Default::default(),
                order: vec!["light".to_string(), "dark".to_string()],
            }),
        ),
        (
            "ftd#color".to_string(),
            ftd::ftd2021::p2::Thing::Record(ftd::ftd2021::p2::Record {
                name: "ftd#color".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    ("light".to_string(), ftd::ftd2021::p2::Kind::caption()),
                    ("dark".to_string(), ftd::ftd2021::p2::Kind::string()),
                ])
                .collect(),
                instances: Default::default(),
                order: vec!["light".to_string(), "dark".to_string()],
            }),
        ),
        (
            "ftd#font-size".to_string(),
            ftd::ftd2021::p2::Thing::Record(ftd::ftd2021::p2::Record {
                name: "ftd#font-size".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    ("line-height".to_string(), ftd::ftd2021::p2::Kind::integer()),
                    ("size".to_string(), ftd::ftd2021::p2::Kind::integer()),
                    (
                        "letter-spacing".to_string(),
                        ftd::ftd2021::p2::Kind::integer().set_default(Some("0".to_string())),
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
            ftd::ftd2021::p2::Thing::Record(ftd::ftd2021::p2::Record {
                name: "ftd#type".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    ("font".to_string(), ftd::ftd2021::p2::Kind::caption()),
                    (
                        "desktop".to_string(),
                        ftd::ftd2021::p2::Kind::record("ftd#font-size"),
                    ),
                    (
                        "mobile".to_string(),
                        ftd::ftd2021::p2::Kind::record("ftd#font-size"),
                    ),
                    (
                        "xl".to_string(),
                        ftd::ftd2021::p2::Kind::record("ftd#font-size"),
                    ),
                    (
                        "weight".to_string(),
                        ftd::ftd2021::p2::Kind::integer().set_default(Some("400".to_string())),
                    ),
                    (
                        "style".to_string(),
                        ftd::ftd2021::p2::Kind::string().into_optional(),
                    ),
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
            ftd::ftd2021::p2::Thing::Record(ftd::ftd2021::p2::Record {
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
            ftd::ftd2021::p2::Thing::Record(ftd::ftd2021::p2::Record {
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
            ftd::ftd2021::p2::Thing::Record(ftd::ftd2021::p2::Record {
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
            ftd::ftd2021::p2::Thing::Record(ftd::ftd2021::p2::Record {
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
            ftd::ftd2021::p2::Thing::Record(ftd::ftd2021::p2::Record {
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
            ftd::ftd2021::p2::Thing::Record(ftd::ftd2021::p2::Record {
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
        common: Box::new(ftd::Common {
            width: Some(ftd::Length::Fill),
            height: Some(ftd::Length::Fill),
            position: Some(ftd::Position::Center),
            ..Default::default()
        }),
        spacing: None,
        ..Default::default()
    }
}

pub mod markdown {
    fn theme_color(light: &str, dark: &str) -> ftd::PropertyValue {
        ftd::PropertyValue::Value {
            value: ftd::Value::Record {
                name: "ftd#color".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    (
                        "light".to_string(),
                        ftd::PropertyValue::Value {
                            value: ftd::Value::String {
                                text: light.to_string(),
                                source: ftd::TextSource::Caption,
                            },
                        },
                    ),
                    (
                        "dark".to_string(),
                        ftd::PropertyValue::Value {
                            value: ftd::Value::String {
                                text: dark.to_string(),
                                source: ftd::TextSource::Header,
                            },
                        },
                    ),
                ])
                .collect(),
            },
        }
    }

    fn link(light: &str, dark: &str) -> (String, ftd::PropertyValue) {
        ("link".to_string(), theme_color(light, dark))
    }

    fn code(light: &str, dark: &str) -> (String, ftd::PropertyValue) {
        ("code".to_string(), theme_color(light, dark))
    }

    fn link_visited(light: &str, dark: &str) -> (String, ftd::PropertyValue) {
        ("link-visited".to_string(), theme_color(light, dark))
    }

    fn link_code(light: &str, dark: &str) -> (String, ftd::PropertyValue) {
        ("link-code".to_string(), theme_color(light, dark))
    }

    fn link_visited_code(light: &str, dark: &str) -> (String, ftd::PropertyValue) {
        ("link-visited-code".to_string(), theme_color(light, dark))
    }

    fn ul_ol_li_before(light: &str, dark: &str) -> (String, ftd::PropertyValue) {
        ("ul-ol-li-before".to_string(), theme_color(light, dark))
    }

    fn blockquote(light: &str, dark: &str) -> (String, ftd::PropertyValue) {
        ("blockquote".to_string(), theme_color(light, dark))
    }

    pub fn color() -> ftd::ftd2021::p2::Thing {
        ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
            name: "ftd#markdown-color".to_string(),
            value: ftd::PropertyValue::Value {
                value: ftd::Value::Record {
                    name: "ftd#markdown-color-data".to_string(),
                    fields: std::iter::IntoIterator::into_iter([
                        link("#6a89b8", "#58a6ff"),
                        code("#f6f7f8", "#f6f7f8"),
                        link_visited("#9475cb", "#a27de7"),
                        link_code("#6a89b8", "#58a6ff"),
                        link_visited_code("#6a89b8", "#a27de7"),
                        ul_ol_li_before("#000000", "#ffffff"),
                    ])
                    .collect(),
                },
            },
            conditions: vec![],
            flags: ftd::VariableFlags {
                always_include: Some(true),
            },
        })
    }

    pub fn background_color() -> ftd::ftd2021::p2::Thing {
        ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
            name: "ftd#markdown-background-color".to_string(),
            value: ftd::PropertyValue::Value {
                value: ftd::Value::Record {
                    name: "ftd#markdown-background-color-data".to_string(),
                    fields: std::iter::IntoIterator::into_iter([
                        link("#18181b", "#18181b"),
                        code("#9f9b9b45", "#9f9b9b45"),
                        link_visited("#18181b", "#18181b"),
                        link_code("#9f9b9b45", "#9f9b9b45"),
                        link_visited_code("#18181b", "#18181b"),
                        ul_ol_li_before("#18181b", "#18181b"),
                        blockquote("#f0f0f0", "#f0f0f0"),
                    ])
                    .collect(),
                },
            },
            conditions: vec![],
            flags: ftd::VariableFlags {
                always_include: Some(true),
            },
        })
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
