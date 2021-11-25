#[derive(Debug, PartialEq)]
pub struct TDoc<'a> {
    pub name: &'a str,
    pub aliases: &'a std::collections::BTreeMap<String, String>,
    pub bag: &'a std::collections::BTreeMap<String, crate::p2::Thing>,
}

impl<'a> TDoc<'a> {
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
            return from_json_(section.line_number, self, &json, v.kind());
        }
        if let Ok(list) = ftd::Variable::list_from_p1(section, self) {
            return from_json_(section.line_number, self, &json, list.value.kind());
        }
        if let Ok(var) = ftd::Variable::from_p1(section, self) {
            return from_json_(section.line_number, self, &json, var.value.kind());
        }

        return ftd::e2(
            "component should be var or list",
            self.name,
            self.name.to_string(),
            section.line_number,
        );

        fn from_json_(
            line_number: usize,
            doc: &TDoc,
            json: &serde_json::Value,
            kind: ftd::p2::Kind,
        ) -> ftd::p1::Result<ftd::Value> {
            Ok(match kind {
                ftd::p2::Kind::String { .. } => ftd::Value::String {
                    text: serde_json::from_value::<String>(json.to_owned()).map_err(|_| {
                        ftd::p1::Error::ParseError {
                            message: format!("Can't parse to string, found: {}", json),
                            doc_id: doc.name.to_string(),
                            line_number,
                        }
                    })?,
                    source: ftd::TextSource::Header,
                },
                ftd::p2::Kind::Integer { .. } => ftd::Value::Integer {
                    value: serde_json::from_value::<i64>(json.to_owned()).map_err(|_| {
                        ftd::p1::Error::ParseError {
                            message: format!("Can't parse to integer, found: {}", json),
                            doc_id: doc.name.to_string(),
                            line_number,
                        }
                    })?,
                },
                ftd::p2::Kind::Decimal { .. } => ftd::Value::Decimal {
                    value: serde_json::from_value::<f64>(json.to_owned()).map_err(|_| {
                        ftd::p1::Error::ParseError {
                            message: format!("Can't parse to decimal, found: {}", json),
                            doc_id: doc.name.to_string(),
                            line_number,
                        }
                    })?,
                },
                ftd::p2::Kind::Boolean { .. } => ftd::Value::Boolean {
                    value: serde_json::from_value::<bool>(json.to_owned()).map_err(|_| {
                        ftd::p1::Error::ParseError {
                            message: format!("Can't parse to boolean,found: {}", json),
                            doc_id: doc.name.to_string(),
                            line_number,
                        }
                    })?,
                },
                ftd::p2::Kind::Record { name, .. } => {
                    let rec_fields = doc.get_record(line_number, &name)?.fields;
                    let mut fields: std::collections::BTreeMap<String, ftd::PropertyValue> =
                        Default::default();
                    if let serde_json::Value::Object(o) = json {
                        for (key, kind) in rec_fields {
                            let val = match o.get(&key) {
                                Some(v) => v,
                                None => {
                                    return ftd::e2(
                                        format!("key not found: {}", key.as_str()),
                                        doc.name,
                                        doc.name.to_string(),
                                        line_number,
                                    )
                                }
                            };
                            fields.insert(
                                key,
                                ftd::PropertyValue::Value {
                                    value: from_json_(line_number, doc, val, kind)?,
                                },
                            );
                        }
                    } else {
                        return ftd::e2(
                            format!("expected object of record type, found: {}", json),
                            doc.name,
                            doc.name.to_string(),
                            line_number,
                        );
                    }
                    ftd::Value::Record { name, fields }
                }
                ftd::p2::Kind::List { kind } => {
                    let kind = kind.as_ref();
                    let mut data: Vec<ftd::Value> = vec![];
                    if let serde_json::Value::Array(list) = json {
                        for item in list {
                            data.push(from_json_(line_number, doc, item, kind.to_owned())?);
                        }
                    } else {
                        return ftd::e2(
                            format!("expected object of list type, found: {}", json),
                            doc.name,
                            doc.name.to_string(),
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
                    doc.name.to_string()
                ),
            })
        }
    }

    pub fn format_name(&self, name: &str) -> String {
        format!("{}#{}", self.name, name)
    }

    pub fn resolve_name_without_full_path(
        &self,
        line_number: usize,
        name: &str,
    ) -> crate::p1::Result<String> {
        if name.contains('#') {
            return Ok(name.to_string());
        }

        Ok(match ftd::split_module(name, line_number)? {
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
    ) -> crate::p1::Result<String> {
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

        Ok(match ftd::split_module(name, line_number)? {
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
            (_, _, Some(_)) => unimplemented!(),
            (None, v, None) => v.to_string(),
        })
    }

    pub fn resolve_name(&self, line_number: usize, name: &str) -> crate::p1::Result<String> {
        if name.contains('#') {
            return Ok(name.to_string());
        }

        Ok(match ftd::split_module(name, line_number)? {
            (Some(m), v, None) => match self.aliases.get(m) {
                Some(m) => format!("{}#{}", m, v),
                None => return self.err("alias not found", m, "resolve_name", line_number),
            },
            (_, _, Some(_)) => unimplemented!(),
            (None, v, None) => format!("{}#{}", self.name, v),
        })
    }

    pub fn get_record(
        &self,
        line_number: usize,
        name: &str,
    ) -> crate::p1::Result<crate::p2::Record> {
        match self.get_thing(line_number, name)? {
            crate::p2::Thing::Record(v) => Ok(v),
            v => self.err("not a record", v, "get_record", line_number),
        }
    }

    pub fn get_or_type(&self, line_number: usize, name: &str) -> crate::p1::Result<crate::OrType> {
        match self.get_thing(line_number, name)? {
            crate::p2::Thing::OrType(v) => Ok(v),
            v => self.err("not an or-type", v, "get_or_type", line_number),
        }
    }

    pub fn is_variable_record_type(
        &self,
        line_number: usize,
        name: &str,
    ) -> crate::p1::Result<bool> {
        match self.get_value(line_number, name)? {
            crate::Value::Record { .. } => Ok(true),
            _ => Ok(false),
        }
    }

    pub fn get_value_and_conditions_with_root(
        &self,
        line_number: usize,
        name: &str,
        root_name: Option<&str>,
    ) -> crate::p1::Result<(crate::Value, Vec<(crate::p2::Boolean, crate::Value)>)> {
        match self.get_thing_with_root(line_number, name, root_name)? {
            crate::p2::Thing::Variable(v) => Ok((v.value, v.conditions)),
            v => self.err("not a variable", v, "get_value", line_number),
        }
    }

    pub fn get_value(&self, line_number: usize, name: &str) -> crate::p1::Result<crate::Value> {
        // TODO: name can be a.b.c, and a and a.b are records with right fields
        self.get_value_with_root(line_number, name, None)
    }

    pub fn get_value_with_root(
        &self,
        line_number: usize,
        name: &str,
        root_name: Option<&str>,
    ) -> crate::p1::Result<crate::Value> {
        // TODO: name can be a.b.c, and a and a.b are records with right fields
        match self.get_thing_with_root(line_number, name, root_name)? {
            crate::p2::Thing::Variable(v) => Ok(v.value),
            v => self.err("not a variable", v, "get_value", line_number),
        }
    }

    fn err<T, T2: std::fmt::Debug>(
        &self,
        msg: &str,
        ctx: T2,
        f: &str,
        line_number: usize,
    ) -> crate::p1::Result<T> {
        crate::e2(
            format!("{}: {} ({:?})", self.name, msg, ctx),
            f,
            self.name.to_string(),
            line_number,
        )
    }

    pub fn get_component(
        &self,
        line_number: usize,
        name: &str,
    ) -> crate::p1::Result<crate::Component> {
        match self.get_thing(line_number, name)? {
            crate::p2::Thing::Component(v) => Ok(v),
            v => self.err("not a component", v, "get_component", line_number),
        }
    }

    pub fn get_component_with_root(
        &self,
        line_number: usize,
        name: &str,
        root_name: Option<&str>,
    ) -> crate::p1::Result<crate::Component> {
        match self.get_thing_with_root(line_number, name, root_name)? {
            crate::p2::Thing::Component(v) => Ok(v),
            v => self.err("not a component", v, "get_component", line_number),
        }
    }

    pub fn get_root(
        &'a self,
        name: &'a str,
        line_number: usize,
    ) -> crate::p1::Result<Option<&str>> {
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
        match ftd::split_module(name, line_number)? {
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
    ) -> crate::p1::Result<crate::p2::Thing> {
        self.get_thing_with_root(line_number, name, None)
    }

    pub fn get_thing_with_root(
        &'a self,
        line_number: usize,
        name: &'a str,
        root_name: Option<&'a str>,
    ) -> crate::p1::Result<crate::p2::Thing> {
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
            match ftd::split_module(name, line_number)? {
                (Some(m), v, None) => match self.aliases.get(m) {
                    Some(m) => self
                        .bag
                        .get(format!("{}#{}", m, v).as_str())
                        .map(ToOwned::to_owned),
                    None => {
                        let thing = self.get_thing(line_number, m)?;
                        match thing.clone() {
                            crate::p2::Thing::OrType(e) => {
                                Some(crate::p2::Thing::OrTypeWithVariant {
                                    e,
                                    variant: v.to_string(),
                                })
                            }
                            crate::p2::Thing::Variable(crate::Variable {
                                name,
                                value,
                                conditions,
                            }) => {
                                let fields = match value {
                                    crate::Value::Record { fields, .. } => fields,
                                    crate::Value::OrType { fields, .. } => fields,
                                    _ => {
                                        return self.err(
                                            "not an record or or-type",
                                            thing,
                                            "get_thing",
                                            line_number,
                                        )
                                    }
                                };
                                if let Some(crate::PropertyValue::Value { value: val }) =
                                    fields.get(v)
                                {
                                    return Ok(crate::p2::Thing::Variable(crate::Variable {
                                        name,
                                        value: val.clone(),
                                        conditions,
                                    }));
                                } else if let Some(crate::PropertyValue::Reference {
                                    name, ..
                                }) = fields.get(v)
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
                        Some(crate::p2::Thing::OrType(e)) => {
                            Some(crate::p2::Thing::OrTypeWithVariant {
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
                    Some(crate::p2::Thing::OrType(e)) => {
                        Some(crate::p2::Thing::OrTypeWithVariant {
                            e: e.to_owned(),
                            variant: v.to_string(),
                        })
                    }
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
    }
}
