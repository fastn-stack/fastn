pub fn processor(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc,
    config: &fpm::Config,
) -> ftd::p1::Result<ftd::Value> {
    let doc_path = match section
        .header
        .str_optional(doc.name, section.line_number, "path")?
    {
        Some(v) => v,
        None => {
            match section
                .header
                .str_optional(doc.name, section.line_number, "$path$")?
            {
                Some(v) => v,
                None => {
                    return ftd::e2(
                        "`path` is not specified".to_string(),
                        doc.name,
                        section.line_number,
                    )
                }
            }
        }
    };
    let mut v: std::collections::BTreeMap<String, ftd::PropertyValue> = Default::default();

    let code_item = IncludeCode::parse(doc_path, config).unwrap();

    v.insert(
        "$body$".to_string(),
        ftd::PropertyValue::Value {
            value: ftd::Value::String {
                text: code_item.body,
                source: ftd::TextSource::Header,
            },
        },
    );
    v.insert(
        "lang".to_string(),
        ftd::PropertyValue::Value {
            value: ftd::Value::String {
                text: code_item.extension,
                source: ftd::TextSource::Header,
            },
        },
    );
    Ok(ftd::Value::Object { values: v })
}

#[derive(PartialEq, Debug, Default, Clone, serde::Serialize)]
pub struct IncludeCode {
    pub extension: String,
    pub body: String,
}

#[derive(PartialEq, Debug, Clone, serde::Serialize)]
enum RangeOrAnchor {
    Range(LineRange),
    Anchor(String),
}

// A range of lines specified with some include directive.
#[derive(PartialEq, Debug, Clone, serde::Serialize)]
enum LineRange {
    SingleLine(i32),
    Range((i32, i32)),
    RangeFrom(i32),
    RangeTo(i32),
    RangeFull,
}

#[derive(PartialEq, Debug, Clone, serde::Serialize)]
pub struct IncludeDocument {
    path: String,
    roa: RangeOrAnchor,
}

impl IncludeDocument {
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        match s.split_once(':') {
            None => {
                // No `:` found. Include full file
                Ok(IncludeDocument {
                    path: s.to_string(),
                    roa: RangeOrAnchor::Range(LineRange::RangeFull),
                })
            }
            Some((doc_path, range_or_anchor)) => {
                match range_or_anchor.split_once(':') {
                    None => {
                        // Can either be an anchor or an individual line
                        let test = range_or_anchor.parse::<i32>();
                        match test {
                            Ok(ok) => Ok(IncludeDocument {
                                path: doc_path.to_string(),
                                roa: RangeOrAnchor::Range(LineRange::SingleLine(ok)),
                            }),
                            Err(_e) => Ok(IncludeDocument {
                                path: doc_path.to_string(),
                                roa: RangeOrAnchor::Anchor(range_or_anchor.to_string()),
                            }),
                        }
                    }
                    Some((start, end)) => match (start, end) {
                        (k, "") => Ok(IncludeDocument {
                            path: doc_path.to_string(),
                            roa: RangeOrAnchor::Range(LineRange::RangeFrom(k.parse::<i32>()?)),
                        }),
                        ("", l) => Ok(IncludeDocument {
                            path: doc_path.to_string(),
                            roa: RangeOrAnchor::Range(LineRange::RangeTo(l.parse::<i32>()?)),
                        }),
                        (k, l) => {
                            let start = k.parse::<i32>()?;
                            let end = l.parse::<i32>()?;
                            if end < start {
                                panic!("End can't be higher than start")
                            }
                            Ok(IncludeDocument {
                                path: doc_path.to_string(),
                                roa: RangeOrAnchor::Range(LineRange::Range((start, end))),
                            })
                        }
                    },
                }
            }
        }
    }
}

lazy_static! {
    static ref ANCHOR_START: regex::Regex =
        regex::Regex::new(r"ANCHOR:\s*(?P<anchor_name>[\w_-]+)").unwrap();
    static ref ANCHOR_END: regex::Regex =
        regex::Regex::new(r"ANCHOR_END:\s*(?P<anchor_name>[\w_-]+)").unwrap();
}

