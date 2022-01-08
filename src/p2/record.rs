#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Record {
    pub name: String,
    pub fields: std::collections::BTreeMap<String, ftd::p2::Kind>,
    pub instances: std::collections::BTreeMap<String, Vec<Invocation>>,
    pub order: Vec<String>,
}

type Invocation = std::collections::BTreeMap<String, ftd::PropertyValue>;

impl Record {
    pub fn variant_name(&self) -> Option<&str> {
        self.name.split_once(".").map(|(_, r)| r)
    }

    pub fn fields(
        &self,
        p1: &ftd::p1::Section,
        doc: &ftd::p2::TDoc,
    ) -> ftd::p1::Result<std::collections::BTreeMap<String, ftd::PropertyValue>> {
        let mut fields: std::collections::BTreeMap<String, ftd::PropertyValue> = Default::default();
        self.assert_no_extra_fields(doc.name, &p1.header, &p1.caption, &p1.body)?;
        for (name, kind) in self.fields.iter() {
            let value = match (
                p1.sub_section_by_name(name, doc.name.to_string()),
                kind.inner(),
            ) {
                (Ok(v), ftd::p2::Kind::String { .. }) => ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: v.body(doc.name)?,
                        source: ftd::TextSource::Body,
                    },
                },
                (Ok(v), ftd::p2::Kind::Record { name, .. }) => {
                    let record = doc.get_record(p1.line_number, name)?;
                    ftd::PropertyValue::Value {
                        value: ftd::Value::Record {
                            name: doc.resolve_name(p1.line_number, record.name.as_str())?,
                            fields: record.fields_from_sub_section(v, doc)?,
                        },
                    }
                }
                (Ok(_), _) => {
                    return ftd::e2(
                        format!("'{:?}' ('{}') can not be a sub-section", kind, name),
                        doc.name,
                        p1.line_number,
                    );
                }
                (
                    Err(ftd::p1::Error::NotFound { .. }),
                    ftd::p2::Kind::List {
                        kind: list_kind, ..
                    },
                ) => match list_kind.as_ref() {
                    ftd::p2::Kind::OrType { name: or_type_name } => {
                        let e = doc.get_or_type(p1.line_number, or_type_name)?;
                        let mut values: Vec<ftd::Value> = vec![];
                        for s in p1.sub_sections.0.iter() {
                            if s.is_commented {
                                continue;
                            }
                            for v in e.variants.iter() {
                                let variant = v.variant_name().expect("record.fields").to_string();
                                if s.name == format!("{}.{}", name, variant.as_str()) {
                                    values.push(ftd::Value::OrType {
                                        variant,
                                        name: e.name.to_string(),
                                        fields: v.fields_from_sub_section(s, doc)?,
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
                    ftd::p2::Kind::Record { .. } => {
                        let mut list = ftd::Value::List {
                            kind: list_kind.inner().to_owned(),
                            data: vec![],
                        };
                        for (i, k, v) in p1.header.0.iter() {
                            if *k != *name || k.starts_with('/') {
                                continue;
                            }
                            list = doc.get_value(i.to_owned(), v)?;
                        }
                        ftd::PropertyValue::Value { value: list }
                    }
                    ftd::p2::Kind::String { .. } => {
                        let mut values: Vec<ftd::Value> = vec![];
                        for (_, k, v) in p1.header.0.iter() {
                            if *k != *name || k.starts_with('/') {
                                continue;
                            }
                            values.push(ftd::Value::String {
                                text: v.to_string(),
                                source: ftd::TextSource::Header,
                            });
                        }
                        ftd::PropertyValue::Value {
                            value: ftd::Value::List {
                                kind: list_kind.inner().to_owned(),
                                data: values,
                            },
                        }
                    }
                    ftd::p2::Kind::Integer { .. } => {
                        return ftd::e2("unexpected integer", doc.name, p1.line_number);
                    }
                    t => {
                        return ftd::e2(
                            format!("not yet implemented: {:?}", t),
                            doc.name,
                            p1.line_number,
                        )
                    }
                },
                (Err(ftd::p1::Error::NotFound { .. }), _) => kind.read_section(
                    p1.line_number,
                    &p1.header,
                    &p1.caption,
                    &p1.body_without_comment(),
                    name,
                    doc,
                )?,
                (
                    Err(ftd::p1::Error::MoreThanOneSubSections { .. }),
                    ftd::p2::Kind::List {
                        kind: list_kind, ..
                    },
                ) => {
                    let mut values: Vec<ftd::Value> = vec![];
                    for s in p1.sub_sections.0.iter() {
                        if s.name != *name || s.is_commented {
                            continue;
                        }
                        let v = match list_kind.inner().string_any() {
                            ftd::p2::Kind::Record { name, .. } => {
                                let record = doc.get_record(p1.line_number, name.as_str())?;
                                ftd::Value::Record {
                                    name: doc.resolve_name(s.line_number, record.name.as_str())?,
                                    fields: record.fields_from_sub_section(s, doc)?,
                                }
                            }
                            k => {
                                match k.read_section(
                                    s.line_number,
                                    &s.header,
                                    &s.caption,
                                    &s.body_without_comment(),
                                    s.name.as_str(),
                                    doc,
                                )? {
                                    ftd::PropertyValue::Value { value: v } => v,
                                    _ => unimplemented!(),
                                }
                            }
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
        p1: &ftd::p1::Section,
        doc: &ftd::p2::TDoc,
    ) -> ftd::p1::Result<()> {
        let fields = self.fields(p1, doc)?;
        self.instances
            .entry(doc.name.to_string())
            .or_default()
            .push(fields);
        Ok(())
    }

    pub fn create(
        &self,
        p1: &ftd::p1::Section,
        doc: &ftd::p2::TDoc,
    ) -> ftd::p1::Result<ftd::Value> {
        Ok(ftd::Value::Record {
            name: doc.resolve_name(p1.line_number, self.name.as_str())?,
            fields: self.fields(p1, doc)?,
        })
    }

    pub fn fields_from_sub_section(
        &self,
        p1: &ftd::p1::SubSection,
        doc: &ftd::p2::TDoc,
    ) -> ftd::p1::Result<std::collections::BTreeMap<String, ftd::PropertyValue>> {
        let mut fields: std::collections::BTreeMap<String, ftd::PropertyValue> = Default::default();
        self.assert_no_extra_fields(doc.name, &p1.header, &p1.caption, &p1.body)?;
        for (name, kind) in self.fields.iter() {
            fields.insert(
                name.to_string(),
                kind.read_section(
                    p1.line_number,
                    &p1.header,
                    &p1.caption,
                    &p1.body_without_comment(),
                    name,
                    doc,
                )?,
            );
        }
        Ok(fields)
    }

    fn assert_no_extra_fields(
        &self,
        doc_id: &str,
        p1: &ftd::p1::Header,
        _caption: &Option<String>,
        _body: &Option<(usize, String)>,
    ) -> ftd::p1::Result<()> {
        // TODO: handle caption
        // TODO: handle body
        for (i, k, _) in p1.0.iter() {
            if k.starts_with('/') {
                continue;
            }

            if !self.fields.contains_key(k) && k != "type" && k != "$processor$" {
                return ftd::e2(
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
        p1_header: &ftd::p1::Header,
        doc: &ftd::p2::TDoc,
        line_number: usize,
    ) -> ftd::p1::Result<Self> {
        let name = ftd::get_name("record", p1_name, doc.name)?;
        let full_name = doc.format_name(name);
        let mut fields = std::collections::BTreeMap::new();
        let mut order = vec![];
        let object_kind = (
            name,
            ftd::p2::Kind::Record {
                name: full_name.clone(),
                default: None,
            },
        );
        for (i, k, v) in p1_header.0.iter() {
            if k.starts_with('/') {
                continue;
            }
            let var_data = match ftd::variable::VariableData::get_name_kind(
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
                ftd::p2::Kind::for_variable(i.to_owned(), k, v, doc, Some(object_kind.clone()))?,
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

        fn normalise_value(s: &str) -> ftd::p1::Result<String> {
            // TODO: normalise spaces in v
            Ok(s.to_string())
        }

        fn validate_key(_k: &str) -> ftd::p1::Result<()> {
            // TODO: ensure k in valid (only alphanumeric, _, and -)
            Ok(())
        }
    }
}

fn assert_fields_valid(
    line_number: usize,
    fields: &std::collections::BTreeMap<String, ftd::p2::Kind>,
    doc_id: &str,
) -> ftd::p1::Result<()> {
    let mut caption_field: Option<String> = None;
    let mut body_field: Option<String> = None;
    for (name, kind) in fields.iter() {
        if let ftd::p2::Kind::String { caption, body, .. } = kind {
            if *caption {
                match &caption_field {
                    Some(c) => {
                        return ftd::e2(
                            format!("both {} and {} are caption fields", name, c),
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
                        return ftd::e2(
                            format!("both {} and {} are body fields", name, c),
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

#[cfg(test)]
mod test {
    use ftd::test::*;

    #[test]
    fn record() {
        let sourabh: super::Invocation = std::array::IntoIter::new([
            (
                s("name"),
                ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "Sourabh Garg".to_string(),
                        source: ftd::TextSource::Caption,
                    },
                },
            ),
            (
                s("address"),
                ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "Ranchi".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
            ),
            (
                s("bio"),
                ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "Frontend developer at fifthtry.".to_string(),
                        source: ftd::TextSource::Body,
                    },
                },
            ),
            (
                s("age"),
                ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 28 },
                },
            ),
        ])
        .collect();

        let mut bag = ftd::p2::interpreter::default_bag();
        bag.insert(
            "foo/bar#abrar".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: "abrar".to_string(),
                value: ftd::Value::Record {
                    name: "foo/bar#person".to_string(),
                    fields: abrar(),
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#person".to_string(),
            ftd::p2::Thing::Record(ftd::p2::Record {
                name: "foo/bar#person".to_string(),
                fields: person_fields(),
                instances: std::array::IntoIter::new([(
                    s("foo/bar"),
                    vec![abrar(), sourabh.clone()],
                )])
                .collect(),
                order: vec![s("name"), s("address"), s("bio"), s("age")],
            }),
        );
        bag.insert(
            "foo/bar#x".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: "x".to_string(),
                value: ftd::Value::Integer { value: 20 },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#employee".to_string(),
            ftd::p2::Thing::Record(ftd::p2::Record {
                name: "foo/bar#employee".to_string(),
                fields: std::array::IntoIter::new([
                    (s("eid"), ftd::p2::Kind::string()),
                    (
                        s("who"),
                        ftd::p2::Kind::Record {
                            name: s("foo/bar#person"),
                            default: None,
                        },
                    ),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![s("eid"), s("who")],
            }),
        );
        bag.insert(
            "foo/bar#abrar_e".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: "abrar_e".to_string(),
                value: ftd::Value::Record {
                    name: "foo/bar#employee".to_string(),
                    fields: std::array::IntoIter::new([
                        (
                            s("eid"),
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: "E04".to_string(),
                                    source: ftd::TextSource::Header,
                                },
                            },
                        ),
                        (
                            s("who"),
                            ftd::PropertyValue::Reference {
                                name: s("foo/bar#abrar"),
                                kind: ftd::p2::Kind::Record {
                                    name: s("foo/bar#person"),
                                    default: None,
                                },
                            },
                        ),
                    ])
                    .collect(),
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#sourabh".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: "sourabh".to_string(),
                value: ftd::Value::Record {
                    name: "foo/bar#employee".to_string(),
                    fields: std::array::IntoIter::new([
                        (
                            s("eid"),
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: "E05".to_string(),
                                    source: ftd::TextSource::Body,
                                },
                            },
                        ),
                        (
                            s("who"),
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: "foo/bar#person".to_string(),
                                    fields: sourabh,
                                },
                            },
                        ),
                    ])
                    .collect(),
                },
                conditions: vec![],
            }),
        );

        p!(
            "
            -- record person:
            caption name:
            string address:
            body bio:
            integer age:

            -- integer x: 10

            -- person: Abrar Khan2
            address: Bihar2
            age: $x

            Software developer working at fifthtry2.

            -- person: Sourabh Garg
            address: Ranchi
            age: 28

            Frontend developer at fifthtry.

            -- person abrar: Abrar Khan
            address: Bihar
            age: $x

            Software developer working at fifthtry.

            -- record employee:
            string eid:
            person who:

            -- employee abrar_e:
            eid: E04
            who: $abrar

            -- employee sourabh:

            --- eid:

            E05

            --- who: Sourabh Garg
            address: Ranchi
            age: 28

            Frontend developer at fifthtry.

            -- integer x: 20

            -- abrar: Abrar Khan2
            address: Bihar2
            age: $x

            Software developer working at fifthtry2.
            ",
            (bag, ftd::p2::interpreter::default_column()),
        );
    }

    #[test]
    fn list() {
        let b = |source: ftd::TextSource| {
            let mut bag = default_bag();

            bag.insert(
                "foo/bar#person".to_string(),
                ftd::p2::Thing::Record(ftd::p2::Record {
                    name: "foo/bar#person".to_string(),
                    fields: std::array::IntoIter::new([
                        (s("name"), ftd::p2::Kind::caption()),
                        (
                            s("friends"),
                            ftd::p2::Kind::List {
                                kind: Box::new(ftd::p2::Kind::string()),
                                default: None,
                            },
                        ),
                    ])
                    .collect(),
                    instances: Default::default(),
                    order: vec![s("name"), s("friends")],
                }),
            );

            bag.insert(
                "foo/bar#abrar".to_string(),
                ftd::p2::Thing::Variable(ftd::Variable {
                    name: "abrar".to_string(),
                    value: ftd::Value::Record {
                        name: "foo/bar#person".to_string(),
                        fields: std::array::IntoIter::new([
                            (
                                s("name"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "Abrar Khan".to_string(),
                                        source: ftd::TextSource::Caption,
                                    },
                                },
                            ),
                            (
                                s("friends"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::List {
                                        kind: ftd::p2::Kind::string(),
                                        data: vec![
                                            ftd::Value::String {
                                                text: "Deepak Angrula".to_string(),
                                                source: source.clone(),
                                            },
                                            ftd::Value::String {
                                                text: "Amit Upadhyay".to_string(),
                                                source: source.clone(),
                                            },
                                            ftd::Value::String {
                                                text: "Saurabh Garg".to_string(),
                                                source,
                                            },
                                        ],
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                    conditions: vec![],
                }),
            );
            bag
        };

        p!(
            "
            -- record person:
            caption name:
            string list friends:

            -- person abrar: Abrar Khan
            friends: Deepak Angrula
            friends: Amit Upadhyay
            friends: Saurabh Garg
            ",
            (b(ftd::TextSource::Header), default_column()),
        );

        p!(
            "
            -- record person:
            caption name:
            string list friends:

            -- person abrar: Abrar Khan

            --- friends: Deepak Angrula
            --- friends: Amit Upadhyay
            --- friends: Saurabh Garg
            ",
            (b(ftd::TextSource::Caption), default_column()),
        );
    }

    #[test]
    fn list_of_records() {
        let mut bag = default_bag();

        bag.insert(
            s("foo/bar#point"),
            ftd::p2::Thing::Record(ftd::p2::Record {
                name: s("foo/bar#point"),
                fields: std::array::IntoIter::new([
                    (s("x"), ftd::p2::Kind::integer()),
                    (s("y"), ftd::p2::Kind::integer()),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![s("x"), s("y")],
            }),
        );

        bag.insert(
            "foo/bar#person".to_string(),
            ftd::p2::Thing::Record(ftd::p2::Record {
                name: s("foo/bar#person"),
                fields: std::array::IntoIter::new([
                    (s("name"), ftd::p2::Kind::caption()),
                    (
                        s("points"),
                        ftd::p2::Kind::List {
                            kind: Box::new(ftd::p2::Kind::Record {
                                name: s("foo/bar#point"),
                                default: None,
                            }),
                            default: None,
                        },
                    ),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![s("name"), s("points")],
            }),
        );

        bag.insert(
            "foo/bar#abrar".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: "abrar".to_string(),
                value: ftd::Value::Record {
                    name: "foo/bar#person".to_string(),
                    fields: std::array::IntoIter::new([
                        (
                            s("name"),
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: "Abrar Khan".to_string(),
                                    source: ftd::TextSource::Caption,
                                },
                            },
                        ),
                        (
                            s("points"),
                            ftd::PropertyValue::Value {
                                value: ftd::Value::List {
                                    kind: ftd::p2::Kind::Record {
                                        name: s("foo/bar#point"),
                                        default: None,
                                    },
                                    data: vec![
                                        ftd::Value::Record {
                                            name: "foo/bar#point".to_string(),
                                            fields: std::array::IntoIter::new([
                                                (
                                                    s("x"),
                                                    ftd::PropertyValue::Value {
                                                        value: ftd::Value::Integer { value: 10 },
                                                    },
                                                ),
                                                (
                                                    s("y"),
                                                    ftd::PropertyValue::Value {
                                                        value: ftd::Value::Integer { value: 20 },
                                                    },
                                                ),
                                            ])
                                            .collect(),
                                        },
                                        ftd::Value::Record {
                                            name: "foo/bar#point".to_string(),
                                            fields: std::array::IntoIter::new([
                                                (
                                                    s("x"),
                                                    ftd::PropertyValue::Value {
                                                        value: ftd::Value::Integer { value: 0 },
                                                    },
                                                ),
                                                (
                                                    s("y"),
                                                    ftd::PropertyValue::Value {
                                                        value: ftd::Value::Integer { value: 0 },
                                                    },
                                                ),
                                            ])
                                            .collect(),
                                        },
                                        ftd::Value::Record {
                                            name: "foo/bar#point".to_string(),
                                            fields: std::array::IntoIter::new([
                                                (
                                                    s("x"),
                                                    ftd::PropertyValue::Value {
                                                        value: ftd::Value::Integer { value: 1 },
                                                    },
                                                ),
                                                (
                                                    s("y"),
                                                    ftd::PropertyValue::Value {
                                                        value: ftd::Value::Integer { value: 22 },
                                                    },
                                                ),
                                            ])
                                            .collect(),
                                        },
                                    ],
                                },
                            },
                        ),
                    ])
                    .collect(),
                },
                conditions: vec![],
            }),
        );

        p!(
            "
            -- record point:
            integer x:
            integer y:

            -- record person:
            caption name:
            point list points:

            -- person abrar: Abrar Khan

            --- points:
            x: 10
            y: 20

            --- points:
            x: 0
            y: 0

            --- points:
            x: 1
            y: 22
            ",
            (bag, default_column()),
        );
    }

    #[test]
    fn list_of_or_types() {
        let mut bag = default_bag();

        bag.insert(s("foo/bar#entity"), entity());
        bag.insert(
            s("foo/bar#sale"),
            ftd::p2::Thing::Record(ftd::p2::Record {
                name: s("foo/bar#sale"),
                fields: std::array::IntoIter::new([
                    (
                        s("party"),
                        ftd::p2::Kind::List {
                            kind: Box::new(ftd::p2::Kind::OrType {
                                name: s("foo/bar#entity"),
                            }),
                            default: None,
                        },
                    ),
                    (s("value"), ftd::p2::Kind::integer()),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![s("party"), s("value")],
            }),
        );
        bag.insert(
            s("foo/bar#jan"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("jan"),
                value: ftd::Value::Record {
                    name: s("foo/bar#sale"),
                    fields: std::array::IntoIter::new([
                        (
                            s("value"),
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Integer { value: 2000 },
                            },
                        ),
                        (
                            s("party"),
                            ftd::PropertyValue::Value {
                                value: ftd::Value::List {
                                    kind: ftd::p2::Kind::OrType {
                                        name: s("foo/bar#entity"),
                                    },
                                    data: vec![
                                        ftd::Value::OrType {
                                            name: s("foo/bar#entity"),
                                            variant: s("person"),
                                            fields: std::array::IntoIter::new([
                                                (
                                                    s("address"),
                                                    ftd::PropertyValue::Value {
                                                        value: ftd::Value::String {
                                                            text: s("123 Lane"),
                                                            source: ftd::TextSource::Header,
                                                        },
                                                    },
                                                ),
                                                (
                                                    s("bio"),
                                                    ftd::PropertyValue::Value {
                                                        value: ftd::Value::String {
                                                            text: s("Owner of Jack Russo\'s Bar"),
                                                            source: ftd::TextSource::Body,
                                                        },
                                                    },
                                                ),
                                                (
                                                    s("name"),
                                                    ftd::PropertyValue::Value {
                                                        value: ftd::Value::String {
                                                            text: s("Jack Russo"),
                                                            source: ftd::TextSource::Caption,
                                                        },
                                                    },
                                                ),
                                                (
                                                    s("age"),
                                                    ftd::PropertyValue::Value {
                                                        value: ftd::Value::Integer { value: 24 },
                                                    },
                                                ),
                                            ])
                                            .collect(),
                                        },
                                        ftd::Value::OrType {
                                            name: s("foo/bar#entity"),
                                            variant: s("company"),
                                            fields: std::array::IntoIter::new([
                                                (
                                                    s("industry"),
                                                    ftd::PropertyValue::Value {
                                                        value: ftd::Value::String {
                                                            text: s("Widgets"),
                                                            source: ftd::TextSource::Header,
                                                        },
                                                    },
                                                ),
                                                (
                                                    s("name"),
                                                    ftd::PropertyValue::Value {
                                                        value: ftd::Value::String {
                                                            text: s("Acme Inc"),
                                                            source: ftd::TextSource::Caption,
                                                        },
                                                    },
                                                ),
                                            ])
                                            .collect(),
                                        },
                                    ],
                                },
                            },
                        ),
                    ])
                    .collect(),
                },
                conditions: vec![],
            }),
        );

        p!(
            "
            -- or-type entity:

            --- person:
            caption name:
            string address:
            body bio:
            integer age:

            --- company:
            caption name:
            string industry:

            -- record sale:
            entity list party:
            integer value:

            -- sale jan:
            value: 2000

            --- party.person: Jack Russo
            address: 123 Lane
            age: 24

            Owner of Jack Russo's Bar

            --- party.company: Acme Inc
            industry: Widgets
            ",
            (bag, default_column()),
        );
    }
}
