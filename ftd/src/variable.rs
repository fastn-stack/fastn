#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Variable {
    pub name: String,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum PropertyValue {
    Value { value: crate::variable::Value },
    Reference { name: String, kind: crate::p2::Kind },
    Argument { name: String, kind: crate::p2::Kind },
}

impl PropertyValue {
    pub fn kind(&self) -> crate::p2::Kind {
        match self {
            Self::Value { value: v } => v.kind(),
            Self::Reference { kind, .. } => kind.to_owned(),
            Self::Argument { kind, .. } => kind.to_owned(),
        }
    }

    pub fn resolve(
        &self,
        arguments: &std::collections::BTreeMap<String, crate::Value>,
        doc: &crate::p2::TDoc,
    ) -> crate::p1::Result<Value> {
        Ok(match self {
            crate::PropertyValue::Value { value: v } => v.to_owned(),
            crate::PropertyValue::Argument {
                name,
                kind: argument_kind,
            } => {
                assert_eq!(self.kind(), *argument_kind);
                match (arguments.get(name.as_str()), argument_kind.is_optional()) {
                    (Some(v), _) => v.to_owned(),
                    (None, true) => Value::None {
                        kind: argument_kind.to_owned(),
                    },
                    (None, false) => {
                        return ftd::e2("is required", name);
                    }
                }
            }
            crate::PropertyValue::Reference {
                name: reference_name,
                kind: reference_kind,
            } => {
                assert_eq!(self.kind(), *reference_kind);
                doc.get_value(reference_name.as_str())?
            }
        })
    }
}

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum TextSource {
    Header,
    Caption,
    Body,
}

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
#[allow(clippy::large_enum_variant)]
pub enum Value {
    None {
        kind: crate::p2::Kind,
    },
    String {
        text: String,
        source: ftd::TextSource,
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
    Record {
        name: String,
        fields: std::collections::BTreeMap<String, PropertyValue>,
    },
    OrType {
        name: String,
        variant: String,
        fields: std::collections::BTreeMap<String, PropertyValue>,
    },
    List {
        data: Vec<Value>,
        kind: crate::p2::Kind,
    },
    Map {
        data: std::collections::BTreeMap<String, Value>,
        kind: crate::p2::Kind,
    },
}

impl Value {
    pub fn is_null(&self) -> bool {
        matches!(self, Self::None { .. })
    }

    pub fn kind(&self) -> crate::p2::Kind {
        match self {
            Value::None { kind: k } => k.to_owned(),
            Value::String { source, .. } => crate::p2::Kind::String {
                caption: *source == TextSource::Caption,
                body: *source == TextSource::Body,
            },
            Value::Integer { .. } => crate::p2::Kind::Integer,
            Value::Decimal { .. } => crate::p2::Kind::Decimal,
            Value::Boolean { .. } => crate::p2::Kind::Boolean,
            Value::Record { name: id, .. } => crate::p2::Kind::Record {
                name: id.to_string(),
            },
            Value::OrType { name: id, .. } => crate::p2::Kind::OrType {
                name: id.to_string(),
            },
            Value::List { kind, .. } => crate::p2::Kind::List {
                kind: Box::new(kind.to_owned()),
            },
            Value::Map { kind, .. } => crate::p2::Kind::Map {
                kind: Box::new(kind.to_owned()),
            },
        }
    }
}

impl Variable {
    pub fn list_from_p1(p1: &crate::p1::Section, doc: &crate::p2::TDoc) -> crate::p1::Result<Self> {
        let name = doc.resolve_name(ftd_rt::get_name("list", p1.name.as_str())?)?;
        Ok(Variable {
            name,
            value: Value::List {
                data: Default::default(),
                kind: crate::p2::Kind::from(p1.header.str("type")?, doc)?,
            },
        })
    }

    pub fn map_from_p1(p1: &crate::p1::Section, doc: &crate::p2::TDoc) -> crate::p1::Result<Self> {
        let name = doc.resolve_name(ftd_rt::get_name("map", p1.name.as_str())?)?;
        Ok(Variable {
            name,
            value: Value::Map {
                data: Default::default(),
                kind: crate::p2::Kind::from(p1.header.str("type")?, doc)?,
            },
        })
    }

