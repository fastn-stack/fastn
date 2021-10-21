#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Variable {
    pub name: String,
    pub value: Value,
    pub conditions: Vec<(crate::p2::Boolean, crate::Value)>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum PropertyValue {
    Value { value: crate::variable::Value },
    Reference { name: String, kind: crate::p2::Kind },
    Argument { name: String, kind: crate::p2::Kind },
    LocalVariable { name: String, kind: crate::p2::Kind },
}

impl PropertyValue {
    pub fn resolve_value(
        value: &str,
        expected_kind: Option<ftd::p2::Kind>,
        doc: &ftd::p2::TDoc,
        arguments: &std::collections::BTreeMap<String, ftd::p2::Kind>,
        locals: &std::collections::BTreeMap<String, ftd::p2::Kind>,
        source: Option<ftd::TextSource>,
        is_data: bool,
    ) -> ftd::p1::Result<ftd::PropertyValue> {
        let property_type = if is_data {
            PropertyType::Value(value.to_string())
        } else if let Some(arg) = value.strip_prefix('$') {
            PropertyType::Argument(arg.to_string())
        } else if let Some(lv) = value.strip_prefix('@') {
            PropertyType::LocalVariable(lv.to_string())
        } else if doc.get_value(value).is_ok() {
            PropertyType::Reference(value.to_string())
        } else {
            let value = if (value.starts_with('\"') && value.ends_with('\"'))
                || (value.starts_with('\'') && value.ends_with('\''))
            {
                value[1..value.len() - 1].to_string()
            } else {
                value.to_string()
            };
            PropertyType::Value(value)
        };

        let (part1, part2) = get_parts(&property_type.string())?;

        return Ok(match property_type {
            PropertyType::Reference(string) => {
                let kind = match doc.get_value(&string) {
                    Ok(val) => val.kind(),
                    Err(e) => return ftd::e(format!("{} in not present in doc, {:?}", part1, e)),
                };

                let found_kind = get_kind(&kind, part2, doc, &expected_kind)?;

                PropertyValue::Reference {
                    name: doc.resolve_name(string.as_str()).unwrap_or(string),
                    kind: found_kind,
                }
            }
            PropertyType::Argument(string) => {
                let kind = match arguments.get(&part1) {
                    None => return ftd::e(format!("{} in not present in locals", part1)),
                    Some(kind) => kind.to_owned(),
                };

                let found_kind = get_kind(&kind, part2, doc, &expected_kind)?;

                PropertyValue::Argument {
                    name: string,
                    kind: found_kind,
                }
            }
            PropertyType::LocalVariable(string) => {
                let kind = match locals.get(&part1) {
                    None => return ftd::e(format!("{} in not present in locals", part1)),
                    Some(kind) => kind.to_owned(),
                };
                let found_kind = get_kind(&kind, part2, doc, &expected_kind)?;

                PropertyValue::LocalVariable {
                    name: string,
                    kind: found_kind.set_default(kind.get_default_value_str()),
                }
            }
            PropertyType::Value(string) => {
                if expected_kind.is_none() {
                    return ftd::e("expected expected_kind while calling resolve_value");
                }
                let expected_kind = expected_kind.unwrap();
                match expected_kind.inner() {
                    ftd::p2::Kind::Integer { .. } => crate::PropertyValue::Value {
                        value: crate::Value::Integer {
                            value: string
                                .parse::<i64>()
                                .map_err(|e| crate::p1::Error::CantParseInt { source: e })?,
                        },
                    },
                    ftd::p2::Kind::Decimal { .. } => crate::PropertyValue::Value {
                        value: crate::Value::Decimal {
                            value: string
                                .parse::<f64>()
                                .map_err(|e| crate::p1::Error::CantParseFloat { source: e })?,
                        },
                    },
                    ftd::p2::Kind::Boolean { .. } => crate::PropertyValue::Value {
                        value: crate::Value::Boolean {
                            value: string
                                .parse::<bool>()
                                .map_err(|_| crate::p1::Error::CantParseBool)?,
                        },
                    },
                    ftd::p2::Kind::String { .. } => crate::PropertyValue::Value {
                        value: crate::Value::String {
                            text: string,
                            source: source.unwrap_or(ftd::TextSource::Header),
                        },
                    },
                    t => {
                        return ftd::e(format!(
                            "can't resolve value {} to expected kind {:?}",
                            string, t
                        ))
                    }
                }
            }
        });

        enum PropertyType {
            Value(String),
            Reference(String),
            Argument(String),
            LocalVariable(String),
        }

        impl PropertyType {
            fn string(&self) -> String {
                match self {
                    PropertyType::Value(s)
                    | PropertyType::Reference(s)
                    | PropertyType::Argument(s)
                    | PropertyType::LocalVariable(s) => s.to_string(),
                }
            }
        }

        fn get_parts(s: &str) -> ftd::p1::Result<(String, Option<String>)> {
            Ok(if s.contains('.') {
                let (p1, p2) = ftd::p2::utils::split(s.to_string(), ".")?;
                (p1, Some(p2))
            } else {
                (s.to_string(), None)
            })
        }

        fn get_kind(
            kind: &ftd::p2::Kind,
            p2: Option<String>,
            doc: &ftd::p2::TDoc,
            expected_kind: &Option<ftd::p2::Kind>,
        ) -> crate::p1::Result<ftd::p2::Kind> {
            let mut found_kind = kind.to_owned();
            if let ftd::p2::Kind::Record { ref name } = kind {
                if let Some(p2) = p2 {
                    let rec = doc.get_record(&doc.resolve_name(name)?)?;
                    found_kind = match rec.fields.get(p2.as_str()) {
                        Some(kind) => kind.to_owned(),
                        _ => {
                            return ftd::e(format!(
                                "{} is not present in {} of type {:?}",
                                p2, name, rec
                            ));
                        }
                    };
                }
            }
            if let Some(e_kind) = expected_kind {
                if !e_kind.is_same_as(&found_kind) {
                    return ftd::e(format!("expected {:?} found {:?}", found_kind, e_kind,));
                }
                return Ok(e_kind.to_owned());
            }
            Ok(found_kind)
        }
    }

