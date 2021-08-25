#[derive(PartialEq, Debug, serde_derive::Serialize, Default, Clone)]
pub struct Document {
    pub sections: Vec<crate::Section>,
    pub pr_sections: linked_hash_map::LinkedHashMap<String, crate::pr::PR>,
}

pub fn get_title(sections: &[crate::Section]) -> crate::Rendered {
    sections
        .iter()
        .filter(|s| crate::Section::is_heading(s))
        .collect::<Vec<_>>()
        .first()
        .map(|s| crate::Rendered::line(s.title().as_str()))
        .unwrap_or_else(crate::Rendered::default)
}

pub fn get_no_index(sections: &[crate::Section]) -> bool {
    for s in sections.iter() {
        if let crate::Section::Meta(m) = s {
            if m.no_index {
                return true;
            }
        }
    }
    false
}

impl Document {
    pub fn convert_to_string(&self) -> String {
        crate::p1::to_string(&self.sections.iter().map(|v| v.to_p1()).collect::<Vec<_>>())
    }

    pub fn get_section_by_id(&self, id: &str) -> Option<&crate::Section> {
        self.sections
            .iter()
            .find(|x| x.id().eq(&Some(id.to_string())))
    }

    pub fn new(sections: &[crate::Section]) -> Self {
        Self {
            sections: sections.to_vec(),
            pr_sections: Self::get_pr_sections_map(sections),
        }
    }

    pub fn set_default_meta(&mut self, meta: crate::Meta) {
        let mut found = false;
        for s in self.sections.iter() {
            if matches!(s, crate::Section::Meta(_)) {
                found = true;
                break;
            }
        }

        if !found {
            self.sections.insert(0, crate::Section::Meta(meta));
        }
    }

    pub fn is_public(&self) -> bool {
        for s in self.sections.iter() {
            if let crate::Section::Meta(m) = s {
                if m.is_public() {
                    return true;
                }
            }
        }

        false
    }

    pub fn can_read(&self, username: Option<String>) -> bool {
        // TODO: email
        if self.is_public() {
            return true;
        }

        if let Some(u) = username {
            for s in self.sections.iter() {
                if let crate::Section::Meta(m) = s {
                    if m.can_read(u.as_str()) {
                        return true;
                    }
                }
            }
        }

        false
    }

    pub fn can_write(&self, username: &str) -> bool {
        // TODO: email

        for s in self.sections.iter() {
            if let crate::Section::Meta(m) = s {
                if m.can_write(username) {
                    return true;
                }
            }
        }

        false
    }

    pub fn can_admin(&self, username: &str) -> bool {
        // TODO: email

        for s in self.sections.iter() {
            if let crate::Section::Meta(m) = s {
                if m.can_admin(username) {
                    return true;
                }
            }
        }

        false
    }

    pub fn get_toc(&self) -> Option<crate::ToC> {
        for section in self.sections.iter() {
            if let crate::Section::ToC(toc) = section {
                return Some(toc.clone());
            }
        }
        None
    }

    pub fn get_header(&self) -> crate::ToC {
        for section in self.sections.iter() {
            if let crate::Section::Header(toc) = section {
                return toc.clone();
            }
        }
        ToC::default()
    }

    pub fn get_second(&self) -> Option<crate::ToC> {
        for section in self.sections.iter() {
            if let crate::Section::Second(toc) = section {
                return Some(toc.clone());
            }
        }
        None
    }

    pub fn get_design(&self) -> crate::meta::Design {
        self.get_meta().and_then(|m| m.design).unwrap_or_default()
    }

    pub fn get_meta(&self) -> Option<crate::Meta> {
        for section in self.sections.iter() {
            if let crate::Section::Meta(meta) = section {
                return Some(meta.clone());
            }
        }
        None
    }

    pub fn get_meta_ref(&self) -> Option<&crate::Meta> {
        for section in self.sections.iter() {
            if let crate::Section::Meta(meta) = section {
                return Some(meta);
            }
        }
        None
    }

    pub fn get_translation(&self) -> Option<&crate::meta::Translation> {
        self.get_meta_ref().map(|ref x| x.get_translation())
    }

    pub fn get_language_with_default(&self) -> realm_lang::Language {
        self.get_language().unwrap_or(realm_lang::Language::English)
    }

    pub fn get_language(&self) -> Option<realm_lang::Language> {
        self.get_meta_ref().map(|x| *x.lang.inner())
    }

    pub fn get_translation_and_lang(
        &self,
    ) -> Option<(&crate::meta::Translation, &realm_lang::Language)> {
        self.get_meta_ref()
            .map(|ref x| (x.get_translation(), x.get_lang().inner()))
    }

    pub fn get_title(&self) -> crate::Rendered {
        get_title(&self.sections)
    }

    pub fn no_index(&self) -> bool {
        get_no_index(&self.sections)
    }