    pub fn update_from_p1(
        &mut self,
        p1: &crate::p1::Section,
        doc: &crate::p2::TDoc,
    ) -> crate::p1::Result<()> {
        fn read_value(
            kind: &crate::p2::Kind,
            p1: &crate::p1::Section,
            doc: &crate::p2::TDoc,
        ) -> crate::p1::Result<crate::Value> {
            Ok(match kind {
                crate::p2::Kind::Integer => read_integer(p1)?,
                crate::p2::Kind::Decimal => read_decimal(p1)?,
                crate::p2::Kind::Boolean => read_boolean(p1)?,
                crate::p2::Kind::String { .. } => read_string(p1)?,
                crate::p2::Kind::Record { name } => doc.get_record(name)?.create(p1, doc)?,
                _ => unimplemented!("{:?}", kind),
            })
        }

        match (self.value.kind().inner(), &mut self.value) {
            (ftd::p2::Kind::Record { name }, _) => {
                self.value = doc.get_record(name)?.create(p1, doc)?
            }
            (ftd::p2::Kind::List { kind }, crate::Value::List { data, .. }) => {
                data.push(read_value(kind, p1, doc)?);
            }
            (ftd::p2::Kind::Map { .. }, _) => todo!(),
            (k, _) => self.value = read_value(k, p1, doc)?,
        };

        Ok(())
    }

    pub fn from_p1(p1: &crate::p1::Section, doc: &crate::p2::TDoc) -> crate::p1::Result<Self> {
        let name = ftd_rt::get_name("var", p1.name.as_str())?.to_string();

        let value = match p1.header.str_optional("type")? {
            Some("string") => read_string(p1)?,
            Some("integer") => read_integer(p1)?,
            Some("decimal") => read_decimal(p1)?,
            Some("boolean") => read_boolean(p1)?,
            Some(t) => match doc.get_thing(t)? {
                crate::p2::Thing::Record(r) => r.create(p1, doc)?,
                crate::p2::Thing::OrTypeWithVariant { e, variant } => e.create(p1, variant, doc)?,
                t => todo!("{:?}", t),
            },
            None => guess_type(p1)?,
        };

        Ok(Variable { name, value })
    }
}

fn guess_type(p1: &crate::p1::Section) -> crate::p1::Result<Value> {
    let caption = match p1.caption.as_deref() {
        Some("true") => return Ok(Value::Boolean { value: true }),
        Some("false") => return Ok(Value::Boolean { value: false }),
        Some(v) => v,
        None => {
            return Ok(Value::String {
                text: p1.body()?,
                source: TextSource::Body,
            });
        }
    };

    if let Ok(v) = caption.parse::<i64>() {
        return Ok(Value::Integer { value: v });
    }

    if let Ok(v) = caption.parse::<f64>() {
        return Ok(Value::Decimal { value: v });
    }

    Ok(Value::String {
        text: caption.to_string(),
        source: TextSource::Caption,
    })
}

fn read_string(p1: &crate::p1::Section) -> crate::p1::Result<Value> {
    match (&p1.caption, &p1.body) {
        (Some(_), Some(_)) => crate::e("' ' is missing".to_string()),
        (Some(caption), None) => Ok(Value::String {
            text: caption.to_string(),
            source: TextSource::Caption,
        }),
        (None, Some(body)) => Ok(Value::String {
            text: body.to_string(),
            source: TextSource::Body,
        }),
        (None, None) => crate::e("either body or caption is required for string".to_string()),
    }
}

fn read_integer(p1: &crate::p1::Section) -> crate::p1::Result<Value> {
    let caption = p1.caption()?;
    if let Ok(v) = caption.parse::<i64>() {
        return Ok(Value::Integer { value: v });
    }

    crate::e("not a valid integer".to_string())
}

fn read_decimal(p1: &crate::p1::Section) -> crate::p1::Result<Value> {
    let caption = p1.caption()?;
    if let Ok(v) = caption.parse::<f64>() {
        return Ok(Value::Decimal { value: v });
    }

    crate::e("not a valid float".to_string())
}

fn read_boolean(p1: &crate::p1::Section) -> crate::p1::Result<Value> {
    let caption = p1.caption()?;
    if let Ok(v) = caption.parse::<bool>() {
        return Ok(Value::Boolean { value: v });
    }

    crate::e("not a valid bool".to_string())
}

#[cfg(test)]
mod test {
    use crate::test::*;

