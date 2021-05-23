#[cfg(feature = "fifthtry")]
pub mod api;
#[cfg(feature = "fifthtry")]
pub mod code;
#[cfg(feature = "fifthtry")]
pub mod definition_list;
#[cfg(feature = "fifthtry")]
pub mod document;
#[cfg(feature = "fifthtry")]
pub mod heading;
#[cfg(feature = "fifthtry")]
pub mod iframe;
#[cfg(feature = "fifthtry")]
pub mod image;
#[cfg(feature = "fifthtry")]
pub mod include;
#[cfg(feature = "fifthtry")]
pub mod latex;
#[cfg(feature = "fifthtry")]
pub mod markdown;
#[cfg(feature = "fifthtry")]
pub mod meta;
pub mod p1;
#[cfg(feature = "fifthtry")]
pub mod p2;
#[cfg(feature = "fifthtry")]
pub mod pr;
#[cfg(feature = "fifthtry")]
pub mod prelude;
#[cfg(feature = "fifthtry")]
pub mod raw;
#[cfg(feature = "fifthtry")]
pub mod render;
#[cfg(feature = "fifthtry")]
pub mod rst;
#[cfg(feature = "fifthtry")]
pub mod section;
#[cfg(feature = "fifthtry")]
pub mod toc;
#[cfg(feature = "fifthtry")]
mod value_with_default;
#[cfg(feature = "fifthtry")]
pub mod youtube;

#[cfg(feature = "fifthtry")]
pub use crate::code::{Code, Highlight, ShowLineNumbers};
#[cfg(feature = "fifthtry")]
pub use crate::definition_list::DefinitionList;
#[cfg(feature = "fifthtry")]
pub use crate::document::{Align, Document, Table, TextDirection};
#[cfg(feature = "fifthtry")]
pub use crate::heading::Heading;
#[cfg(feature = "fifthtry")]
pub use crate::iframe::IFrame;
#[cfg(feature = "fifthtry")]
pub use crate::image::Image;
#[cfg(feature = "fifthtry")]
pub use crate::latex::Latex;
#[cfg(feature = "fifthtry")]
pub use crate::markdown::Markdown;
#[cfg(feature = "fifthtry")]
pub use crate::meta::{Admin, Meta, Reader, Someone, Surfer, Writer};
#[cfg(feature = "fifthtry")]
pub use crate::rst::Rst;
#[cfg(feature = "fifthtry")]
pub use crate::section::Section;
#[cfg(feature = "fifthtry")]
pub use crate::toc::{ToC, TocItem};
#[cfg(feature = "fifthtry")]
pub use crate::value_with_default::ValueWithDefault;
#[cfg(feature = "fifthtry")]
pub use crate::youtube::YouTube;

#[cfg(feature = "fifthtry")]
#[derive(serde_derive::Serialize, Eq, PartialEq, Debug, Default, Clone)]
pub struct Rendered {
    pub original: String,
    pub rendered: String,
}

#[cfg(not(feature = "fifthtry"))]
#[derive(Eq, PartialEq, Debug, Default, Clone)]
pub struct Rendered {
    pub original: String,
    pub rendered: String,
}

#[cfg(feature = "fifthtry")]
impl Rendered {
    pub fn rst(s: &str) -> Rendered {
        let p = match rst_parser::parse(s) {
            Ok(p) => p,
            Err(e) => return Rendered::line(format!("invalid rst: {}", e).as_str()),
        };

        let mut o: Vec<u8> = Vec::new();

        if let Err(e) = rst_renderer::render_html(&p, &mut o, true) {
            return Rendered::line(format!("invalid rst: {}", e).as_str());
        };

        Rendered {
            original: s.to_string(),
            rendered: match std::str::from_utf8(&o) {
                Ok(v) => v.to_string(),
                Err(e) => return Rendered::line(format!("invalid rst: {}", e).as_str()),
            },
        }
    }

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
