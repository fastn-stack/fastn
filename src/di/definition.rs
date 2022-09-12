use crate::p11::Header;

#[derive(Debug, PartialEq)]
pub struct Definition {
    name: String,
    kind: String,
    properties: Vec<Property>,
    children: Vec<ftd::di::DI>,
}

impl Definition {
    pub(crate) fn is_definition(section: &ftd::p11::Section) -> bool {
        if ftd::di::Import::is_import(section) || ftd::di::Record::is_record(section) {
            return false;
        }
        section.kind.is_some()
    }

    pub(crate) fn from_p1(
        section: &ftd::p11::Section,
        doc_id: &str,
    ) -> ftd::di::Result<Definition> {
        if !Self::is_definition(section) {
            return ftd::di::parse_error(
                format!("Section is not `definition`, found `{:?}`", section),
                doc_id,
                section.line_number,
            );
        }

        let properties = Property::from_p1(section, doc_id)?;
        let children = ftd::di::DI::from_sections(section.sub_sections.as_slice(), doc_id)?;

        let kind = if let Some(ref kind) = section.kind {
            kind
        } else {
            return ftd::di::parse_error(
                format!(
                    "Section is not `definition`, kind not found, found `{:?}`",
                    section
                ),
                doc_id,
                section.line_number,
            );
        };

        Ok(Definition {
            name: section.name.to_string(),
            kind: kind.to_string(),
            properties,
            children,
        })
    }

    #[cfg(test)]
    pub(crate) fn new(name: &str, kind: &str) -> Definition {
        Definition {
            name: name.to_string(),
            kind: kind.to_string(),
            properties: vec![],
            children: vec![],
        }
    }

    #[cfg(test)]
    pub(crate) fn add_body(self, s: &str) -> Definition {
        let mut definition = self;
        definition.properties.push(Property::from_body(s));
        definition
    }

    #[cfg(test)]
    pub(crate) fn add_value_property(
        self,
        key: &str,
        kind: Option<String>,
        value: Option<String>,
    ) -> Definition {
        let mut definition = self;
        definition.properties.push(Property::from_kv(
            &ftd::p11::header::KV::new(key, kind, value, 0),
            Source::Header,
        ));
        definition
    }

    #[cfg(test)]
    pub(crate) fn add_caption_str(self, s: &str) -> Definition {
        let mut definition = self;
        definition.properties.push(Property::from_caption_str(s));
        definition
    }
}

#[derive(Debug, PartialEq)]
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
            Header::KV(kv) => Ok(Property::from_kv(kv, source)),
            Header::Section(section) => Property::from_section(section, doc_id, source),
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

#[derive(Debug, PartialEq)]
pub enum PropertyValue {
    Value(Option<String>),
    DI(Vec<ftd::di::DI>),
}

#[derive(Debug, PartialEq)]
pub enum Source {
    Header,
    Caption,
    Body,
}
