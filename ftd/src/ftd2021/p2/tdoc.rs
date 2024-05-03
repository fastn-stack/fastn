#[derive(Debug, PartialEq)]
pub struct TDoc<'a> {
    pub name: &'a str,
    pub aliases: &'a ftd::Map<String>,
    pub bag: &'a ftd::Map<ftd::ftd2021::p2::Thing>,
    pub local_variables: &'a mut ftd::Map<ftd::ftd2021::p2::Thing>,
    /// string $msg: $message
    /// then msg is a referenced_variable that is won't become new local_variable
    pub referenced_local_variables: &'a mut ftd::Map<String>,
}

impl TDoc<'_> {
    fn get_local_variable<'b>(
        &'b self,
        key: &'b str,
    ) -> Option<(&'b str, &'b ftd::ftd2021::p2::Thing)> {
        if let Some(thing) = self.local_variables.get(key) {
            return Some((key, thing));
        }
        if let Some(thing) = self.bag.get(key) {
            return Some((key, thing));
        }
        if let Some(key) = self.referenced_local_variables.get(key) {
            return self.get_local_variable(key);
        }
        None
    }

    fn insert_local_variable(
        &mut self,
        root: &str,
        arguments: &mut ftd::Map<ftd::ftd2021::p2::Kind>,
        properties: &ftd::Map<ftd::ftd2021::component::Property>,
        string_container: &str,
        local_container: &[usize],
        external_children_count: &Option<usize>,
    ) -> ftd::ftd2021::p1::Result<()> {
        for (k, arg) in arguments.iter() {
            let mut default = if let Some(d) = properties.get(k) {
                let default = if let Some(ref d) = d.default {
                    d.to_owned()
                } else {
                    //todo
                    return ftd::ftd2021::p2::utils::e2(
                        format!(
                            "expected default value for local variable {}: {:?} in {}",
                            k, arg, root
                        ),
                        self.name,
                        0,
                    );
                };
                if matches!(default.kind().inner(), ftd::ftd2021::p2::Kind::UI { .. }) {
                    let root = match &default {
                        ftd::PropertyValue::Value {
                            value: ftd::Value::UI { name, .. },
                        }
                        | ftd::PropertyValue::Reference { name, .. }
                        | ftd::PropertyValue::Variable { name, .. } => name.to_string(),
                        ftd::PropertyValue::Value { value } => {
                            if let Some(ftd::Value::UI { name, .. }) = value.to_owned().inner() {
                                name
                            } else {
                                return ftd::ftd2021::p2::utils::e2(
                                    format!(
                                        "expected UI for local variable {}: {:?} in {}, found: `{:?}`",
                                        k, arg, root, value
                                    ),
                                    self.name,
                                    0,
                                );
                            }
                        }
                    }
                    .to_string();

                    let c = ftd::ftd2021::p2::Thing::Component(ftd::Component {
                        root,
                        full_name: self.resolve_local_variable_name(0, k, string_container)?,
                        arguments: Default::default(),
                        locals: Default::default(),
                        properties: d.nested_properties.clone(),
                        instructions: vec![],
                        events: vec![],
                        condition: None,
                        kernel: false,
                        invocations: vec![],
                        line_number: 0,
                    });
                    self.local_variables
                        .entry(self.resolve_local_variable_name(0, k, string_container)?)
                        .or_insert(c);
                    continue;
                }
                default
            } else if let Some(default) = arg.get_default_value_str() {
                ftd::PropertyValue::resolve_value(
                    0,
                    default.as_str(),
                    Some(arg.to_owned()),
                    self,
                    arguments,
                    None,
                )?
            } else if let Ok(value) = arg.to_value(0, self.name) {
                ftd::PropertyValue::Value { value }
            } else {
                return ftd::ftd2021::p2::utils::e2(
                    format!(
                        "expected default value for local variable 2 {}: {:?} in {}",
                        k, arg, root
                    ),
                    self.name,
                    0,
                );
            };
            if let ftd::PropertyValue::Variable { ref mut name, .. } = default {
                if !self.local_variables.contains_key(name) && !self.bag.contains_key(name) {
                    *name = self.resolve_local_variable_name(0, name, string_container)?;
                }
            }
            if let Some(name) = default.get_passed_by_variable() {
                self.referenced_local_variables.insert(
                    self.resolve_local_variable_name(0, k, string_container)?,
                    name,
                );
            } else {
                let local_variable = ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                    name: k.to_string(),
                    value: default,
                    conditions: vec![],
                    flags: Default::default(),
                });
                self.local_variables
                    .entry(self.resolve_local_variable_name(0, k, string_container)?)
                    .or_insert(local_variable);
            }
        }
        let sibling_index =
            external_children_count.unwrap_or(*local_container.last().unwrap_or(&0)) as i64;
        self.local_variables
            .entry(self.resolve_local_variable_name(0, "SIBLING-INDEX", string_container)?)
            .or_insert_with(|| {
                ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                    name: "SIBLING-INDEX".to_string(),
                    value: ftd::PropertyValue::Value {
                        value: ftd::Value::Integer {
                            value: sibling_index + 1,
                        },
                    },
                    conditions: vec![],
                    flags: Default::default(),
                })
            });

        self.local_variables
            .entry(self.resolve_local_variable_name(0, "SIBLING-INDEX-0", string_container)?)
            .or_insert_with(|| {
                ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                    name: "SIBLING-INDEX-0".to_string(),
                    value: ftd::PropertyValue::Value {
                        value: ftd::Value::Integer {
                            value: sibling_index,
                        },
                    },
                    conditions: vec![],
                    flags: Default::default(),
                })
            });

        self.local_variables
            .entry(self.resolve_local_variable_name(0, "CHILDREN-COUNT", string_container)?)
            .or_insert_with(|| {
                ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                    name: "CHILDREN-COUNT".to_string(),
                    value: ftd::PropertyValue::Value {
                        value: ftd::Value::Integer { value: 0 },
                    },
                    conditions: vec![],
                    flags: Default::default(),
                })
            });

        self.local_variables
            .entry(self.resolve_local_variable_name(
                0,
                "CHILDREN-COUNT-MINUS-ONE",
                string_container,
            )?)
            .or_insert_with(|| {
                ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                    name: "CHILDREN-COUNT-MINUS-ONE".to_string(),
                    value: ftd::PropertyValue::Value {
                        value: ftd::Value::Integer { value: -1 },
                    },
                    conditions: vec![],
                    flags: Default::default(),
                })
            });

        *arguments = Default::default();
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn update_component_data(
        &mut self,
        current_container: &str,
        parent_container: &str,
        properties: &mut ftd::Map<ftd::ftd2021::component::Property>,
        reference: &mut Option<(String, ftd::ftd2021::p2::Kind)>,
        condition: &mut Option<ftd::ftd2021::p2::Boolean>,
        events: &mut [ftd::ftd2021::p2::Event],
        insert_only: bool,
        ignore_loop: bool,
        ignore_mouse_in: bool,
    ) -> ftd::ftd2021::p1::Result<()> {
        for (_, property) in properties.iter_mut() {
            if let Some(ref mut default) = property.default {
                rename_property_value(
                    default,
                    self,
                    parent_container,
                    current_container,
                    insert_only,
                    ignore_loop,
                    ignore_mouse_in,
                )?;
            }
            for (boolean, condition) in property.conditions.iter_mut() {
                edit_condition(
                    boolean,
                    self,
                    parent_container,
                    current_container,
                    insert_only,
                    ignore_loop,
                    ignore_mouse_in,
                )?;
                rename_property_value(
                    condition,
                    self,
                    parent_container,
                    current_container,
                    insert_only,
                    ignore_loop,
                    ignore_mouse_in,
                )?;
            }
        }
        if let Some((ref mut c, _)) = reference {
            *c = self.resolve_name(0, format!("{}@{}", c, parent_container).as_str())?;
        }
        if let Some(ref mut condition) = condition {
            edit_condition(
                condition,
                self,
                parent_container,
                current_container,
                insert_only,
                ignore_loop,
                ignore_mouse_in,
            )?;
        }
        for event in events.iter_mut() {
            rename_property_value(
                &mut event.action.target,
                self,
                parent_container,
                current_container,
                insert_only,
                ignore_loop,
                ignore_mouse_in,
            )?;
            for (_, parameters) in event.action.parameters.iter_mut() {
                for parameter in parameters.iter_mut() {
                    rename_property_value(
                        parameter,
                        self,
                        parent_container,
                        current_container,
                        insert_only,
                        ignore_loop,
                        ignore_mouse_in,
                    )?;
                }
            }
        }
        return Ok(());

        fn edit_condition(
            condition: &mut ftd::ftd2021::p2::Boolean,
            doc: &mut ftd::ftd2021::p2::TDoc,
            parent_container: &str,
            current_container: &str,
            insert_only: bool,
            ignore_loop: bool,
            ignore_mouse_in: bool,
        ) -> ftd::ftd2021::p1::Result<()> {
            match condition {
                ftd::ftd2021::p2::Boolean::IsNotNull { value }
                | ftd::ftd2021::p2::Boolean::IsNull { value }
                | ftd::ftd2021::p2::Boolean::IsNotEmpty { value }
                | ftd::ftd2021::p2::Boolean::IsEmpty { value }
                | ftd::ftd2021::p2::Boolean::ListIsEmpty { value } => {
                    rename_property_value(
                        value,
                        doc,
                        parent_container,
                        current_container,
                        insert_only,
                        ignore_loop,
                        ignore_mouse_in,
                    )?;
                }
                ftd::ftd2021::p2::Boolean::Equal { left, right }
                | ftd::ftd2021::p2::Boolean::NotEqual { left, right } => {
                    rename_property_value(
                        left,
                        doc,
                        parent_container,
                        current_container,
                        insert_only,
                        ignore_loop,
                        ignore_mouse_in,
                    )?;
                    rename_property_value(
                        right,
                        doc,
                        parent_container,
                        current_container,
                        insert_only,
                        ignore_loop,
                        ignore_mouse_in,
                    )?;
                }
                ftd::ftd2021::p2::Boolean::Not { of } => edit_condition(
                    of,
                    doc,
                    parent_container,
                    current_container,
                    insert_only,
                    ignore_loop,
                    ignore_mouse_in,
                )?,
                ftd::ftd2021::p2::Boolean::Literal { .. } => {}
            }
            Ok(())
        }

        fn rename_property_value(
            property_value: &mut ftd::PropertyValue,
            doc: &mut ftd::ftd2021::p2::TDoc,
            parent_container: &str,
            current_container: &str,
            insert_only: bool,
            ignore_loop: bool,
            ignore_mouse_in: bool,
        ) -> ftd::ftd2021::p1::Result<()> {
            if let ftd::PropertyValue::Variable { ref mut name, kind } = property_value {
                if (ignore_loop && name.contains("$loop$"))
                    || (insert_only && !name.as_str().eq("MOUSE-IN"))
                    || (ignore_mouse_in && name.contains("MOUSE-IN"))
                // in case of recursive_child_component
                {
                    return Ok(());
                }

                let (part1, part2) = ftd::ftd2021::p2::utils::get_doc_name_and_remaining(name)?;
                if part1.eq("PARENT") {
                    if let Some(part2) = part2 {
                        let parents_parent_container =
                            parent_container.rsplit_once(',').map(|v| v.0).unwrap_or("");
                        let key = doc.resolve_local_variable_name(
                            0,
                            part2.as_str(),
                            parents_parent_container,
                        )?;
                        if parents_parent_container.is_empty() {
                            let value =
                                kind.to_value(0, doc.name).unwrap_or(ftd::Value::Optional {
                                    data: Box::new(None),
                                    kind: kind.clone(),
                                });
                            let local_variable = ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                                name: key.clone(),
                                value: ftd::PropertyValue::Value { value },
                                conditions: vec![],
                                flags: Default::default(),
                            });
                            doc.local_variables.insert(key.clone(), local_variable);
                        }
                        *name = key;
                    } else {
                        return ftd::ftd2021::p2::utils::e2(
                            "PARENT should have variable",
                            doc.name,
                            0,
                        );
                    }
                } else if name.as_str().eq("MOUSE-IN") {
                    let key =
                        doc.resolve_local_variable_name(0, name.as_str(), current_container)?;
                    let local_variable = ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                        name: key.clone(),
                        value: ftd::PropertyValue::Value {
                            value: ftd::Value::Boolean { value: false },
                        },
                        conditions: vec![],
                        flags: Default::default(),
                    });
                    doc.local_variables.insert(key.clone(), local_variable);
                    *name = key;
                } else if let Some((key, _)) = doc.get_local_variable(
                    &doc.resolve_name(0, format!("{}@{}", part1, parent_container).as_str())?,
                ) {
                    let key = if let Some(part2) = part2 {
                        format!("{}.{}", key, part2)
                    } else {
                        key.to_string()
                    };
                    *name = key;
                }
            }
            Ok(())
        }
    }

    pub(crate) fn insert_local_from_childcomponent(
        &mut self,
        local_container: &[usize],
        child: &mut ftd::ChildComponent,
    ) -> ftd::ftd2021::p1::Result<()> {
        let string_container = ftd::ftd2021::p2::utils::get_string_container(local_container);

        self.update_component_data(
            string_container.as_str(),
            string_container.as_str(),
            &mut child.properties,
            &mut child.reference,
            &mut child.condition,
            &mut child.events,
            true,
            true,
            false,
        )?;
        Ok(())
    }

    pub(crate) fn insert_local_from_component(
        &mut self,
        component: &mut ftd::Component,
        child_component_properties: &ftd::Map<ftd::ftd2021::component::Property>,
        local_container: &[usize],
        external_children_count: &Option<usize>,
    ) -> ftd::ftd2021::p1::Result<()> {
        let string_container = ftd::ftd2021::p2::utils::get_string_container(local_container);
        if component.root == "ftd.kernel" {
            return Ok(());
        }
        self.insert_local_variable(
            component.root.as_str(),
            &mut component.arguments,
            child_component_properties,
            string_container.as_str(),
            local_container,
            external_children_count,
        )?;

        ftd::ftd2021::component::Property::add_default_properties(
            child_component_properties,
            &mut component.properties,
        );
        self.update_component_data(
            string_container.as_str(),
            string_container.as_str(),
            &mut component.properties,
            &mut Default::default(),
            &mut component.condition,
            &mut component.events,
            false,
            true,
            false,
        )?;
        for (idx, instruction) in component.instructions.iter_mut().enumerate() {
            let local_container = {
                let mut local_container = local_container.to_vec();
                local_container.push(idx);
                local_container
            };
            let current_container =
                ftd::ftd2021::p2::utils::get_string_container(local_container.as_slice());
            let child = match instruction {
                ftd::Instruction::ChildComponent { child }
                | ftd::Instruction::RecursiveChildComponent { child } => child,
                _ => continue,
            };
            self.update_component_data(
                current_container.as_str(),
                string_container.as_str(),
                &mut child.properties,
                &mut child.reference,
                &mut child.condition,
                &mut child.events,
                false,
                true,
                true,
            )?;
        }
        Ok(())
    }

    pub(crate) fn insert_local(
        &mut self,
        parent: &mut ftd::ChildComponent,
        children: &mut [ftd::ChildComponent],
        local_container: &[usize],
        external_children_count: &Option<usize>,
    ) -> ftd::ftd2021::p1::Result<()> {
        let string_container = ftd::ftd2021::p2::utils::get_string_container(local_container);
        if parent.root == "ftd.kernel" {
            return Ok(());
        }
        self.insert_local_variable(
            parent.root.as_str(),
            &mut parent.arguments,
            &Default::default(),
            string_container.as_str(),
            local_container,
            external_children_count,
        )?;
        self.update_component_data(
            string_container.as_str(),
            string_container.as_str(),
            &mut parent.properties,
            &mut Default::default(),
            &mut parent.condition,
            &mut parent.events,
            false,
            true,
            false,
        )?;
        for (idx, child) in children.iter_mut().enumerate() {
            let local_container = {
                let mut local_container = local_container.to_vec();
                local_container.push(idx);
                local_container
            };
            let current_container =
                ftd::ftd2021::p2::utils::get_string_container(local_container.as_slice());
            self.update_component_data(
                current_container.as_str(),
                string_container.as_str(),
                &mut child.properties,
                &mut child.reference,
                &mut child.condition,
                &mut child.events,
                false,
                true,
                false,
            )?;
        }
        Ok(())
    }

    pub fn get_variable_kind(
        &self,
        section: &ftd::ftd2021::p1::Section,
    ) -> ftd::ftd2021::p1::Result<ftd::ftd2021::p2::Kind> {
        if let Ok(v) = self.get_value(0, section.name.as_str()) {
            return Ok(v.kind());
        }
        if let Ok(list) = ftd::Variable::list_from_p1(section, self) {
            return Ok(list.value.kind());
        }
        if let Ok(var) = ftd::Variable::from_p1(section, self) {
            return Ok(var.value.kind());
        }
        ftd::ftd2021::p2::Kind::for_variable(
            section.line_number,
            &section.name,
            None,
            self,
            None,
            &Default::default(),
        )
    }

    pub fn from_json<T>(
        &self,
        json: &T,
        section: &ftd::ftd2021::p1::Section,
    ) -> ftd::ftd2021::p1::Result<ftd::Value>
    where
        T: serde::Serialize + std::fmt::Debug,
    {
        let json = serde_json::to_value(json).map_err(|e| ftd::ftd2021::p1::Error::ParseError {
            message: format!("Can't serialize to json: {:?}, found: {:?}", e, json),
            doc_id: self.name.to_string(),
            line_number: section.line_number,
        })?;

        if let Ok(v) = self.get_value(0, section.name.as_str()) {
            return self.from_json_(section.line_number, &json, v.kind());
        }
        if let Ok(list) = ftd::Variable::list_from_p1(section, self) {
            return self.from_json_(section.line_number, &json, list.value.kind());
        }
        if let Ok(var) = ftd::Variable::from_p1(section, self) {
            return self.from_json_(section.line_number, &json, var.value.kind());
        }
        if let Ok(kind) = ftd::ftd2021::p2::Kind::for_variable(
            section.line_number,
            &section.name,
            None,
            self,
            None,
            &Default::default(),
        ) {
            return self.from_json_(section.line_number, &json, kind);
        }

        ftd::ftd2021::p2::utils::e2(
            "component should be var or list",
            self.name,
            section.line_number,
        )
    }

    #[allow(clippy::wrong_self_convention)]
    fn from_json_(
        &self,
        line_number: usize,
        json: &serde_json::Value,
        kind: ftd::ftd2021::p2::Kind,
    ) -> ftd::ftd2021::p1::Result<ftd::Value> {
        Ok(match kind {
            ftd::ftd2021::p2::Kind::String { .. } => ftd::Value::String {
                text: serde_json::from_value::<String>(json.to_owned()).map_err(|_| {
                    ftd::ftd2021::p1::Error::ParseError {
                        message: format!("Can't parse to string, found: {}", json),
                        doc_id: self.name.to_string(),
                        line_number,
                    }
                })?,
                source: ftd::TextSource::Header,
            },
            ftd::ftd2021::p2::Kind::Integer { .. } => ftd::Value::Integer {
                value: serde_json::from_value::<i64>(json.to_owned()).map_err(|_| {
                    ftd::ftd2021::p1::Error::ParseError {
                        message: format!("Can't parse to integer, found: {}", json),
                        doc_id: self.name.to_string(),
                        line_number,
                    }
                })?,
            },
            ftd::ftd2021::p2::Kind::Decimal { .. } => ftd::Value::Decimal {
                value: serde_json::from_value::<f64>(json.to_owned()).map_err(|_| {
                    ftd::ftd2021::p1::Error::ParseError {
                        message: format!("Can't parse to decimal, found: {}", json),
                        doc_id: self.name.to_string(),
                        line_number,
                    }
                })?,
            },
            ftd::ftd2021::p2::Kind::Boolean { .. } => ftd::Value::Boolean {
                value: serde_json::from_value::<bool>(json.to_owned()).map_err(|_| {
                    ftd::ftd2021::p1::Error::ParseError {
                        message: format!("Can't parse to boolean,found: {}", json),
                        doc_id: self.name.to_string(),
                        line_number,
                    }
                })?,
            },
            ftd::ftd2021::p2::Kind::Record { name, .. } => {
                let rec_fields = self.get_record(line_number, &name)?.fields;
                let mut fields: ftd::Map<ftd::PropertyValue> = Default::default();
                if let serde_json::Value::Object(o) = json {
                    for (key, kind) in rec_fields {
                        let val = match o.get(&key) {
                            Some(v) => v,
                            None => {
                                return ftd::ftd2021::p2::utils::e2(
                                    format!("key not found: {}", key.as_str()),
                                    self.name,
                                    line_number,
                                )
                            }
                        };
                        fields.insert(
                            key,
                            ftd::PropertyValue::Value {
                                value: self.from_json_(line_number, val, kind)?,
                            },
                        );
                    }
                } else {
                    return ftd::ftd2021::p2::utils::e2(
                        format!("expected object of record type, found: {}", json),
                        self.name,
                        line_number,
                    );
                }
                ftd::Value::Record { name, fields }
            }
            ftd::ftd2021::p2::Kind::List { kind, .. } => {
                let kind = kind.as_ref();
                let mut data: Vec<ftd::PropertyValue> = vec![];
                if let serde_json::Value::Array(list) = json {
                    for item in list {
                        data.push(ftd::PropertyValue::Value {
                            value: self.from_json_(line_number, item, kind.to_owned())?,
                        });
                    }
                } else {
                    return ftd::ftd2021::p2::utils::e2(
                        format!("expected object of list type, found: {}", json),
                        self.name,
                        line_number,
                    );
                }
                ftd::Value::List {
                    data,
                    kind: kind.to_owned(),
                }
            }
            ftd::ftd2021::p2::Kind::Optional { kind, .. } => {
                let kind = kind.as_ref().to_owned();
                match json {
                    serde_json::Value::Null => ftd::Value::Optional {
                        kind,
                        data: Box::new(None),
                    },
                    _ => self.from_json_(line_number, json, kind)?,
                }
            }
            t => unimplemented!(
                "{:?} not yet implemented, line number: {}, doc: {}",
                t,
                line_number,
                self.name.to_string()
            ),
        })
    }

    pub fn from_json_rows(
        &self,
        section: &ftd::ftd2021::p1::Section,
        rows: &[Vec<serde_json::Value>],
    ) -> ftd::ftd2021::p1::Result<ftd::Value> {
        if let Ok(v) = self.get_value(0, section.name.as_str()) {
            return from_json_rows_(section.line_number, self, rows, v.kind());
        }
        if let Ok(list) = ftd::Variable::list_from_p1(section, self) {
            return from_json_rows_(section.line_number, self, rows, list.value.kind());
        }

        return ftd::ftd2021::p2::utils::e2(
            "component should be list",
            self.name,
            section.line_number,
        );

        fn from_json_rows_(
            line_number: usize,
            doc: &ftd::ftd2021::p2::TDoc,
            rows: &[Vec<serde_json::Value>],
            kind: ftd::ftd2021::p2::Kind,
        ) -> ftd::ftd2021::p1::Result<ftd::Value> {
            Ok(match kind {
                ftd::ftd2021::p2::Kind::List { kind, .. } => {
                    let kind = kind.as_ref();
                    let mut data: Vec<ftd::PropertyValue> = vec![];
                    for row in rows {
                        data.push(ftd::PropertyValue::Value {
                            value: doc.from_json_row_(line_number, row, kind.to_owned())?,
                        });
                    }

                    ftd::Value::List {
                        data,
                        kind: kind.to_owned(),
                    }
                }
                t => unimplemented!(
                    "{:?} not yet implemented, line number: {}, doc: {}",
                    t,
                    line_number,
                    doc.name.to_string()
                ),
            })
        }
    }

    pub fn from_json_row(
        &self,
        section: &ftd::ftd2021::p1::Section,
        row: &[serde_json::Value],
    ) -> ftd::ftd2021::p1::Result<ftd::Value> {
        if let Ok(v) = self.get_value(0, section.name.as_str()) {
            return self.from_json_row_(section.line_number, row, v.kind());
        }
        if let Ok(var) = ftd::Variable::from_p1(section, self) {
            return self.from_json_row_(section.line_number, row, var.value.kind());
        }

        ftd::ftd2021::p2::utils::e2(
            "component should be var of record type",
            self.name,
            section.line_number,
        )
    }

    #[allow(clippy::wrong_self_convention)]
    fn from_json_row_(
        &self,
        line_number: usize,
        row: &[serde_json::Value],
        kind: ftd::ftd2021::p2::Kind,
    ) -> ftd::ftd2021::p1::Result<ftd::Value> {
        Ok(match kind {
            ftd::ftd2021::p2::Kind::Record { name, .. } => {
                let rec = self.get_record(line_number, &name)?;
                let rec_fields = rec.fields;
                let mut fields: ftd::Map<ftd::PropertyValue> = Default::default();
                for (idx, key) in rec.order.iter().enumerate() {
                    if let Some(kind) = rec_fields.get(key) {
                        let val = match row.get(idx) {
                            Some(v) => v,
                            None => {
                                return ftd::ftd2021::p2::utils::e2(
                                    format!("key not found: {}", key.as_str()),
                                    self.name,
                                    line_number,
                                )
                            }
                        };
                        fields.insert(
                            key.to_string(),
                            ftd::PropertyValue::Value {
                                value: self.from_json_(line_number, val, kind.to_owned())?,
                            },
                        );
                    } else {
                        return ftd::ftd2021::p2::utils::e2(
                            format!("field `{}` not found", key),
                            self.name,
                            line_number,
                        );
                    }
                }
                ftd::Value::Record { name, fields }
            }
            ftd::ftd2021::p2::Kind::String { .. } if row.first().is_some() => ftd::Value::String {
                text: serde_json::from_value::<String>(row.first().unwrap().to_owned()).map_err(
                    |_| ftd::ftd2021::p1::Error::ParseError {
                        message: format!("Can't parse to string, found: {:?}", row),
                        doc_id: self.name.to_string(),
                        line_number,
                    },
                )?,
                source: ftd::TextSource::Header,
            },
            ftd::ftd2021::p2::Kind::Integer { .. } if row.first().is_some() => {
                ftd::Value::Integer {
                    value: serde_json::from_value::<i64>(row.first().unwrap().to_owned()).map_err(
                        |_| ftd::ftd2021::p1::Error::ParseError {
                            message: format!("Can't parse to integer, found: {:?}", row),
                            doc_id: self.name.to_string(),
                            line_number,
                        },
                    )?,
                }
            }
            ftd::ftd2021::p2::Kind::Decimal { .. } if row.first().is_some() => {
                ftd::Value::Decimal {
                    value: serde_json::from_value::<f64>(row.first().unwrap().to_owned()).map_err(
                        |_| ftd::ftd2021::p1::Error::ParseError {
                            message: format!("Can't parse to decimal, found: {:?}", row),
                            doc_id: self.name.to_string(),
                            line_number,
                        },
                    )?,
                }
            }
            ftd::ftd2021::p2::Kind::Boolean { .. } if row.first().is_some() => {
                ftd::Value::Boolean {
                    value: serde_json::from_value::<bool>(row.first().unwrap().to_owned())
                        .map_err(|_| ftd::ftd2021::p1::Error::ParseError {
                            message: format!("Can't parse to boolean,found: {:?}", row),
                            doc_id: self.name.to_string(),
                            line_number,
                        })?,
                }
            }
            t => unimplemented!(
                "{:?} not yet implemented, line number: {}, doc: {}",
                t,
                line_number,
                self.name.to_string()
            ),
        })
    }

    pub fn format_name(&self, name: &str) -> String {
        format!("{}#{}", self.name, name)
    }

    pub fn resolve_name_without_full_path(
        &self,
        line_number: usize,
        name: &str,
    ) -> ftd::ftd2021::p1::Result<String> {
        if name.contains('#') {
            return Ok(name.to_string());
        }

        Ok(
            match ftd::ftd2021::p2::utils::split_module(name, self.name, line_number)? {
                (Some(m), v, None) => match self.aliases.get(m) {
                    Some(m) => format!("{}#{}", m, v),
                    None => {
                        return self.err(
                            "alias not found",
                            m,
                            "resolve_name_without_full_path",
                            line_number,
                        )
                    }
                },
                (_, _, Some(_)) => unimplemented!(),
                (None, v, None) => v.to_string(),
            },
        )
    }

    pub fn resolve_name_with_instruction(
        &self,
        line_number: usize,
        name: &str,
        instructions: &[ftd::Instruction],
    ) -> ftd::ftd2021::p1::Result<String> {
        if name.contains('#') {
            return Ok(name.to_string());
        }
        let mut available_components: ftd::Map<String> = Default::default();
        for instruction in instructions {
            if let Some(text) = instruction.resolve_id() {
                available_components.insert(text.to_string(), text.to_string());
            }
        }

        Ok(
            match ftd::ftd2021::p2::utils::split_module(name, self.name, line_number)? {
                (Some(m), v, None) => match self.aliases.get(m) {
                    Some(m) => format!("{}#{}", m, v),
                    None => match available_components.get(m) {
                        Some(a) => format!("{}#{}", a, v),
                        None => {
                            return self.err(
                                "alias not found",
                                m,
                                "resolve_name_with_instruction",
                                line_number,
                            );
                        }
                    },
                },
                (Some(m), v, Some(c)) => match self.aliases.get(m) {
                    Some(m) => format!("{}#{}.{}", m, v, c),
                    None => match available_components.get(m) {
                        Some(a) => format!("{}#{}.{}", a, v, c),
                        None => {
                            return self.err(
                                "alias not found",
                                m,
                                "resolve_name_with_instruction",
                                line_number,
                            );
                        }
                    },
                },
                (None, v, None) => v.to_string(),
                _ => unimplemented!(),
            },
        )
    }

    pub(crate) fn resolve_reference_name(
        &self,
        line_number: usize,
        name: &str,
        arguments: &ftd::Map<ftd::ftd2021::p2::Kind>,
    ) -> ftd::ftd2021::p1::Result<String> {
        return Ok(if let Some(l) = name.strip_prefix('$') {
            /*let (part1, part2) = ftd::ftd2021::p2::utils::get_doc_name_and_remaining(l)?;
            if get_special_variable().iter().any(|v| part1.starts_with(v)) {
                let part2 = part2.map(|v| format!(".{}", v)).unwrap_or("".to_string());
                return Ok(format!("${}{}", part1, part2));
            } else if arguments.contains_key(part1.as_str()) {
                return Ok(format!("${}", l));
            }
            let s = self.resolve_name(line_number, l)?;
            format!("${}", s)*/
            let d = ftd::ftd2021::p2::utils::get_doc_name_and_remaining(l)?.0;
            if arguments.contains_key(d.as_str()) || get_special_variable().contains(&d.as_str()) {
                return Ok(format!("${}", l));
            }
            let s = self.resolve_name(line_number, l)?;
            format!("${}", s)
        } else {
            name.to_string()
        });

        fn get_special_variable() -> Vec<&'static str> {
            vec![
                "MOUSE-IN",
                "SIBLING-INDEX",
                "SIBLING-INDEX-0",
                "CHILDREN-COUNT",
                "CHILDREN-COUNT-MINUS-ONE",
                "PARENT",
            ]
        }
    }

    pub(crate) fn resolve_local_variable_name(
        &self,
        line_number: usize,
        name: &str,
        container: &str,
    ) -> ftd::ftd2021::p1::Result<String> {
        ftd::ftd2021::p2::utils::resolve_local_variable_name(
            line_number,
            name,
            container,
            self.name,
            self.aliases,
        )
    }

    pub fn resolve_name(&self, line_number: usize, name: &str) -> ftd::ftd2021::p1::Result<String> {
        ftd::ftd2021::p2::utils::resolve_name(line_number, name, self.name, self.aliases)
    }

    pub fn get_record(
        &self,
        line_number: usize,
        name: &str,
    ) -> ftd::ftd2021::p1::Result<ftd::ftd2021::p2::Record> {
        match self.get_thing(line_number, name)? {
            ftd::ftd2021::p2::Thing::Record(v) => Ok(v),
            v => self.err("not a record", v, "get_record", line_number),
        }
    }

    pub fn get_or_type(
        &self,
        line_number: usize,
        name: &str,
    ) -> ftd::ftd2021::p1::Result<ftd::ftd2021::OrType> {
        match self.get_thing(line_number, name)? {
            ftd::ftd2021::p2::Thing::OrType(v) => Ok(v),
            v => self.err("not an or-type", v, "get_or_type", line_number),
        }
    }

    pub fn get_or_type_with_variant(
        &self,
        line_number: usize,
        name: &str,
    ) -> ftd::ftd2021::p1::Result<ftd::ftd2021::OrType> {
        match self.get_thing(line_number, name)? {
            ftd::ftd2021::p2::Thing::OrTypeWithVariant { e, .. } => Ok(e),
            v => self.err("not an or-type", v, "get_or_type", line_number),
        }
    }

    pub fn is_variable_record_type(
        &self,
        line_number: usize,
        name: &str,
    ) -> ftd::ftd2021::p1::Result<bool> {
        Ok(match self.get_thing(line_number, name)? {
            ftd::ftd2021::p2::Thing::Variable(v) => v.value.kind().is_record(),
            _ => false,
        })
    }

    pub fn get_value_and_conditions(
        &self,
        line_number: usize,
        name: &str,
    ) -> ftd::ftd2021::p1::Result<(ftd::Value, Vec<(ftd::ftd2021::p2::Boolean, ftd::Value)>)> {
        match self.get_thing(line_number, name)? {
            ftd::ftd2021::p2::Thing::Variable(v) => Ok((
                v.value.resolve(line_number, self)?,
                v.conditions
                    .into_iter()
                    .flat_map(|(b, v)| {
                        if let Ok(v) = v.resolve(line_number, self) {
                            Some((b, v))
                        } else {
                            None
                        }
                    })
                    .collect(),
            )),
            v => self.err("not a variable", v, "get_value", line_number),
        }
    }

    pub fn get_value(
        &self,
        line_number: usize,
        name: &str,
    ) -> ftd::ftd2021::p1::Result<ftd::Value> {
        // TODO: name can be a.b.c, and a and a.b are records with right fields
        match self.get_thing(line_number, name)? {
            ftd::ftd2021::p2::Thing::Variable(v) => v.value.partial_resolve(line_number, self),
            v => self.err("not a variable", v, "get_value", line_number),
        }
    }

    fn err<T, T2: std::fmt::Debug>(
        &self,
        msg: &str,
        ctx: T2,
        f: &str,
        line_number: usize,
    ) -> ftd::ftd2021::p1::Result<T> {
        ftd::ftd2021::p2::utils::e2(
            format!("{}: {} ({:?}), f: {}", self.name, msg, ctx, f),
            self.name,
            line_number,
        )
    }

    pub fn get_component(
        &self,
        line_number: usize,
        name: &str,
    ) -> ftd::ftd2021::p1::Result<ftd::Component> {
        match self.get_thing(line_number, name)? {
            ftd::ftd2021::p2::Thing::Component(v) => Ok(v),
            v => self.err("not a component", v, "get_component", line_number),
        }
    }

    pub fn get_root<'a>(
        &'a self,
        name: &'a str,
        line_number: usize,
    ) -> ftd::ftd2021::p1::Result<Option<&str>> {
        if name.contains('#') {
            match name.split_once('#') {
                Some((p1, _)) => {
                    for (k, v) in self.aliases.iter() {
                        if p1 == v.as_str() {
                            return Ok(Some(k.as_str()));
                        }
                    }
                }
                _ => {
                    return Ok(None);
                }
            }
            return Ok(None);
        }
        match ftd::ftd2021::p2::utils::split_module(name, self.name, line_number)? {
            (Some(m), _, _) => {
                if self.aliases.contains_key(m) {
                    Ok(Some(m))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }
    // name = foo | alias.foo | a/b#foo

    pub fn get_initial_thing(
        &self,
        line_number: usize,
        name: &str,
    ) -> ftd::ftd2021::p1::Result<(ftd::ftd2021::p2::Thing, Option<String>)> {
        if name.contains('#') {
            let (name, remaining_value) = {
                let mut full_name = (name.to_string(), None);
                if let Some((s, n)) = name.split_once('#') {
                    if let Some((v, remaining_value)) = n.split_once('.') {
                        full_name.0 = format!("{}#{}", s, v);
                        full_name.1 = Some(remaining_value.to_string());
                    }
                }
                full_name
            };
            return match self.bag.get(name.as_str()) {
                Some(a) => Ok((a.to_owned(), remaining_value)),
                None => match self.local_variables.get(name.as_str()) {
                    Some(a) => Ok((a.to_owned(), remaining_value)),
                    None => self.err("not found", name, "get_thing", line_number),
                },
            };
        }
        return Ok(match get_initial_thing_(self, None, self.name, name) {
            Some(a) => a,
            None => {
                if let Some((m, v)) = name.split_once('.') {
                    match get_initial_thing_(self, Some(m), m, v) {
                        None => return self.err("not found", name, "get_thing", line_number),
                        Some(a) => a,
                    }
                } else {
                    return self.err("not found", name, "get_thing", line_number);
                }
            }
        });

        fn get_initial_thing_(
            doc: &ftd::ftd2021::p2::TDoc,
            root_name: Option<&str>,
            doc_name: &str,
            name: &str,
        ) -> Option<(ftd::ftd2021::p2::Thing, Option<String>)> {
            let (name, remaining_value) = if let Some((v, remaining_value)) = name.split_once('.') {
                (v, Some(remaining_value.to_string()))
            } else {
                (name, None)
            };

            match doc
                .bag
                .get(format!("{}#{}", doc_name, name).as_str())
                .or_else(|| {
                    doc.local_variables
                        .get(format!("{}#{}", doc_name, name).as_str())
                })
                .map(ToOwned::to_owned)
            {
                Some(a) => Some((a, remaining_value)),
                None => match root_name {
                    Some(doc_name) => match doc.aliases.get(doc_name) {
                        Some(g) => doc
                            .bag
                            .get(format!("{}#{}", g, name).as_str())
                            .map(|v| (v.clone(), remaining_value)),
                        None => None,
                    },
                    None => None,
                },
            }
        }
    }

    pub fn set_value(
        &self,
        line_number: usize,
        name: &str,
        value: ftd::Variable,
    ) -> ftd::ftd2021::p1::Result<ftd::Variable> {
        let (initial_thing, remaining) = self.get_initial_thing(line_number, name)?;

        let remaining = if let Some(remaining) = remaining {
            remaining
        } else {
            return Ok(value);
        };

        let mut variable = if let ftd::ftd2021::p2::Thing::Variable(variable) = initial_thing {
            variable
        } else {
            return ftd::ftd2021::p2::utils::e2(
                format!("Expected variable, found: `{:#?}`", initial_thing),
                self.name,
                line_number,
            );
        };

        variable.value = set_value_(
            self,
            line_number,
            remaining.as_str(),
            &variable.value,
            value.value,
        )?;

        return Ok(variable);

        fn set_value_(
            doc: &ftd::ftd2021::p2::TDoc,
            line_number: usize,
            name: &str,
            var_value: &ftd::PropertyValue,
            set_value: ftd::PropertyValue,
        ) -> ftd::ftd2021::p1::Result<ftd::PropertyValue> {
            let (v, remaining) = name
                .split_once('.')
                .map(|(v, n)| (v, Some(n)))
                .unwrap_or((name, None));
            let value = var_value.resolve(line_number, doc)?;
            let mut inner_value = if let Some(val) = value.to_owned().inner() {
                val
            } else {
                return doc.err(
                    "Need value for optional variable",
                    value,
                    "set_variable",
                    line_number,
                );
            };
            let fields = match &mut inner_value {
                ftd::Value::Record { fields, .. } => fields,
                ftd::Value::OrType { fields, .. } => fields,
                ftd::Value::Object { values } => values,
                _ => return doc.err("not an record or or-type", value, "set_thing", line_number),
            };

            if let Some(data) = fields.get_mut(v) {
                if let Some(remaining) = remaining {
                    *data = set_value_(doc, line_number, remaining, data, set_value)?;
                } else {
                    *data = set_value;
                }
            }

            Ok(ftd::PropertyValue::Value {
                value: if value.is_optional() {
                    ftd::Value::Optional {
                        data: Box::new(Some(inner_value.to_owned())),
                        kind: inner_value.kind(),
                    }
                } else {
                    inner_value
                },
            })
        }
    }

    pub fn get_thing(
        &self,
        line_number: usize,
        name: &str,
    ) -> ftd::ftd2021::p1::Result<ftd::ftd2021::p2::Thing> {
        let name = if let Some(name) = name.strip_prefix('$') {
            name
        } else {
            name
        };

        let (initial_thing, remaining) = self.get_initial_thing(line_number, name)?;

        if let Some(remaining) = remaining {
            return get_thing_(self, line_number, remaining.as_str(), &initial_thing);
        }
        return Ok(initial_thing);

        fn get_thing_(
            doc: &ftd::ftd2021::p2::TDoc,
            line_number: usize,
            name: &str,
            thing: &ftd::ftd2021::p2::Thing,
        ) -> ftd::ftd2021::p1::Result<ftd::ftd2021::p2::Thing> {
            let (v, remaining) = name
                .split_once('.')
                .map(|(v, n)| (v, Some(n)))
                .unwrap_or((name, None));
            let thing = match thing.clone() {
                ftd::ftd2021::p2::Thing::OrType(e) => ftd::ftd2021::p2::Thing::OrTypeWithVariant {
                    e,
                    variant: v.to_string(),
                },
                ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                    name,
                    value,
                    conditions,
                    ..
                }) => {
                    let fields = match value.resolve(line_number, doc)?.inner_with_none() {
                        ftd::Value::Record { fields, .. } => fields,
                        ftd::Value::OrType { fields, .. } => fields,
                        ftd::Value::Object { values } => values,
                        ftd::Value::None { kind } => {
                            let kind_name = match kind {
                                ftd::ftd2021::p2::Kind::Record { ref name, .. } => name,
                                ftd::ftd2021::p2::Kind::OrType { ref name, .. } => name,
                                ftd::ftd2021::p2::Kind::OrTypeWithVariant { ref name, .. } => name,
                                _ => {
                                    return doc.err(
                                        "not an record or or-type",
                                        thing,
                                        "get_thing",
                                        line_number,
                                    );
                                }
                            };
                            let kind_thing = doc.get_thing(line_number, kind_name)?;
                            let kind = if let Some(fields_kind) = match kind_thing {
                                ftd::ftd2021::p2::Thing::Record(ftd::ftd2021::p2::Record {
                                    fields,
                                    ..
                                }) => fields.get(v).cloned(),
                                _ => None,
                            } {
                                fields_kind
                            } else {
                                return doc.err(
                                    "not an record or or-type",
                                    thing,
                                    "get_thing",
                                    line_number,
                                );
                            };
                            let thing = ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                                name,
                                value: ftd::PropertyValue::Value {
                                    value: ftd::Value::None { kind },
                                },
                                conditions,
                                flags: ftd::VariableFlags::default(),
                            });
                            if let Some(remaining) = remaining {
                                return get_thing_(doc, line_number, remaining, &thing);
                            }
                            return Ok(thing);
                        }
                        _ => {
                            return doc.err(
                                "not an record or or-type",
                                thing,
                                "get_thing",
                                line_number,
                            )
                        }
                    };
                    if let Some(ftd::PropertyValue::Value { value: val }) = fields.get(v) {
                        ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                            name,
                            value: ftd::PropertyValue::Value { value: val.clone() },
                            conditions,
                            flags: ftd::VariableFlags::default(),
                        })
                    } else if let Some(ftd::PropertyValue::Reference { name, .. }) = fields.get(v) {
                        let (initial_thing, name) = doc.get_initial_thing(line_number, name)?;
                        if let Some(remaining) = name {
                            get_thing_(doc, line_number, remaining.as_str(), &initial_thing)?
                        } else {
                            initial_thing
                        }
                    } else {
                        thing.clone()
                    }
                }
                _ => {
                    return doc.err("not an or-type", thing, "get_thing", line_number);
                }
            };
            if let Some(remaining) = remaining {
                return get_thing_(doc, line_number, remaining, &thing);
            }
            Ok(thing)
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn string_list_from_rows() {
        let data: Vec<Vec<serde_json::Value>> = vec![
            vec![serde_json::json!("Prayagraj")],
            vec![serde_json::json!("Varanasi")],
        ];
        let doc = ftd::ftd2021::p2::TDoc {
            name: "foo/bar",
            aliases: &Default::default(),
            bag: &Default::default(),
            local_variables: &mut Default::default(),
            referenced_local_variables: &mut Default::default(),
        };
        let section = ftd::ftd2021::p1::parse(
            indoc::indoc!(
                "
            -- string list city:
            "
            ),
            "foo/bar",
        )
        .unwrap();
        let value_from_json = doc.from_json_rows(&section[0], &data).unwrap();
        let value = ftd::Value::List {
            data: vec![
                ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "Prayagraj".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
                ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "Varanasi".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
            ],
            kind: ftd::ftd2021::p2::Kind::String {
                caption: false,
                body: false,
                default: None,
                is_reference: false,
            },
        };
        pretty_assertions::assert_eq!(value_from_json, value);
    }
    #[test]
    fn record_list_from_rows() {
        let source = indoc::indoc!(
            "
            -- record person:
            string name:
            integer age:
            string address:
            string bio:
            "
        )
        .to_string();

        let (g_bag, _g_col) =
            ftd::ftd2021::test::interpret("foo/bar", &source, &ftd::ftd2021::p2::TestLibrary {})
                .expect("found error");
        let data: Vec<Vec<serde_json::Value>> = vec![
            vec![
                serde_json::json!("Amitu"),
                serde_json::json!(20_i64),
                serde_json::json!("Bangalore"),
                serde_json::json!("CEO of fifthTry"),
            ],
            vec![
                serde_json::json!("Arpita"),
                serde_json::json!(20_i64),
                serde_json::json!("Varanasi"),
                serde_json::json!("Software Developer of fifthTry"),
            ],
        ];
        let doc = ftd::ftd2021::p2::TDoc {
            name: "foo/bar",
            aliases: &Default::default(),
            bag: &g_bag,
            local_variables: &mut Default::default(),
            referenced_local_variables: &mut Default::default(),
        };
        let section = ftd::ftd2021::p1::parse(
            indoc::indoc!(
                "
            -- person list people:
            "
            ),
            "foo/bar",
        )
        .unwrap();
        let value_from_json = doc.from_json_rows(&section[0], &data).unwrap();
        let value = ftd::Value::List {
            data: vec![
                ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: "foo/bar#person".to_string(),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                "name".to_string(),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "Amitu".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                "age".to_string(),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::Integer { value: 20 },
                                },
                            ),
                            (
                                "bio".to_string(),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "CEO of fifthTry".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                "address".to_string(),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "Bangalore".to_string(),
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
                        name: "foo/bar#person".to_string(),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                "name".to_string(),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "Arpita".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                "age".to_string(),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::Integer { value: 20 },
                                },
                            ),
                            (
                                "bio".to_string(),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "Software Developer of fifthTry".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                "address".to_string(),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "Varanasi".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
            ],
            kind: ftd::ftd2021::p2::Kind::Record {
                name: "foo/bar#person".to_string(),
                default: None,
                is_reference: false,
            },
        };
        pretty_assertions::assert_eq!(value_from_json, value);
    }
}
