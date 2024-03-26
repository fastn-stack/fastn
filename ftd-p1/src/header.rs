#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Header {
    KV(ftd_p1::header::KV),
    Section(ftd_p1::header::SectionHeader),
    BlockRecordHeader(ftd_p1::header::BlockRecordHeader),
}

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize, Default)]
#[serde(default)]
pub struct BlockRecordHeader {
    pub key: String,
    pub kind: Option<String>,
    pub caption: Option<String>,
    pub body: (Option<String>, Option<usize>),
    pub fields: Vec<Header>,
    pub condition: Option<String>,
    pub line_number: usize,
}

impl BlockRecordHeader {
    pub fn new(
        key: String,
        kind: Option<String>,
        caption: Option<String>,
        body: (Option<String>, Option<usize>),
        fields: Vec<Header>,
        condition: Option<String>,
        line_number: usize,
    ) -> BlockRecordHeader {
        BlockRecordHeader {
            key,
            kind,
            caption,
            body,
            fields,
            condition,
            line_number,
        }
    }
}

#[derive(Debug, Default, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub enum KVSource {
    Caption,
    Body,
    #[default]
    Header,
}

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize, Default)]
#[serde(default)]
pub struct KV {
    pub line_number: usize,
    pub key: String,
    pub kind: Option<String>,
    pub value: Option<String>,
    pub condition: Option<String>,
    pub access_modifier: AccessModifier,
    pub source: KVSource,
}

impl KV {
    pub fn new(
        key: &str,
        mut kind: Option<String>,
        value: Option<String>,
        line_number: usize,
        condition: Option<String>,
        source: Option<KVSource>,
    ) -> KV {
        let mut access_modifier = AccessModifier::Public;
        if let Some(k) = kind.as_ref() {
            let (rest_kind, access) = AccessModifier::get_kind_and_modifier(k.as_str());
            kind = Some(rest_kind);
            access_modifier = access.unwrap_or(AccessModifier::Public);
        }

        KV {
            line_number,
            key: key.to_string(),
            kind,
            value,
            condition,
            access_modifier,
            source: source.unwrap_or_default(),
        }
    }
}

#[derive(Debug, Default, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub enum AccessModifier {
    #[default]
    Public,
    Private,
}

impl AccessModifier {
    pub fn is_public(&self) -> bool {
        matches!(self, AccessModifier::Public)
    }

    pub fn remove_modifiers(name: &str) -> String {
        let mut result = vec![];
        for part in name.split(' ') {
            if !AccessModifier::is_modifier(part) {
                result.push(part)
            }
        }
        result.join(" ")
    }

    pub fn is_modifier(s: &str) -> bool {
        matches!(s, "public" | "private")
    }

    pub fn get_modifier_from_string(modifier: &str) -> Option<AccessModifier> {
        match modifier {
            "public" => Some(AccessModifier::Public),
            "private" => Some(AccessModifier::Private),
            _ => None,
        }
    }

    pub fn get_kind_and_modifier(kind: &str) -> (String, Option<AccessModifier>) {
        let mut access_modifier: Option<AccessModifier> = None;

        let mut rest_kind = vec![];
        for part in kind.split(' ') {
            if !AccessModifier::is_modifier(part) {
                rest_kind.push(part);
                continue;
            }
            access_modifier = AccessModifier::get_modifier_from_string(part)
        }
        (rest_kind.join(" "), access_modifier)
    }
}

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize, Default)]
#[serde(default)]
pub struct SectionHeader {
    pub line_number: usize,
    pub key: String,
    pub kind: Option<String>,
    pub section: Vec<ftd_p1::Section>,
    pub condition: Option<String>,
}

impl Header {
    pub fn from_caption(value: &str, line_number: usize) -> Header {
        Header::kv(
            line_number,
            ftd_p1::utils::CAPTION,
            None,
            Some(value.to_string()),
            None,
            Some(KVSource::Caption),
        )
    }

    pub fn kv(
        line_number: usize,
        key: &str,
        kind: Option<String>,
        value: Option<String>,
        condition: Option<String>,
        source: Option<KVSource>,
    ) -> Header {
        Header::KV(KV::new(key, kind, value, line_number, condition, source))
    }

