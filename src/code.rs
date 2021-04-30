use crate::document::ParseError;

#[derive(PartialEq, Debug, Clone, Serialize)]
pub struct Code {
    pub caption: crate::Rendered,
    pub lang: String,                       // file extension
    pub break_lines: bool,                  // default false
    pub show_line_numbers: ShowLineNumbers, // default No
    pub show_invisibles: bool,              // default false
    pub show_tabs: bool,                    // some languages have special meaning for tabs
    pub tab_size: u8,                       // default: depends on language
    pub pointers: std::collections::HashMap<String, crate::Rendered>,
    pub show: Vec<i32>,
    // link to where to see the whole code, eg a github link
    pub full: Option<String>,
    pub highlights: Vec<Highlight>,

    pub code: crate::Rendered,
}

fn fix_body(body: &str) -> String {
    let mut body = body
        .replace("\n-- ", "\n\\-- ")
        .replace("\n--- ", "\n\\--- ");
    if body.starts_with("-- ") {
        body = body.replacen("-- ", "\\-- ", 1);
    }
    if body.starts_with("--- ") {
        body = body.replacen("--- ", "\\--- ", 1);
    }
    body
}

impl ToString for Code {
    fn to_string(&self) -> String {
        format!(
            "-- code:{}\nlang: {}\n\n{}",
            if self.caption.original.is_empty() {
                "".to_string()
            } else {
                format!(" {}", self.caption.original)
            },
            self.lang,
            fix_body(self.code.original.as_str()),
        )
    }
}

impl Default for Code {
    fn default() -> Code {
        Code {
            caption: crate::Rendered::default(),
            lang: "".to_string(),
            break_lines: false,
            show_line_numbers: ShowLineNumbers::No,
            show_invisibles: false,
            show_tabs: false,
            tab_size: 4,
            pointers: std::collections::HashMap::new(),
            show: vec![],
            full: None,
            highlights: vec![],
            code: crate::Rendered::default(),
        }
    }
}

impl Code {
    pub fn to_p1(&self) -> crate::p1::Section {
        crate::p1::Section::with_name("code")
            .and_caption(self.caption.original.as_str())
            .add_header("lang", self.lang.as_str())
            .and_body(fix_body(self.code.original.as_str()).as_str())
    }

    pub fn from_p1(p1: &crate::p1::Section) -> Result<Self, crate::document::ParseError> {
        let mut c = Code {
            lang: p1.header.str_with_default("lang", "")?.to_string(),
            ..Default::default()
        };
        if let Some(ref caption) = p1.caption {
            c.caption = crate::Rendered::line(caption.as_str());
        }
        match p1.body {
            Some(ref b) => c.code = crate::Rendered::code(b, c.lang.as_str()),
            None => {
                return Err(crate::document::ParseError::ValidationError(
                    "body must be present for code".to_string(),
                ))
            }
        }

        c.break_lines = p1.header.bool_with_default("break_lines", c.break_lines)?;
        c.show_line_numbers = p1
            .header
            .str_with_default(
                "show_line_numbers",
                c.show_line_numbers.to_string().as_str(),
            )?
            .parse()?;
        c.show_invisibles = p1
            .header
            .bool_with_default("show_invisibles", c.show_invisibles)?;
        c.show_tabs = p1.header.bool_with_default("show_tabs", c.show_tabs)?;
        c.tab_size = p1.header.i32_with_default("tab_size", c.tab_size as i32)? as u8;
        c.full = p1.header.str_optional("full")?.map(ToString::to_string);

        // TODO: pointers
        // TODO: show
        // TODO: highlights

        Ok(c)
    }

    pub fn with_caption(mut self, caption: &str) -> Self {
        self.caption = crate::Rendered::line(caption);
        self
    }

    pub fn with_code(mut self, code: &str) -> Self {
        // use .lang to render
        self.code = crate::Rendered::code(code, self.lang.as_str());
        self
    }

    pub fn with_lang(mut self, lang: &str) -> Self {
        self.code = crate::Rendered::code(self.code.original.as_str(), lang);
        self.lang = lang.to_string();
        self
    }

    pub fn with_break_lines(mut self, v: bool) -> Self {
        self.break_lines = v;
        self
    }

    pub fn with_tab_size(mut self, tab_size: u8) -> Self {
        self.tab_size = tab_size;
        self
    }

    pub fn with_show_tabs(mut self, v: bool) -> Self {
        self.show_tabs = v;
        self
    }

    pub fn with_show_invisibles(mut self, v: bool) -> Self {
        self.show_invisibles = v;
        self
    }

    pub fn with_full(mut self, v: &str) -> Self {
        self.full = Some(v.to_string());
        self
    }

    pub fn with_show_line_numbers(mut self, v: crate::ShowLineNumbers) -> Self {
        self.show_line_numbers = v;
        self
    }
}

