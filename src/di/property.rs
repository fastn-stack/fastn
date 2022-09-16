#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Property {
    name: String,
    kind: Option<String>,
    value: PropertyValue,
    source: Source,
}

impl Property {
    pub(crate) fn from_p1(
        section: &ftd::p11::Section,
        doc_id: &str,
    ) -> ftd::di::Result<Vec<Property>> {
        let mut properties = vec![];
        for header in section.headers.0.iter() {
            properties.push(Property::from_header(header, doc_id, Source::Header)?)
        }
        if let Some(ref caption) = section.caption {
            properties.push(Property::from_header(caption, doc_id, Source::Caption)?)
        }
        if let Some(ref body) = section.body {
            properties.push(Property::from_body(body.value.as_str()))
        }

        Ok(properties)
    }

    pub(crate) fn from_header(
        header: &ftd::p11::Header,
        doc_id: &str,
        source: Source,
    ) -> ftd::di::Result<Property> {
        match header {
            ftd::p11::Header::KV(kv) => Ok(Property::from_kv(kv, source)),
            ftd::p11::Header::Section(section) => Property::from_section(section, doc_id, source),
        }
    }

    pub(crate) fn from_kv(kv: &ftd::p11::header::KV, source: Source) -> Property {
        Property {
            name: kv.key.to_string(),
            kind: kv.kind.clone(),
            value: PropertyValue::Value(kv.value.clone()),
            source,
        }
    }

    #[cfg(test)]
    pub(crate) fn from_di_list(
        key: &str,
        kind: Option<String>,
        di: Vec<ftd::di::DI>,
        source: Source,
    ) -> Property {
        Property {
            name: key.to_string(),
            kind,
            value: PropertyValue::DI(di),
            source,
        }
    }

    pub(crate) fn from_body(body: &str) -> Property {
        Property {
            name: ftd::di::utils::BODY.to_string(),
            kind: None,
            value: PropertyValue::Value(Some(body.to_string())),
            source: Source::Body,
        }
    }

    #[cfg(test)]
    pub(crate) fn from_caption_str(caption: &str) -> Property {
        Property {
            name: ftd::di::utils::CAPTION.to_string(),
            kind: None,
            value: PropertyValue::Value(Some(caption.to_string())),
            source: Source::Caption,
        }
    }

    pub(crate) fn from_section(
        section: &ftd::p11::header::Section,
        doc_id: &str,
        source: Source,
    ) -> ftd::di::Result<Property> {
        let di = ftd::di::DI::from_sections(section.section.as_slice(), doc_id)?;
        Ok(Property {
            name: section.key.to_string(),
            kind: section.kind.clone(),
            value: PropertyValue::DI(di),
            source,
        })
    }
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "pv", content = "value")]
pub enum PropertyValue {
    Value(Option<String>),
    DI(Vec<ftd::di::DI>),
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "source")]
pub enum Source {
    Header,
    Caption,
    Body,
}
