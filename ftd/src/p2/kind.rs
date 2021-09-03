#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Kind {
    String { caption: bool, body: bool },
    Integer,
    Decimal,
    Boolean,
    Element,
    Elements,
    Message,
    StringMessage,           // message that takes a string
    IntMessage,              // message that takes an int
    Record { name: String }, // the full name of the record (full document name.record name)
    OrType { name: String }, // the full name of the or-type
    Map { kind: Box<Kind> }, // map of String to Kind
    List { kind: Box<Kind> },
    Optional { kind: Box<Kind> },
}

impl Kind {
    pub fn is_optional(&self) -> bool {
        matches!(self, Kind::Optional { .. })
    }
}

impl Kind {
    pub fn is_same_as(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String { .. }, Self::String { .. }) => matches!(other, Self::String { .. }),
            (Self::Optional { kind }, _) => kind.is_same_as(other),
            (_, Self::Optional { kind: other }) => self.is_same_as(other),
            _ => self == other,
        }
    }

    pub fn string() -> Self {
        Kind::String {
            caption: false,
            body: false,
        }
    }

    pub fn caption() -> Self {
        Kind::String {
            caption: true,
            body: false,
        }
    }

    pub fn body() -> Self {
        Kind::String {
            caption: false,
            body: true,
        }
    }

    pub fn caption_or_body() -> Self {
        Kind::String {
            caption: true,
            body: true,
        }
    }

    pub fn optional(k: Self) -> Self {
        Kind::Optional { kind: Box::new(k) }
    }

    pub fn list(k: Self) -> Self {
        Kind::List { kind: Box::new(k) }
    }

    pub fn map(k: Self) -> Self {
        Kind::Map { kind: Box::new(k) }
    }

    pub fn into_optional(self) -> Self {
        Kind::Optional {
            kind: Box::new(self),
        }
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self, Kind::Boolean { .. })
    }

    pub fn inner(&self) -> &Self {
        match self {
            Kind::Optional { kind } => kind,
            _ => self,
        }
    }

    pub fn string_any(&self) -> Self {
        match self {
            Self::String { .. } => Self::String {
                caption: true,
                body: true,
            },
            _ => self.to_owned(),
        }
    }

    pub fn read_section(
        &self,
        p1: &crate::p1::Header,
        p1_caption: &Option<String>,
        p1_body: &Option<String>,
        name: &str,
        doc: &crate::p2::TDoc,
    ) -> crate::p1::Result<crate::PropertyValue> {
        let (v, source) = match p1.str_optional(name)? {
            Some(v) => (v, crate::TextSource::Header),
            None => {
                let (optional, caption, body) = match self {
                    Kind::Optional { kind } => match kind.as_ref() {
                        crate::p2::Kind::String { caption, body } => (true, caption, body),
                        _ => {
                            return Ok(crate::PropertyValue::Value {
                                value: crate::Value::None {
                                    kind: *kind.clone(),
                                },
                            })
                        }
                    },
                    Kind::String { caption, body } => (false, caption, body),
                    t => return crate::e2(format!("`{}` is {:?}", name, t), "two"),
                };

                if *caption && p1_caption.is_some() {
                    (
                        p1_caption.as_ref().expect("asd").as_str(),
                        crate::TextSource::Caption,
                    )
                } else if *body && p1_body.is_some() {
                    (
                        p1_body.as_ref().expect("asd").as_str(),
                        crate::TextSource::Body,
                    )
                } else {
                    if optional {
                        return Ok(crate::PropertyValue::Value {
                            value: crate::Value::None {
                                kind: self.inner().to_owned(),
                            },
                        });
                    }
                    return crate::e2(format!("`{}` is required", name), "one");
                }
            }
        };

        if v.starts_with("ref ") {
            let reference = ftd_rt::get_name("ref", v)?;
            let referred = doc.get_value(reference)?;
            if referred.kind().string_any() != self.string_any() {
                return crate::e(format!(
                    "`{}` is of wrong kind: {:?}, expected: {:?}",
                    name,
                    referred.kind(),
                    self
                ));
            }
            return Ok(crate::PropertyValue::Reference {
                name: doc.resolve_name(reference)?,
                kind: self.to_owned(),
            });
        }

        match self.inner() {
            Kind::Integer => Ok(crate::PropertyValue::Value {
                value: crate::Value::Integer {
                    value: p1.i64(name)?,
                },
            }),
            Kind::Decimal => Ok(crate::PropertyValue::Value {
                value: crate::Value::Decimal {
                    value: p1.f64(name)?,
                },
            }),
            Kind::Boolean => Ok(crate::PropertyValue::Value {
                value: crate::Value::Boolean {
                    value: p1.bool(name)?,
                },
            }),
            Kind::String { .. } => Ok(crate::PropertyValue::Value {
                value: crate::Value::String {
                    text: v.to_string(),
                    source,
                },
            }),
            _ => todo!(),
        }
    }

    pub fn from(s: &str, doc: &crate::p2::TDoc) -> crate::p1::Result<Self> {
        let (optional, k) = if s.starts_with("optional ") {
            (true, ftd_rt::get_name("optional", s)?)
        } else {
            (false, s)
        };

        if k.starts_with("list ") {
            return Ok(Kind::List {
                kind: Box::new(Self::from(ftd_rt::get_name("list", k)?, doc)?),
            });
        }

        let k = match k {
            "string" => Kind::string(),
            "caption" => Kind::caption(),
            "body" => Kind::body(),
            "body or caption" | "caption or body" => Kind::caption_or_body(),
            "integer" => Kind::Integer,
            "decimal" => Kind::Decimal,
            "boolean" => Kind::Boolean,
            "element" => Kind::Element,
            "elements" => Kind::Elements,
            "message" => Kind::Message,
            "string-message" => Kind::StringMessage,
            "int-message" => Kind::IntMessage,
            _ => match doc.get_thing(k)? {
                crate::p2::Thing::Record(r) => Kind::Record { name: r.name },
                crate::p2::Thing::OrType(e) => Kind::OrType { name: e.name },
                t => unimplemented!("{} is {:?}", k, t),
            },
        };

        Ok(if optional { Self::optional(k) } else { k })
    }
}
