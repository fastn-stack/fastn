#![allow(dead_code)]

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub enum Boolean {
    // if: $caption is not null
    IsNotNull {
        value: ftd::interpreter2::PropertyValue,
    },
    // if: $caption is null
    IsNull {
        value: ftd::interpreter2::PropertyValue,
    },
    // if: $list is not empty
    IsNotEmpty {
        value: ftd::interpreter2::PropertyValue,
    },
    // if: $list is empty
    IsEmpty {
        value: ftd::interpreter2::PropertyValue,
    },
    // if: $caption == hello | if: $foo
    Equal {
        left: ftd::interpreter2::PropertyValue,
        right: ftd::interpreter2::PropertyValue,
    },
    // if: $caption != hello
    NotEqual {
        left: ftd::interpreter2::PropertyValue,
        right: ftd::interpreter2::PropertyValue,
    },
    // if: not $show_something
    Not {
        of: Box<Boolean>,
    },
    // if: false
    Literal {
        value: bool,
    },
    // if: $array is empty
    ListIsEmpty {
        value: ftd::interpreter2::PropertyValue,
    },
}

impl Boolean {
    pub(crate) fn from_ast_condition(
        condition: ftd::ast::Condition,
        definition_name_with_arguments: Option<(&str, &[ftd::interpreter2::Argument])>,
        doc: &ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::Boolean> {
        let (boolean, mut left, mut right) = ftd::interpreter2::Boolean::boolean_left_right(
            condition.line_number,
            condition.expression.as_str(),
            doc.name,
        )?;
        left = doc.resolve_reference_name(left.as_str(), condition.line_number)?;
        if let Some(ref r) = right {
            right = doc.resolve_reference_name(r, condition.line_number).ok();
        }

        Ok(match boolean.as_str() {
            "Literal" => Boolean::Literal {
                value: left == "true",
            },
            "IsNotNull" | "IsNull" => {
                let value = ftd::interpreter2::PropertyValue::from_string_with_argument(
                    left.as_str(),
                    doc,
                    None,
                    false,
                    condition.line_number,
                    definition_name_with_arguments,
                )?;
                if !value.kind().is_optional() {
                    return ftd::interpreter2::utils::e2(
                        format!("'{}' is not to a list", left),
                        doc.name,
                        condition.line_number,
                    );
                }

                if boolean.as_str() == "IsNotNull" {
                    Boolean::IsNotNull { value }
                } else {
                    Boolean::IsNull { value }
                }
            }
            "IsNotEmpty" | "IsEmpty" => {
                let value = ftd::interpreter2::PropertyValue::from_string_with_argument(
                    left.as_str(),
                    doc,
                    None,
                    false,
                    condition.line_number,
                    definition_name_with_arguments,
                )?;
                if !value.kind().is_list() {
                    return ftd::interpreter2::utils::e2(
                        format!("'{}' is not to a list", left),
                        doc.name,
                        condition.line_number,
                    );
                }
                if boolean.as_str() == "IsNotEmpty" {
                    Boolean::IsNotEmpty { value }
                } else {
                    Boolean::IsEmpty { value }
                }
            }
            "NotEqual" | "Equal" => {
                if let Some(right) = right {
                    let left = ftd::interpreter2::PropertyValue::from_string_with_argument(
                        left.as_str(),
                        doc,
                        None,
                        false,
                        condition.line_number,
                        definition_name_with_arguments,
                    )?;
                    let right = ftd::interpreter2::PropertyValue::from_string_with_argument(
                        right.as_str(),
                        doc,
                        Some(&ftd::interpreter2::KindData {
                            kind: left.kind(),
                            caption: false,
                            body: false,
                        }),
                        false,
                        condition.line_number,
                        definition_name_with_arguments,
                    )?;
                    Boolean::Equal { left, right }
                } else {
                    Boolean::Equal {
                        left: ftd::interpreter2::PropertyValue::from_string_with_argument(
                            left.as_str(),
                            doc,
                            None,
                            false,
                            condition.line_number,
                            definition_name_with_arguments,
                        )?,
                        right: ftd::interpreter2::PropertyValue::Value {
                            value: ftd::interpreter2::Value::Boolean {
                                value: boolean.as_str() == "Equal",
                            },
                            line_number: condition.line_number,
                        },
                    }
                }
            }
            _ => {
                return ftd::interpreter2::utils::e2(
                    format!("'{}' is not valid condition", condition.expression),
                    doc.name,
                    condition.line_number,
                )
            }
        })
    }

    pub fn boolean_left_right(
        line_number: usize,
        expr: &str,
        doc_id: &str,
    ) -> ftd::interpreter2::Result<(String, String, Option<String>)> {
        let expr: String = expr.split_whitespace().collect::<Vec<&str>>().join(" ");
        if expr == "true" || expr == "false" {
            return Ok(("Literal".to_string(), expr, None));
        }
        let (left, rest) = match expr.split_once(' ') {
            None => return Ok(("Equal".to_string(), expr.to_string(), None)),
            Some(v) => v,
        };
        if left == "not" {
            return Ok(("NotEqual".to_string(), rest.to_string(), None));
        }
        Ok(match rest {
            "is not null" => ("IsNotNull".to_string(), left.to_string(), None),
            "is null" => ("IsNull".to_string(), left.to_string(), None),
            "is not empty" => ("IsNotEmpty".to_string(), left.to_string(), None),
            "is empty" => ("IsEmpty".to_string(), left.to_string(), None),
            _ if rest.starts_with("==") => (
                "Equal".to_string(),
                left.to_string(),
                Some(rest.replace("==", "").trim().to_string()),
            ),
            _ => {
                return ftd::interpreter2::utils::e2(
                    format!("'{}' is not valid condition", rest),
                    doc_id,
                    line_number,
                )
            }
        })
    }
}
