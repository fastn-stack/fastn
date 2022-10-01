#[derive(Debug, PartialEq)]
pub struct TDoc<'a> {
    pub name: &'a str,
    pub aliases: &'a ftd::Map<String>,
    pub bag: &'a ftd::Map<ftd::interpreter2::Thing>,
}

impl<'a> TDoc<'a> {
    pub fn resolve_name(&self, name: &str) -> String {
        ftd::interpreter2::utils::resolve_name(name, self.name, self.aliases)
    }

    pub fn get_record(
        &'a self,
        line_number: usize,
        name: &'a str,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::Record> {
        match self.get_thing(line_number, name)? {
            ftd::interpreter2::Thing::Record(r) => Ok(r),
            t => self.err(
                format!("Expected Record, found: `{:?}`", t).as_str(),
                name,
                "get_record",
                line_number,
            ),
        }
    }

    pub fn get_variable(
        &'a self,
        name: &'a str,
        line_number: usize,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::Variable> {
        match self.get_thing(line_number, name)? {
            ftd::interpreter2::Thing::Variable(r) => Ok(r),
            t => self.err(
                format!("Expected Variable, found: `{:?}`", t).as_str(),
                name,
                "get_variable",
                line_number,
            ),
        }
    }

    pub fn eq(&'a self, name1: &'a str, name2: &'a str) -> bool {
        let name1 = self.resolve_name(name1);
        let name2 = self.resolve_name(name2);
        name1.eq(&name2)
    }

    pub(crate) fn resolve_reference_name(
        &self,
        name: &str,
        line_number: usize,
    ) -> ftd::interpreter2::Result<String> {
        Ok(if let Some(l) = name.strip_prefix('$') {
            let d =
                ftd::interpreter2::utils::get_doc_name_and_remaining(l, self.name, line_number)?.0;
            if ftd::interpreter2::utils::get_special_variable().contains(&d.as_str()) {
                return Ok(format!("${}", l));
            }
            format!("${}", self.resolve_name(l))
        } else {
            name.to_string()
        })
    }
    pub(crate) fn resolve(
        &self,
        name: &str,
        line_number: usize,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::Value> {
        let (variable, remaining) = self.get_initial_variable(name, line_number)?;
        let value = variable.value.resolve(self, line_number)?.inner().ok_or(
            ftd::interpreter2::Error::ParseError {
                message: format!(
                    "Expected value for variable {} in line number {}",
                    variable.name, variable.line_number
                ),
                doc_id: self.name.to_string(),
                line_number,
            },
        )?;
        if let Some(remaining) = remaining {
            return resolve_(remaining.as_str(), &value, line_number, self);
        }
        return Ok(value);

        fn resolve_(
            name: &str,
            value: &ftd::interpreter2::Value,
            line_number: usize,
            doc: &ftd::interpreter2::TDoc,
        ) -> ftd::interpreter2::Result<ftd::interpreter2::Value> {
            match value {
                ftd::interpreter2::Value::Record {
                    name: rec_name,
                    fields,
                } => {
                    let (p1, p2) = ftd::interpreter2::utils::split_at(name, ".");
                    let field = fields
                        .get(p1.as_str())
                        .ok_or(ftd::interpreter2::Error::ParseError {
                            message: format!("Can't find field `{}` in record `{}`", p1, rec_name),
                            doc_id: doc.name.to_string(),
                            line_number,
                        })?
                        .clone()
                        .resolve(doc, line_number)?;
                    if let Some(p2) = p2 {
                        return resolve_(p2.as_str(), &field, line_number, doc);
                    }
                    Ok(field)
                }
                t => ftd::interpreter2::utils::e2(
                    format!("Expected record found `{:?}`", t).as_str(),
                    doc.name,
                    line_number,
                ),
            }
        }
    }

    pub fn set_value(
        &'a self,
        name: &'a str,
        value: ftd::interpreter2::PropertyValue,
        line_number: usize,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::Variable> {
        let (mut variable, mut remaining) = self.get_initial_variable(name, line_number)?;

        if !variable.mutable {
            return ftd::interpreter2::utils::e2(
                format!(
                    "The variable declaration `{}` is not mutable in line number {}",
                    variable.name, variable.line_number
                )
                .as_str(),
                self.name,
                line_number,
            );
        }

        if let Some((var, rem)) =
            find_variable_reference(&variable.value, remaining.clone(), self, line_number)?
        {
            variable = var;
            remaining = rem;
        }

        set_value_(&mut variable, value, remaining, self, line_number)?;

        return Ok(variable.clone());

        fn find_variable_reference(
            value: &ftd::interpreter2::PropertyValue,
            name: Option<String>,
            doc: &ftd::interpreter2::TDoc,
            line_number: usize,
        ) -> ftd::interpreter2::Result<Option<(ftd::interpreter2::Variable, Option<String>)>>
        {
            let mut variable = None;
            let mut remaining = name;
            let mut value = value.clone();
            while let Some(reference) = value.reference_name() {
                let (var, rem) = doc.get_initial_variable(reference, line_number)?;
                value = var.value.clone();
                variable = Some(var);
                remaining = if let Some(remaining) = remaining {
                    Some(
                        rem.map(|v| format!("{}.{}", v, remaining))
                            .unwrap_or(remaining),
                    )
                } else {
                    rem
                };
            }

            if let ftd::interpreter2::PropertyValue::Clone { .. } = value {
                return Ok(variable.map(|v| (v, remaining)));
            }

            if let Some(ref remaining) = remaining {
                let (p1, p2) = ftd::interpreter2::utils::split_at(remaining, ".");
                let value = value.value(doc.name, line_number)?.inner().ok_or(
                    ftd::interpreter2::Error::ParseError {
                        message: format!(
                            "Value expected found null, `{:?}` in line number {}",
                            value, line_number
                        ),
                        doc_id: doc.name.to_string(),
                        line_number,
                    },
                )?;

                match value {
                    ftd::interpreter2::Value::Record {
                        name: rec_name,
                        fields,
                    } => {
                        let field_value = fields
                            .get(p1.as_str())
                            .ok_or(ftd::interpreter2::Error::ParseError {
                                message: format!(
                                    "Expected field {} in record `{}` in line number {}",
                                    p1, rec_name, line_number
                                ),
                                doc_id: doc.name.to_string(),
                                line_number,
                            })?
                            .to_owned();
                        if let Some(variable) =
                            find_variable_reference(&field_value, p2, doc, line_number)?
                        {
                            return Ok(Some(variable));
                        }
                    }
                    t => {
                        return ftd::interpreter2::utils::e2(
                            format!(
                                "Expected record, found `{:?}` in line number {}",
                                t, line_number
                            )
                            .as_str(),
                            doc.name,
                            line_number,
                        )
                    }
                }
            }

            Ok(variable.map(|v| (v, remaining)))
        }

        fn set_value_(
            variable: &mut ftd::interpreter2::Variable,
            value: ftd::interpreter2::PropertyValue,
            remaining: Option<String>,
            doc: &ftd::interpreter2::TDoc,
            line_number: usize,
        ) -> ftd::interpreter2::Result<()> {
            change_value(&mut variable.value, value, remaining, doc, line_number)?;
            Ok(())
        }

        fn change_value(
            value: &mut ftd::interpreter2::PropertyValue,
            set: ftd::interpreter2::PropertyValue,
            remaining: Option<String>,
            doc: &ftd::interpreter2::TDoc,
            line_number: usize,
        ) -> ftd::interpreter2::Result<()> {
            if let Some(remaining) = remaining {
                let (p1, p2) = ftd::interpreter2::utils::split_at(remaining.as_str(), ".");
                match value {
                    ftd::interpreter2::PropertyValue::Value { value, .. } => match value {
                        ftd::interpreter2::Value::Record { name, fields } => {
                            let field = fields.get_mut(p1.as_str()).ok_or(
                                ftd::interpreter2::Error::ParseError {
                                    message: format!(
                                        "Can't find field `{}` in record `{}`",
                                        p1, name
                                    ),
                                    doc_id: doc.name.to_string(),
                                    line_number,
                                },
                            )?;
                            change_value(field, set, p2, doc, line_number)?;
                        }
                        t => {
                            return ftd::interpreter2::utils::e2(
                                format!("Expected record, found `{:?}`", t).as_str(),
                                doc.name,
                                line_number,
                            )
                        }
                    },
                    ftd::interpreter2::PropertyValue::Reference { name, .. }
                    | ftd::interpreter2::PropertyValue::Clone { name, .. } => {
                        let resolved_value = doc.resolve(name, line_number)?;
                        *value = ftd::interpreter2::PropertyValue::Value {
                            value: resolved_value,
                            line_number,
                        };
                        change_value(value, set, Some(remaining), doc, line_number)?;
                    }
                }
            } else {
                assert_eq!(value.kind(), set.kind());
                *value = set;
            }
            Ok(())
        }
    }

    pub fn get_kind(
        &'a self,
        name: &'a str,
        line_number: usize,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::KindData> {
        let name = if let Some(name) = name.strip_prefix('$') {
            name
        } else {
            name
        };

        let (initial_thing, remaining) = self.get_initial_thing(name, line_number)?;

        let initial_kind = match initial_thing {
            ftd::interpreter2::Thing::Record(r) => ftd::interpreter2::KindData {
                kind: ftd::interpreter2::Kind::Record { name: r.name },
                caption: true,
                body: true,
            },
            ftd::interpreter2::Thing::Variable(v) => v.kind,
            ftd::interpreter2::Thing::Component(c) => ftd::interpreter2::KindData {
                kind: ftd::interpreter2::Kind::UI { name: Some(c.name) },
                caption: true,
                body: true,
            },
        };

        if let Some(remaining) = remaining {
            return get_kind_(initial_kind.kind, remaining.as_str(), self, line_number);
        }

        return Ok(initial_kind);

        fn get_kind_(
            kind: ftd::interpreter2::Kind,
            name: &str,
            doc: &ftd::interpreter2::TDoc,
            line_number: usize,
        ) -> ftd::interpreter2::Result<ftd::interpreter2::KindData> {
            let (v, remaining) = name
                .split_once('.')
                .map(|(v, n)| (v, Some(n)))
                .unwrap_or((name, None));

            match kind {
                ftd::interpreter2::Kind::Record { name: rec_name } => {
                    let record = doc.get_record(line_number, rec_name.as_str())?;
                    let field_kind = record.get_field(v, doc.name, line_number)?.kind.to_owned();
                    if let Some(remaining) = remaining {
                        get_kind_(field_kind.kind, remaining, doc, line_number)
                    } else {
                        Ok(field_kind)
                    }
                }
                t => ftd::interpreter2::utils::e2(
                    format!("Expected Record field `{}`, found: `{:?}`", name, t),
                    doc.name,
                    line_number,
                ),
            }
        }
    }

    pub fn get_thing(
        &'a self,
        line_number: usize,
        name: &'a str,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::Thing> {
        let name = if let Some(name) = name.strip_prefix('$') {
            name
        } else {
            name
        };

        let (initial_thing, _remaining) = self.get_initial_thing(name, line_number)?;

        Ok(initial_thing)
    }

    pub fn get_initial_variable(
        &'a self,
        name: &'a str,
        line_number: usize,
    ) -> ftd::interpreter2::Result<(ftd::interpreter2::Variable, Option<String>)> {
        let (initial_thing, remaining) = self.get_initial_thing(name, line_number)?;
        Ok((initial_thing.variable(self.name, line_number)?, remaining))
    }

    pub fn get_initial_thing(
        &'a self,
        name: &'a str,
        line_number: usize,
    ) -> ftd::interpreter2::Result<(ftd::interpreter2::Thing, Option<String>)> {
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
                None => self.err("not found", name, "get_thing", line_number),
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
            doc: &ftd::interpreter2::TDoc,
            root_name: Option<&str>,
            doc_name: &str,
            name: &str,
        ) -> Option<(ftd::interpreter2::Thing, Option<String>)> {
            let (name, remaining_value) = if let Some((v, remaining_value)) = name.split_once('.') {
                (v, Some(remaining_value.to_string()))
            } else {
                (name, None)
            };

            match doc
                .bag
                .get(format!("{}#{}", doc_name, name).as_str())
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

    fn err<T, T2: std::fmt::Debug>(
        &self,
        msg: &str,
        ctx: T2,
        f: &str,
        line_number: usize,
    ) -> ftd::interpreter2::Result<T> {
        ftd::interpreter2::utils::e2(
            format!("{}: {} ({:?}), f: {}", self.name, msg, ctx, f),
            self.name,
            line_number,
        )
    }
}
