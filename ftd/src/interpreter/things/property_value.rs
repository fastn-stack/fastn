#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    Value {
        value: ftd::interpreter::Value,
    },
    Reference {
        name: String,
        kind: ftd::interpreter::KindData,
    },
}

impl PropertyValue {
    pub(crate) fn from_p1_section(
        s: &ftd::p11::Section,
        doc_id: &str,
    ) -> ftd::interpreter::Result<PropertyValue> {
        let kind = s
            .kind
            .as_ref()
            .ok_or(ftd::interpreter::Error::InvalidKind {
                doc_id: doc_id.to_string(),
                line_number: s.line_number,
                message: format!("Kind not found for section: {}", s.name),
            })?;
        let kind_data =
            ftd::interpreter::KindData::from_p1_kind(kind.as_str(), doc_id, s.line_number)?;
        PropertyValue::from_p1_section_with_kind(s, doc_id, &kind_data)
    }

    #[allow(dead_code)]
    pub(crate) fn for_header(
        s: &ftd::p11::Section,
        doc_id: &str,
        key: &str,
    ) -> ftd::interpreter::Result<PropertyValue> {
        let header = s.headers.find_once(key, doc_id, s.line_number)?;
        let kind = header
            .get_kind()
            .ok_or(ftd::interpreter::Error::InvalidKind {
                doc_id: doc_id.to_string(),
                line_number: s.line_number,
                message: format!("Kind not found for section: {}", s.name),
            })?;
        let kind_data =
            ftd::interpreter::KindData::from_p1_kind(kind.as_str(), doc_id, s.line_number)?;
        PropertyValue::from_header_with_kind(header, doc_id, &kind_data)
    }

    pub(crate) fn for_header_with_kind(
        s: &ftd::p11::Section,
        doc_id: &str,
        key: &str,
        kind_data: &ftd::interpreter::KindData,
    ) -> ftd::interpreter::Result<PropertyValue> {
        let header = s.headers.find_once(key, doc_id, s.line_number)?;
        PropertyValue::from_header_with_kind(header, doc_id, kind_data)
    }

    pub(crate) fn from_header_with_kind(
        header: &ftd::p11::Header,
        doc_id: &str,
        kind_data: &ftd::interpreter::KindData,
    ) -> ftd::interpreter::Result<PropertyValue> {
        Ok(match header.get_value(doc_id) {
            Ok(Some(value)) if get_reference(value.as_str()).is_some() => PropertyValue::reference(
                get_reference(value.as_str()).unwrap().to_string(),
                kind_data.to_owned(),
            ),
            _ => {
                let value = Value::from_p1_header(header, doc_id, kind_data)?;
                PropertyValue::value(value)
            }
        })
    }

    pub(crate) fn from_p1_section_with_kind(
        s: &ftd::p11::Section,
        doc_id: &str,
        kind_data: &ftd::interpreter::KindData,
    ) -> ftd::interpreter::Result<PropertyValue> {
        Ok(match section_value_from_caption_or_body(s, doc_id) {
            Ok(value) if get_reference(value.as_str()).is_some() => PropertyValue::reference(
                get_reference(value.as_str()).unwrap().to_string(),
                kind_data.to_owned(),
            ),
            _ => {
                let value = Value::from_p1_section(s, doc_id, kind_data)?;
                PropertyValue::value(value)
            }
        })
    }

    pub(crate) fn reference(name: String, kind: ftd::interpreter::KindData) -> PropertyValue {
        PropertyValue::Reference { name, kind }
    }

