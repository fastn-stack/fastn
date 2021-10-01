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

pub fn latex(s: &str) -> Result<ftd_rt::Rendered, crate::p1::Error> {
    let opts = katex::Opts::builder()
        .throw_on_error(false)
        .display_mode(true)
        .build()
        .unwrap();

    Ok(ftd_rt::Rendered {
        original: s.to_string(),
        rendered: katex::render_with_opts(s, &opts).map_err(|e| match e {
            katex::Error::JsValueError(e)
            | katex::Error::JsExecError(e)
            | katex::Error::JsInitError(e) => crate::p1::Error::InvalidInput {
                message: e,
                context: s.to_string(),
            },
            _ => todo!(),
        })?,
    })
}

pub fn code(code: &str, ext: &str) -> ftd_rt::Rendered {
    code_with_theme(code, ext, crate::render::DEFAULT_THEME).unwrap()
}

pub fn code_with_theme(code: &str, ext: &str, theme: &str) -> crate::p1::Result<ftd_rt::Rendered> {
    Ok(ftd_rt::Rendered {
        original: code.to_string(),
        rendered: render::code_with_theme(code.replace("\n\\-- ", "\n-- ").as_str(), ext, theme)?,
    })
}

pub fn markdown_line(s: &str) -> ftd_rt::Rendered {
    ftd_rt::Rendered {
        original: s.to_string(),
        rendered: render::inline(s),
    }
}

pub fn e<T, S>(m: S) -> crate::p1::Result<T>
where
    S: Into<String>,
{
    Err(crate::p1::Error::InvalidInput {
        message: m.into(),
        context: "".to_string(),
    })
}

pub fn e2<T, S>(m: S, c: &str) -> crate::p1::Result<T>
where
    S: Into<String>,
{
    Err(crate::p1::Error::InvalidInput {
        message: format!("{}: {}", m.into(), c),
        context: c.to_string(),
    })
}

pub fn unknown_processor_error<T, S>(m: S) -> crate::p1::Result<T>
where
    S: Into<String>,
{
    Err(crate::p1::Error::UnknownProcessor { message: m.into() })
}

pub fn split_module(id: &str) -> crate::p1::Result<(Option<&str>, &str, Option<&str>)> {
    if id.chars().filter(|v| *v == '.').count() > 2 {
        return crate::e("id contains more than two dots".to_string());
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

impl ftd::p2::Library for ExampleLibrary {
    fn get(&self, name: &str) -> Option<String> {
        std::fs::read_to_string(format!("./examples/{}.ftd", name)).ok()
    }
}