    pub fn section(
        line_number: usize,
        key: &str,
        kind: Option<String>,
        section: Vec<ftd_p1::Section>,
        condition: Option<String>,
    ) -> Header {
        Header::Section(SectionHeader {
            line_number,
            key: key.to_string(),
            kind,
            section,
            condition,
        })
    }

    pub fn block_record_header(
        key: &str,
        kind: Option<String>,
        caption: Option<String>,
        body: (Option<String>, Option<usize>),
        fields: Vec<Header>,
        condition: Option<String>,
        line_number: usize,
    ) -> Header {
        Header::BlockRecordHeader(BlockRecordHeader::new(
            key.to_string(),
            kind,
            caption,
            body,
            fields,
            condition,
            line_number,
        ))
    }

    pub fn without_line_number(&self) -> Self {
        use itertools::Itertools;

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
            Header::BlockRecordHeader(b) => {
                let mut blockrecord = (*b).clone();
                blockrecord.line_number = 0;
                Header::BlockRecordHeader(blockrecord)
            }
        }
    }

    pub fn get_key(&self) -> String {
        match self {
            Header::KV(ftd_p1::header::KV { key, .. })
            | Header::Section(ftd_p1::header::SectionHeader { key, .. })
            | Header::BlockRecordHeader(ftd_p1::header::BlockRecordHeader { key, .. }) => {
                key.to_string()
            }
        }
    }

    pub fn get_access_modifier(&self) -> AccessModifier {
        match self {
            Header::KV(ftd_p1::header::KV {
                access_modifier, ..
            }) => access_modifier.clone(),
            Header::Section(ftd_p1::header::SectionHeader { .. })
            | Header::BlockRecordHeader(ftd_p1::header::BlockRecordHeader { .. }) => {
                AccessModifier::Public
            }
        }
    }

    pub(crate) fn set_key(&mut self, value: &str) {
        match self {
            Header::KV(ftd_p1::header::KV { key, .. })
            | Header::Section(ftd_p1::header::SectionHeader { key, .. })
            | Header::BlockRecordHeader(ftd_p1::header::BlockRecordHeader { key, .. }) => {
                *key = value.to_string();
            }
        }
    }

    pub fn set_kind(&mut self, value: &str) {
        match self {
            Header::KV(ftd_p1::header::KV {
                kind: Some(kind), ..
            })
            | Header::Section(ftd_p1::header::SectionHeader {
                kind: Some(kind), ..
            })
            | Header::BlockRecordHeader(ftd_p1::header::BlockRecordHeader {
                kind: Some(kind),
                ..
            }) => {
                *kind = value.to_string();
            }
            _ => {}
        }
    }

    pub fn get_value(&self, doc_id: &str) -> ftd_p1::Result<Option<String>> {
        match self {
            Header::KV(ftd_p1::header::KV { value, .. }) => Ok(value.to_owned()),
            Header::Section(_) => Err(ftd_p1::Error::ParseError {
                message: format!(
                    "Expected Header of type: KV, found: Section {}",
                    self.get_key()
                ),
                doc_id: doc_id.to_string(),
                line_number: self.get_line_number(),
            }),
            Header::BlockRecordHeader(_) => Err(ftd_p1::Error::ParseError {
                message: format!(
                    "Expected Header of type: KV, found: BlockRecordHeader {}",
                    self.get_key()
                ),
                doc_id: doc_id.to_string(),
                line_number: self.get_line_number(),
            }),
        }
    }

    pub fn get_sections(&self, doc_id: &str) -> ftd_p1::Result<&Vec<ftd_p1::Section>> {
        match self {
            Header::KV(_) | Header::BlockRecordHeader(_) => Err(ftd_p1::Error::ParseError {
                message: format!(
                    "Expected Header of type: Sections, found: KV {}",
                    self.get_key()
                ),
                doc_id: doc_id.to_string(),
                line_number: self.get_line_number(),
            }),
            Header::Section(ftd_p1::header::SectionHeader { section, .. }) => Ok(section),
        }
    }

    pub fn get_line_number(&self) -> usize {
        match self {
            Header::KV(ftd_p1::header::KV { line_number, .. })
            | Header::Section(ftd_p1::header::SectionHeader { line_number, .. })
            | Header::BlockRecordHeader(ftd_p1::header::BlockRecordHeader {
                line_number, ..
            }) => *line_number,
        }
    }

    pub fn get_kind(&self) -> Option<String> {
        match self {
            Header::KV(ftd_p1::header::KV { kind, .. })
            | Header::Section(ftd_p1::header::SectionHeader { kind, .. })
            | Header::BlockRecordHeader(ftd_p1::header::BlockRecordHeader { kind, .. }) => {
                kind.to_owned()
            }
        }
    }

    pub fn is_module_kind(&self) -> bool {
        match self {
            Header::KV(ftd_p1::header::KV { kind, .. })
            | Header::Section(ftd_p1::header::SectionHeader { kind, .. })
            | Header::BlockRecordHeader(ftd_p1::header::BlockRecordHeader { kind, .. }) => {
                match kind {
                    Some(k) => k.trim().eq("module"),
                    None => false,
                }
            }
        }
    }

    pub fn get_condition(&self) -> Option<String> {
        match self {
            Header::KV(ftd_p1::header::KV { condition, .. })
            | Header::Section(ftd_p1::header::SectionHeader { condition, .. })
            | Header::BlockRecordHeader(ftd_p1::header::BlockRecordHeader { condition, .. }) => {
                condition.to_owned()
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Header::KV(ftd_p1::header::KV { value, .. }) => value.is_none(),
            Header::Section(ftd_p1::header::SectionHeader { section, .. }) => section.is_empty(),
            Header::BlockRecordHeader(ftd_p1::header::BlockRecordHeader { fields, .. }) => {
                fields.is_empty()
            }
        }
    }

    pub fn remove_comments(&self) -> Option<Header> {
        let mut header = self.clone();
        let key = header.get_key().trim().to_string();
        if let Some(kind) = header.get_kind() {
            if kind.starts_with('/') {
                return None;
            }
            if key.starts_with(r"\/") {
                header.set_kind(kind.trim_start_matches('\\'));
            }
        } else {
            if key.starts_with('/') {
                return None;
            }

            if key.starts_with(r"\/") {
                header.set_key(key.trim_start_matches('\\'));
            }
        }

        match &mut header {
            Header::KV(ftd_p1::header::KV { .. })
            | Header::BlockRecordHeader(ftd_p1::header::BlockRecordHeader { .. }) => {}
            Header::Section(ftd_p1::header::SectionHeader { section, .. }) => {
                *section = section
                    .iter_mut()
                    .filter_map(|s| s.remove_comments())
                    .collect();
            }
        }
        Some(header)
    }
}

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Headers(pub Vec<Header>);