    pub(crate) fn value(value: ftd::interpreter::Value) -> PropertyValue {
        PropertyValue::Value { value }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    String {
        text: String,
    },
    Integer {
        value: i64,
    },
    Decimal {
        value: f64,
    },
    Boolean {
        value: bool,
    },
    Object {
        values: ftd::Map<PropertyValue>,
    },
    Record {
        name: String,
        fields: ftd::Map<PropertyValue>,
    },
    OrType {
        name: String,
        variant: String,
        fields: ftd::Map<PropertyValue>,
    },
    List {
        data: Vec<PropertyValue>,
        kind: ftd::interpreter::KindData,
    },
    Optional {
        data: Box<Option<Value>>,
        kind: ftd::interpreter::KindData,
    },
    Map {
        data: ftd::Map<Value>,
        kind: ftd::p2::Kind,
    },
    // TODO: UI
    // UI {
    //     name: String,
    //     component: ftd::interpreter::Component,
    // },
}

impl Value {
    pub(crate) fn from_p1_header(
        s: &ftd::p11::Header,
        doc_id: &str,
        kind_data: &ftd::interpreter::KindData,
    ) -> ftd::interpreter::Result<Value> {
        match &kind_data.kind {
            ftd::interpreter::Kind::String
            | ftd::interpreter::Kind::Integer
            | ftd::interpreter::Kind::Decimal
            | ftd::interpreter::Kind::Boolean => {
                let value = s
                    .get_value(doc_id)?
                    .ok_or(ftd::interpreter::Error::ValueNotFound {
                        doc_id: doc_id.to_string(),
                        line_number: s.get_line_number(),
                        message: format!("Can't find value for key: `{}`", s.get_key()),
                    })?;
                Value::to_value_for_basic_kind(value.as_str(), &kind_data.kind)
            }
            ftd::interpreter::Kind::Optional { kind } => {
                let kind_data = kind
                    .to_owned()
                    .into_kind_data(kind_data.caption, kind_data.body);
                if s.is_empty() {
                    Ok(Value::Optional {
                        data: Box::new(None),
                        kind: kind_data,
                    })
                } else {
                    let value = Value::from_p1_header(s, doc_id, &kind_data)?;
                    Ok(Value::Optional {
                        data: Box::new(Some(value)),
                        kind: kind_data,
                    })
                }
            }
            ftd::interpreter::Kind::List { kind } => {
                let mut data = vec![];
                let sections = if let Ok(sections) = s.get_sections(doc_id) {
                    sections
                } else {
                    return Ok(Value::List {
                        data,
                        kind: kind_data.to_owned(),
                    });
                };
                for subsection in sections.iter() {
                    let found_kind = ftd::interpreter::KindData::from_p1_kind(
                        &subsection.name,
                        doc_id,
                        subsection.line_number,
                    )?;

                    if found_kind.kind.ne(kind) {
                        return Err(ftd::interpreter::utils::invalid_kind_error(
                            format!(
                                "List kind mismatch, expected kind `{:?}`, found kind `{:?}`",
                                kind, found_kind.kind
                            ),
                            doc_id,
                            subsection.line_number,
                        ));
                    }
                    data.push(PropertyValue::from_p1_section_with_kind(
                        subsection,
                        doc_id,
                        &kind
                            .to_owned()
                            .into_kind_data(kind_data.caption, kind_data.body),
                    )?);
                }
                Ok(Value::List {
                    data,
                    kind: kind_data.to_owned(),
                })
            }
            _ => unimplemented!(),
        }
    }
    pub(crate) fn from_p1_section(
        s: &ftd::p11::Section,
        doc_id: &str,
        kind_data: &ftd::interpreter::KindData,
    ) -> ftd::interpreter::Result<Value> {
        match &kind_data.kind {
            ftd::interpreter::Kind::String
            | ftd::interpreter::Kind::Integer
            | ftd::interpreter::Kind::Decimal
            | ftd::interpreter::Kind::Boolean => {
                let value = section_value_from_caption_or_body(s, doc_id)?;
                Value::to_value_for_basic_kind(value.as_str(), &kind_data.kind)
            }
            ftd::interpreter::Kind::Optional { kind } => {
                let kind_data = kind
                    .to_owned()
                    .into_kind_data(kind_data.caption, kind_data.body);
                if section_value_from_caption_or_body(s, doc_id).is_err() {
                    Ok(Value::Optional {
                        data: Box::new(None),
                        kind: kind_data,
                    })
                } else {
                    let value = Value::from_p1_section(s, doc_id, &kind_data)?;
                    Ok(Value::Optional {
                        data: Box::new(Some(value)),
                        kind: kind_data,
                    })
                }
            }
            ftd::interpreter::Kind::List { kind } => {
                let mut data = vec![];
                for subsection in s.sub_sections.iter() {
                    let found_kind = ftd::interpreter::KindData::from_p1_kind(
                        &subsection.name,
                        doc_id,
                        subsection.line_number,
                    )?;

                    if found_kind.kind.ne(kind) {
                        return Err(ftd::interpreter::utils::invalid_kind_error(
                            format!(
                                "List kind mismatch, expected kind `{:?}`, found kind `{:?}`",
                                kind, found_kind.kind
                            ),
                            doc_id,
                            subsection.line_number,
                        ));
                    }
                    data.push(PropertyValue::from_p1_section_with_kind(
                        subsection,
                        doc_id,
                        &kind
                            .to_owned()
                            .into_kind_data(kind_data.caption, kind_data.body),
                    )?);
                }
                Ok(Value::List {
                    data,
                    kind: kind_data.to_owned(),
                })
            }
            _ => unimplemented!(),
        }
    }