    pub fn get_pr_sections(&self) -> Vec<&crate::pr::PR> {
        let mut v = vec![];
        for section in self.sections.iter() {
            if let crate::Section::PR(pr) = section {
                v.push(pr)
            }
        }
        v
    }

    pub fn get_pr_sections_map(
        sections: &[crate::Section],
    ) -> linked_hash_map::LinkedHashMap<String, crate::pr::PR> {
        let mut map: linked_hash_map::LinkedHashMap<String, crate::pr::PR> =
            linked_hash_map::LinkedHashMap::new();
        for section in sections.iter() {
            if let crate::Section::PR(pr) = section.clone() {
                map.insert(pr.unique_id(), pr);
            }
        }
        map
    }

    pub fn parse(s: &str, id: &str) -> Result<Self, ParseError> {
        Self::parse_(s).map_err(|e| {
            observer::log("failed to parse ftd document");
            observer::observe_string("id", id);
            observer::observe_string("err", e.to_string().as_str());
            e
        })
    }

    fn parse_(s: &str) -> Result<Self, ParseError> {
        let p1 = crate::p1::parse(s)?;
        let mut sections = vec![];
        for s in p1 {
            let section = crate::Section::from_p1(&s)?;
            let body = if section.is_heading() && s.body.is_some() {
                s.body.clone()
            } else {
                None
            };
            sections.push(section);
            if let Some(b) = body {
                sections.push(crate::Section::Markdown(crate::Markdown {
                    id: None,
                    body: crate::Rendered::from(b.as_str()),
                    hard_breaks: false,
                    auto_links: true,
                    align: Align::default(),
                    direction: TextDirection::default(),
                    two_columns: false,
                    collapsed: false,
                    caption: None,
                }))
            }
        }
        let pr_sections = Self::get_pr_sections_map(&sections);
        Ok(Document {
            sections,
            pr_sections,
        })
    }

    pub fn is_deleted(&self) -> bool {
        match self.get_meta_ref() {
            Some(x) => x.deleted,
            None => false,
        }
    }
}

#[derive(PartialEq, Debug, Clone, serde_derive::Serialize)]
#[serde(tag = "type")]
pub enum Align {
    Left,
    Center,
    Right,
}

impl Default for Align {
    fn default() -> Align {
        Align::Left
    }
}

impl Align {
    pub fn as_str(&self) -> &'static str {
        match self {
            Align::Left => "left",
            Align::Center => "center",
            Align::Right => "right",
        }
    }
}

impl std::str::FromStr for Align {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "left" => Ok(Align::Left),
            "right" => Ok(Align::Right),
            "center" => Ok(Align::Center),
            "centre" => Ok(Align::Center),
            _ => Err(
                format!("accepted values: left | right | center, found: {}", s)
                    .as_str()
                    .into(),
            ),
        }
    }
}

#[derive(PartialEq, Debug, Clone, serde_derive::Serialize)]
#[serde(tag = "type")]
pub enum TextDirection {
    RightToLeft,
    LeftToRight,
}

impl std::str::FromStr for TextDirection {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rtl" => Ok(TextDirection::RightToLeft),
            "ltr" => Ok(TextDirection::LeftToRight),
            _ => Err(format!("accepted values: ltr | rtl, found: {}", s)
                .as_str()
                .into()),
        }
    }
}

impl Default for TextDirection {
    fn default() -> TextDirection {
        TextDirection::LeftToRight
    }
}

impl TextDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            TextDirection::LeftToRight => "ltr",
            TextDirection::RightToLeft => "rtl",
        }
    }
}

#[derive(PartialEq, Debug, Clone, serde_derive::Serialize)]
pub struct Table {
    pub id: Option<String>,
    pub caption: crate::Rendered,
    pub header: Vec<crate::Rendered>,
    pub rows: Vec<Vec<crate::Rendered>>,
}

impl ToString for Table {
    fn to_string(&self) -> String {
        todo!()
    }
}

impl Table {
    pub fn to_p1(&self) -> crate::p1::Section {
        todo!()
    }
}

use crate::ToC;
use thiserror::Error as Error_;

#[derive(Error_, Debug)]
pub enum ParseError {
    #[error("P1Error: {0}")]
    P1Error(crate::p1::Error),
    #[error("IntError: {0}")]
    IntError(std::num::ParseIntError),
    #[error("LangError: {0}")]
    LangError(realm_lang::Error),
    #[error("ValidationError: {0}")]
    ValidationError(String),
    #[error("ColorParseError: {0}")]
    ColorParseError(css_color_parser::ColorParseError),
    #[error("ToCError: {0}")]
    ToCError(crate::toc::ParseError),
}

impl From<css_color_parser::ColorParseError> for ParseError {
    fn from(p: css_color_parser::ColorParseError) -> ParseError {
        ParseError::ColorParseError(p)
    }
}

impl From<crate::toc::ParseError> for ParseError {
    fn from(p: crate::toc::ParseError) -> ParseError {
        ParseError::ToCError(p)
    }
}

