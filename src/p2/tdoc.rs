#[derive(Debug, PartialEq)]
pub struct TDoc<'a> {
    pub name: &'a str,
    pub aliases: &'a std::collections::BTreeMap<String, String>,
    pub bag: &'a std::collections::BTreeMap<String, ftd::p2::Thing>,
    pub local_variables: &'a mut std::collections::BTreeMap<String, ftd::p2::Thing>,
}

impl<'a> TDoc<'a> {
    pub(crate) fn insert_local_from_component(
        &mut self,
        component: &mut ftd::Component,
        child_component_properties: &std::collections::BTreeMap<String, ftd::component::Property>,
        local_container: &[usize],
    ) -> ftd::p1::Result<()> {
        let string_container: String = local_container
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>()
            .join(",");
        if component.root == "ftd.kernel" {
            return Ok(());
        }
        for (k, arg) in component.arguments.iter() {
            let mut default = if let Some(d) = child_component_properties.get(k) {
                if let Some(ref d) = d.default {
                    d.to_owned()
                } else {
                    //todo
                    return ftd::e2(
                        format!(
                            "expected default value for local variable {}: {:?} in {}",
                            k, arg, component.root
                        ),
                        self.name,
                        0,
                    );
                }
            } else {
                if let Some(default) = arg.get_default_value_str() {
                    ftd::PropertyValue::resolve_value(
                        0,
                        default.as_str(),
                        Some(arg.to_owned()),
                        self,
                        &component.arguments,
                        None,
                    )?
                } else {
                    return ftd::e2(
                        format!(
                            "expected default value for local variable {}: {:?} in {}",
                            k, arg, component.root
                        ),
                        self.name,
                        0,
                    );
                }
            };
            if let ftd::PropertyValue::Variable { ref mut name, .. } = default {
                *name = self.resolve_name(0, format!("{}@{}", name, string_container).as_str())?;
            }
            let local_variable = ftd::p2::Thing::Variable(ftd::Variable {
                name: k.to_string(),
                value: default,
                conditions: vec![],
                flags: Default::default(),
            });
            self.local_variables.insert(
                self.resolve_name(0, format!("{}@{}", k, string_container).as_str())?,
                local_variable,
            );
        }
        for (_, property) in component.properties.iter_mut() {
            if let Some(ftd::PropertyValue::Variable { ref mut name, .. }) = property.default {
                *name = self.resolve_name(0, format!("{}@{}", name, string_container).as_str())?;
            }
            for (_, condition) in property.conditions.iter_mut() {
                if let ftd::PropertyValue::Variable { ref mut name, .. } = condition {
                    *name =
                        self.resolve_name(0, format!("{}@{}", name, string_container).as_str())?
                }
            }
        }
        if let Some(ref mut condition) = component.condition {
            edit_condition(condition, self, &string_container)?;
        }

        component.arguments = Default::default();
        for instruction in component.instructions.iter_mut() {
            let child = match instruction {
                ftd::Instruction::ChildComponent { child } => child,
                _ => continue,
            };
            for (_, property) in child.properties.iter_mut() {
                if let Some(ftd::PropertyValue::Variable { ref mut name, .. }) = property.default {
                    *name =
                        self.resolve_name(0, format!("{}@{}", name, string_container).as_str())?;
                }
                for (_, condition) in property.conditions.iter_mut() {
                    if let ftd::PropertyValue::Variable { ref mut name, .. } = condition {
                        *name =
                            self.resolve_name(0, format!("{}@{}", name, string_container).as_str())?
                    }
                }
            }
            if let Some((ref mut c, _)) = child.reference {
                *c = self.resolve_name(0, format!("{}@{}", c, string_container).as_str())?;
            }
            if let Some(ref mut condition) = child.condition {
                edit_condition(condition, self, &string_container)?;
            }
        }
        return Ok(());

        fn edit_condition(
            condition: &mut ftd::p2::Boolean,
            doc: &ftd::p2::TDoc,
            string_container: &str,
        ) -> ftd::p1::Result<()> {
            match condition {
                ftd::p2::Boolean::IsNotNull { value }
                | ftd::p2::Boolean::IsNull { value }
                | ftd::p2::Boolean::IsNotEmpty { value }
                | ftd::p2::Boolean::IsEmpty { value }
                | ftd::p2::Boolean::ListIsEmpty { value } => {
                    if let ftd::PropertyValue::Variable { ref mut name, .. } = value {
                        *name =
                            doc.resolve_name(0, format!("{}@{}", name, string_container).as_str())?;
                    }
                }
                ftd::p2::Boolean::Equal { left, right }
                | ftd::p2::Boolean::NotEqual { left, right } => {
                    if let ftd::PropertyValue::Variable { ref mut name, .. } = left {
                        *name =
                            doc.resolve_name(0, format!("{}@{}", name, string_container).as_str())?;
                    }
                    if let ftd::PropertyValue::Variable { ref mut name, .. } = right {
                        *name =
                            doc.resolve_name(0, format!("{}@{}", name, string_container).as_str())?;
                    }
                }
                ftd::p2::Boolean::Not { of } => edit_condition(of, doc, string_container)?,
                ftd::p2::Boolean::Literal { .. } => {}
            }
            Ok(())
        }
    }

