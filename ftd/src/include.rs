#[derive(PartialEq, Debug, Clone, serde_derive::Serialize)]
pub struct Include {
    pub id: String,
    pub document: String,
    pub level: Option<i32>,
}

impl Include {
    pub fn from_p1(p1: &crate::p1::Section) -> Result<Self, crate::document::ParseError> {
        Ok(Self {
            id: p1.header.string("id")?,
            document: p1.header.string("document")?,
            level: p1.header.i32_optional("level")?,
        })
    }

    pub fn to_p1(&self) -> crate::p1::Section {
        crate::p1::Section::with_name("include")
            .add_header("id", self.id.as_str())
            .add_header("document", self.document.as_str())
            .add_optional_header_i32("level", &self.level)
    }
}
