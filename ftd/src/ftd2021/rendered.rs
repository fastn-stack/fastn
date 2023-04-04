#[derive(serde::Serialize, serde::Deserialize, Eq, PartialEq, Debug, Default, Clone)]
pub struct Rendered {
    pub original: String,
    pub rendered: String,
}

pub fn code_with_theme(
    code: &str,
    ext: &str,
    theme: &str,
    doc_id: &str,
) -> crate::ftd2021::p1::Result<ftd::Rendered> {
    Ok(ftd::Rendered {
        original: code.to_string(),
        rendered: crate::ftd2021::code::code(
            code.replace("\n\\-- ", "\n-- ").as_str(),
            ext,
            theme,
            doc_id,
        )?,
    })
}

pub fn markup(s: &str) -> ftd::Rendered {
    ftd::Rendered {
        original: s.to_string(),
        rendered: crate::ftd2021::markup::markup(s),
    }
}

pub fn markup_line(s: &str) -> ftd::Rendered {
    ftd::Rendered {
        original: s.to_string(),
        rendered: crate::ftd2021::markup::markup_inline(s),
    }
}
