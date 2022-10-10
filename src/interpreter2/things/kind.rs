#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Kind {
    String,
    Object,
    Integer,
    Decimal,
    Boolean,
    Record { name: String }, // the full name of the record (full document name.record name)
    List { kind: Box<Kind> },
    Optional { kind: Box<Kind> },
    UI { name: Option<String> },
}

impl Kind {
    pub fn into_kind_data(self) -> KindData {
        KindData::new(self)
    }

    pub fn is_same_as(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::UI { .. }, Self::UI { .. }) => matches!(other, Self::UI { .. }),
            (Self::Optional { kind, .. }, _) => kind.is_same_as(other),
            (_, Self::Optional { kind: other, .. }) => self.is_same_as(other),
            _ => self.eq(other),
        }
    }

    pub fn string() -> Kind {
        Kind::String
    }

    pub fn integer() -> Kind {
        Kind::Integer
    }

    pub fn decimal() -> Kind {
        Kind::Decimal
    }

    pub fn boolean() -> Kind {
        Kind::Boolean
    }

    pub fn record(name: &str) -> Kind {
        Kind::Record {
            name: name.to_string(),
        }
    }

    pub fn into_optional(self) -> Kind {
        Kind::Optional {
            kind: Box::new(self),
        }
    }

    pub fn inner(self) -> Kind {
        match self {
            Kind::Optional { kind } => kind.as_ref().to_owned(),
            t => t,
        }
    }

    pub(crate) fn is_list(&self) -> bool {
        matches!(self, Kind::List { .. })
    }

    pub fn is_optional(&self) -> bool {
        matches!(self, Kind::Optional { .. })
    }

    pub fn get_record_name(&self) -> Option<&str> {
        match self {
            ftd::interpreter2::Kind::Record { ref name, .. } => Some(name),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct KindData {
    pub kind: Kind,
    pub caption: bool,
    pub body: bool,
}

impl KindData {
    pub fn new(kind: Kind) -> KindData {
        KindData {
            kind,
            caption: false,
            body: false,
        }
    }

    pub fn caption(self) -> KindData {
        let mut kind = self;
        kind.caption = true;
        kind
    }

    pub fn body(self) -> KindData {
        let mut kind = self;
        kind.body = true;
        kind
    }

    pub fn caption_or_body(self) -> KindData {
        let mut kind = self;
        kind.caption = true;
        kind.body = true;
        kind
    }

    pub(crate) fn into_by_ast_modifier(self, modifier: &ftd::ast::VariableModifier) -> Self {
        match modifier {
            ftd::ast::VariableModifier::Optional => self.optional(),
            ftd::ast::VariableModifier::List => self.list(),
        }
    }
    pub(crate) fn from_ast_kind(
        var_kind: ftd::ast::VariableKind,
        doc: &ftd::interpreter2::TDoc,
        line_number: usize,
    ) -> ftd::interpreter2::Result<KindData> {
        let mut ast_kind = var_kind.kind.clone();
        let (caption, body) = check_for_caption_and_body(&mut ast_kind);
        if ast_kind.is_empty() {
            if !(caption || body) {
                return Err(ftd::interpreter2::utils::invalid_kind_error(
                    ast_kind,
                    doc.name,
                    line_number,
                ));
            }

            let mut kind_data = KindData {
                kind: Kind::String,
                caption,
                body,
            };

            if let Some(ref modifier) = var_kind.modifier {
                kind_data = kind_data.into_by_ast_modifier(modifier);
            }

            return Ok(kind_data);
        }
        let kind = match ast_kind.as_ref() {
            "string" => Kind::String,
            "object" => Kind::Object,
            "integer" => Kind::Integer,
            "decimal" => Kind::Decimal,
            "boolean" => Kind::Boolean,
            "ftd.ui" => Kind::UI { name: None },
            k => match doc.get_thing(k, line_number)? {
                ftd::interpreter2::Thing::Record(r) => Kind::Record { name: r.name },
                t => {
                    return ftd::interpreter2::utils::e2(
                        format!("Can't get find for `{:?}`", t),
                        doc.name,
                        line_number,
                    )
                }
            },
        };

        let mut kind_data = KindData {
            kind,
            caption,
            body,
        };

        if let Some(ref modifier) = var_kind.modifier {
            kind_data = kind_data.into_by_ast_modifier(modifier);
        }

        Ok(kind_data)
    }

    fn optional(self) -> KindData {
        KindData {
            kind: Kind::Optional {
                kind: Box::new(self.kind),
            },
            caption: self.caption,
            body: self.body,
        }
    }

    fn list(self) -> KindData {
        KindData {
            kind: Kind::List {
                kind: Box::new(self.kind),
            },
            caption: self.caption,
            body: self.body,
        }
    }

    pub fn is_list(&self) -> bool {
        self.kind.is_list()
    }

    pub fn is_optional(&self) -> bool {
        self.kind.is_optional()
    }
}

pub fn check_for_caption_and_body(s: &mut String) -> (bool, bool) {
    use itertools::Itertools;

    let mut caption = false;
    let mut body = false;

    let mut expr = s.split_whitespace().collect_vec();

    if expr.is_empty() {
        return (caption, body);
    }

    if is_caption_or_body(expr.as_slice()) {
        caption = true;
        body = true;
        expr = expr[3..].to_vec();
    } else if is_caption(expr[0]) {
        caption = true;
        expr = expr[1..].to_vec();
    } else if is_body(expr[0]) {
        body = true;
        expr = expr[1..].to_vec();
    }

    *s = expr.join(" ");

    (caption, body)
}

pub(crate) fn is_caption_or_body(expr: &[&str]) -> bool {
    if expr.len() < 3 {
        return false;
    }
    if is_caption(expr[0]) && expr[1].eq("or") && is_body(expr[2]) {
        return true;
    }

    if is_body(expr[0]) && expr[1].eq("or") && is_caption(expr[2]) {
        return true;
    }

    false
}

pub(crate) fn is_caption(s: &str) -> bool {
    s.eq("caption")
}

pub fn is_body(s: &str) -> bool {
    s.eq("body")
}
