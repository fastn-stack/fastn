use crate::document::ParseError;
use crate::{Code, DefinitionList, Heading, IFrame, Image, Latex, Markdown, Table, YouTube};

#[allow(clippy::large_enum_variant)]
#[derive(PartialEq, Debug, Clone, Serialize)]
pub enum Section {
    Heading(Heading),
    DefinitionList(DefinitionList),
    Markdown(Markdown),
    Latex(Latex),
    Code(Code),
    Image(Image),
    Table(Table),
    YouTube(YouTube),
    IFrame(IFrame),
    ToC(crate::ToC),
    Header(crate::ToC),
    Second(crate::ToC),
    Meta(crate::Meta),
    Api(crate::api::Api),
    ApiObject(crate::api::ApiObject),
    PR(crate::pr::PR),
}

impl ToString for Section {
    fn to_string(&self) -> String {
        match self {
            Section::Heading(i) => i.to_string(),
            Section::DefinitionList(i) => i.to_string(),
            Section::Markdown(i) => i.to_string(),
            Section::Latex(i) => i.to_string(),
            Section::Code(i) => i.to_string(),
            Section::Image(i) => i.to_string(),
            Section::Table(i) => i.to_string(),
            Section::YouTube(i) => i.to_string(),
            Section::IFrame(i) => i.to_string(),
            Section::ToC(i) => i.to_string("toc"),
            Section::Meta(t) => t.to_string(),
            Section::Header(t) => t.to_string("header"),
            Section::Second(t) => t.to_string("second"),
            Section::Api(a) => a.to_string(),
            Section::ApiObject(a) => a.to_string(),
            Section::PR(a) => a.to_string(),
        }
    }
}

impl Section {
    pub fn is_heading(&self) -> bool {
        matches!(self, Section::Heading(_))
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

    pub fn is_toc(&self) -> bool {
        matches!(self, Section::ToC(_))
    }

    pub fn title(&self) -> String {
        match self {
            Section::Heading(h) => h.title.original.clone(),
            _ => unreachable!(),
        }
    }

    pub fn from_p1(p1: &crate::p1::Section) -> Result<Self, ParseError> {
        Ok(match p1.name.as_str() {
            "h0" => Section::Heading(Heading::with_level(p1, 0)?),
            "h1" => Section::Heading(Heading::with_level(p1, 1)?),
            "h2" => Section::Heading(Heading::with_level(p1, 2)?),
            "h3" => Section::Heading(Heading::with_level(p1, 3)?),
            "h4" => Section::Heading(Heading::with_level(p1, 4)?),
            "h5" => Section::Heading(Heading::with_level(p1, 5)?),
            "heading" => Section::Heading(Heading::from_p1(p1)?),
            "markdown" => Section::Markdown(Markdown::from_p1(p1)?),
            "code" => Section::Code(Code::from_p1(p1)?),
            "image" => Section::Image(Image::from_p1(p1)?),
            "iframe" => Section::IFrame(IFrame::from_p1(p1)?),
            "latex" => Section::Latex(Latex::from_p1(p1)?),
            "youtube" => Section::YouTube(YouTube::from_p1(p1)?),
            "toc" => Section::ToC(crate::ToC::from_p1(p1)?),
            "meta" => Section::Meta(crate::Meta::from_p1(p1)?),
            "header" => Section::Header(crate::ToC::from_p1(p1)?),
            "second" => Section::Second(crate::ToC::from_p1(p1)?),
            "api" => Section::Api(crate::api::Api::from_p1(p1)?),
            "api-object" => Section::ApiObject(crate::api::ApiObject::from_p1(p1)?),
            "pr" => Section::PR(crate::pr::PR::from_p1(p1)?),
            t => {
                return Err(ParseError::ValidationError(format!(
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
        }
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;

    #[test]
    fn test_unknown_section() {
        let s = indoc! {"
                        -- markdown:

                        Hello World!

                        -- unknown:

                        abc
        "};

        f(s, "ValidationError: unknown section unknown");
    }
}
