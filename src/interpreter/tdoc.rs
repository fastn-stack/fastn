#[derive(Debug, PartialEq)]
pub struct TDoc<'a> {
    pub name: &'a str,
    pub aliases: &'a ftd::Map<String>,
    pub bag: &'a ftd::Map<ftd::interpreter::Thing>,
    pub local_variables: &'a mut ftd::Map<ftd::interpreter::Thing>,
    /// string $msg: $message
    /// then msg is a referenced_variable that is won't become new local_variable
    pub referenced_local_variables: &'a mut ftd::Map<String>,
}

impl<'a> TDoc<'a> {
    fn get_local_variable<'b>(
        &'b self,
        key: &'b str,
    ) -> Option<(&'b str, &'b ftd::interpreter::Thing)> {
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
        arguments: &mut ftd::Map<ftd::interpreter::Kind>,
        properties: &ftd::Map<ftd::interpreter::Property>,
        string_container: &str,
        local_container: &[usize],
        external_children_count: &Option<usize>,
    ) -> ftd::p11::Result<()> {
        for (k, arg) in arguments.iter() {
            let mut default = if let Some(d) = properties.get(k) {
                let default = if let Some(ref d) = d.default {
                    d.to_owned()
                } else {
                    //todo
                    return ftd::interpreter::utils::e2(
                        format!(
                            "expected default value for local variable {}: {:?} in {}",
                            k, arg, root
                        ),
                        self.name,
                        0,
                    );
                };
                if matches!(default.kind().inner(), ftd::interpreter::Kind::UI { .. }) {
                    let root = match &default {
                        ftd::interpreter::PropertyValue::Value {
                            value: ftd::interpreter::Value::UI { name, .. },
                        }
                        | ftd::interpreter::PropertyValue::Reference { name, .. }
                        | ftd::interpreter::PropertyValue::Variable { name, .. } => name.to_string(),
                        ftd::interpreter::PropertyValue::Value { value } => {
                            if let Some(ftd::interpreter::Value::UI { name, .. }) = value.to_owned().inner() {
                                name
                            } else {
                                return ftd::interpreter::utils::e2(
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

                    let c = ftd::interpreter::Thing::Component(ftd::interpreter::Component {
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
                ftd::interpreter::PropertyValue::resolve_value(
                    0,
                    default.as_str(),
                    Some(arg.to_owned()),
                    self,
                    arguments,
                    None,
                )?
            } else if let Ok(value) = arg.to_value(0, self.name) {
                ftd::interpreter::PropertyValue::Value { value }
            } else {
                return ftd::interpreter::utils::e2(
                    format!(
                        "expected default value for local variable 2 {}: {:?} in {}",
                        k, arg, root
                    ),
                    self.name,
                    0,
                );
            };
            if let ftd::interpreter::PropertyValue::Variable { ref mut name, .. } = default {
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
                let local_variable =
                    ftd::interpreter::Thing::Variable(ftd::interpreter::Variable {
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
                ftd::interpreter::Thing::Variable(ftd::interpreter::Variable {
                    name: "SIBLING-INDEX".to_string(),
                    value: ftd::interpreter::PropertyValue::Value {
                        value: ftd::interpreter::Value::Integer {
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
                ftd::interpreter::Thing::Variable(ftd::interpreter::Variable {
                    name: "SIBLING-INDEX-0".to_string(),
                    value: ftd::interpreter::PropertyValue::Value {
                        value: ftd::interpreter::Value::Integer {
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
                ftd::interpreter::Thing::Variable(ftd::interpreter::Variable {
                    name: "CHILDREN-COUNT".to_string(),
                    value: ftd::interpreter::PropertyValue::Value {
                        value: ftd::interpreter::Value::Integer { value: 0 },
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
                ftd::interpreter::Thing::Variable(ftd::interpreter::Variable {
                    name: "CHILDREN-COUNT-MINUS-ONE".to_string(),
                    value: ftd::interpreter::PropertyValue::Value {
                        value: ftd::interpreter::Value::Integer { value: -1 },
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
        properties: &mut ftd::Map<ftd::interpreter::Property>,
        reference: &mut Option<(String, ftd::interpreter::Kind)>,
        condition: &mut Option<ftd::interpreter::Boolean>,
        events: &mut [ftd::interpreter::Event],
        insert_only: bool,
        ignore_loop: bool,
        ignore_mouse_in: bool,
    ) -> ftd::p11::Result<()> {
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
            condition: &mut ftd::interpreter::Boolean,
            doc: &mut ftd::interpreter::TDoc,
            parent_container: &str,
            current_container: &str,
            insert_only: bool,
            ignore_loop: bool,
            ignore_mouse_in: bool,
        ) -> ftd::p11::Result<()> {
            match condition {
                ftd::interpreter::Boolean::IsNotNull { value }
                | ftd::interpreter::Boolean::IsNull { value }
                | ftd::interpreter::Boolean::IsNotEmpty { value }
                | ftd::interpreter::Boolean::IsEmpty { value }
                | ftd::interpreter::Boolean::ListIsEmpty { value } => {
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
                ftd::interpreter::Boolean::Equal { left, right }
                | ftd::interpreter::Boolean::NotEqual { left, right } => {
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
                ftd::interpreter::Boolean::Not { of } => edit_condition(
                    of,
                    doc,
                    parent_container,
                    current_container,
                    insert_only,
                    ignore_loop,
                    ignore_mouse_in,
                )?,
                ftd::interpreter::Boolean::Literal { .. } => {}
            }
            Ok(())
        }

        fn rename_property_value(
            property_value: &mut ftd::interpreter::PropertyValue,
            doc: &mut ftd::interpreter::TDoc,
            parent_container: &str,
            current_container: &str,
            insert_only: bool,
            ignore_loop: bool,
            ignore_mouse_in: bool,
        ) -> ftd::p11::Result<()> {
            if let ftd::interpreter::PropertyValue::Variable { ref mut name, kind } = property_value
            {
                if (ignore_loop && name.contains("$loop$"))
                    || (insert_only && !name.as_str().eq("MOUSE-IN"))
                    || (ignore_mouse_in && name.contains("MOUSE-IN"))
                // in case of recursive_child_component
                {
                    return Ok(());
                }

                let (part1, part2) = ftd::interpreter::utils::get_doc_name_and_remaining(name)?;
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
                            let value = kind.to_value(0, doc.name).unwrap_or(
                                ftd::interpreter::Value::Optional {
                                    data: Box::new(None),
                                    kind: kind.clone(),
                                },
                            );
                            let local_variable =
                                ftd::interpreter::Thing::Variable(ftd::interpreter::Variable {
                                    name: key.clone(),
                                    value: ftd::interpreter::PropertyValue::Value { value },
                                    conditions: vec![],
                                    flags: Default::default(),
                                });
                            doc.local_variables.insert(key.clone(), local_variable);
                        }
                        *name = key;
                    } else {
                        return ftd::interpreter::utils::e2(
                            "PARENT should have variable",
                            doc.name,
                            0,
                        );
                    }
                } else if name.as_str().eq("MOUSE-IN") {
                    let key =
                        doc.resolve_local_variable_name(0, name.as_str(), current_container)?;
                    let local_variable =
                        ftd::interpreter::Thing::Variable(ftd::interpreter::Variable {
                            name: key.clone(),
                            value: ftd::interpreter::PropertyValue::Value {
                                value: ftd::interpreter::Value::Boolean { value: false },
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

    pub fn get_variable_kind(
        &self,
        section: &ftd::p11::Section,
    ) -> ftd::p11::Result<ftd::interpreter::Kind> {
        if let Ok(v) = self.get_value(0, section.name.as_str()) {
            return Ok(v.kind());
        }
        if let Ok(list) = ftd::interpreter::Variable::list_from_p1(section, self) {
            return Ok(list.value.kind());
        }
        if let Ok(var) = ftd::interpreter::Variable::from_p1(section, self) {
            return Ok(var.value.kind());
        }
        ftd::interpreter::Kind::for_variable(
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
        section: &ftd::p11::Section,
    ) -> ftd::p11::Result<ftd::interpreter::Value>
    where
        T: serde::Serialize + std::fmt::Debug,
    {
        let json = serde_json::to_value(json).map_err(|e| ftd::p11::Error::ParseError {
            message: format!("Can't serialize to json: {:?}, found: {:?}", e, json),
            doc_id: self.name.to_string(),
            line_number: section.line_number,
        })?;

        if let Ok(v) = self.get_value(0, section.name.as_str()) {
            return self.from_json_(section.line_number, &json, v.kind());
        }
        if let Ok(list) = ftd::interpreter::Variable::list_from_p1(section, self) {
            return self.from_json_(section.line_number, &json, list.value.kind());
        }
        if let Ok(var) = ftd::interpreter::Variable::from_p1(section, self) {
            return self.from_json_(section.line_number, &json, var.value.kind());
        }
        if let Ok(kind) = ftd::interpreter::Kind::for_variable(
            section.line_number,
            &section.name,
            None,
            self,
            None,
            &Default::default(),
        ) {
            return self.from_json_(section.line_number, &json, kind);
        }

        ftd::interpreter::utils::e2(
            "component should be var or list",
            self.name,
            section.line_number,
        )
    }

    #[cfg_attr(feature = "cargo-clippy", allow(clippy::wrong_self_convention))]
    fn from_json_(
        &self,
        line_number: usize,
        json: &serde_json::Value,
        kind: ftd::interpreter::Kind,
    ) -> ftd::p11::Result<ftd::interpreter::Value> {
        Ok(match kind {
            ftd::interpreter::Kind::String { .. } => ftd::interpreter::Value::String {
                text: serde_json::from_value::<String>(json.to_owned()).map_err(|_| {
                    ftd::p11::Error::ParseError {
                        message: format!("Can't parse to string, found: {}", json),
                        doc_id: self.name.to_string(),
                        line_number,
                    }
                })?,
                source: ftd::interpreter::TextSource::Header,
            },
            ftd::interpreter::Kind::Integer { .. } => ftd::interpreter::Value::Integer {
                value: serde_json::from_value::<i64>(json.to_owned()).map_err(|_| {
                    ftd::p11::Error::ParseError {
                        message: format!("Can't parse to integer, found: {}", json),
                        doc_id: self.name.to_string(),
                        line_number,
                    }
                })?,
            },
            ftd::interpreter::Kind::Decimal { .. } => ftd::interpreter::Value::Decimal {
                value: serde_json::from_value::<f64>(json.to_owned()).map_err(|_| {
                    ftd::p11::Error::ParseError {
                        message: format!("Can't parse to decimal, found: {}", json),
                        doc_id: self.name.to_string(),
                        line_number,
                    }
                })?,
            },
            ftd::interpreter::Kind::Boolean { .. } => ftd::interpreter::Value::Boolean {
                value: serde_json::from_value::<bool>(json.to_owned()).map_err(|_| {
                    ftd::p11::Error::ParseError {
                        message: format!("Can't parse to boolean,found: {}", json),
                        doc_id: self.name.to_string(),
                        line_number,
                    }
                })?,
            },
            ftd::interpreter::Kind::Record { name, .. } => {
                let rec_fields = self.get_record(line_number, &name)?.fields;
                let mut fields: ftd::Map<ftd::interpreter::PropertyValue> = Default::default();
                if let serde_json::Value::Object(o) = json {
                    for (key, kind) in rec_fields {
                        let val = match o.get(&key) {
                            Some(v) => v,
                            None => {
                                return ftd::interpreter::utils::e2(
                                    format!("key not found: {}", key.as_str()),
                                    self.name,
                                    line_number,
                                )
                            }
                        };
                        fields.insert(
                            key,
                            ftd::interpreter::PropertyValue::Value {
                                value: self.from_json_(line_number, val, kind)?,
                            },
                        );
                    }
                } else {
                    return ftd::interpreter::utils::e2(
                        format!("expected object of record type, found: {}", json),
                        self.name,
                        line_number,
                    );
                }
                ftd::interpreter::Value::Record { name, fields }
            }
            ftd::interpreter::Kind::List { kind, .. } => {
                let kind = kind.as_ref();
                let mut data: Vec<ftd::interpreter::PropertyValue> = vec![];
                if let serde_json::Value::Array(list) = json {
                    for item in list {
                        data.push(ftd::interpreter::PropertyValue::Value {
                            value: self.from_json_(line_number, item, kind.to_owned())?,
                        });
                    }
                } else {
                    return ftd::interpreter::utils::e2(
                        format!("expected object of list type, found: {}", json),
                        self.name,
                        line_number,
                    );
                }
                ftd::interpreter::Value::List {
                    data,
                    kind: kind.to_owned(),
                }
            }
            ftd::interpreter::Kind::Optional { kind, .. } => {
                let kind = kind.as_ref().to_owned();
                match json {
                    serde_json::Value::Null => ftd::interpreter::Value::Optional {
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
        section: &ftd::p11::Section,
        rows: &[Vec<serde_json::Value>],
    ) -> ftd::p11::Result<ftd::interpreter::Value> {
        if let Ok(v) = self.get_value(0, section.name.as_str()) {
            return from_json_rows_(section.line_number, self, rows, v.kind());
        }
        if let Ok(list) = ftd::interpreter::Variable::list_from_p1(section, self) {
            return from_json_rows_(section.line_number, self, rows, list.value.kind());
        }

        return ftd::interpreter::utils::e2(
            "component should be list",
            self.name,
            section.line_number,
        );

        fn from_json_rows_(
            line_number: usize,
            doc: &ftd::interpreter::TDoc,
            rows: &[Vec<serde_json::Value>],
            kind: ftd::interpreter::Kind,
        ) -> ftd::p11::Result<ftd::interpreter::Value> {
            Ok(match kind {
                ftd::interpreter::Kind::List { kind, .. } => {
                    let kind = kind.as_ref();
                    let mut data: Vec<ftd::interpreter::PropertyValue> = vec![];
                    for row in rows {
                        data.push(ftd::interpreter::PropertyValue::Value {
                            value: doc.from_json_row_(line_number, row, kind.to_owned())?,
                        });
                    }

                    ftd::interpreter::Value::List {
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
        section: &ftd::p11::Section,
        row: &[serde_json::Value],
    ) -> ftd::p11::Result<ftd::interpreter::Value> {
        if let Ok(v) = self.get_value(0, section.name.as_str()) {
            return self.from_json_row_(section.line_number, row, v.kind());
        }
        if let Ok(var) = ftd::interpreter::Variable::from_p1(section, self) {
            return self.from_json_row_(section.line_number, row, var.value.kind());
        }

        ftd::interpreter::utils::e2(
            "component should be var of record type",
            self.name,
            section.line_number,
        )
    }

    #[cfg_attr(feature = "cargo-clippy", allow(clippy::wrong_self_convention))]
    fn from_json_row_(
        &self,
        line_number: usize,
        row: &[serde_json::Value],
        kind: ftd::interpreter::Kind,
    ) -> ftd::p11::Result<ftd::interpreter::Value> {
        Ok(match kind {
            ftd::interpreter::Kind::Record { name, .. } => {
                let rec = self.get_record(line_number, &name)?;
                let rec_fields = rec.fields;
                let mut fields: ftd::Map<ftd::interpreter::PropertyValue> = Default::default();
                for (idx, key) in rec.order.iter().enumerate() {
                    if let Some(kind) = rec_fields.get(key) {
                        let val = match row.get(idx) {
                            Some(v) => v,
                            None => {
                                return ftd::interpreter::utils::e2(
                                    format!("key not found: {}", key.as_str()),
                                    self.name,
                                    line_number,
                                )
                            }
                        };
                        fields.insert(
                            key.to_string(),
                            ftd::interpreter::PropertyValue::Value {
                                value: self.from_json_(line_number, val, kind.to_owned())?,
                            },
                        );
                    } else {
                        return ftd::interpreter::utils::e2(
                            format!("field `{}` not found", key),
                            self.name,
                            line_number,
                        );
                    }
                }
                ftd::interpreter::Value::Record { name, fields }
            }
            ftd::interpreter::Kind::String { .. } if row.first().is_some() => {
                ftd::interpreter::Value::String {
                    text: serde_json::from_value::<String>(row.first().unwrap().to_owned())
                        .map_err(|_| ftd::p11::Error::ParseError {
                            message: format!("Can't parse to string, found: {:?}", row),
                            doc_id: self.name.to_string(),
                            line_number,
                        })?,
                    source: ftd::interpreter::TextSource::Header,
                }
            }
            ftd::interpreter::Kind::Integer { .. } if row.first().is_some() => {
                ftd::interpreter::Value::Integer {
                    value: serde_json::from_value::<i64>(row.first().unwrap().to_owned()).map_err(
                        |_| ftd::p11::Error::ParseError {
                            message: format!("Can't parse to integer, found: {:?}", row),
                            doc_id: self.name.to_string(),
                            line_number,
                        },
                    )?,
                }
            }
            ftd::interpreter::Kind::Decimal { .. } if row.first().is_some() => {
                ftd::interpreter::Value::Decimal {
                    value: serde_json::from_value::<f64>(row.first().unwrap().to_owned()).map_err(
                        |_| ftd::p11::Error::ParseError {
                            message: format!("Can't parse to decimal, found: {:?}", row),
                            doc_id: self.name.to_string(),
                            line_number,
                        },
                    )?,
                }
            }
            ftd::interpreter::Kind::Boolean { .. } if row.first().is_some() => {
                ftd::interpreter::Value::Boolean {
                    value: serde_json::from_value::<bool>(row.first().unwrap().to_owned())
                        .map_err(|_| ftd::p11::Error::ParseError {
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
    ) -> ftd::p11::Result<String> {
        if name.contains('#') {
            return Ok(name.to_string());
        }

        Ok(
            match ftd::interpreter::utils::split_module(name, self.name, line_number)? {
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
        instructions: &[ftd::interpreter::Instruction],
    ) -> ftd::p11::Result<String> {
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
            match ftd::interpreter::utils::split_module(name, self.name, line_number)? {
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
        arguments: &ftd::Map<ftd::interpreter::Kind>,
    ) -> ftd::p11::Result<String> {
        return Ok(if let Some(l) = name.strip_prefix('$') {
            /*let (part1, part2) = ftd::interpreter::utils::get_doc_name_and_remaining(l)?;
            if get_special_variable().iter().any(|v| part1.starts_with(v)) {
                let part2 = part2.map(|v| format!(".{}", v)).unwrap_or("".to_string());
                return Ok(format!("${}{}", part1, part2));
            } else if arguments.contains_key(part1.as_str()) {
                return Ok(format!("${}", l));
            }
            let s = self.resolve_name(line_number, l)?;
            format!("${}", s)*/
            let d = ftd::interpreter::utils::get_doc_name_and_remaining(l)?.0;
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
    ) -> ftd::p11::Result<String> {
        ftd::interpreter::utils::resolve_local_variable_name(
            line_number,
            name,
            container,
            self.name,
            self.aliases,
        )
    }

    pub fn resolve_name(&self, line_number: usize, name: &str) -> ftd::p11::Result<String> {
        ftd::interpreter::utils::resolve_name(line_number, name, self.name, self.aliases)
    }

    pub fn get_record(
        &self,
        line_number: usize,
        name: &str,
    ) -> ftd::p11::Result<ftd::interpreter::Record> {
        match self.get_thing(line_number, name)? {
            ftd::interpreter::Thing::Record(v) => Ok(v),
            v => self.err("not a record", v, "get_record", line_number),
        }
    }

    pub fn get_or_type(
        &self,
        line_number: usize,
        name: &str,
    ) -> ftd::p11::Result<ftd::interpreter::OrType> {
        match self.get_thing(line_number, name)? {
            ftd::interpreter::Thing::OrType(v) => Ok(v),
            v => self.err("not an or-type", v, "get_or_type", line_number),
        }
    }

    pub fn get_or_type_with_variant(
        &self,
        line_number: usize,
        name: &str,
    ) -> ftd::p11::Result<ftd::interpreter::OrType> {
        match self.get_thing(line_number, name)? {
            ftd::interpreter::Thing::OrTypeWithVariant { e, .. } => Ok(e),
            v => self.err("not an or-type", v, "get_or_type", line_number),
        }
    }

    pub fn is_variable_record_type(
        &self,
        line_number: usize,
        name: &str,
    ) -> ftd::p11::Result<bool> {
        Ok(match self.get_thing(line_number, name)? {
            ftd::interpreter::Thing::Variable(v) => v.value.kind().is_record(),
            _ => false,
        })
    }

    pub fn get_value_and_conditions(
        &self,
        line_number: usize,
        name: &str,
    ) -> ftd::p11::Result<(
        ftd::interpreter::Value,
        Vec<(ftd::interpreter::Boolean, ftd::interpreter::Value)>,
    )> {
        match self.get_thing(line_number, name)? {
            ftd::interpreter::Thing::Variable(v) => Ok((
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
    ) -> ftd::p11::Result<ftd::interpreter::Value> {
        // TODO: name can be a.b.c, and a and a.b are records with right fields
        match self.get_thing(line_number, name)? {
            ftd::interpreter::Thing::Variable(v) => v.value.partial_resolve(line_number, self),
            v => self.err("not a variable", v, "get_value", line_number),
        }
    }

    fn err<T, T2: std::fmt::Debug>(
        &self,
        msg: &str,
        ctx: T2,
        f: &str,
        line_number: usize,
    ) -> ftd::p11::Result<T> {
        ftd::interpreter::utils::e2(
            format!("{}: {} ({:?}), f: {}", self.name, msg, ctx, f),
            self.name,
            line_number,
        )
    }

    pub fn get_component(
        &self,
        line_number: usize,
        name: &str,
    ) -> ftd::p11::Result<ftd::interpreter::Component> {
        match self.get_thing(line_number, name)? {
            ftd::interpreter::Thing::Component(v) => Ok(v),
            v => self.err("not a component", v, "get_component", line_number),
        }
    }

    pub fn get_root(&'a self, name: &'a str, line_number: usize) -> ftd::p11::Result<Option<&str>> {
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
        match ftd::interpreter::utils::split_module(name, self.name, line_number)? {
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
        &'a self,
        line_number: usize,
        name: &'a str,
    ) -> ftd::p11::Result<(ftd::interpreter::Thing, Option<String>)> {
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
            doc: &ftd::interpreter::TDoc,
            root_name: Option<&str>,
            doc_name: &str,
            name: &str,
        ) -> Option<(ftd::interpreter::Thing, Option<String>)> {
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
        &'a self,
        line_number: usize,
        name: &'a str,
        value: ftd::interpreter::Variable,
    ) -> ftd::p11::Result<ftd::interpreter::Variable> {
        let (initial_thing, remaining) = self.get_initial_thing(line_number, name)?;

        let remaining = if let Some(remaining) = remaining {
            remaining
        } else {
            return Ok(value);
        };

        let mut variable = if let ftd::interpreter::Thing::Variable(variable) = initial_thing {
            variable
        } else {
            return ftd::interpreter::utils::e2(
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
            doc: &ftd::interpreter::TDoc,
            line_number: usize,
            name: &str,
            var_value: &ftd::interpreter::PropertyValue,
            set_value: ftd::interpreter::PropertyValue,
        ) -> ftd::p11::Result<ftd::interpreter::PropertyValue> {
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
                ftd::interpreter::Value::Record { fields, .. } => fields,
                ftd::interpreter::Value::OrType { fields, .. } => fields,
                ftd::interpreter::Value::Object { values } => values,
                _ => return doc.err("not an record or or-type", value, "set_thing", line_number),
            };

            if let Some(data) = fields.get_mut(v) {
                if let Some(remaining) = remaining {
                    *data = set_value_(doc, line_number, remaining, data, set_value)?;
                } else {
                    *data = set_value;
                }
            }

            Ok(ftd::interpreter::PropertyValue::Value {
                value: if value.is_optional() {
                    ftd::interpreter::Value::Optional {
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
        &'a self,
        line_number: usize,
        name: &'a str,
    ) -> ftd::p11::Result<ftd::interpreter::Thing> {
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
            doc: &ftd::interpreter::TDoc,
            line_number: usize,
            name: &str,
            thing: &ftd::interpreter::Thing,
        ) -> ftd::p11::Result<ftd::interpreter::Thing> {
            let (v, remaining) = name
                .split_once('.')
                .map(|(v, n)| (v, Some(n)))
                .unwrap_or((name, None));
            let thing = match thing.clone() {
                ftd::interpreter::Thing::OrType(e) => ftd::interpreter::Thing::OrTypeWithVariant {
                    e,
                    variant: v.to_string(),
                },
                ftd::interpreter::Thing::Variable(ftd::interpreter::Variable {
                    name,
                    value,
                    conditions,
                    ..
                }) => {
                    let fields = match value.resolve(line_number, doc)?.inner_with_none() {
                        ftd::interpreter::Value::Record { fields, .. } => fields,
                        ftd::interpreter::Value::OrType { fields, .. } => fields,
                        ftd::interpreter::Value::Object { values } => values,
                        ftd::interpreter::Value::None { kind } => {
                            let kind_name = match kind {
                                ftd::interpreter::Kind::Record { ref name, .. } => name,
                                ftd::interpreter::Kind::OrType { ref name, .. } => name,
                                ftd::interpreter::Kind::OrTypeWithVariant { ref name, .. } => name,
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
                                ftd::interpreter::Thing::Record(ftd::interpreter::Record {
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
                            let thing =
                                ftd::interpreter::Thing::Variable(ftd::interpreter::Variable {
                                    name,
                                    value: ftd::interpreter::PropertyValue::Value {
                                        value: ftd::interpreter::Value::None { kind },
                                    },
                                    conditions,
                                    flags: ftd::interpreter::VariableFlags::default(),
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
                    if let Some(ftd::interpreter::PropertyValue::Value { value: val }) =
                        fields.get(v)
                    {
                        ftd::interpreter::Thing::Variable(ftd::interpreter::Variable {
                            name,
                            value: ftd::interpreter::PropertyValue::Value { value: val.clone() },
                            conditions,
                            flags: ftd::interpreter::VariableFlags::default(),
                        })
                    } else if let Some(ftd::interpreter::PropertyValue::Reference {
                        name, ..
                    }) = fields.get(v)
                    {
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
        let doc = ftd::interpreter::TDoc {
            name: "foo/bar",
            aliases: &Default::default(),
            bag: &Default::default(),
            local_variables: &mut Default::default(),
            referenced_local_variables: &mut Default::default(),
        };
        let section = ftd::p11::parse(
            indoc::indoc!(
                "
            -- string list city:
            "
            ),
            "foo/bar",
        )
        .unwrap();
        let value_from_json = doc.from_json_rows(&section[0], &data).unwrap();
        let value = ftd::interpreter::Value::List {
            data: vec![
                ftd::interpreter::PropertyValue::Value {
                    value: ftd::interpreter::Value::String {
                        text: "Prayagraj".to_string(),
                        source: ftd::interpreter::TextSource::Header,
                    },
                },
                ftd::interpreter::PropertyValue::Value {
                    value: ftd::interpreter::Value::String {
                        text: "Varanasi".to_string(),
                        source: ftd::interpreter::TextSource::Header,
                    },
                },
            ],
            kind: ftd::interpreter::Kind::String {
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
            ftd::test::interpret("foo/bar", &source, &ftd::interpreter::TestLibrary {})
                .expect("found error");
        let data: Vec<Vec<serde_json::Value>> = vec![
            vec![
                serde_json::json!("Amitu"),
                serde_json::json!(20 as i64),
                serde_json::json!("Bangalore"),
                serde_json::json!("CEO of fifthTry"),
            ],
            vec![
                serde_json::json!("Arpita"),
                serde_json::json!(20 as i64),
                serde_json::json!("Varanasi"),
                serde_json::json!("Software Developer of fifthTry"),
            ],
        ];
        let doc = ftd::interpreter::TDoc {
            name: "foo/bar",
            aliases: &Default::default(),
            bag: &g_bag,
            local_variables: &mut Default::default(),
            referenced_local_variables: &mut Default::default(),
        };
        let section = ftd::p11::parse(
            indoc::indoc!(
                "
            -- person list people:
            "
            ),
            "foo/bar",
        )
        .unwrap();
        let value_from_json = doc.from_json_rows(&section[0], &data).unwrap();
        let value = ftd::interpreter::Value::List {
            data: vec![
                ftd::interpreter::PropertyValue::Value {
                    value: ftd::interpreter::Value::Record {
                        name: "foo/bar#person".to_string(),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                "name".to_string(),
                                ftd::interpreter::PropertyValue::Value {
                                    value: ftd::interpreter::Value::String {
                                        text: "Amitu".to_string(),
                                        source: ftd::interpreter::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                "age".to_string(),
                                ftd::interpreter::PropertyValue::Value {
                                    value: ftd::interpreter::Value::Integer { value: 20 },
                                },
                            ),
                            (
                                "bio".to_string(),
                                ftd::interpreter::PropertyValue::Value {
                                    value: ftd::interpreter::Value::String {
                                        text: "CEO of fifthTry".to_string(),
                                        source: ftd::interpreter::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                "address".to_string(),
                                ftd::interpreter::PropertyValue::Value {
                                    value: ftd::interpreter::Value::String {
                                        text: "Bangalore".to_string(),
                                        source: ftd::interpreter::TextSource::Header,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                ftd::interpreter::PropertyValue::Value {
                    value: ftd::interpreter::Value::Record {
                        name: "foo/bar#person".to_string(),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                "name".to_string(),
                                ftd::interpreter::PropertyValue::Value {
                                    value: ftd::interpreter::Value::String {
                                        text: "Arpita".to_string(),
                                        source: ftd::interpreter::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                "age".to_string(),
                                ftd::interpreter::PropertyValue::Value {
                                    value: ftd::interpreter::Value::Integer { value: 20 },
                                },
                            ),
                            (
                                "bio".to_string(),
                                ftd::interpreter::PropertyValue::Value {
                                    value: ftd::interpreter::Value::String {
                                        text: "Software Developer of fifthTry".to_string(),
                                        source: ftd::interpreter::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                "address".to_string(),
                                ftd::interpreter::PropertyValue::Value {
                                    value: ftd::interpreter::Value::String {
                                        text: "Varanasi".to_string(),
                                        source: ftd::interpreter::TextSource::Header,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
            ],
            kind: ftd::interpreter::Kind::Record {
                name: "foo/bar#person".to_string(),
                default: None,
                is_reference: false,
            },
        };
        pretty_assertions::assert_eq!(value_from_json, value);
    }
}