    pub(crate) fn insert_local(
        &mut self,
        parent: &mut ftd::ChildComponent,
        children: &mut Vec<ftd::ChildComponent>,
        local_container: &[usize],
    ) -> ftd::p1::Result<()> {
        let string_container: String = local_container
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>()
            .join(",");
        for (k, arg) in parent.arguments.iter() {
            let default = {
                if let Some(default) = arg.get_default_value_str() {
                    ftd::PropertyValue::resolve_value(
                        0,
                        default.as_str(),
                        Some(arg.to_owned()),
                        self,
                        &parent.arguments,
                        None,
                    )?
                } else {
                    return ftd::e2(
                        format!(
                            "expected default value for local variable {}: {:?} in {}",
                            k, arg, parent.root
                        ),
                        self.name,
                        0,
                    );
                }
            };
            let local_variable = ftd::p2::Thing::Variable(ftd::Variable {
                name: k.to_string(),
                value: default,
                conditions: vec![],
                flags: Default::default(),
            });
            self.local_variables.insert(
                self.resolve_name(0, format!("{}@{}", k, string_container).as_str())?,
                local_variable,
            );
        }
        if let Some(ref mut condition) = parent.condition {
            edit_condition(condition, self, &string_container);
        }
        for (_, property) in parent.properties.iter_mut() {
            if let Some(ftd::PropertyValue::Variable { ref mut name, .. }) = property.default {
                *name = self.resolve_name(0, format!("{}@{}", name, string_container).as_str())?;
            }
            for (_, condition) in property.conditions.iter_mut() {
                if let ftd::PropertyValue::Variable { ref mut name, .. } = condition {
                    *name =
                        self.resolve_name(0, format!("{}@{}", name, string_container).as_str())?;
                }
            }
        }
        parent.arguments = Default::default();
        for child in children.iter_mut() {
            for (_, property) in child.properties.iter_mut() {
                if let Some(ftd::PropertyValue::Variable { ref mut name, .. }) = property.default {
                    *name =
                        self.resolve_name(0, format!("{}@{}", name, string_container).as_str())?;
                }
                for (_, condition) in property.conditions.iter_mut() {
                    if let ftd::PropertyValue::Variable { ref mut name, .. } = condition {
                        *name = self
                            .resolve_name(0, format!("{}@{}", name, string_container).as_str())?;
                    }
                }
            }
            if let Some((ref mut c, _)) = child.reference {
                *c = self.resolve_name(0, format!("{}@{}", c, string_container).as_str())?;
            }
            if let Some(ref mut condition) = child.condition {
                edit_condition(condition, self, &string_container);
            }
        }
        return Ok(());

        fn edit_condition(
            condition: &mut ftd::p2::Boolean,
            doc: &ftd::p2::TDoc,
            string_container: &str,
        ) -> ftd::p1::Result<()> {
            match condition {
                ftd::p2::Boolean::IsNotNull { value }
                | ftd::p2::Boolean::IsNull { value }
                | ftd::p2::Boolean::IsNotEmpty { value }
                | ftd::p2::Boolean::IsEmpty { value }
                | ftd::p2::Boolean::ListIsEmpty { value } => {
                    if let ftd::PropertyValue::Variable { ref mut name, .. } = value {
                        *name =
                            doc.resolve_name(0, format!("{}@{}", name, string_container).as_str())?
                    }
                }
                ftd::p2::Boolean::Equal { left, right }
                | ftd::p2::Boolean::NotEqual { left, right } => {
                    if let ftd::PropertyValue::Variable { ref mut name, .. } = left {
                        *name =
                            doc.resolve_name(0, format!("{}@{}", name, string_container).as_str())?
                    }
                    if let ftd::PropertyValue::Variable { ref mut name, .. } = right {
                        *name =
                            doc.resolve_name(0, format!("{}@{}", name, string_container).as_str())?
                    }
                }
                ftd::p2::Boolean::Not { of } => edit_condition(of, doc, string_container)?,
                ftd::p2::Boolean::Literal { .. } => {}
            }
            Ok(())
        }
    }

