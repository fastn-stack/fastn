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
    Variable { name: String, kind: crate::p2::Kind },
}

impl PropertyValue {
    pub fn resolve_value(
        line_number: usize,
        value: &str,
        expected_kind: Option<ftd::p2::Kind>,
        doc: &ftd::p2::TDoc,
        arguments: &std::collections::BTreeMap<String, ftd::p2::Kind>,
        source: Option<ftd::TextSource>,
    ) -> ftd::p1::Result<ftd::PropertyValue> {
        let property_type = if let Some(arg) = value.strip_prefix('$') {
            PropertyType::Variable(arg.to_string())
        } else {
            let value = if let Some(value) = value.strip_prefix('\\') {
                value.to_string()
            } else {
                value.to_string()
            };
            PropertyType::Value(value)
        };

        let (part1, part2) = get_parts(&property_type.string())?;

        return Ok(match property_type {
            PropertyType::Variable(string) => {
                let (kind, is_doc) = match arguments.get(&part1) {
                    _ if part1.eq("MOUSE-IN") => (
                        ftd::p2::Kind::Boolean {
                            default: Some("false".to_string()),
                        },
                        false,
                    ),
                    None => match doc.get_value(line_number, &string) {
                        Ok(val) => (val.kind(), true),
                        Err(e) => {
                            return ftd::e2(
                                format!("{} is not present in doc, {:?}", part1, e),
                                doc.name,
                                line_number,
                            )
                        }
                    },
                    Some(kind) => (kind.to_owned(), false),
                };

                let found_kind = get_kind(line_number, &kind, part2, doc, &expected_kind)?;

                if is_doc {
                    PropertyValue::Reference {
                        name: doc
                            .resolve_name(line_number, string.as_str())
                            .unwrap_or(string),
                        kind: found_kind,
                    }
                } else {
                    PropertyValue::Variable {
                        name: string,
                        kind: found_kind,
                    }
                }
            }
            PropertyType::Value(string) => {
                if expected_kind.is_none() {
                    return ftd::e2(
                        "expected expected_kind while calling resolve_value",
                        doc.name,
                        line_number,
                    );
                }
                let expected_kind = expected_kind.unwrap();
                match expected_kind.inner() {
                    ftd::p2::Kind::Integer { .. } => crate::PropertyValue::Value {
                        value: crate::Value::Integer {
                            value: string.parse::<i64>().map_err(|e| {
                                crate::p1::Error::ParseError {
                                    message: e.to_string(),
                                    doc_id: doc.name.to_string(),
                                    line_number,
                                }
                            })?,
                        },
                    },
                    ftd::p2::Kind::Decimal { .. } => crate::PropertyValue::Value {
                        value: crate::Value::Decimal {
                            value: string.parse::<f64>().map_err(|e| {
                                crate::p1::Error::ParseError {
                                    message: e.to_string(),
                                    doc_id: doc.name.to_string(),
                                    line_number,
                                }
                            })?,
                        },
                    },
                    ftd::p2::Kind::Boolean { .. } => crate::PropertyValue::Value {
                        value: crate::Value::Boolean {
                            value: string.parse::<bool>().map_err(|e| {
                                crate::p1::Error::ParseError {
                                    message: e.to_string(),
                                    doc_id: doc.name.to_string(),
                                    line_number,
                                }
                            })?,
                        },
                    },
                    ftd::p2::Kind::String { .. } => crate::PropertyValue::Value {
                        value: crate::Value::String {
                            text: string,
                            source: source.unwrap_or(ftd::TextSource::Header),
                        },
                    },
                    t => {
                        return ftd::e2(
                            format!("can't resolve value {} to expected kind {:?}", string, t),
                            doc.name,
                            line_number,
                        )
                    }
                }
            }
        });

        #[derive(Debug)]
        enum PropertyType {
            Value(String),
            Variable(String),
        }

        impl PropertyType {
            fn string(&self) -> String {
                match self {
                    PropertyType::Value(s) | PropertyType::Variable(s) => s.to_string(),
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
            line_number: usize,
            kind: &ftd::p2::Kind,
            p2: Option<String>,
            doc: &ftd::p2::TDoc,
            expected_kind: &Option<ftd::p2::Kind>,
        ) -> crate::p1::Result<ftd::p2::Kind> {
            let mut found_kind = kind.to_owned();
            if let ftd::p2::Kind::Record { ref name } = kind {
                if let Some(p2) = p2 {
                    let rec = doc.get_record(line_number, &doc.resolve_name(line_number, name)?)?;
                    found_kind = match rec.fields.get(p2.as_str()) {
                        Some(kind) => kind.to_owned(),
                        _ => {
                            return ftd::e2(
                                format!("{} is not present in {} of type {:?}", p2, name, rec),
                                doc.name,
                                line_number,
                            );
                        }
                    };
                }
            }
            if let Some(e_kind) = expected_kind {
                if !e_kind.is_same_as(&found_kind) {
                    return ftd::e2(
                        format!("expected {:?} found {:?}", found_kind, e_kind),
                        doc.name,
                        line_number,
                    );
                }
                return Ok(e_kind.to_owned().set_default({
                    if found_kind.get_default_value_str().is_some() {
                        found_kind.get_default_value_str()
                    } else {
                        e_kind.get_default_value_str()
                    }
                }));
            }
            Ok(found_kind)
        }
    }

    pub fn kind(&self) -> crate::p2::Kind {
        match self {
            Self::Value { value: v } => v.kind(),
            Self::Reference { kind, .. } => kind.to_owned(),
            Self::Variable { kind, .. } => kind.to_owned(),
        }
    }
    pub fn resolve(
        &self,
        line_number: usize,
        arguments: &std::collections::BTreeMap<String, crate::Value>,
        doc: &crate::p2::TDoc,
    ) -> crate::p1::Result<Value> {
        self.resolve_with_root(line_number, arguments, doc, None)
    }

    pub fn resolve_with_root(
        &self,
        line_number: usize,
        arguments: &std::collections::BTreeMap<String, crate::Value>,
        doc: &crate::p2::TDoc,
        root_name: Option<&str>,
    ) -> crate::p1::Result<Value> {
        Ok(match self {
            crate::PropertyValue::Value { value: v } => v.to_owned(),
            crate::PropertyValue::Variable {
                name,
                kind: argument_kind,
            } => {
                assert_eq!(self.kind(), *argument_kind);
                if name.contains('.') {
                    let (part_1, part_2) = ftd::p2::utils::split(name.to_string(), ".")?;
                    match arguments.get(&part_1) {
                        Some(Value::Record { name, fields }) => match fields.get(&part_2) {
                            Some(pv) => {
                                return pv.resolve_with_root(line_number, arguments, doc, root_name)
                            }
                            None => {
                                return ftd::e2(
                                    format!(
                                        "{} is not present in record {} [name: {}]",
                                        part_2, part_1, name
                                    ),
                                    doc.name,
                                    line_number,
                                )
                            }
                        },
                        None => {
                            return ftd::e2(
                                format!("{} is not present in argument [name: {}]", part_1, name),
                                doc.name,
                                line_number,
                            );
                        }
                        _ => {
                            return ftd::e2(
                                format!("{} is not a record [name: {}]", part_1, name),
                                doc.name,
                                line_number,
                            )
                        }
                    }
                } else {
                    match (arguments.get(name.as_str()), argument_kind.is_optional()) {
                        (Some(v), _) => v.to_owned(),
                        (None, t) => {
                            if let Ok(val) = argument_kind.to_value(line_number, doc.name) {
                                val
                            } else {
                                if !t {
                                    return ftd::e2(
                                        format!("{} is required", name),
                                        doc.name,
                                        line_number,
                                    );
                                }
                                Value::None {
                                    kind: argument_kind.to_owned(),
                                }
                            }
                        }
                    }
                }
            }
            crate::PropertyValue::Reference {
                name: reference_name,
                kind: reference_kind,
            } => {
                assert_eq!(self.kind(), *reference_kind);
                let (default, condition) = match doc
                    .
                        get_value_and_conditions_with_root(0, reference_name.as_str(), root_name) //todo
                {
                    Ok(d) => d,
                    _ => return reference_kind.to_value(line_number, doc.name),
                };
                let mut value = default;
                for (boolean, property) in condition {
                    if boolean.eval(line_number, arguments, doc)? {
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

    pub fn is_empty(&self) -> bool {
        if let Self::List { data, .. } = self {
            if data.is_empty() {
                return true;
            }
        }
        false
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

    pub fn is_equal(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::String { text: a, .. }, Value::String { text: b, .. }) => a == b,
            (a, b) => a == b,
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
        let var_data =
            ftd::variable::VariableData::get_name_kind(&p1.name, doc.name, p1.line_number, true)?;
        let name = doc.resolve_name(p1.line_number, &var_data.name)?;
        let kind = crate::p2::Kind::for_variable(p1.line_number, &p1.name, None, doc, None, true)?;
        if !kind.is_list() {
            return ftd::e2(
                format!("Expected list found: {:?}", p1),
                doc.name,
                p1.line_number,
            );
        }
        Ok(Variable {
            name,
            value: Value::List {
                data: Default::default(),
                kind: kind.list_kind().to_owned(),
            },
            conditions: vec![],
        })
    }

    pub fn map_from_p1(p1: &crate::p1::Section, doc: &crate::p2::TDoc) -> crate::p1::Result<Self> {
        let name = doc.resolve_name(
            p1.line_number,
            ftd_rt::get_name("map", p1.name.as_str(), doc.name)?,
        )?;
        Ok(Variable {
            name,
            value: Value::Map {
                data: Default::default(),
                kind: crate::p2::Kind::from(
                    p1.line_number,
                    p1.header.str(doc.name, p1.line_number, "type")?,
                    doc,
                    None,
                )?,
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
            line_number: usize,
            kind: &crate::p2::Kind,
            p1: &crate::p1::Section,
            doc: &crate::p2::TDoc,
        ) -> crate::p1::Result<crate::Value> {
            Ok(match kind {
                crate::p2::Kind::Integer { .. } => read_integer(p1, doc.name)?,
                crate::p2::Kind::Decimal { .. } => read_decimal(p1, doc.name)?,
                crate::p2::Kind::Boolean { .. } => read_boolean(p1, doc.name)?,
                crate::p2::Kind::String { .. } => read_string(p1, doc.name)?,
                crate::p2::Kind::Record { name } => {
                    doc.get_record(line_number, name)?.create(p1, doc)?
                }
                _ => unimplemented!("{:?}", kind),
            })
        }

        let p1 = {
            let mut p1 = p1.clone();
            p1.name = if let Some(n) = p1.name.strip_prefix('$') {
                n.to_string()
            } else {
                p1.name
            };
            p1
        };

        match (self.value.kind().inner(), &mut self.value) {
            (ftd::p2::Kind::Record { name }, _) => {
                self.value = doc.get_record(p1.line_number, name)?.create(&p1, doc)?
            }
            (ftd::p2::Kind::List { kind }, crate::Value::List { data, .. }) => {
                data.push(read_value(p1.line_number, kind, &p1, doc)?);
            }
            (ftd::p2::Kind::Map { .. }, _) => {
                return ftd::e2("unexpected map", doc.name, p1.line_number)
            }
            (k, _) => self.value = read_value(p1.line_number, k, &p1, doc)?,
        };

        Ok(())
    }

    pub fn from_p1(p1: &crate::p1::Section, doc: &crate::p2::TDoc) -> crate::p1::Result<Self> {
        let var_data =
            ftd::variable::VariableData::get_name_kind(&p1.name, doc.name, p1.line_number, true)?;
        let name = var_data.name;
        let value = match var_data.kind.as_deref() {
            Some("string") => read_string(p1, doc.name)?,
            Some("integer") => read_integer(p1, doc.name)?,
            Some("decimal") => read_decimal(p1, doc.name)?,
            Some("boolean") => read_boolean(p1, doc.name)?,
            Some(t) => match doc.get_thing(p1.line_number, t)? {
                crate::p2::Thing::Record(r) => r.create(p1, doc)?,
                crate::p2::Thing::OrTypeWithVariant { e, variant } => e.create(p1, variant, doc)?,
                t => {
                    return ftd::e2(
                        format!("unexpected thing found: {:?}", t),
                        doc.name,
                        p1.line_number,
                    )
                }
            },
            None => {
                let (caption, is_body) = match p1.caption.as_deref() {
                    Some(v) => (v.to_string(), false),
                    None => (p1.body(p1.line_number, doc.name)?, true),
                };
                guess_type(&caption, is_body)?
            }
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
            ftd::p2::Kind::String { .. } => read_string(p1, doc.name),
            ftd::p2::Kind::Integer { .. } => read_integer(p1, doc.name),
            ftd::p2::Kind::Decimal { .. } => read_decimal(p1, doc.name),
            ftd::p2::Kind::Boolean { .. } => read_boolean(p1, doc.name),
            ftd::p2::Kind::Record { name } => match doc.get_thing(p1.line_number, &name)? {
                crate::p2::Thing::Record(r) => r.create(p1, doc),
                t => ftd::e2(
                    format!("expected record type, found: {:?}", t),
                    doc.name,
                    p1.line_number,
                ),
            },
            ftd::p2::Kind::OrType { name } => match doc.get_thing(p1.line_number, &name)? {
                crate::p2::Thing::OrTypeWithVariant { e, variant } => e.create(p1, variant, doc),
                t => ftd::e2(
                    format!("expected or-type type, found: {:?}", t),
                    doc.name,
                    p1.line_number,
                ),
            },
            t => ftd::e2(
                format!("unexpected type found: {:?}", t),
                doc.name,
                p1.line_number,
            ),
        }
    }
}

pub fn guess_type(s: &str, is_body: bool) -> crate::p1::Result<Value> {
    if is_body {
        return Ok(Value::String {
            text: s.to_string(),
            source: TextSource::Body,
        });
    }
    let caption = match s {
        "true" => return Ok(Value::Boolean { value: true }),
        "false" => return Ok(Value::Boolean { value: false }),
        v => v,
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

fn read_string(p1: &crate::p1::Section, doc_id: &str) -> crate::p1::Result<Value> {
    match (&p1.caption, &p1.body_without_comment()) {
        (Some(c), Some(b)) => ftd::e2(
            format!("both caption: `{}` and body: `{}` present", c, b.1),
            doc_id,
            p1.line_number,
        ),
        (Some(caption), None) => Ok(Value::String {
            text: caption.to_string(),
            source: TextSource::Caption,
        }),
        (None, Some(body)) => Ok(Value::String {
            text: body.1.to_string(),
            source: TextSource::Body,
        }),
        (None, None) => ftd::e2(
            "either body or caption is required for string",
            doc_id,
            p1.line_number,
        ),
    }
}

fn read_integer(p1: &crate::p1::Section, doc_id: &str) -> crate::p1::Result<Value> {
    let caption = p1.caption(p1.line_number, doc_id)?;
    if let Ok(v) = caption.parse::<i64>() {
        return Ok(Value::Integer { value: v });
    }

    ftd::e2("not a valid integer", doc_id, p1.line_number)
}

fn read_decimal(p1: &crate::p1::Section, doc_id: &str) -> crate::p1::Result<Value> {
    let caption = p1.caption(p1.line_number, doc_id)?;
    if let Ok(v) = caption.parse::<f64>() {
        return Ok(Value::Decimal { value: v });
    }

    ftd::e2("not a valid float", doc_id, p1.line_number)
}

fn read_boolean(p1: &crate::p1::Section, doc_id: &str) -> crate::p1::Result<Value> {
    let caption = p1.caption(p1.line_number, doc_id)?;
    if let Ok(v) = caption.parse::<bool>() {
        return Ok(Value::Boolean { value: v });
    }

    ftd::e2("not a valid bool", doc_id, p1.line_number)
}

#[derive(Debug, Clone)]
pub struct VariableData {
    pub name: String,
    pub kind: Option<String>,
    pub modifier: VariableModifier,
}

#[derive(Debug, Clone)]
pub enum VariableModifier {
    None,
    List,
    Optional,
}

impl VariableData {
    pub fn get_name_kind(
        s: &str,
        doc_id: &str,
        line_number: usize,
        is_variable: bool,
    ) -> ftd::p1::Result<VariableData> {
        let expr = s.split_whitespace().collect::<Vec<&str>>();
        if expr.len() > 4 {
            return ftd::e2(
                format!("invalid variable or list declaration, found: `{}`", s),
                doc_id,
                line_number,
            );
        }
        let mut name = expr.get(0);
        let mut kind = None;
        let mut modifier = VariableModifier::None;
        if expr.len() == 4 {
            if expr.get(1).unwrap().eq(&"or") {
                kind = Some(expr[..3].join(" "));
                name = expr.get(3);
            } else {
                return ftd::e2(
                    format!("invalid variable or list declaration, found: `{}`", s),
                    doc_id,
                    line_number,
                );
            }
        } else if expr.len() == 3 {
            if expr.get(1).unwrap().eq(&"list") {
                modifier = VariableModifier::List;
                name = expr.get(2);
                kind = expr.get(0).map(|k| k.to_string());
            } else if expr.get(0).unwrap().eq(&"optional") {
                modifier = VariableModifier::Optional;
                name = expr.get(2);
                kind = expr.get(1).map(|k| k.to_string());
            } else {
                return ftd::e2(
                    format!("invalid variable or list declaration, found: `{}`", s),
                    doc_id,
                    line_number,
                );
            }
        } else if expr.len() == 2 {
            name = expr.get(1);
            kind = expr.get(0).map(|k| k.to_string());
        }

        let var_name = {
            let error = ftd::e2(
                if is_variable {
                    format!(
                        "list name should be present and should start with $, found: `{}`",
                        s
                    )
                } else {
                    format!("name should not start with $, found: `{}`", s)
                },
                doc_id,
                line_number,
            );
            if name.is_none() {
                return error;
            }

            let name = name.unwrap();
            let mut var_name = name.to_string();
            if let Some(n) = name.strip_prefix('$') {
                if !is_variable {
                    return error;
                }
                var_name = n.to_string();
            } else if is_variable {
                return error;
            }
            var_name
        };

        Ok(VariableData {
            name: var_name,
            kind,
            modifier,
        })
    }

    pub fn is_none(&self) -> bool {
        matches!(self.modifier, VariableModifier::None)
    }

    pub fn is_list(&self) -> bool {
        matches!(self.modifier, VariableModifier::List)
    }

    pub fn is_optional(&self) -> bool {
        matches!(self.modifier, VariableModifier::Optional)
    }
}
#[cfg(test)]
mod test {
    use crate::test::*;

    macro_rules! p2 {
        ($s:expr, $n: expr, $v: expr, $c: expr,) => {
            p2!($s, $n, $v, $c)
        };
        ($s:expr, $n: expr, $v: expr, $c: expr) => {
            let p1 = crate::p1::parse(indoc::indoc!($s), "foo").unwrap();
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
        p2!("-- $x: 10", "x", Integer { value: 10 }, vec![],);
        p2!("-- integer $x: 10", "x", Integer { value: 10 }, vec![],);
    }

    #[test]
    fn float() {
        use super::Value::Decimal;
        p2!("-- $x: 10.0", "x", Decimal { value: 10.0 }, vec![],);
        p2!("-- decimal $x: 10", "x", Decimal { value: 10.0 }, vec![],);
    }

    #[test]
    fn bool() {
        use super::Value::Boolean;
        p2!("-- $x: true", "x", Boolean { value: true }, vec![],);
        p2!(
            "-- boolean $x: false",
            "x",
            Boolean { value: false },
            vec![],
        );
    }

    #[test]
    fn str() {
        use super::Value::String;
        p2!(
            "-- $x: hello",
            "x",
            String {
                text: "hello".to_string(),
                source: crate::TextSource::Caption
            },
            vec![],
        );
        p2!(
            "-- $x:\n\nhello world\nyo!",
            "x",
            String {
                text: "hello world\nyo!".to_string(),
                source: crate::TextSource::Body
            },
            vec![],
        );
        p2!(
            "-- string $x: 10",
            "x",
            String {
                text: "10".to_string(),
                source: crate::TextSource::Caption
            },
            vec![],
        );
        p2!(
            "-- string $x: true",
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
            caption title:
            body about:

            -- component pr-view:
            pull-request $pr:
            component: ftd.column

            --- ftd.text:
            text: $pr.title

            --- ftd.text:
            text: $pr.about

            -- list pr:
            type: pull-request

            -- pr: some pr

            yo yo
            ",
            (bag, default_column()),
        );
    }
}
