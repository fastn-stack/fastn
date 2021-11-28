extern crate self as ftd;

#[cfg(test)]
#[macro_use]
pub(crate) mod test;

mod component;
mod execute_doc;
pub mod main;
mod or_type;
pub mod p1;
pub mod p2;
pub mod render;
mod rt;
mod value_with_default;
pub(crate) mod variable;
mod youtube_id;

pub use crate::value_with_default::ValueWithDefault;
pub use component::{ChildComponent, Component, Instruction};
pub use or_type::OrType;
pub use rt::RT;
pub use variable::{PropertyValue, TextSource, Value, Variable};

pub fn js() -> &'static str {
    include_str!("../ftd.js")
}

pub fn html() -> &'static str {
    include_str!("../ftd.html")
}

pub fn rst(s: &str) -> ftd_rt::Rendered {
    // TODO: use pandoc to render
    ftd_rt::Rendered {
        original: s.to_string(),
        rendered: s.to_string(),
    }
}

pub fn markdown(s: &str) -> ftd_rt::Rendered {
    ftd_rt::Rendered {
        original: s.to_string(),
        rendered: render::render(s, true, false),
    }
}

pub fn markdown_extra(s: &str, auto_links: bool, hard_breaks: bool) -> ftd_rt::Rendered {
    ftd_rt::Rendered {
        original: s.to_string(),
        rendered: render::render(s, auto_links, hard_breaks),
    }
}

pub fn latex(s: &str, doc_id: &str) -> ftd::p1::Result<ftd_rt::Rendered> {
    let opts = katex::Opts::builder()
        .throw_on_error(false)
        .display_mode(true)
        .build()
        .unwrap();

    Ok(ftd_rt::Rendered {
        original: s.to_string(),
        rendered: match katex::render_with_opts(s, &opts) {
            Ok(v) => v,
            Err(e) => match e {
                katex::Error::JsValueError(e)
                | katex::Error::JsExecError(e)
                | katex::Error::JsInitError(e) => {
                    return Err(ftd::p1::Error::ParseError {
                        message: format!("{}: {}", e, s),
                        doc_id: doc_id.to_string(),
                        line_number: 0,
                    })
                }
                _ => return ftd::e2("katex error", e, doc_id.to_string(), 0),
            },
        },
    })
}

pub fn code(code: &str, ext: &str, doc_id: &str) -> ftd_rt::Rendered {
    code_with_theme(code, ext, crate::render::DEFAULT_THEME, doc_id).unwrap()
}

pub fn code_with_theme(
    code: &str,
    ext: &str,
    theme: &str,
    doc_id: &str,
) -> crate::p1::Result<ftd_rt::Rendered> {
    Ok(ftd_rt::Rendered {
        original: code.to_string(),
        rendered: render::code_with_theme(
            code.replace("\n\\-- ", "\n-- ").as_str(),
            ext,
            theme,
            doc_id,
        )?,
    })
}

pub fn markdown_line(s: &str) -> ftd_rt::Rendered {
    ftd_rt::Rendered {
        original: s.to_string(),
        rendered: render::inline(s),
    }
}

pub fn e2<T, S1, S2>(m: S1, c: S2, doc_id: String, line_number: usize) -> crate::p1::Result<T>
where
    S1: std::fmt::Debug,
    S2: std::fmt::Debug,
{
    Err(crate::p1::Error::ParseError {
        message: format!("{:?}: {:?}", m, c),
        doc_id,
        line_number,
    })
}

pub fn unknown_processor_error<T, S>(
    m: S,
    doc_id: String,
    line_number: usize,
) -> crate::p1::Result<T>
where
    S: Into<String>,
{
    Err(crate::p1::Error::ParseError {
        message: m.into(),
        doc_id,
        line_number,
    })
}

pub fn split_module(
    id: &str,
    line_number: usize,
) -> crate::p1::Result<(Option<&str>, &str, Option<&str>)> {
    if id.chars().filter(|v| *v == '.').count() > 2 {
        return ftd::e2(
            "id contains more than two dots",
            id,
            "".to_string(),
            line_number,
        );
    }

    match id.split_once('.') {
        Some((p1, p2)) => match p2.split_once(".") {
            Some((p21, p22)) => Ok((Some(p1), p21, Some(p22))),
            None => Ok((Some(p1), p2, None)),
        },
        None => Ok((None, id, None)),
    }
}

pub struct ExampleLibrary {}

#[cfg(feature = "async")]
use async_trait::async_trait;

#[cfg(feature = "async")]
#[async_trait]
impl ftd::p2::Library for ExampleLibrary {
    async fn get(&self, name: &str) -> Option<String> {
        if name == "fifthtry/ft" {
            return Some(std::fs::read_to_string("../ft.ftd").unwrap());
        }
        if name == "fifthtry/ft-core" {
            return Some(std::fs::read_to_string("../ft-core.ftd").unwrap());
        }

        std::fs::read_to_string(format!("./examples/{}.ftd", name)).ok()
    }
}

#[cfg(not(feature = "async"))]
impl ftd::p2::Library for ExampleLibrary {
    fn get(&self, name: &str) -> Option<String> {
        if name == "fifthtry/ft" {
            return Some(std::fs::read_to_string("../ft.ftd").unwrap());
        }
        if name == "fifthtry/ft-core" {
            return Some(std::fs::read_to_string("../ft-core.ftd").unwrap());
        }

        std::fs::read_to_string(format!("./examples/{}.ftd", name)).ok()
    }
}
