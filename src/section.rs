#[allow(clippy::large_enum_variant)]
#[allow(clippy::upper_case_acronyms)]
#[derive(PartialEq, Debug, Clone, serde_derive::Serialize)]
pub enum Section {
    Heading(crate::Heading),
    DefinitionList(crate::DefinitionList),
    Markdown(crate::Markdown),
    Rst(crate::Rst),
    Latex(crate::Latex),
    Code(crate::Code),
    Image(crate::Image),
    Table(crate::Table),
    YouTube(crate::YouTube),
    IFrame(crate::IFrame),
    ToC(crate::ToC),
    Header(crate::ToC),
    Second(crate::ToC),
    Meta(crate::Meta),
    Api(crate::api::Api),
    ApiObject(crate::api::ApiObject),
    PR(crate::pr::PR),
    Include(crate::include::Include),
}

impl Section {
    pub fn is_heading(&self) -> bool {
        matches!(self, Section::Heading(_))
    }

    pub fn id(&self) -> Option<String> {
        match self {
            Section::Heading(i) => Some(i.id.inner().to_string()),
            Section::DefinitionList(i) => i.id.clone(),
            Section::Markdown(i) => i.id.clone(),
            Section::Rst(i) => i.id.clone(),
            Section::Latex(i) => i.id.clone(),
            Section::Code(i) => i.id.clone(),
            Section::Image(i) => i.id.clone(),
            Section::Table(i) => i.id.clone(),
            Section::YouTube(i) => i.id.clone(),
            Section::IFrame(i) => i.id.clone(),
            Section::Api(i) => i.id.clone(),
            Section::ApiObject(i) => i.id.clone(),
            Section::PR(_) => None,
            Section::Header(_) => None,
            Section::Second(_) => None,
            Section::ToC(_) => None,
            Section::Meta(_) => None,
            Section::Include(_) => None,
        }
    }

    pub fn is_meta(&self) -> bool {
        matches!(self, Section::Meta(_))
    }

    pub fn is_header(&self) -> bool {
        matches!(self, Section::Header(_))
    }

    pub fn is_second(&self) -> bool {
        matches!(self, Section::Second(_))
    }

    pub fn is_include(&self) -> bool {
        matches!(self, Section::Include(_))
    }

    pub fn is_toc(&self) -> bool {
        matches!(self, Section::ToC(_))
    }

    pub fn title(&self) -> String {
        match self {
            Section::Heading(h) => h.title.original.clone(),
            _ => unreachable!(),
        }
    }

    pub fn from_p1(p1: &crate::p1::Section) -> Result<Self, crate::document::ParseError> {
        Ok(match p1.name.as_str() {
            "h0" => Section::Heading(crate::Heading::with_level(p1, 0)?),
            "h1" => Section::Heading(crate::Heading::with_level(p1, 1)?),
            "h2" => Section::Heading(crate::Heading::with_level(p1, 2)?),
            "h3" => Section::Heading(crate::Heading::with_level(p1, 3)?),
            "h4" => Section::Heading(crate::Heading::with_level(p1, 4)?),
            "h5" => Section::Heading(crate::Heading::with_level(p1, 5)?),
            "heading" => Section::Heading(crate::Heading::from_p1(p1)?),
            "markdown" => Section::Markdown(crate::Markdown::from_p1(p1)?),
            "rst" => Section::Rst(crate::Rst::from_p1(p1)?),
            "code" => Section::Code(crate::Code::from_p1(p1)?),
            "image" => Section::Image(crate::Image::from_p1(p1)?),
            "iframe" => Section::IFrame(crate::IFrame::from_p1(p1)?),
            "latex" => Section::Latex(crate::Latex::from_p1(p1)?),
            "youtube" => Section::YouTube(crate::YouTube::from_p1(p1)?),
            "toc" => Section::ToC(crate::ToC::from_p1(p1)?),
            "meta" => Section::Meta(crate::Meta::from_p1(p1)?),
            "header" => Section::Header(crate::ToC::from_p1(p1)?),
            "second" => Section::Second(crate::ToC::from_p1(p1)?),
            "api" => Section::Api(crate::api::Api::from_p1(p1)?),
            "api-object" => Section::ApiObject(crate::api::ApiObject::from_p1(p1)?),
            "pr" => Section::PR(crate::pr::PR::from_p1(p1)?),
            "include" => Section::Include(crate::include::Include::from_p1(p1)?),
            t => {
                return Err(crate::document::ParseError::ValidationError(format!(
                    "unknown section {}",
                    t
                )))
            }
        })
    }

    pub fn to_p1(&self) -> crate::p1::Section {
        match self {
            Section::Heading(h) => h.to_p1(),
            Section::Code(c) => c.to_p1(),
            Section::DefinitionList(d) => d.to_p1(),
            Section::Markdown(d) => d.to_p1(),
            Section::Rst(d) => d.to_p1(),
            Section::Latex(d) => d.to_p1(),
            Section::Image(d) => d.to_p1(),
            Section::Table(d) => d.to_p1(),
            Section::YouTube(d) => d.to_p1(),
            Section::IFrame(d) => d.to_p1(),
            Section::ToC(d) => d.to_p1("toc"),
            Section::Meta(d) => d.to_p1(),
            Section::Header(d) => d.to_p1("header"),
            Section::Second(d) => d.to_p1("second"),
            Section::Api(d) => d.to_p1(),
            Section::ApiObject(d) => d.to_p1(),
            Section::PR(d) => d.to_p1(),
            Section::Include(d) => d.to_p1(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;

    #[test]
    fn test_unknown_section() {
        let s = indoc::indoc! {"
                        -- markdown:

                        Hello World!

                        -- unknown:

                        abc
        "};

        f(s, "ValidationError: unknown section unknown");
    }
}