    pub fn kind(&self) -> crate::p2::Kind {
        match self {
            Self::Value { value: v } => v.kind(),
            Self::Reference { kind, .. } => kind.to_owned(),
            Self::Argument { kind, .. } => kind.to_owned(),
            Self::LocalVariable { kind, .. } => kind.to_owned(),
        }
    }
    pub fn resolve(
        &self,
        arguments: &std::collections::BTreeMap<String, crate::Value>,
        doc: &crate::p2::TDoc,
    ) -> crate::p1::Result<Value> {
        self.resolve_with_root(arguments, doc, None)
    }

    pub fn resolve_with_root(
        &self,
        arguments: &std::collections::BTreeMap<String, crate::Value>,
        doc: &crate::p2::TDoc,
        root_name: Option<&str>,
    ) -> crate::p1::Result<Value> {
        Ok(match self {
            crate::PropertyValue::Value { value: v } => v.to_owned(),
            crate::PropertyValue::Argument {
                name,
                kind: argument_kind,
            } => {
                assert_eq!(self.kind(), *argument_kind);
                if name.contains('.') {
                    let (part_1, part_2) = ftd::p2::utils::split(name.to_string(), ".")?;
                    match arguments.get(&part_1) {
                        Some(Value::Record { name, fields }) => match fields.get(&part_2) {
                            Some(pv) => return pv.resolve_with_root(arguments, doc, root_name),
                            None => {
                                return ftd::e2(
                                    format!("{} is not present in record {}", part_2, part_1),
                                    name,
                                )
                            }
                        },
                        None => {
                            return ftd::e2(format!("{} is not present in argument", part_1), name);
                        }
                        _ => return ftd::e2(format!("{} is not a record", part_1), name),
                    }
                } else {
                    match (arguments.get(name.as_str()), argument_kind.is_optional()) {
                        (Some(v), _) => v.to_owned(),
                        (None, t) => {
                            if let Ok(val) = argument_kind.to_value() {
                                val
                            } else {
                                if !t {
                                    return ftd::e2("is required", name);
                                }
                                Value::None {
                                    kind: argument_kind.to_owned(),
                                }
                            }
                        }
                    }
                }
            }
            crate::PropertyValue::LocalVariable { kind, .. } => {
                assert_eq!(self.kind(), *kind);
                kind.to_value()?
            }
            crate::PropertyValue::Reference {
                name: reference_name,
                kind: reference_kind,
            } => {
                assert_eq!(self.kind(), *reference_kind);
                let (default, condition) = match doc
                    .get_value_and_conditions_with_root(reference_name.as_str(), root_name)
                {
                    Ok(d) => d,
                    _ => return reference_kind.to_value(),
                };
                let mut value = default;
                for (boolean, property) in condition {
                    if boolean.eval(arguments, doc)? {
                        value = property;
                    }
                }
                value
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
    Default,
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
                default: None,
            },
            Value::Integer { .. } => crate::p2::Kind::integer(),
            Value::Decimal { .. } => crate::p2::Kind::decimal(),
            Value::Boolean { .. } => crate::p2::Kind::boolean(),
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

    pub fn to_string(&self) -> Option<String> {
        match self {
            Value::String { text, .. } => Some(text.to_string()),
            Value::Integer { value } => Some(value.to_string()),
            Value::Decimal { value } => Some(value.to_string()),
            Value::Boolean { value } => Some(value.to_string()),
            _ => None,
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
                kind: crate::p2::Kind::from(p1.header.str("type")?, doc, None)?,
            },
            conditions: vec![],
        })
    }

    pub fn map_from_p1(p1: &crate::p1::Section, doc: &crate::p2::TDoc) -> crate::p1::Result<Self> {
        let name = doc.resolve_name(ftd_rt::get_name("map", p1.name.as_str())?)?;
        Ok(Variable {
            name,
            value: Value::Map {
                data: Default::default(),
                kind: crate::p2::Kind::from(p1.header.str("type")?, doc, None)?,
            },
            conditions: vec![],
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
                crate::p2::Kind::Integer { .. } => read_integer(p1)?,
                crate::p2::Kind::Decimal { .. } => read_decimal(p1)?,
                crate::p2::Kind::Boolean { .. } => read_boolean(p1)?,
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
            (ftd::p2::Kind::Map { .. }, _) => return ftd::e("unexpected map"),
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
                t => return ftd::e2("unexpected thing found", t),
            },
            None => guess_type(p1)?,
        };

        Ok(Variable {
            name,
            value,
            conditions: vec![],
        })
    }

    pub fn get_value(
        &self,
        p1: &crate::p1::Section,
        doc: &crate::p2::TDoc,
    ) -> ftd::p1::Result<ftd::Value> {
        match self.value.kind() {
            ftd::p2::Kind::String { .. } => read_string(p1),
            ftd::p2::Kind::Integer { .. } => read_integer(p1),
            ftd::p2::Kind::Decimal { .. } => read_decimal(p1),
            ftd::p2::Kind::Boolean { .. } => read_boolean(p1),
            ftd::p2::Kind::Record { name } => match doc.get_thing(&name)? {
                crate::p2::Thing::Record(r) => r.create(p1, doc),
                t => crate::e(format!("expected record type, found: {:?}", t)),
            },
            ftd::p2::Kind::OrType { name } => match doc.get_thing(&name)? {
                crate::p2::Thing::OrTypeWithVariant { e, variant } => e.create(p1, variant, doc),
                t => crate::e(format!("expected or-type type, found: {:?}", t)),
            },
            t => crate::e(format!("unexpected type found: {:?}", t)),
        }
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
    match (&p1.caption, &p1.body_without_comment()) {
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
        ($s:expr, $n: expr, $v: expr, $c: expr,) => {
            p2!($s, $n, $v, $c)
        };
        ($s:expr, $n: expr, $v: expr, $c: expr) => {
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
                    value: $v,
                    conditions: $c
                }
            )
        };
    }