impl From<std::num::ParseIntError> for ParseError {
    fn from(p: std::num::ParseIntError) -> ParseError {
        ParseError::IntError(p)
    }
}

impl From<crate::p1::Error> for ParseError {
    fn from(p: crate::p1::Error) -> ParseError {
        ParseError::P1Error(p)
    }
}

impl From<&str> for ParseError {
    fn from(s: &str) -> ParseError {
        ParseError::ValidationError(s.to_string())
    }
}

impl From<String> for ParseError {
    fn from(s: String) -> ParseError {
        ParseError::ValidationError(s)
    }
}

impl From<realm_lang::Error> for ParseError {
    fn from(e: realm_lang::Error) -> Self {
        ParseError::LangError(e)
    }
}

#[cfg(test)]
#[track_caller]
pub fn p(s: &str, t: &[crate::Section]) {
    use pretty_assertions::assert_eq;

    assert_eq!(
        Document::parse(s, "foo/bar")
            .unwrap_or_else(|e| panic!("{:?}", e))
            .sections,
        t
    )
}

#[cfg(test)]
#[track_caller]
pub fn f(s: &str, m: &str) {
    use pretty_assertions::assert_eq;

    match Document::parse(s, "foo/bar") {
        Ok(r) => panic!("expected failure, found: {:?}", r),
        Err(e) => assert_eq!(e.to_string(), m.trim()),
    }
}

pub fn err<T>(msg: &str) -> Result<T, ParseError> {
    Err(crate::document::ParseError::ValidationError(
        msg.to_string(),
    ))
}

#[cfg(test)]
mod test {
    use crate::prelude::*;

    #[test]
    fn escaping() {
        p(
            &indoc::indoc!(
                "
            -- code:
            lang: py

            \\-- hello: world
            \\--- damn: man
            "
            ),
            &vec![crate::Section::Code(
                crate::Code::default()
                    .with_code("-- hello: world\n--- damn: man")
                    .with_lang("py"),
            )],
        );
    }

    #[test]
    #[ignore]
    fn definition_list() {
        p(
            &indoc::indoc!(
                "
                 -- definition-list: hello list
                 hello:
                     world is
                     not enough

                     lol

                 super:
                    awesome

                 this: is another test
            "
            ),
            &vec![crate::Section::DefinitionList(crate::DefinitionList {
                id: None,
                caption: crate::Rendered::line("hello list"),
                list: vec![
                    (
                        crate::Rendered::line("hello"),
                        crate::Rendered::from("world is\nnot enough\n\nlol"),
                    ),
                    (
                        crate::Rendered::line("super"),
                        crate::Rendered::from("awesome"),
                    ),
                    (
                        crate::Rendered::line("this"),
                        crate::Rendered::from("is another test"),
                    ),
                ],
            })],
        );
        p(
            &indoc::indoc!(
                "
                 -- definition-list:
                 without: title
            "
            ),
            &vec![crate::Section::DefinitionList(crate::DefinitionList {
                id: None,
                caption: crate::Rendered::default(),
                list: vec![(
                    crate::Rendered::line("without"),
                    crate::Rendered::from("title"),
                )],
            })],
        );

        f(
            "-- definition-list: items are required",
            indoc::indoc!(
                "
                 PestError:  --> 1:1
                   |
                 1 | -- definition-list: items are required
                   | ^---
                   |
                   = expected section
             "
            ),
        );
    }

    // #[test] -- TODO
    #[allow(dead_code)]
    fn latex() {
        p(
            &indoc::indoc!(
                "
                 -- latex:
                 hello world is

                     not enough

                     lol
            "
            ),
            &vec![crate::Section::Latex(crate::Latex {
                id: None,
                caption: None,
                body: crate::Rendered::from("hello world is\n\n    not enough\n\n    lol\n"),
            })],
        );

        p(
            &indoc::indoc!(
                "
                 -- latex: some title
                 id: temp
                 hello world is

                     not enough

                     lol
            "
            ),
            &vec![crate::Section::Latex(crate::Latex {
                id: Some("temp".to_string()),
                caption: Some(crate::Rendered::line("some title")),
                body: crate::Rendered::from("hello world is\n\n    not enough\n\n    lol\n"),
            })],
        );

        f(
            "-- latex: without body",
            indoc::indoc!(
                "
                   --> 1:1
                   |
                 1 | -- latex: without body
                   | ^---
                   |
                   = expected section
             "
            ),
        );
        f(
            "-- latex:\n-- latex:",
            indoc::indoc!(
                "
                   --> 1:10
                   |
                 1 | -- latex:␊
                   |          ^---
                   |
                   = expected text_till_eol
             "
            ),
        );
        f(
            "-- latex:  \n-- latex:",
            indoc::indoc!(
                "
                   --> 1:1
                   |
                 1 | -- latex:  ␊
                   | ^---
                   |
                   = expected section
             "
            ),
        );
    }
}
