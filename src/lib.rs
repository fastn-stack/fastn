#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
#[cfg(test)]
#[macro_use]
extern crate indoc;

pub mod api;
pub mod code;
pub mod definition_list;
pub mod document;
pub mod heading;
pub mod iframe;
pub mod image;
pub mod latex;
pub mod markdown;
pub mod meta;
pub mod p1;
pub mod p2;
pub mod pr;
pub mod prelude;
pub mod raw;
pub mod render;
pub mod section;
pub mod toc;
mod value_with_default;
pub mod youtube;

pub use crate::code::{Code, Highlight, ShowLineNumbers};
pub use crate::definition_list::DefinitionList;
pub use crate::document::{Align, Document, Table, TextDirection};
pub use crate::heading::Heading;
pub use crate::iframe::IFrame;
pub use crate::image::Image;
pub use crate::latex::Latex;
pub use crate::markdown::Markdown;
pub use crate::meta::{Admin, Meta, Reader, Someone, Surfer, Writer};
pub use crate::section::Section;
pub use crate::toc::{ToC, TocItem};
pub use crate::value_with_default::ValueWithDefault;
pub use crate::youtube::YouTube;

#[derive(Serialize, Eq, PartialEq, Debug, Default, Clone)]
pub struct Rendered {
    pub original: String,
    pub rendered: String,
}

impl Rendered {
    pub fn from(s: &str) -> Rendered {
        Rendered {
            original: s.to_string(),
            rendered: render::render(s, true, false),
        }
    }

    pub fn from_extra(s: &str, auto_links: bool, hard_breaks: bool) -> Rendered {
        Rendered {
            original: s.to_string(),
            rendered: render::render(s, auto_links, hard_breaks),
        }
    }

    pub fn latex(s: &str) -> Result<Rendered, crate::document::ParseError> {
        let opts = katex::Opts::builder()
            .throw_on_error(false)
            .display_mode(true)
            .build()
            .unwrap();

        Ok(Rendered {
            original: s.to_string(),
            rendered: katex::render_with_opts(s, &opts).map_err(|e| match e {
                katex::Error::JsValueError(s) | katex::Error::JsExecError(s) => {
                    crate::document::ParseError::ValidationError(s)
                }
                katex::Error::JsInitError(s) => {
                    panic!("{}", s)
                }
                _ => todo!(),
            })?,
        })
    }

    pub fn code(code: &str, ext: &str) -> Rendered {
        Rendered {
            original: code.to_string(),
            rendered: render::code(code.replace("\n\\-- ", "\n-- ").as_str(), ext),
        }
    }

    pub fn line(s: &str) -> Rendered {
        Rendered {
            original: s.to_string(),
            rendered: render::inline(s),
        }
    }
}