    pub(crate) fn to_value_for_basic_kind(
        s: &str,
        kind: &ftd::interpreter::Kind,
    ) -> ftd::interpreter::Result<Value> {
        Ok(match kind {
            ftd::interpreter::Kind::String => Value::String {
                text: s.to_string(),
            },
            ftd::interpreter::Kind::Integer => Value::Integer {
                value: s.parse::<i64>()?,
            },
            ftd::interpreter::Kind::Decimal => Value::Decimal {
                value: s.parse::<f64>()?,
            },
            ftd::interpreter::Kind::Boolean => Value::Boolean {
                value: s.parse::<bool>()?,
            },
            _ => unreachable!(),
        })
    }
}

fn section_value_from_caption_or_body(
    section: &ftd::p11::Section,
    doc_id: &str,
) -> ftd::interpreter::Result<String> {
    if let Some(ref header) = section.caption {
        if let Some(value) = header.get_value(doc_id)? {
            return Ok(value);
        }
    }

    if let Some(ref body) = section.body {
        return Ok(body.value.to_string());
    }

    Err(ftd::interpreter::Error::ValueNotFound {
        doc_id: doc_id.to_string(),
        line_number: section.line_number,
        message: format!("Caption and body not found {}", section.name),
    })
}

pub(crate) fn get_reference(s: &str) -> Option<&str> {
    s.trim().strip_prefix('$')
}

#[cfg(test)]
mod test {
    #[track_caller]
    fn p(s: &str, t: ftd::interpreter::PropertyValue) {
        let section = ftd::p11::parse(s, "foo")
            .unwrap_or_else(|e| panic!("{:?}", e))
            .first()
            .unwrap()
            .to_owned();
        assert_eq!(
            super::PropertyValue::from_p1_section(&section, "foo")
                .unwrap_or_else(|e| panic!("{:?}", e)),
            t
        )
    }

    #[track_caller]
    fn f(s: &str, m: &str) {
        let section = ftd::p11::parse(s, "foo")
            .unwrap_or_else(|e| panic!("{:?}", e))
            .first()
            .unwrap()
            .to_owned();
        match super::PropertyValue::from_p1_section(&section, "foo") {
            Ok(r) => panic!("expected failure, found: {:?}", r),
            Err(e) => {
                let expected = m.trim();
                let f2 = e.to_string();
                let found = f2.trim();
                if expected != found {
                    let patch = diffy::create_patch(expected, found);
                    let f = diffy::PatchFormatter::new().with_color();
                    print!(
                        "{}",
                        f.fmt_patch(&patch)
                            .to_string()
                            .replace("\\ No newline at end of file", "")
                    );
                    println!("expected:\n{}\nfound:\n{}\n", expected, f2);
                    panic!("test failed")
                }
            }
        }
    }

    #[test]
    fn integer() {
        p(
            "-- integer age: 40",
            super::PropertyValue::Value {
                value: super::Value::Integer { value: 40 },
            },
        )
    }

    #[test]
    fn integer_list() {
        p(
            indoc::indoc!(
                "
            -- integer list ages: 
            
            -- integer: 40

            -- integer: 50

            -- end: ages
            "
            ),
            super::PropertyValue::Value {
                value: super::Value::List {
                    data: vec![
                        ftd::interpreter::PropertyValue::Value {
                            value: ftd::interpreter::Value::Integer { value: 40 },
                        },
                        ftd::interpreter::PropertyValue::Value {
                            value: ftd::interpreter::Value::Integer { value: 50 },
                        },
                    ],
                    kind: ftd::interpreter::KindData {
                        kind: ftd::interpreter::Kind::List {
                            kind: Box::new(ftd::interpreter::Kind::Integer),
                        },
                        caption: false,
                        body: false,
                    },
                },
            },
        );

        f(indoc::indoc!(
            "
            -- integer list ages: 
            
            -- integer: 40

            -- string: 50

            -- end: ages
            "
            ),
          "InvalidKind: foo:5 -> List kind mismatch, expected kind `Integer`, found kind `String`"
        )
    }

    #[test]
    fn optional() {
        p(
            "-- optional integer age: 40",
            super::PropertyValue::Value {
                value: super::Value::Optional {
                    data: Box::new(Some(super::Value::Integer { value: 40 })),
                    kind: ftd::interpreter::KindData {
                        kind: ftd::interpreter::Kind::Integer,
                        caption: false,
                        body: false,
                    },
                },
            },
        );

        p(
            "-- optional integer age: ",
            super::PropertyValue::Value {
                value: super::Value::Optional {
                    data: Box::new(None),
                    kind: ftd::interpreter::KindData {
                        kind: ftd::interpreter::Kind::Integer,
                        caption: false,
                        body: false,
                    },
                },
            },
        )
    }
}