    #[test]
    fn int() {
        use super::Value::Integer;
        p2!("-- var x: 10", "x", Integer { value: 10 }, vec![],);
        p2!(
            "-- var x: 10\ntype: integer",
            "x",
            Integer { value: 10 },
            vec![],
        );
    }

    #[test]
    fn float() {
        use super::Value::Decimal;
        p2!("-- var x: 10.0", "x", Decimal { value: 10.0 }, vec![],);
        p2!(
            "-- var x: 10\ntype: decimal",
            "x",
            Decimal { value: 10.0 },
            vec![],
        );
    }

    #[test]
    fn bool() {
        use super::Value::Boolean;
        p2!("-- var x: true", "x", Boolean { value: true }, vec![],);
        p2!(
            "-- var x: false\ntype: boolean",
            "x",
            Boolean { value: false },
            vec![],
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
            vec![],
        );
        p2!(
            "-- var x:\n\nhello world\nyo!",
            "x",
            String {
                text: "hello world\nyo!".to_string(),
                source: crate::TextSource::Body
            },
            vec![],
        );
        p2!(
            "-- var x: 10\ntype: string",
            "x",
            String {
                text: "10".to_string(),
                source: crate::TextSource::Caption
            },
            vec![],
        );
        p2!(
            "-- var x: true\ntype: string",
            "x",
            String {
                text: "true".to_string(),
                source: crate::TextSource::Caption
            },
            vec![],
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
                conditions: vec![],
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
            (bag, default_column()),
        );
    }
}
