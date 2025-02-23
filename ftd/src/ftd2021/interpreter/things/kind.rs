#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    String,
    Object,
    Integer,
    Decimal,
    Boolean,
    Record { name: String }, // the full name of the record (full document name.record name)
    OrType { name: String }, // the full name of the or-type
    OrTypeWithVariant { name: String, variant: String },
    Map { kind: Box<Kind> }, // map of String to Kind
    List { kind: Box<Kind> },
    Optional { kind: Box<Kind> },
    UI,
}

impl Kind {
    pub(crate) fn into_kind_data(self, caption: bool, body: bool) -> KindData {
        KindData {
            kind: self,
            caption,
            body,
        }
    }

    pub(crate) fn get_kind(s: &str) -> Option<Kind> {
        match s {
            "string" => Some(Kind::String),
            "integer" => Some(Kind::Integer),
            "decimal" => Some(Kind::Decimal),
            "object" => Some(Kind::Object),
            "boolean" => Some(Kind::Boolean),
            "ftd.ui" => Some(Kind::UI),
            _ => None,
        }
    }

    pub(crate) fn inner(&self) -> &Self {
        match self {
            Kind::Optional { kind, .. } => kind,
            _ => self,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindData {
    pub kind: Kind,
    pub caption: bool,
    pub body: bool,
}

impl KindData {
    pub(crate) fn from_p1_kind(
        str_kind: &str,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::ftd2021::interpreter::Result<KindData> {
        use itertools::Itertools;

        let mut s = str_kind.split_whitespace().join(" ");
        if s.is_empty() {
            return Err(ftd::ftd2021::interpreter::utils::invalid_kind_error(
                str_kind,
                doc_id,
                line_number,
            ));
        }

        let optional = check_for_optional(&mut s);

        if s.is_empty() {
            return Err(ftd::ftd2021::interpreter::utils::invalid_kind_error(
                str_kind,
                doc_id,
                line_number,
            ));
        }

        let (caption, body) = check_for_caption_and_body(&mut s);

        if s.is_empty() {
            if !(caption || body) {
                return Err(ftd::ftd2021::interpreter::utils::invalid_kind_error(
                    str_kind,
                    doc_id,
                    line_number,
                ));
            }

            let mut kind_data = KindData {
                kind: Kind::String,
                caption,
                body,
            };
            if optional {
                kind_data = kind_data.optional();
            }
            return Ok(kind_data);
        }

        let kind = match check_for_kind(&mut s) {
            Some(kind) => kind,
            _ if caption || body => Kind::String,
            _ => {
                return Err(ftd::ftd2021::interpreter::utils::invalid_kind_error(
                    str_kind,
                    doc_id,
                    line_number,
                ));
            }
        };

        let list = check_for_list(&mut s);

        let mut kind_data = KindData {
            kind,
            caption,
            body,
        };

        if optional {
            kind_data = kind_data.optional();
        }

        if list {
            kind_data = kind_data.list();
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

    pub(crate) fn boolean() -> KindData {
        KindData {
            kind: Kind::Boolean,
            caption: false,
            body: false,
        }
    }

    fn integer() -> KindData {
        KindData {
            kind: Kind::Integer,
            caption: false,
            body: false,
        }
    }

    fn string() -> KindData {
        KindData {
            kind: Kind::String,
            caption: false,
            body: false,
        }
    }

    fn caption(self) -> KindData {
        KindData {
            kind: self.kind,
            caption: true,
            body: self.body,
        }
    }

    fn body(self) -> KindData {
        KindData {
            kind: self.kind,
            caption: self.caption,
            body: true,
        }
    }
}

pub fn check_for_optional(s: &mut String) -> bool {
    use itertools::Itertools;

    let expr = s.split_whitespace().collect_vec();

    if expr.is_empty() {
        return false;
    }

    if is_optional(expr[0]) {
        *s = expr[1..].join(" ");
        return true;
    }

    false
}

pub(crate) fn is_optional(s: &str) -> bool {
    s.eq("optional")
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

pub fn check_for_kind(s: &mut String) -> Option<Kind> {
    use itertools::Itertools;

    let expr = s.split_whitespace().collect_vec();

    if expr.is_empty() {
        return None;
    }

    let kind = Kind::get_kind(expr[0])?;

    *s = expr[1..].join(" ");

    Some(kind)
}

pub fn check_for_list(s: &mut String) -> bool {
    use itertools::Itertools;

    let expr = s.split_whitespace().collect_vec();

    if expr.is_empty() {
        return false;
    }

    if is_list(expr[0]) {
        *s = expr[1..].join(" ");
        return true;
    }

    false
}

pub(crate) fn is_list(s: &str) -> bool {
    s.eq("list")
}

#[cfg(test)]
mod test {
    #[track_caller]
    fn p(s: &str, t: super::KindData) {
        assert_eq!(
            super::KindData::from_p1_kind(s, "foo", 0).unwrap_or_else(|e| panic!("{:?}", e)),
            t
        )
    }

    #[track_caller]
    fn f(s: &str, m: &str) {
        match super::KindData::from_p1_kind(s, "foo", 0) {
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
        p("integer", super::KindData::integer())
    }

    #[test]
    fn caption_integer() {
        p("caption integer", super::KindData::integer().caption())
    }

    #[test]
    fn caption_or_body_integer() {
        p(
            "caption or body integer",
            super::KindData::integer().caption().body(),
        );

        p(
            "body or caption integer",
            super::KindData::integer().caption().body(),
        );
    }

    #[test]
    fn integer_list() {
        p("integer list", super::KindData::integer().list())
    }

    #[test]
    fn optional_integer() {
        p("optional integer", super::KindData::integer().optional())
    }

    #[test]
    fn optional_failure() {
        f("optional", "InvalidKind: foo:0 -> optional");
    }

    #[test]
    fn caption() {
        p("caption", super::KindData::string().caption());

        p("caption string", super::KindData::string().caption());
    }

    #[test]
    fn caption_or_body() {
        p(
            "caption or body",
            super::KindData::string().caption().body(),
        );

        p(
            "body or caption",
            super::KindData::string().caption().body(),
        );

        p(
            "caption or body string",
            super::KindData::string().caption().body(),
        );

        p(
            "body or caption string",
            super::KindData::string().caption().body(),
        );
    }
}
