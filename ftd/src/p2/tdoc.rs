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
        let json = serde_json::to_value(json).map_err(|e| ftd::p1::Error::InvalidInput {
            message: format!("Can't serialize to json: {:?}", e),
            context: format!("{:?}", json),
        })?;

        if let Ok(v) = self.get_value(section.name.as_str()) {
            return from_json_(self, &json, v.kind());
        }
        if let Ok(list) = ftd::Variable::list_from_p1(section, self) {
            return from_json_(self, &json, list.value.kind());
        }
        if let Ok(var) = ftd::Variable::from_p1(section, self) {
            return from_json_(self, &json, var.value.kind());
        }

        return ftd::e("component should be var or list");
        fn from_json_(
            doc: &TDoc,
            json: &serde_json::Value,
            kind: ftd::p2::Kind,
        ) -> ftd::p1::Result<ftd::Value> {
            Ok(match kind {
                ftd::p2::Kind::String { .. } => ftd::Value::String {
                    text: serde_json::from_value::<String>(json.to_owned()).map_err(|_| {
                        ftd::p1::Error::InvalidInput {
                            message: "Can't parse to string".into(),
                            context: format!("found: {}", json),
                        }
                    })?,
                    source: ftd::TextSource::Header,
                },
                ftd::p2::Kind::Integer { .. } => ftd::Value::Integer {
                    value: serde_json::from_value::<i64>(json.to_owned()).map_err(|_| {
                        ftd::p1::Error::InvalidInput {
                            message: "Can't parse to integer".into(),
                            context: format!("found: {}", json),
                        }
                    })?,
                },
                ftd::p2::Kind::Decimal { .. } => ftd::Value::Decimal {
                    value: serde_json::from_value::<f64>(json.to_owned()).map_err(|_| {
                        ftd::p1::Error::InvalidInput {
                            message: "Can't parse to decimal".into(),
                            context: format!("found: {}", json),
                        }
                    })?,
                },
                ftd::p2::Kind::Boolean { .. } => ftd::Value::Boolean {
                    value: serde_json::from_value::<bool>(json.to_owned()).map_err(|_| {
                        ftd::p1::Error::InvalidInput {
                            message: "Can't parse to boolean".into(),
                            context: format!("found: {}", json),
                        }
                    })?,
                },
                ftd::p2::Kind::Record { name, .. } => {
                    let rec_fields = doc.get_record(&name)?.fields;
                    let mut fields: std::collections::BTreeMap<String, ftd::PropertyValue> =
                        Default::default();
                    if let serde_json::Value::Object(o) = json {
                        for (key, kind) in rec_fields {
                            let val = o.get(&key).unwrap();
                            fields.insert(
                                key,
                                ftd::PropertyValue::Value {
                                    value: from_json_(doc, val, kind)?,
                                },
                            );
                        }
                    } else {
                        return ftd::e(format!("expected object of record type, found: {}", json));
                    }
                    ftd::Value::Record { name, fields }
                }
                ftd::p2::Kind::List { kind } => {
                    let kind = kind.as_ref();
                    let mut data: Vec<ftd::Value> = vec![];
                    if let serde_json::Value::Array(list) = json {
                        for item in list {
                            data.push(from_json_(doc, item, kind.to_owned())?);
                        }
                    } else {
                        return ftd::e(format!("expected object of list type, found: {}", json));
                    }
                    ftd::Value::List {
                        data,
                        kind: kind.to_owned(),
                    }
                }
                t => unimplemented!("{:?} not yet implemented", t),
            })
        }
    }

    pub fn format_name(&self, name: &str) -> String {
        format!("{}#{}", self.name, name)
    }

    pub fn resolve_name_without_full_path(&self, name: &str) -> crate::p1::Result<String> {
        if name.contains('#') {
            return Ok(name.to_string());
        }

        Ok(match ftd::split_module(name)? {
            (Some(m), v, None) => match self.aliases.get(m) {
                Some(m) => format!("{}#{}", m, v),
                None => return self.err("alias not found", m, "resolve_name_without_full_path"),
            },
            (_, _, Some(_)) => unimplemented!(),
            (None, v, None) => v.to_string(),
        })
    }

    pub fn resolve_name_with_instruction(
        &self,
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

        Ok(match ftd::split_module(name)? {
            (Some(m), v, None) => match self.aliases.get(m) {
                Some(m) => format!("{}#{}", m, v),
                None => match available_components.get(m) {
                    Some(a) => format!("{}#{}", a, v),
                    None => {
                        return self.err("alias not found", m, "resolve_name_with_instruction");
                    }
                },
            },
            (_, _, Some(_)) => unimplemented!(),
            (None, v, None) => v.to_string(),
        })
    }

    pub fn resolve_name(&self, name: &str) -> crate::p1::Result<String> {
        if name.contains('#') {
            return Ok(name.to_string());
        }

        Ok(match ftd::split_module(name)? {
            (Some(m), v, None) => match self.aliases.get(m) {
                Some(m) => format!("{}#{}", m, v),
                None => return self.err("alias not found", m, "resolve_name"),
            },
            (_, _, Some(_)) => unimplemented!(),
            (None, v, None) => format!("{}#{}", self.name, v),
        })
    }

    pub fn get_record(&self, name: &str) -> crate::p1::Result<crate::p2::Record> {
        match self.get_thing(name)? {
            crate::p2::Thing::Record(v) => Ok(v),
            v => self.err("not a record", v, "get_record"),
        }
    }

    pub fn get_or_type(&self, name: &str) -> crate::p1::Result<crate::OrType> {
        match self.get_thing(name)? {
            crate::p2::Thing::OrType(v) => Ok(v),
            v => self.err("not an or-type", v, "get_or_type"),
        }
    }

    pub fn is_variable_record_type(&self, name: &str) -> crate::p1::Result<bool> {
        match self.get_value(name)? {
            crate::Value::Record { .. } => Ok(true),
            _ => Ok(false),
        }
    }

    pub fn get_value_and_conditions_with_root(
        &self,
        name: &str,
        root_name: Option<&str>,
    ) -> crate::p1::Result<(crate::Value, Vec<(crate::p2::Boolean, crate::Value)>)> {
        match self.get_thing_with_root(name, root_name)? {
            crate::p2::Thing::Variable(v) => Ok((v.value, v.conditions)),
            v => self.err("not a variable", v, "get_value"),
        }
    }

    pub fn get_value(&self, name: &str) -> crate::p1::Result<crate::Value> {
        // TODO: name can be a.b.c, and a and a.b are records with right fields
        self.get_value_with_root(name, None)
    }

    pub fn get_value_with_root(
        &self,
        name: &str,
        root_name: Option<&str>,
    ) -> crate::p1::Result<crate::Value> {
        // TODO: name can be a.b.c, and a and a.b are records with right fields
        match self.get_thing_with_root(name, root_name)? {
            crate::p2::Thing::Variable(v) => Ok(v.value),
            v => self.err("not a variable", v, "get_value"),
        }
    }

    fn err<T, T2: std::fmt::Debug>(&self, msg: &str, ctx: T2, f: &str) -> crate::p1::Result<T> {
        crate::e2(format!("{}: {} ({:?})", self.name, msg, ctx), f)
    }

    pub fn get_component(&self, name: &str) -> crate::p1::Result<crate::Component> {
        match self.get_thing(name)? {
            crate::p2::Thing::Component(v) => Ok(v),
            v => self.err("not a component", v, "get_component"),
        }
    }

    pub fn get_component_with_root(
        &self,
        name: &str,
        root_name: Option<&str>,
    ) -> crate::p1::Result<crate::Component> {
        match self.get_thing_with_root(name, root_name)? {
            crate::p2::Thing::Component(v) => Ok(v),
            v => self.err("not a component", v, "get_component"),
        }
    }

    pub fn get_root(&'a self, name: &'a str) -> crate::p1::Result<Option<&str>> {
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
        match ftd::split_module(name)? {
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
    pub fn get_thing(&'a self, name: &'a str) -> crate::p1::Result<crate::p2::Thing> {
        self.get_thing_with_root(name, None)
    }

    pub fn get_thing_with_root(
        &'a self,
        name: &'a str,
        root_name: Option<&'a str>,
    ) -> crate::p1::Result<crate::p2::Thing> {
        match if name.contains('#') {
            self.bag.get(name).map(ToOwned::to_owned)
        } else {
            match ftd::split_module(name)? {
                (Some(m), v, None) => match self.aliases.get(m) {
                    Some(m) => self
                        .bag
                        .get(format!("{}#{}", m, v).as_str())
                        .map(ToOwned::to_owned),
                    None => {
                        let thing = self.get_thing(m)?;
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
                                return self.err("not an or-type", thing, "get_thing");
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
                            return self.err("not an or-type", t, "get_thing2");
                        }
                        None => return self.err("not found", format!("{}#{}", m, e), "get_thing3"),
                    },
                    None => return self.err("not found", name, "get_thing4"),
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
                        return self.err("expected or-type, found", t, "get_thing5");
                    }
                    None => return self.err("not found", name, "get_thing6"),
                },
                // None => return crate::e2(format!("{} not found", name), "get_thing"),
            }
        } {
            Some(v) => Ok(v),
            None => self.err("not found", name, "get_thing"),
        }
    }
}
