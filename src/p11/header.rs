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

    pub(crate) fn get_key(&self) -> String {
        match self {
            Header::KV(ftd::p11::header::KV { key, .. })
            | Header::Section(ftd::p11::header::Section { key, .. }) => key.to_string(),
        }
    }

    pub(crate) fn get_value(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::p11::Result<Option<String>> {
        match self {
            Header::KV(ftd::p11::header::KV { value, .. }) => Ok(value.to_owned()),
            Header::Section(_) => Err(ftd::p11::Error::ParseError {
                message: format!(
                    "Expected Header of type: KV, found: Section {}",
                    self.get_key()
                ),
                doc_id: doc_id.to_string(),
                line_number,
            }),
        }
    }
}
