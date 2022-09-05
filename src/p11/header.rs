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
    condition: Option<String>,
}

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize, Default)]
#[serde(default)]
pub struct Section {
    line_number: usize,
    key: String,
    kind: Option<String>,
    section: Vec<ftd::p11::Section>,
    condition: Option<String>,
}

pub enum HeaderValue {
    KV(Option<String>),
    Section(Vec<ftd::p11::Section>),
}

impl Header {
    pub fn is_section(&self) -> bool {
        matches!(self, Header::Section(_))
    }

    pub fn get_key(&self) -> String {
        match self {
            Header::KV(KV { key, .. }) => key,
            Header::Section(Section { key, .. }) => key,
        }
        .to_string()
    }

    pub fn get_line_number(&self) -> usize {
        match self {
            Header::KV(KV { line_number, .. }) => *line_number,
            Header::Section(Section { line_number, .. }) => *line_number,
        }
    }

    pub fn get_value(&self, doc_id: &str) -> ftd::p11::Result<Option<String>> {
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
            condition: None,
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
        condition: Option<String>,
    ) -> Header {
        Header::KV(KV {
            line_number,
            key: key.to_string(),
            kind,
            value,
            condition,
        })
    }

    pub(crate) fn section(
        line_number: usize,
        key: &str,
        kind: Option<String>,
        section: Vec<ftd::p11::Section>,
        condition: Option<String>,
    ) -> Header {
        Header::Section(Section {
            line_number,
            key: key.to_string(),
            kind,
            section,
            condition,
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

    pub fn get_header_value(&self) -> HeaderValue {
        match self {
            Header::KV(KV { value, .. }) => HeaderValue::KV(value.to_owned()),
            Header::Section(Section { section, .. }) => HeaderValue::Section(section.to_owned()),
        }
    }

    pub fn get_kind(&self) -> Option<String> {
        match self {
            Header::KV(KV { kind, .. }) => kind,
            Header::Section(Section { kind, .. }) => kind,
        }
        .to_owned()
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
}
