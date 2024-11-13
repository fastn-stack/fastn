#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Record {
    pub name: String,
    pub fields: Vec<fastn_type::Field>,
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
    pub kind: fastn_type::KindData,
    pub mutable: bool,
    pub value: Option<fastn_type::PropertyValue>,
    pub line_number: usize,
    pub access_modifier: AccessModifier,
}

impl Field {
    pub fn new(
        name: &str,
        kind: fastn_type::KindData,
        mutable: bool,
        value: Option<fastn_type::PropertyValue>,
        line_number: usize,
    ) -> Field {
        Field {
            name: name.to_string(),
            kind,
            mutable,
            value,
            line_number,
            access_modifier: Default::default(),
        }
    }

    pub fn to_sources(&self) -> Vec<fastn_type::PropertySource> {
        let mut sources = vec![fastn_type::PropertySource::Header {
            name: self.name.to_string(),
            mutable: self.mutable,
        }];
        if self.is_caption() {
            sources.push(fastn_type::PropertySource::Caption);
        }

        if self.is_body() {
            sources.push(fastn_type::PropertySource::Body);
        }

        if self.is_subsection_ui() {
            sources.push(fastn_type::PropertySource::Subsection);
        }

        sources
    }

    pub fn default(name: &str, kind: fastn_type::KindData) -> fastn_type::Field {
        fastn_type::Field {
            name: name.to_string(),
            kind,
            mutable: false,
            value: None,
            line_number: 0,
            access_modifier: Default::default(),
        }
    }

    pub fn default_with_value(
        name: &str,
        kind: fastn_type::KindData,
        value: fastn_type::PropertyValue,
    ) -> Field {
        Field {
            name: name.to_string(),
            kind,
            mutable: false,
            value: Some(value),
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
        self.value.is_none()
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
