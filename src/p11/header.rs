#[derive(Debug, PartialEq, Clone, serde::Serialize)]
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
}