impl Headers {
    pub fn find(&self, key: &str) -> Vec<&ftd_p1::Header> {
        use itertools::Itertools;

        self.0.iter().filter(|v| v.get_key().eq(key)).collect_vec()
    }

    pub fn find_once(
        &self,
        key: &str,
        doc_id: &str,
        line_number: usize,
    ) -> ftd_p1::Result<&ftd_p1::Header> {
        let headers = self.find(key);
        let header = headers.first().ok_or(ftd_p1::Error::HeaderNotFound {
            key: key.to_string(),
            doc_id: doc_id.to_string(),
            line_number,
        })?;
        if headers.len() > 1 {
            return Err(ftd_p1::Error::MoreThanOneHeader {
                key: key.to_string(),
                doc_id: doc_id.to_string(),
                line_number: header.get_line_number(),
            });
        }
        Ok(header)
    }

    pub fn find_once_mut(
        &mut self,
        key: &str,
        doc_id: &str,
        line_number: usize,
    ) -> ftd_p1::Result<&mut ftd_p1::Header> {
        self.0
            .iter_mut()
            .find(|v| v.get_key().eq(key))
            .ok_or(ftd_p1::Error::HeaderNotFound {
                key: key.to_string(),
                doc_id: doc_id.to_string(),
                line_number,
            })
    }

    pub fn push(&mut self, item: ftd_p1::Header) {
        self.0.push(item)
    }

    /// returns a copy of Header after processing comments "/" and escape "\\/" (if any)
    ///
    /// only used by [`Section::remove_comments()`] and [`SubSection::remove_comments()`]
    ///
    /// [`SubSection::remove_comments()`]: ftd_p1::sub_section::SubSection::remove_comments
    /// [`Section::remove_comments()`]: ftd_p1::section::Section::remove_comments
    pub fn remove_comments(self) -> Headers {
        use itertools::Itertools;

        Headers(
            self.0
                .into_iter()
                .filter_map(|h| h.remove_comments())
                .collect_vec(),
        )
    }
}