/// Take anchored lines from a string.
/// Lines containing anchor are ignored.
pub fn take_anchored_lines(s: &str, anchor: &str) -> String {
    let mut retained = Vec::<&str>::new();
    let mut anchor_found = false;

    for l in s.lines() {
        if anchor_found {
            match ANCHOR_END.captures(l) {
                Some(cap) => {
                    if &cap["anchor_name"] == anchor {
                        break;
                    }
                }
                None => {
                    if !ANCHOR_START.is_match(l) {
                        retained.push(l);
                    }
                }
            }
        } else if let Some(cap) = ANCHOR_START.captures(l) {
            if &cap["anchor_name"] == anchor {
                anchor_found = true;
            }
        }
    }

    retained.join("\n")
}

pub fn sanitize_anchored_lines(s: &str) -> String {
    let mut retained = Vec::<&str>::new();
    for l in s.lines() {
        if let (None, None) = (ANCHOR_END.captures(l), ANCHOR_START.captures(l)) {
            retained.push(l)
        }
    }
    retained.join("\n")
}

impl IncludeCode {
    pub fn parse(s: &str, config: &fpm::Config) -> Result<Self, ParseError> {
        let doc = IncludeDocument::parse(s)?;
        let extension = match &doc.path.rsplit_once('.') {
            Some((_, ex)) => ex,
            None => "txt",
        };
        let file_path = config.root.join(
            doc.path
                .replace('/', std::path::MAIN_SEPARATOR.to_string().as_str()),
        );
        let file_content = std::fs::read_to_string(file_path)?
            .lines()
            .into_iter()
            .fold(String::new(), |accumulator, instance| {
                let should_escape = instance.starts_with('$') || instance.starts_with('/');
                format!(
                    "{}{}{}\n",
                    accumulator.as_str(),
                    if should_escape { "\\" } else { "" },
                    instance
                )
            });
        let output = match (file_content, doc.roa) {
            (fc, RangeOrAnchor::Anchor(anchor_name)) => take_anchored_lines(&fc, &anchor_name),
            (fc, RangeOrAnchor::Range(LineRange::RangeFull)) => fc,
            (fc, RangeOrAnchor::Range(r)) => {
                let lines = fc.lines();
                let len = lines.clone().count();
                let (start, end) = match r {
                    LineRange::Range((s, e)) => (s - 1, e),
                    LineRange::RangeFrom(s) => (s - 1, len as i32),
                    LineRange::RangeTo(e) => (0, e),
                    LineRange::SingleLine(e) => (e - 1, e),
                    LineRange::RangeFull => (0, len as i32),
                };
                lines
                    .skip(start as usize)
                    .take(std::cmp::min(end as usize, len))
                    .collect::<Vec<&str>>()
                    .join("\n")
            }
        };
        Ok(IncludeCode {
            extension: extension.to_string(),
            body: sanitize_anchored_lines(output.as_str()),
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("Integer Parsing Error: {}", _0)]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("File Not Found Error: {}", _0)]
    FileNotFound(#[from] std::io::Error),

    #[error("{}", _0)]
    FTDError(#[from] ftd::p1::Error),
}

#[cfg(test)]
mod test {
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    macro_rules! p {
        ($s:expr, $t: expr,) => {
            p!($s, $t)
        };
        ($s:expr, $t: expr) => {
            assert_eq!(
                super::IncludeDocument::parse($s).unwrap_or_else(|e| panic!("{}", e)),
                $t
            )
        };
    }

    #[test]
    fn parse_path() {
        // File Import
        p!(
            &indoc!("code.rs"),
            super::IncludeDocument {
                path: "code.rs".to_string(),
                roa: super::RangeOrAnchor::Range(super::LineRange::RangeFull)
            }
        );
        // Line Import
        p!(
            &indoc!("code.rs:4"),
            super::IncludeDocument {
                path: "code.rs".to_string(),
                roa: super::RangeOrAnchor::Range(super::LineRange::SingleLine(4))
            }
        );
        // Anchor Import
        p!(
            &indoc!("code.rs:MAIN_CODE"),
            super::IncludeDocument {
                path: "code.rs".to_string(),
                roa: super::RangeOrAnchor::Anchor("MAIN_CODE".to_string())
            }
        );
        // Range From Import
        p!(
            &indoc!("code.rs:2:"),
            super::IncludeDocument {
                path: "code.rs".to_string(),
                roa: super::RangeOrAnchor::Range(super::LineRange::RangeFrom(2))
            }
        );
        // Range To Import
        p!(
            &indoc!("code.rs::2"),
            super::IncludeDocument {
                path: "code.rs".to_string(),
                roa: super::RangeOrAnchor::Range(super::LineRange::RangeTo(2))
            }
        );
    }
}