    macro_rules! p2 {
        ($s:expr, $n: expr, $v: expr,) => {
            p2!($s, $n, $v)
        };
        ($s:expr, $n: expr, $v: expr) => {
            let p1 = crate::p1::parse(indoc::indoc!($s)).unwrap();
            let mut bag = std::collections::BTreeMap::new();
            let aliases = std::collections::BTreeMap::new();
            let mut d = crate::p2::TDoc {
                name: "foo",
                bag: &mut bag,
                aliases: &aliases,
            };
            pretty_assertions::assert_eq!(
                super::Variable::from_p1(&p1[0], &mut d).unwrap(),
                super::Variable {
                    name: $n.to_string(),
                    value: $v
                }
            )
        };
    }

    #[test]
    fn int() {
        use super::Value::Integer;
        p2!("-- var x: 10", "x", Integer { value: 10 },);
        p2!("-- var x: 10\ntype: integer", "x", Integer { value: 10 },);
    }

    #[test]
    fn float() {
        use super::Value::Decimal;
        p2!("-- var x: 10.0", "x", Decimal { value: 10.0 },);
        p2!("-- var x: 10\ntype: decimal", "x", Decimal { value: 10.0 },);
    }

    #[test]
    fn bool() {
        use super::Value::Boolean;
        p2!("-- var x: true", "x", Boolean { value: true },);
        p2!(
            "-- var x: false\ntype: boolean",
            "x",
            Boolean { value: false },
        );
    }

    #[test]
    fn str() {
        use super::Value::String;
        p2!(
            "-- var x: hello",
            "x",
            String {
                text: "hello".to_string(),
                source: crate::TextSource::Caption
            },
        );
        p2!(
            "-- var x:\n\nhello world\nyo!",
            "x",
            String {
                text: "hello world\nyo!".to_string(),
                source: crate::TextSource::Body
            },
        );
        p2!(
            "-- var x: 10\ntype: string",
            "x",
            String {
                text: "10".to_string(),
                source: crate::TextSource::Caption
            }
        );
        p2!(
            "-- var x: true\ntype: string",
            "x",
            String {
                text: "true".to_string(),
                source: crate::TextSource::Caption
            }
        );
    }

    #[test]
    #[ignore]
    fn list_with_component() {
        let mut bag = default_bag();
        bag.insert(
            s("foo/bar#pull-request"),
            crate::p2::Thing::Record(crate::p2::Record {
                name: s("foo/bar#pull-request"),
                fields: std::array::IntoIter::new([
                    (s("title"), crate::p2::Kind::caption()),
                    (s("about"), crate::p2::Kind::body()),
                ])
                .collect(),
                instances: Default::default(),
            }),
        );

        bag.insert(
            "foo/bar#pr".to_string(),
            crate::p2::Thing::Variable(crate::Variable {
                name: "foo/bar#pr".to_string(),
                value: crate::Value::List {
                    data: vec![crate::Value::Record {
                        name: s("foo/bar#pull-request"),
                        fields: std::array::IntoIter::new([
                            (
                                s("title"),
                                crate::PropertyValue::Value {
                                    value: crate::Value::String {
                                        text: "some pr".to_string(),
                                        source: crate::TextSource::Caption,
                                    },
                                },
                            ),
                            (
                                s("about"),
                                crate::PropertyValue::Value {
                                    value: crate::Value::String {
                                        text: "yo yo".to_string(),
                                        source: crate::TextSource::Body,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    }],
                    kind: crate::p2::Kind::Record {
                        name: s("foo/bar#pull-request"),
                    },
                },
            }),
        );

        p!(
            "
            -- record pull-request:
            title: caption
            about: body

            -- component pr-view:
            $pr: pull-request
            component: ftd.column

            --- ftd.text:
            text: ref $pr.title

            --- ftd.text:
            text: ref $pr.about

            -- list pr:
            type: pull-request

            -- pr: some pr

            yo yo
            ",
            &test_library(),
            (bag, default_column()),
        );
    }
}