#[derive(PartialEq, Debug, Clone, Serialize)]
pub struct Highlight {
    pub line: i32,
    pub start: Option<i32>,
    pub end: Option<i32>,
}

#[derive(PartialEq, Debug, Clone, Serialize)]
pub enum ShowLineNumbers {
    Yes,
    No,
    StartingWith(i32),
}

impl ToString for ShowLineNumbers {
    fn to_string(&self) -> String {
        match self {
            ShowLineNumbers::Yes => "true".to_string(),
            ShowLineNumbers::No => "false".to_string(),
            ShowLineNumbers::StartingWith(i) => format!("starting_with {}", i),
        }
    }
}

impl std::str::FromStr for ShowLineNumbers {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "true" => Ok(ShowLineNumbers::Yes),
            "false" => Ok(ShowLineNumbers::No),
            _ => {
                if s.starts_with("starting_with") {
                    Ok(ShowLineNumbers::StartingWith(
                        s.get("starting_with".len()..)
                            .ok_or_else(|| ParseError::ValidationError("impossibru".to_string()))?
                            .trim()
                            .parse::<i32>()
                            .map_err(ParseError::IntError)?,
                    ))
                } else {
                    Err("unknown string".into())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn code() {
        assert_eq!(
            "-- code:\nlang: py\n\nimport os\nprint('hello world')\n",
            crate::Code::default()
                .with_code("import os\nprint('hello world')\n")
                .with_lang("py")
                .to_string()
        );

        p(
            &indoc!(
                "
                 -- code: some caption
                 import os
                 print('hello world')
            "
            ),
            &vec![crate::Section::Code(
                crate::Code::default()
                    .with_caption("some caption")
                    .with_code("import os\nprint('hello world')"),
            )],
        );

        p(
            &indoc!(
                "
                 -- code: some caption
                 lang: rs
                 break_lines: true

                 import os
                 print('hello world')
            "
            ),
            &vec![crate::Section::Code(
                crate::Code::default()
                    .with_caption("some caption")
                    .with_lang("rs")
                    .with_break_lines(true)
                    .with_code("import os\nprint('hello world')"),
            )],
        );

        p(
            &indoc!(
                "
                 -- code: some caption
                 lang: rs
                 show_tabs: true
                 show_invisibles: true
                 tab_size: 8
                 import os
            "
            ),
            &vec![crate::Section::Code(
                crate::Code::default()
                    .with_caption("some caption")
                    .with_lang("rs")
                    .with_code("import os")
                    .with_show_invisibles(true)
                    .with_show_tabs(true)
                    .with_tab_size(8),
            )],
        );

        p(
            &indoc!(
                "
                 -- code: some caption
                 lang: rs
                 full: http://www.google.com
                 show_line_numbers: true
                 import os
            "
            ),
            &vec![crate::Section::Code(
                crate::Code::default()
                    .with_caption("some caption")
                    .with_lang("rs")
                    .with_show_line_numbers(crate::ShowLineNumbers::Yes)
                    .with_full("http://www.google.com")
                    .with_code("import os"),
            )],
        );

        p(
            &indoc!(
                "
                 -- code: some caption
                 lang: rs
                 show_line_numbers: false
                 import os
            "
            ),
            &vec![crate::Section::Code(
                crate::Code::default()
                    .with_caption("some caption")
                    .with_lang("rs")
                    .with_show_line_numbers(crate::ShowLineNumbers::No)
                    .with_code("import os"),
            )],
        );

        p(
            &indoc!(
                "
                 -- code: some caption
                 lang: rs
                 show_line_numbers: starting_with 20
                 import os
            "
            ),
            &vec![crate::Section::Code(
                crate::Code::default()
                    .with_caption("some caption")
                    .with_lang("rs")
                    .with_show_line_numbers(crate::ShowLineNumbers::StartingWith(20))
                    .with_code("import os"),
            )],
        );

        let f = crate::Code::default()
            .with_caption("")
            .with_lang("5d")
            .with_code(&indoc!(
                "
                -- amitu/pricing: Simple pricing
                ---- plan: Unlimited
                -monthly: $49
                -annual: $40
                -link: /buy-something/?"
            ));

        p(
            &indoc!(
                "
        -- code:
        lang: 5d

        \\-- amitu/pricing: Simple pricing
        ---- plan: Unlimited
        -monthly: $49
        -annual: $40
        -link: /buy-something/?"
            ),
            &vec![crate::Section::Code(f.clone())],
        );

        assert_eq!(
            indoc!(
                "
        -- code:
        lang: 5d

        \\-- amitu/pricing: Simple pricing
        ---- plan: Unlimited
        -monthly: $49
        -annual: $40
        -link: /buy-something/?"
            ),
            f.to_string(),
        )
    }
}
