use itertools::Itertools;

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Header {
    KV(ftd::p11::header::KV),
    Section(ftd::p11::header::Section),
}

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize, Default)]
#[serde(default)]
pub struct KV {
    line_number: usize,
    key: String,
    kind: Option<String>,
    value: Option<String>,
}

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize, Default)]
#[serde(default)]
pub struct Section {
    line_number: usize,
    key: String,
    kind: Option<String>,
    section: Vec<ftd::p11::Section>,
}

impl Header {
    pub(crate) fn is_section(&self) -> bool {
        matches!(self, Header::Section(_))
    }

    pub(crate) fn get_key(&self) -> String {
        match self {
            Header::KV(KV { key, .. }) => key,
            Header::Section(Section { key, .. }) => key,
        }
        .to_string()
    }

    pub(crate) fn get_line_number(&self) -> usize {
        match self {
            Header::KV(KV { line_number, .. }) => *line_number,
            Header::Section(Section { line_number, .. }) => *line_number,
        }
    }

    pub(crate) fn get_value(&self, doc_id: &str) -> ftd::p11::Result<Option<String>> {
        match self {
            Header::KV(KV { value, .. }) => Ok(value.to_owned()),
            t => {
                return ftd::interpreter::utils::e2(
                    format!("Expected key value for header, found: {:?}", t),
                    doc_id,
                    t.get_line_number(),
                )
            }
        }
    }

    pub(crate) fn from_string(
        key: &str,
        kind: Option<String>,
        value: &str,
        line_number: usize,
    ) -> Header {
        Header::KV(KV {
            line_number,
            key: key.to_string(),
            kind,
            value: Some(value.to_string()),
        })
    }

    pub(crate) fn from_caption(value: &str, line_number: usize) -> Header {
        Header::from_string("$caption$", None, value, line_number)
    }

    pub(crate) fn kv(
        line_number: usize,
        key: &str,
        kind: Option<String>,
        value: Option<String>,
    ) -> Header {
        Header::KV(KV {
            line_number,
            key: key.to_string(),
            kind,
            value,
        })
    }

    pub(crate) fn section(
        line_number: usize,
        key: &str,
        kind: Option<String>,
        section: Vec<ftd::p11::Section>,
    ) -> Header {
        Header::Section(Section {
            line_number,
            key: key.to_string(),
            kind,
            section,
        })
    }

    pub fn without_line_number(&self) -> Self {
        match self {
            Header::KV(kv) => {
                let mut kv = (*kv).clone();
                kv.line_number = 0;
                Header::KV(kv)
            }
            Header::Section(s) => {
                let mut s = (*s).clone();
                s.line_number = 0;
                s.section = s
                    .section
                    .iter()
                    .map(|v| v.without_line_number())
                    .collect_vec();
                Header::Section(s)
            }
        }
    }
}

pub struct Headers(pub Vec<Header>);

impl Headers {
    pub fn str(
        &self,
        doc_id: &str,
        line_number: usize,
        name: &str,
    ) -> ftd::p11::Result<&Option<String>> {
        for header in self.0.iter() {
            let (k, v) =
                if let ftd::p11::Header::KV(ftd::p11::header::KV { key, value, .. }) = header {
                    (key, value)
                } else {
                    return Err(ftd::p11::Error::NotFound {
                        doc_id: doc_id.to_string(),
                        line_number,
                        key: format!("`{}` header has section, expected: key value", name),
                    });
                };
            if k == name {
                return Ok(v);
            }
        }

        Err(ftd::p11::Error::NotFound {
            doc_id: doc_id.to_string(),
            line_number,
            key: format!("`{}` header is missing", name),
        })
    }

    pub fn push(&mut self, header: ftd::p11::Header) {
        self.0.push(header);
    }

    #[allow(clippy::type_complexity)]
    pub fn conditional_str(
        &self,
        doc: &ftd::interpreter::TDoc,
        line_number: usize,
        name: &str,
        arguments: &ftd::Map<ftd::interpreter::Kind>,
    ) -> ftd::p11::Result<Vec<(usize, String, Option<String>, bool)>> {
        let mut conditional_vector = vec![];
        for (idx, header) in self.0.iter().enumerate() {
            let k = header.get_key();
            let v = header.get_value(doc.name)?;
            let v = doc.resolve_reference_name(line_number, v, arguments)?;
            let (k, is_referenced) = if let Some(k) = k.strip_prefix('$') {
                (k.to_string(), true)
            } else {
                (k.to_string(), false)
            };
            if k.eq(name) {
                conditional_vector.push((idx, v.to_string(), None, is_referenced));
            }
            if k.contains(" if ") {
                let mut parts = k.splitn(2, " if ");
                let property_name = parts.next().unwrap().trim();
                if property_name == name {
                    let conditional_attribute = parts.next().unwrap().trim();
                    conditional_vector.push((
                        idx,
                        v.to_string(),
                        Some(conditional_attribute.to_string()),
                        is_referenced,
                    ));
                }
            }
        }
        if conditional_vector.is_empty() {
            Err(ftd::p11::Error::NotFound {
                doc_id: doc.name.to_string(),
                line_number,
                key: format!("`{}` header is missing", name),
            })
        } else {
            Ok(conditional_vector)
        }
    }
}
