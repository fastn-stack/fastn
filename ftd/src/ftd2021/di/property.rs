#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Property {
    name: String,
    kind: Option<String>,
    value: PropertyValue,
    source: Source,
}

impl Property {
    pub(crate) fn from_p1(
        section: &ftd_p1::Section,
        doc_id: &str,
    ) -> ftd::ftd2021::di::Result<Vec<Property>> {
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
        header: &ftd_p1::Header,
        doc_id: &str,
        source: Source,
    ) -> ftd::ftd2021::di::Result<Property> {
        match header {
            ftd_p1::Header::KV(kv) => Ok(Property::from_kv(kv, source)),
            ftd_p1::Header::Section(section) => Property::from_section(section, doc_id, source),
            ftd_p1::Header::BlockRecordHeader(_) => todo!(),
        }
    }

    pub(crate) fn from_kv(kv: &ftd_p1::KV, source: Source) -> Property {
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
        di: Vec<ftd::ftd2021::di::DI>,
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
            name: ftd::ftd2021::di::utils::BODY.to_string(),
            kind: None,
            value: PropertyValue::Value(Some(body.to_string())),
            source: Source::Body,
        }
    }

    #[cfg(test)]
    pub(crate) fn from_caption_str(caption: &str) -> Property {
        Property {
            name: ftd::ftd2021::di::utils::CAPTION.to_string(),
            kind: None,
            value: PropertyValue::Value(Some(caption.to_string())),
            source: Source::Caption,
        }
    }

    pub(crate) fn from_section(
        section: &ftd_p1::SectionHeader,
        doc_id: &str,
        source: Source,
    ) -> ftd::ftd2021::di::Result<Property> {
        let di = ftd::ftd2021::di::DI::from_sections(section.section.as_slice(), doc_id)?;
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
    DI(Vec<ftd::ftd2021::di::DI>),
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "source")]
pub enum Source {
    Header,
    Caption,
    Body,
}