    pub fn from_json<T>(&self, json: &T, section: &ftd::p1::Section) -> ftd::p1::Result<ftd::Value>
    where
        T: serde::Serialize + std::fmt::Debug,
    {
        let json = serde_json::to_value(json).map_err(|e| ftd::p1::Error::ParseError {
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

        ftd::e2(
            "component should be var or list",
            self.name,
            section.line_number,
        )
    }

    fn from_json_(
        &self,
        line_number: usize,
        json: &serde_json::Value,
        kind: ftd::p2::Kind,
    ) -> ftd::p1::Result<ftd::Value> {
        Ok(match kind {
            ftd::p2::Kind::String { .. } => ftd::Value::String {
                text: serde_json::from_value::<String>(json.to_owned()).map_err(|_| {
                    ftd::p1::Error::ParseError {
                        message: format!("Can't parse to string, found: {}", json),
                        doc_id: self.name.to_string(),
                        line_number,
                    }
                })?,
                source: ftd::TextSource::Header,
            },
            ftd::p2::Kind::Integer { .. } => ftd::Value::Integer {
                value: serde_json::from_value::<i64>(json.to_owned()).map_err(|_| {
                    ftd::p1::Error::ParseError {
                        message: format!("Can't parse to integer, found: {}", json),
                        doc_id: self.name.to_string(),
                        line_number,
                    }
                })?,
            },
            ftd::p2::Kind::Decimal { .. } => ftd::Value::Decimal {
                value: serde_json::from_value::<f64>(json.to_owned()).map_err(|_| {
                    ftd::p1::Error::ParseError {
                        message: format!("Can't parse to decimal, found: {}", json),
                        doc_id: self.name.to_string(),
                        line_number,
                    }
                })?,
            },
            ftd::p2::Kind::Boolean { .. } => ftd::Value::Boolean {
                value: serde_json::from_value::<bool>(json.to_owned()).map_err(|_| {
                    ftd::p1::Error::ParseError {
                        message: format!("Can't parse to boolean,found: {}", json),
                        doc_id: self.name.to_string(),
                        line_number,
                    }
                })?,
            },
            ftd::p2::Kind::Record { name, .. } => {
                let rec_fields = self.get_record(line_number, &name)?.fields;
                let mut fields: std::collections::BTreeMap<String, ftd::PropertyValue> =
                    Default::default();
                if let serde_json::Value::Object(o) = json {
                    for (key, kind) in rec_fields {
                        let val = match o.get(&key) {
                            Some(v) => v,
                            None => {
                                return ftd::e2(
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
                    return ftd::e2(
                        format!("expected object of record type, found: {}", json),
                        self.name,
                        line_number,
                    );
                }
                ftd::Value::Record { name, fields }
            }
            ftd::p2::Kind::List { kind, .. } => {
                let kind = kind.as_ref();
                let mut data: Vec<ftd::PropertyValue> = vec![];
                if let serde_json::Value::Array(list) = json {
                    for item in list {
                        data.push(ftd::PropertyValue::Value {
                            value: self.from_json_(line_number, item, kind.to_owned())?,
                        });
                    }
                } else {
                    return ftd::e2(
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
        section: &ftd::p1::Section,
        rows: &[Vec<serde_json::Value>],
    ) -> ftd::p1::Result<ftd::Value> {
        if let Ok(v) = self.get_value(0, section.name.as_str()) {
            return from_json_rows_(section.line_number, self, rows, v.kind());
        }
        if let Ok(list) = ftd::Variable::list_from_p1(section, self) {
            return from_json_rows_(section.line_number, self, rows, list.value.kind());
        }

        return ftd::e2("component should be list", self.name, section.line_number);

        fn from_json_rows_(
            line_number: usize,
            doc: &ftd::p2::TDoc,
            rows: &[Vec<serde_json::Value>],
            kind: ftd::p2::Kind,
        ) -> ftd::p1::Result<ftd::Value> {
            Ok(match kind {
                ftd::p2::Kind::List { kind, .. } => {
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
        section: &ftd::p1::Section,
        row: &[serde_json::Value],
    ) -> ftd::p1::Result<ftd::Value> {
        if let Ok(v) = self.get_value(0, section.name.as_str()) {
            return self.from_json_row_(section.line_number, row, v.kind());
        }
        if let Ok(var) = ftd::Variable::from_p1(section, self) {
            return self.from_json_row_(section.line_number, row, var.value.kind());
        }

        ftd::e2(
            "component should be var of record type",
            self.name,
            section.line_number,
        )
    }

    fn from_json_row_(
        &self,
        line_number: usize,
        row: &[serde_json::Value],
        kind: ftd::p2::Kind,
    ) -> ftd::p1::Result<ftd::Value> {
        Ok(match kind {
            ftd::p2::Kind::Record { name, .. } => {
                let rec = self.get_record(line_number, &name)?;
                let rec_fields = rec.fields;
                let mut fields: std::collections::BTreeMap<String, ftd::PropertyValue> =
                    Default::default();
                for (idx, key) in rec.order.iter().enumerate() {
                    if let Some(kind) = rec_fields.get(key) {
                        let val = match row.get(idx) {
                            Some(v) => v,
                            None => {
                                return ftd::e2(
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
                        return ftd::e2(
                            format!("field `{}` not found", key),
                            self.name,
                            line_number,
                        );
                    }
                }
                ftd::Value::Record { name, fields }
            }
            ftd::p2::Kind::String { .. } if row.first().is_some() => ftd::Value::String {
                text: serde_json::from_value::<String>(row.first().unwrap().to_owned()).map_err(
                    |_| ftd::p1::Error::ParseError {
                        message: format!("Can't parse to string, found: {:?}", row),
                        doc_id: self.name.to_string(),
                        line_number,
                    },
                )?,
                source: ftd::TextSource::Header,
            },
            ftd::p2::Kind::Integer { .. } if row.first().is_some() => ftd::Value::Integer {
                value: serde_json::from_value::<i64>(row.first().unwrap().to_owned()).map_err(
                    |_| ftd::p1::Error::ParseError {
                        message: format!("Can't parse to integer, found: {:?}", row),
                        doc_id: self.name.to_string(),
                        line_number,
                    },
                )?,
            },
            ftd::p2::Kind::Decimal { .. } if row.first().is_some() => ftd::Value::Decimal {
                value: serde_json::from_value::<f64>(row.first().unwrap().to_owned()).map_err(
                    |_| ftd::p1::Error::ParseError {
                        message: format!("Can't parse to decimal, found: {:?}", row),
                        doc_id: self.name.to_string(),
                        line_number,
                    },
                )?,
            },
            ftd::p2::Kind::Boolean { .. } if row.first().is_some() => ftd::Value::Boolean {
                value: serde_json::from_value::<bool>(row.first().unwrap().to_owned()).map_err(
                    |_| ftd::p1::Error::ParseError {
                        message: format!("Can't parse to boolean,found: {:?}", row),
                        doc_id: self.name.to_string(),
                        line_number,
                    },
                )?,
            },
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
    ) -> ftd::p1::Result<String> {
        if name.contains('#') {
            return Ok(name.to_string());
        }

        Ok(match ftd::split_module(name, self.name, line_number)? {
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
        })
    }

    pub fn resolve_name_with_instruction(
        &self,
        line_number: usize,
        name: &str,
        instructions: &[ftd::Instruction],
    ) -> ftd::p1::Result<String> {
        if name.contains('#') {
            return Ok(name.to_string());
        }
        let mut available_components: std::collections::BTreeMap<String, String> =
            std::collections::BTreeMap::new();
        for instruction in instructions {
            if let Some(text) = instruction.resolve_id() {
                available_components.insert(text.to_string(), text.to_string());
            }
        }

        Ok(match ftd::split_module(name, self.name, line_number)? {
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
        })
    }

    pub(crate) fn resolve_reference_name(
        &self,
        line_number: usize,
        name: &str,
        arguments: &std::collections::BTreeMap<String, ftd::p2::Kind>,
    ) -> ftd::p1::Result<String> {
        return Ok(if let Some(l) = name.strip_prefix('$') {
            let d = ftd::p2::utils::get_doc_name_and_remaining(l)?.0;
            if arguments.contains_key(d.as_str()) || get_special_variable().contains(&d.as_str()) {
                return Ok(format!("@{}", l));
            }
            let s = self.resolve_name(line_number, l)?;
            format!("${}", s)
        } else {
            name.to_string()
        });

        fn get_special_variable() -> Vec<&'static str> {
            vec!["MOUSE-IN"]
        }
    }

    pub fn resolve_name(&self, line_number: usize, name: &str) -> ftd::p1::Result<String> {
        if name.contains('#') {
            return Ok(name.to_string());
        }

        Ok(match ftd::split_module(name, self.name, line_number)? {
            (Some(m), v, None) => match self.aliases.get(m) {
                Some(m) => format!("{}#{}", m, v),
                None => format!("{}#{}.{}", self.name, m, v),
            },
            (Some(m), v, Some(c)) => match self.aliases.get(m) {
                Some(m) => format!("{}#{}.{}", m, v, c),
                None => format!("{}#{}.{}", self.name, v, c),
            },
            (None, v, None) => format!("{}#{}", self.name, v),
            _ => unimplemented!(),
        })
    }

    pub fn get_record(&self, line_number: usize, name: &str) -> ftd::p1::Result<ftd::p2::Record> {
        match self.get_thing(line_number, name)? {
            ftd::p2::Thing::Record(v) => Ok(v),
            v => self.err("not a record", v, "get_record", line_number),
        }
    }

    pub fn get_or_type(&self, line_number: usize, name: &str) -> ftd::p1::Result<ftd::OrType> {
        match self.get_thing(line_number, name)? {
            ftd::p2::Thing::OrType(v) => Ok(v),
            v => self.err("not an or-type", v, "get_or_type", line_number),
        }
    }

    pub fn get_or_type_with_variant(
        &self,
        line_number: usize,
        name: &str,
    ) -> ftd::p1::Result<ftd::OrType> {
        match self.get_thing(line_number, name)? {
            ftd::p2::Thing::OrTypeWithVariant { e, .. } => Ok(e),
            v => self.err("not an or-type", v, "get_or_type", line_number),
        }
    }

    pub fn is_variable_record_type(&self, line_number: usize, name: &str) -> ftd::p1::Result<bool> {
        Ok(match self.get_thing(line_number, name)? {
            ftd::p2::Thing::Variable(v) => v.value.kind().is_record(),
            _ => false,
        })
    }

    pub fn get_value_and_conditions(
        &self,
        line_number: usize,
        name: &str,
    ) -> ftd::p1::Result<(ftd::Value, Vec<(ftd::p2::Boolean, ftd::Value)>)> {
        match self.get_thing(line_number, name)? {
            ftd::p2::Thing::Variable(v) => Ok((
                v.value.resolve(line_number, self)?,
                v.conditions
                    .into_iter()
                    .map(|(b, v)| {
                        if let Ok(v) = v.resolve(line_number, self) {
                            Some((b, v))
                        } else {
                            None
                        }
                    })
                    .flatten()
                    .collect(),
            )),
            v => self.err("not a variable", v, "get_value", line_number),
        }
    }

    pub fn get_value(&self, line_number: usize, name: &str) -> ftd::p1::Result<ftd::Value> {
        // TODO: name can be a.b.c, and a and a.b are records with right fields
        match self.get_thing(line_number, name)? {
            ftd::p2::Thing::Variable(v) => v.value.partial_resolve(line_number, self),
            v => self.err("not a variable", v, "get_value", line_number),
        }
    }

    fn err<T, T2: std::fmt::Debug>(
        &self,
        msg: &str,
        ctx: T2,
        f: &str,
        line_number: usize,
    ) -> ftd::p1::Result<T> {
        ftd::e2(
            format!("{}: {} ({:?}), f: {}", self.name, msg, ctx, f),
            self.name,
            line_number,
        )
    }

    pub fn get_component(&self, line_number: usize, name: &str) -> ftd::p1::Result<ftd::Component> {
        match self.get_thing(line_number, name)? {
            ftd::p2::Thing::Component(v) => Ok(v),
            v => self.err("not a component", v, "get_component", line_number),
        }
    }

    pub fn get_root(&'a self, name: &'a str, line_number: usize) -> ftd::p1::Result<Option<&str>> {
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
        match ftd::split_module(name, self.name, line_number)? {
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
    pub fn get_thing(
        &'a self,
        line_number: usize,
        name: &'a str,
    ) -> ftd::p1::Result<ftd::p2::Thing> {
        let name = if let Some(name) = name.strip_prefix('$') {
            name
        } else {
            name
        };

        let (initial_thing, name) = get_initial_thing(self, line_number, name)?;

        if let Some(remaining) = name {
            return get_thing(self, line_number, remaining.as_str(), &initial_thing);
        }
        return Ok(initial_thing);

        fn get_initial_thing(
            doc: &ftd::p2::TDoc,
            line_number: usize,
            name: &str,
        ) -> ftd::p1::Result<(ftd::p2::Thing, Option<String>)> {
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
                return match doc.bag.get(name.as_str()) {
                    Some(a) => Ok((a.to_owned(), remaining_value)),
                    None => match doc.local_variables.get(name.as_str()) {
                        Some(a) => Ok((a.to_owned(), remaining_value)),
                        None => doc.err("not found", name, "get_thing", line_number),
                    },
                };
            }
            return Ok(match get_initial_thing_(doc, None, doc.name, name) {
                Some(a) => a,
                None => {
                    if let Some((m, v)) = name.split_once('.') {
                        match get_initial_thing_(doc, Some(m), m, v) {
                            None => return doc.err("not found", name, "get_thing", line_number),
                            Some(a) => a,
                        }
                    } else {
                        return doc.err("not found", name, "get_thing", line_number);
                    }
                }
            });

            fn get_initial_thing_(
                doc: &ftd::p2::TDoc,
                root_name: Option<&str>,
                doc_name: &str,
                name: &str,
            ) -> Option<(ftd::p2::Thing, Option<String>)> {
                let (name, remaining_value) =
                    if let Some((v, remaining_value)) = name.split_once('.') {
                        (v, Some(remaining_value.to_string()))
                    } else {
                        (name, None)
                    };
                let bag = {
                    let mut bag = doc.bag.to_owned();
                    bag.extend(doc.local_variables.to_owned());
                    bag
                };
                match bag
                    .get(format!("{}#{}", doc_name, name).as_str())
                    .map(ToOwned::to_owned)
                {
                    Some(a) => Some((a, remaining_value)),
                    None => match root_name {
                        Some(doc_name) => match doc.aliases.get(doc_name) {
                            Some(g) => bag
                                .get(format!("{}#{}", g, name).as_str())
                                .map(|v| (v.clone(), remaining_value)),
                            None => None,
                        },
                        None => None,
                    },
                }
            }
        }

        fn get_thing(
            doc: &ftd::p2::TDoc,
            line_number: usize,
            name: &str,
            thing: &ftd::p2::Thing,
        ) -> ftd::p1::Result<ftd::p2::Thing> {
            let (v, remaining) = name
                .split_once('.')
                .map(|(v, n)| (v, Some(n)))
                .unwrap_or((name, None));
            let thing = match thing.clone() {
                ftd::p2::Thing::OrType(e) => ftd::p2::Thing::OrTypeWithVariant {
                    e,
                    variant: v.to_string(),
                },
                ftd::p2::Thing::Variable(ftd::Variable {
                    name,
                    value,
                    conditions,
                    ..
                }) => {
                    let fields = match value.resolve(line_number, doc)? {
                        ftd::Value::Record { fields, .. } => fields,
                        ftd::Value::OrType { fields, .. } => fields,
                        ftd::Value::Object { values } => values,
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
                        ftd::p2::Thing::Variable(ftd::Variable {
                            name,
                            value: ftd::PropertyValue::Value { value: val.clone() },
                            conditions,
                            flags: ftd::variable::VariableFlags {
                                always_include: None,
                            },
                        })
                    } else if let Some(ftd::PropertyValue::Reference { name, .. }) = fields.get(v) {
                        get_initial_thing(doc, line_number, name)?.0
                    } else {
                        thing.clone()
                    }
                }
                _ => {
                    return doc.err("not an or-type", thing, "get_thing", line_number);
                }
            };
            if let Some(remaining) = remaining {
                return get_thing(doc, line_number, remaining, &thing);
            }
            Ok(thing)
        }
    }

    /*pub fn get_thing_with_root(
        &'a self,
        line_number: usize,
        name: &'a str,
        root_name: Option<&'a str>,
    ) -> ftd::p1::Result<ftd::p2::Thing> {
        let name = if let Some(name) = name.strip_prefix('$') {
            name
        } else {
            name
        };

        match if name.contains('#') {
            self.bag.get(name).map(ToOwned::to_owned)
        } else if self
            .bag
            .get(format!("{}#{}", self.name, name).as_str())
            .is_some()
        {
            self.bag
                .get(format!("{}#{}", self.name, name).as_str())
                .map(ToOwned::to_owned)
        } else {
            match ftd::split_module(name, self.name, line_number)? {
                (Some(m), v, None) => match self.aliases.get(m) {
                    Some(m) => self
                        .bag
                        .get(format!("{}#{}", m, v).as_str())
                        .map(ToOwned::to_owned),
                    None => {
                        let thing = self.get_thing(line_number, m)?;
                        match thing.clone() {
                            ftd::p2::Thing::OrType(e) => Some(ftd::p2::Thing::OrTypeWithVariant {
                                e,
                                variant: v.to_string(),
                            }),
                            ftd::p2::Thing::Variable(ftd::Variable {
                                name,
                                value,
                                conditions,
                            }) => {
                                let fields = match value {
                                    ftd::Value::Record { fields, .. } => fields,
                                    ftd::Value::OrType { fields, .. } => fields,
                                    _ => {
                                        return self.err(
                                            "not an record or or-type",
                                            thing,
                                            "get_thing",
                                            line_number,
                                        )
                                    }
                                };
                                if let Some(ftd::PropertyValue::Value { value: val }) =
                                    fields.get(v)
                                {
                                    return Ok(ftd::p2::Thing::Variable(ftd::Variable {
                                        name,
                                        value: val.clone(),
                                        conditions,
                                    }));
                                } else if let Some(ftd::PropertyValue::Reference { name, .. }) =
                                    fields.get(v)
                                {
                                    self.bag.get(name).map(ToOwned::to_owned)
                                } else {
                                    Some(thing)
                                }
                            }
                            _ => {
                                return self.err("not an or-type", thing, "get_thing", line_number);
                            }
                        }
                    }
                },
                (Some(m), e, Some(v)) => match self.aliases.get(m) {
                    Some(m) => match self.bag.get(format!("{}#{}", m, e).as_str()) {
                        Some(ftd::p2::Thing::OrType(e)) => {
                            Some(ftd::p2::Thing::OrTypeWithVariant {
                                e: e.to_owned(),
                                variant: v.to_string(),
                            })
                        }
                        Some(t) => {
                            return self.err("not an or-type", t, "get_thing", line_number);
                        }
                        None => {
                            return self.err(
                                "not found",
                                format!("{}#{}", m, e),
                                "get_thing",
                                line_number,
                            )
                        }
                    },
                    None => return self.err("not found", name, "get_thing", line_number),
                },
                (None, v, None) => {
                    match self
                        .bag
                        .get(format!("{}#{}", self.name, v).as_str())
                        .map(|v| v.to_owned())
                    {
                        Some(a) => Some(a),
                        None => match root_name {
                            Some(name) => match self.aliases.get(name) {
                                Some(g) => self
                                    .bag
                                    .get(format!("{}#{}", g, v).as_str())
                                    .map(|b| b.to_owned()),
                                None => None,
                            },
                            None => None,
                        },
                    }
                }
                (None, e, Some(v)) => match self.bag.get(format!("{}#{}", self.name, e).as_str()) {
                    Some(ftd::p2::Thing::OrType(e)) => Some(ftd::p2::Thing::OrTypeWithVariant {
                        e: e.to_owned(),
                        variant: v.to_string(),
                    }),
                    Some(t) => {
                        return self.err("expected or-type, found", t, "get_thing", line_number);
                    }
                    None => return self.err("not found", name, "get_thing", line_number),
                },
            }
        } {
            Some(v) => Ok(v),
            None => self.err("not found", name, "get_thing", line_number),
        }
    }*/
}

#[cfg(test)]
mod test {
    #[test]
    fn string_list_from_rows() {
        let data: Vec<Vec<serde_json::Value>> = vec![
            vec![serde_json::json!("Prayagraj")],
            vec![serde_json::json!("Varanasi")],
        ];
        let doc = ftd::p2::TDoc {
            name: "foo/bar",
            aliases: &Default::default(),
            bag: &Default::default(),
            local_variables: &mut Default::default(),
        };
        let section = ftd::p1::parse(
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
            kind: ftd::p2::Kind::String {
                caption: false,
                body: false,
                default: None,
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
            ftd::p2::interpreter::interpret("foo/bar", &source, &ftd::p2::TestLibrary {})
                .expect("found error");
        let data: Vec<Vec<serde_json::Value>> = vec![
            vec![
                serde_json::json!("Amitu"),
                serde_json::json!(20),
                serde_json::json!("Bangalore"),
                serde_json::json!("CEO of fifthTry"),
            ],
            vec![
                serde_json::json!("Arpita"),
                serde_json::json!(20),
                serde_json::json!("Varanasi"),
                serde_json::json!("Software Developer of fifthTry"),
            ],
        ];
        let doc = ftd::p2::TDoc {
            name: "foo/bar",
            aliases: &Default::default(),
            bag: &g_bag,
            local_variables: &mut Default::default(),
        };
        let section = ftd::p1::parse(
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
                        fields: std::array::IntoIter::new([
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
                        fields: std::array::IntoIter::new([
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
            kind: ftd::p2::Kind::Record {
                name: "foo/bar#person".to_string(),
                default: None,
            },
        };
        pretty_assertions::assert_eq!(value_from_json, value);
    }
}
