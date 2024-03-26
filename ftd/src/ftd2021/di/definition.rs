#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Definition {
    name: String,
    kind: String,
    properties: Vec<ftd::ftd2021::di::Property>,
    children: Vec<ftd::ftd2021::di::DI>,
}

impl Definition {
    pub(crate) fn is_definition(section: &ftd::p1::Section) -> bool {
        if ftd::ftd2021::di::Import::is_import(section)
            || ftd::ftd2021::di::Record::is_record(section)
        {
            return false;
        }
        section.kind.is_some()
    }

    pub(crate) fn from_p1(
        section: &ftd::p1::Section,
        doc_id: &str,
    ) -> ftd::ftd2021::di::Result<Definition> {
        if !Self::is_definition(section) {
            return ftd::ftd2021::di::parse_error(
                format!("Section is not `definition`, found `{:?}`", section),
                doc_id,
                section.line_number,
            );
        }

        let properties = ftd::ftd2021::di::Property::from_p1(section, doc_id)?;
        let children =
            ftd::ftd2021::di::DI::from_sections(section.sub_sections.as_slice(), doc_id)?;

        let kind = if let Some(ref kind) = section.kind {
            kind
        } else {
            return ftd::ftd2021::di::parse_error(
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
        definition
            .properties
            .push(ftd::ftd2021::di::Property::from_body(s));
        definition
    }

    #[cfg(test)]
    pub(crate) fn add_value_property(
        self,
        key: &str,
        kind: Option<String>,
        value: Option<String>,
        condition: Option<String>,
    ) -> Definition {
        let mut definition = self;
        definition
            .properties
            .push(ftd::ftd2021::di::Property::from_kv(
                &ftd0::p1::KV::new(key, kind, value, 0, condition, Default::default()),
                ftd::ftd2021::di::Source::Header,
            ));
        definition
    }

    #[cfg(test)]
    pub(crate) fn add_di_property(
        self,
        key: &str,
        kind: Option<String>,
        di: Vec<ftd::ftd2021::di::DI>,
    ) -> Definition {
        let mut definition = self;
        definition
            .properties
            .push(ftd::ftd2021::di::Property::from_di_list(
                key,
                kind,
                di,
                ftd::ftd2021::di::Source::Header,
            ));
        definition
    }

    #[cfg(test)]
    pub(crate) fn add_caption_str(self, s: &str) -> Definition {
        let mut definition = self;
        definition
            .properties
            .push(ftd::ftd2021::di::Property::from_caption_str(s));
        definition
    }

    #[cfg(test)]
    pub(crate) fn add_child(self, di: ftd::ftd2021::di::DI) -> Definition {
        let mut definition = self;
        definition.children.push(di);
        definition
    }
}
