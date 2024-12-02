#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Record {
    pub name: String,
    pub fields: Vec<fastn_resolved::Field>,
    pub line_number: usize,
}

impl Record {
    pub fn new(name: &str, fields: Vec<Field>, line_number: usize) -> Record {
        Record {
            name: name.to_string(),
            fields,
            line_number,
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Field {
    pub name: String,
    pub kind: fastn_resolved::KindData,
    pub mutable: bool,
    pub default: Option<fastn_resolved::PropertyValue>,
    pub line_number: usize,
    pub access_modifier: AccessModifier,
}

impl Field {
    pub fn new(
        name: &str,
        kind: fastn_resolved::KindData,
        mutable: bool,
        value: Option<fastn_resolved::PropertyValue>,
        line_number: usize,
    ) -> Field {
        Field {
            name: name.to_string(),
            kind,
            mutable,
            default: value,
            line_number,
            access_modifier: Default::default(),
        }
    }

    pub fn to_sources(&self) -> Vec<fastn_resolved::PropertySource> {
        let mut sources = vec![fastn_resolved::PropertySource::Header {
            name: self.name.to_string(),
            mutable: self.mutable,
        }];
        if self.is_caption() {
            sources.push(fastn_resolved::PropertySource::Caption);
        }

        if self.is_body() {
            sources.push(fastn_resolved::PropertySource::Body);
        }

        if self.is_subsection_ui() {
            sources.push(fastn_resolved::PropertySource::Subsection);
        }

        sources
    }

    pub fn default(name: &str, kind: fastn_resolved::KindData) -> fastn_resolved::Field {
        fastn_resolved::Field {
            name: name.to_string(),
            kind,
            mutable: false,
            default: None,
            line_number: 0,
            access_modifier: Default::default(),
        }
    }

    pub fn default_with_value(
        name: &str,
        kind: fastn_resolved::KindData,
        value: fastn_resolved::PropertyValue,
    ) -> Field {
        Field {
            name: name.to_string(),
            kind,
            mutable: false,
            default: Some(value),
            line_number: 0,
            access_modifier: Default::default(),
        }
    }

    pub fn is_caption(&self) -> bool {
        self.kind.caption
    }

    pub fn is_subsection_ui(&self) -> bool {
        self.kind.kind.clone().inner_list().is_subsection_ui()
    }

    pub fn is_body(&self) -> bool {
        self.kind.body
    }

    pub fn is_value_required(&self) -> bool {
        if self.kind.is_optional() || self.kind.is_list() {
            return false;
        }
        self.default.is_none()
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
}
