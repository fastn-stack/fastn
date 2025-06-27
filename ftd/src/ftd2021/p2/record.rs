#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Record {
    pub name: String,
    pub fields: ftd::Map<ftd::ftd2021::p2::Kind>,
    pub instances: ftd::Map<Vec<Invocation>>,
    pub order: Vec<String>,
}

pub(crate) type Invocation = ftd::Map<ftd::PropertyValue>;

impl Record {
    pub fn variant_name(&self) -> Option<&str> {
        self.name.split_once('.').map(|(_, r)| r)
    }

    pub fn fields(
        &self,
        p1: &ftd::ftd2021::p1::Section,
        doc: &ftd::ftd2021::p2::TDoc,
    ) -> ftd::ftd2021::p1::Result<ftd::Map<ftd::PropertyValue>> {
        let mut fields: ftd::Map<ftd::PropertyValue> = Default::default();
        self.assert_no_extra_fields(doc.name, &p1.header, &p1.caption, &p1.body)?;
        for (name, kind) in self.fields.iter() {
            let subsections = p1.sub_sections_by_name(name);
            let value = match (
                p1.sub_section_by_name(name, doc.name.to_string()),
                kind.inner(),
            ) {
                (Ok(v), ftd::ftd2021::p2::Kind::String { .. }) => ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: v.body(doc.name)?,
                        source: ftd::TextSource::Body,
                    },
                },
                (Ok(v), ftd::ftd2021::p2::Kind::Record { name, .. }) => {
                    let record = doc.get_record(p1.line_number, name.as_str())?;
                    ftd::PropertyValue::Value {
                        value: ftd::Value::Record {
                            name: doc.resolve_name(p1.line_number, record.name.as_str())?,
                            fields: record.fields_from_sub_section(v, doc)?,
                        },
                    }
                }
                (
                    Err(ftd::ftd2021::p1::Error::NotFound { .. }),
                    ftd::ftd2021::p2::Kind::List {
                        kind: list_kind, ..
                    },
                ) => match list_kind.as_ref() {
                    ftd::ftd2021::p2::Kind::OrType {
                        name: or_type_name, ..
                    }
                    | ftd::ftd2021::p2::Kind::OrTypeWithVariant {
                        name: or_type_name, ..
                    } => {
                        let e = doc.get_or_type(p1.line_number, or_type_name)?;
                        let mut values: Vec<ftd::PropertyValue> = vec![];
                        for s in p1.sub_sections.0.iter() {
                            if s.is_commented {
                                continue;
                            }
                            for v in e.variants.iter() {
                                let variant = v.variant_name().expect("record.fields").to_string();
                                if s.name == format!("{}.{}", name, variant.as_str()) {
                                    values.push(ftd::PropertyValue::Value {
                                        value: ftd::Value::OrType {
                                            variant,
                                            name: e.name.to_string(),
                                            fields: v.fields_from_sub_section(s, doc)?,
                                        },
                                    })
                                }
                            }
                        }
                        ftd::PropertyValue::Value {
                            value: ftd::Value::List {
                                kind: list_kind.inner().to_owned(),
                                data: values,
                            },
                        }
                    }
                    ftd::ftd2021::p2::Kind::Record { .. } => {
                        let mut list = ftd::Value::List {
                            kind: list_kind.inner().to_owned(),
                            data: vec![],
                        };
                        for (i, k, v) in p1.header.0.iter() {
                            if *k != *name {
                                continue;
                            }
                            list = doc.get_value(i.to_owned(), v)?;
                        }
                        ftd::PropertyValue::Value { value: list }
                    }
                    ftd::ftd2021::p2::Kind::String { .. } => {
                        let mut values: Vec<ftd::PropertyValue> = vec![];
                        for (_, k, v) in p1.header.0.iter() {
                            if *k != *name {
                                continue;
                            }
                            values.push(ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: v.to_string(),
                                    source: ftd::TextSource::Header,
                                },
                            });
                        }
                        ftd::PropertyValue::Value {
                            value: ftd::Value::List {
                                kind: list_kind.inner().to_owned(),
                                data: values,
                            },
                        }
                    }
                    ftd::ftd2021::p2::Kind::Integer { .. } => {
                        return ftd::ftd2021::p2::utils::e2(
                            "unexpected integer",
                            doc.name,
                            p1.line_number,
                        );
                    }
                    t => {
                        return ftd::ftd2021::p2::utils::e2(
                            format!("not yet implemented: {t:?}"),
                            doc.name,
                            p1.line_number,
                        );
                    }
                },
                (
                    _,
                    ftd::ftd2021::p2::Kind::List {
                        kind: list_kind, ..
                    },
                ) if !subsections.is_empty() => match list_kind.as_ref() {
                    ftd::ftd2021::p2::Kind::OrType {
                        name: or_type_name, ..
                    }
                    | ftd::ftd2021::p2::Kind::OrTypeWithVariant {
                        name: or_type_name, ..
                    } => {
                        let e = doc.get_or_type(p1.line_number, or_type_name)?;
                        let mut values: Vec<ftd::PropertyValue> = vec![];
                        for s in p1.sub_sections.0.iter() {
                            for v in e.variants.iter() {
                                let variant = v.variant_name().expect("record.fields").to_string();
                                if s.name == format!("{}.{}", name, variant.as_str()) {
                                    values.push(ftd::PropertyValue::Value {
                                        value: ftd::Value::OrType {
                                            variant,
                                            name: e.name.to_string(),
                                            fields: v.fields_from_sub_section(s, doc)?,
                                        },
                                    })
                                }
                            }
                        }
                        ftd::PropertyValue::Value {
                            value: ftd::Value::List {
                                kind: list_kind.inner().to_owned(),
                                data: values,
                            },
                        }
                    }
                    ftd::ftd2021::p2::Kind::Record { name, .. } => {
                        let mut list = vec![];
                        for v in subsections {
                            let record = doc.get_record(p1.line_number, name.as_str())?;
                            list.push(ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: doc.resolve_name(p1.line_number, record.name.as_str())?,
                                    fields: record.fields_from_sub_section(v, doc)?,
                                },
                            });
                        }
                        ftd::PropertyValue::Value {
                            value: ftd::Value::List {
                                kind: list_kind.inner().to_owned(),
                                data: list,
                            },
                        }
                    }
                    ftd::ftd2021::p2::Kind::String { .. } => {
                        let mut list = vec![];
                        for v in subsections {
                            let (text, from_caption) = v.body_or_caption(doc.name)?;
                            list.push(ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text,
                                    source: match from_caption {
                                        true => ftd::TextSource::Caption,
                                        false => ftd::TextSource::Body,
                                    },
                                },
                            });
                        }
                        ftd::PropertyValue::Value {
                            value: ftd::Value::List {
                                kind: list_kind.inner().to_owned(),
                                data: list,
                            },
                        }
                    }
                    ftd::ftd2021::p2::Kind::Integer { .. } => {
                        return ftd::ftd2021::p2::utils::e2(
                            "unexpected integer",
                            doc.name,
                            p1.line_number,
                        );
                    }
                    t => {
                        return ftd::ftd2021::p2::utils::e2(
                            format!("not yet implemented: {t:?}"),
                            doc.name,
                            p1.line_number,
                        );
                    }
                },
                (Ok(_), _) => {
                    return ftd::ftd2021::p2::utils::e2(
                        format!("'{kind:?}' ('{name}') can not be a sub-section"),
                        doc.name,
                        p1.line_number,
                    );
                }
                (Err(ftd::ftd2021::p1::Error::NotFound { .. }), _) => {
                    kind.read_section(p1.line_number, &p1.header, &p1.caption, &p1.body, name, doc)?
                }
                (
                    Err(ftd::ftd2021::p1::Error::MoreThanOneSubSections { .. }),
                    ftd::ftd2021::p2::Kind::List {
                        kind: list_kind, ..
                    },
                ) => {
                    let mut values: Vec<ftd::PropertyValue> = vec![];
                    for s in p1.sub_sections.0.iter() {
                        if s.name != *name || s.is_commented {
                            continue;
                        }
                        let v = match list_kind.inner().string_any() {
                            ftd::ftd2021::p2::Kind::Record { name, .. } => {
                                let record = doc.get_record(p1.line_number, name.as_str())?;
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::Record {
                                        name: doc
                                            .resolve_name(s.line_number, record.name.as_str())?,
                                        fields: record.fields_from_sub_section(s, doc)?,
                                    },
                                }
                            }
                            k => k.read_section(
                                s.line_number,
                                &s.header,
                                &s.caption,
                                &s.body,
                                s.name.as_str(),
                                doc,
                            )?,
                        };
                        values.push(v);
                    }
                    ftd::PropertyValue::Value {
                        value: ftd::Value::List {
                            kind: list_kind.inner().to_owned(),
                            data: values,
                        },
                    }
                }
                (Err(e), _) => return Err(e),
            };
            fields.insert(name.to_string(), value);
        }
        Ok(fields)
    }

    pub fn add_instance(
        &mut self,
        p1: &ftd::ftd2021::p1::Section,
        doc: &ftd::ftd2021::p2::TDoc,
    ) -> ftd::ftd2021::p1::Result<()> {
        let fields = self.fields(p1, doc)?;
        self.instances
            .entry(doc.name.to_string())
            .or_default()
            .push(fields);
        Ok(())
    }

    pub fn create(
        &self,
        p1: &ftd::ftd2021::p1::Section,
        doc: &ftd::ftd2021::p2::TDoc,
    ) -> ftd::ftd2021::p1::Result<ftd::PropertyValue> {
        // todo: check if the its reference to other variable
        Ok(ftd::PropertyValue::Value {
            value: ftd::Value::Record {
                name: doc.resolve_name(p1.line_number, self.name.as_str())?,
                fields: self.fields(p1, doc)?,
            },
        })
    }

    pub fn fields_from_sub_section(
        &self,
        p1: &ftd::ftd2021::p1::SubSection,
        doc: &ftd::ftd2021::p2::TDoc,
    ) -> ftd::ftd2021::p1::Result<ftd::Map<ftd::PropertyValue>> {
        let mut fields: ftd::Map<ftd::PropertyValue> = Default::default();
        self.assert_no_extra_fields(doc.name, &p1.header, &p1.caption, &p1.body)?;
        for (name, kind) in self.fields.iter() {
            fields.insert(
                name.to_string(),
                kind.read_section(p1.line_number, &p1.header, &p1.caption, &p1.body, name, doc)?,
            );
        }
        Ok(fields)
    }

    fn assert_no_extra_fields(
        &self,
        doc_id: &str,
        p1: &ftd::ftd2021::p1::Header,
        _caption: &Option<String>,
        _body: &Option<(usize, String)>,
    ) -> ftd::ftd2021::p1::Result<()> {
        // TODO: handle caption
        // TODO: handle body
        for (i, k, _) in p1.0.iter() {
            if !self.fields.contains_key(k) && k != "type" && k != "$processor$" {
                return ftd::ftd2021::p2::utils::e2(
                    format!(
                        "unknown key passed: '{}' to '{}', allowed: {:?}",
                        k,
                        self.name,
                        self.fields.keys()
                    ),
                    doc_id,
                    i.to_owned(),
                );
            }
        }
        Ok(())
    }

    pub fn from_p1(
        p1_name: &str,
        p1_header: &ftd::ftd2021::p1::Header,
        doc: &ftd::ftd2021::p2::TDoc,
        line_number: usize,
    ) -> ftd::ftd2021::p1::Result<Self> {
        let name = ftd::ftd2021::p2::utils::get_name("record", p1_name, doc.name)?;
        let full_name = doc.format_name(name);
        let mut fields = ftd::Map::new();
        let mut order = vec![];
        let object_kind = (
            name,
            ftd::ftd2021::p2::Kind::Record {
                name: full_name.clone(),
                default: None,
                is_reference: false,
            },
        );
        for (i, k, v) in p1_header.0.iter() {
            let var_data = match ftd::ftd2021::variable::VariableData::get_name_kind(
                k,
                doc,
                i.to_owned(),
                vec![].as_slice(),
            ) {
                Ok(v) => v,
                _ => continue,
            };
            let v = normalise_value(v)?;
            validate_key(k)?;
            let v = if v.is_empty() {
                None
            } else {
                Some(v.to_string())
            };
            fields.insert(
                var_data.name.to_string(),
                ftd::ftd2021::p2::Kind::for_variable(
                    i.to_owned(),
                    k,
                    v,
                    doc,
                    Some(object_kind.clone()),
                    &Default::default(),
                )?,
            );
            order.push(var_data.name.to_string());
        }
        assert_fields_valid(line_number, &fields, doc.name)?;
        return Ok(Record {
            name: full_name,
            fields,
            instances: Default::default(),
            order,
        });

        fn normalise_value(s: &str) -> ftd::ftd2021::p1::Result<String> {
            // TODO: normalise spaces in v
            Ok(s.to_string())
        }

        fn validate_key(_k: &str) -> ftd::ftd2021::p1::Result<()> {
            // TODO: ensure k in valid (only alphanumeric, _, and -)
            Ok(())
        }
    }
}

fn assert_fields_valid(
    line_number: usize,
    fields: &ftd::Map<ftd::ftd2021::p2::Kind>,
    doc_id: &str,
) -> ftd::ftd2021::p1::Result<()> {
    let mut caption_field: Option<String> = None;
    let mut body_field: Option<String> = None;
    for (name, kind) in fields.iter() {
        if let ftd::ftd2021::p2::Kind::String { caption, body, .. } = kind {
            if *caption {
                match &caption_field {
                    Some(c) => {
                        return ftd::ftd2021::p2::utils::e2(
                            format!("both {name} and {c} are caption fields"),
                            doc_id,
                            line_number,
                        );
                    }
                    None => caption_field = Some(name.to_string()),
                }
            }
            if *body {
                match &body_field {
                    Some(c) => {
                        return ftd::ftd2021::p2::utils::e2(
                            format!("both {name} and {c} are body fields"),
                            doc_id,
                            line_number,
                        );
                    }
                    None => body_field = Some(name.to_string()),
                }
            }
        }
    }
    Ok(())
}
