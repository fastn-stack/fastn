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
    UI,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct KindData {
    pub kind: Kind,
    pub caption: bool,
    pub body: bool,
}

impl KindData {
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
            "ftd.ui" => Kind::UI,
            k => match doc.get_thing(line_number, k)? {
                ftd::interpreter2::Thing::Record(r) => Kind::Record { name: r.name },
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
        matches!(self.kind, Kind::List { .. })
    }

    pub fn is_optional(&self) -> bool {
        matches!(self.kind, Kind::Optional { .. })
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
