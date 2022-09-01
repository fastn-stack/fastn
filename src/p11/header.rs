use itertools::Itertools;

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub enum Header {
    KV {
        line_number: usize,
        key: String,
        kind: Option<String>,
        value: Option<String>,
    },
    Section {
        line_number: usize,
        key: String,
        kind: Option<String>,
        section: Vec<ftd::p11::Section>,
    },
}

impl Header {
    pub(crate) fn from_string(
        key: &str,
        kind: Option<String>,
        value: &str,
        line_number: usize,
    ) -> Header {
        Header::KV {
            line_number,
            key: key.to_string(),
            kind,
            value: Some(value.to_string()),
        }
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
        Header::KV {
            line_number,
            key: key.to_string(),
            kind,
            value,
        }
    }

    pub(crate) fn section(
        line_number: usize,
        key: &str,
        kind: Option<String>,
        section: Vec<ftd::p11::Section>,
    ) -> Header {
        Header::Section {
            line_number,
            key: key.to_string(),
            kind,
            section,
        }
    }

    pub fn without_line_number(&self) -> Self {
        match self {
            Header::KV {
                line_number: _,
                key,
                kind,
                value,
            } => Header::KV {
                line_number: 0,
                key: key.to_owned(),
                kind: kind.to_owned(),
                value: value.to_owned(),
            },
            Header::Section {
                line_number: _,
                key,
                kind,
                section,
            } => Header::Section {
                line_number: 0,
                key: key.to_owned(),
                kind: kind.to_owned(),
                section: section
                    .iter()
                    .map(|v| v.without_line_number())
                    .collect_vec(),
            },
        }
    }
}
