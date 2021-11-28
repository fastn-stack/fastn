#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Boolean {
    // if: $caption is not null
    IsNotNull {
        value: ftd::PropertyValue,
    },
    // if: $caption is null
    IsNull {
        value: ftd::PropertyValue,
    },
    // if: $list is not empty
    IsNotEmpty {
        value: ftd::PropertyValue,
    },
    // if: $list is empty
    IsEmpty {
        value: ftd::PropertyValue,
    },
    // if: $caption == hello | if: $foo
    Equal {
        left: ftd::PropertyValue,
        right: ftd::PropertyValue,
    },
    // if: $caption != hello
    NotEqual {
        left: ftd::PropertyValue,
        right: ftd::PropertyValue,
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
        value: ftd::PropertyValue,
    },
}

impl Boolean {
    pub fn to_condition(
        &self,
        line_number: usize,
        all_locals: &mut ftd_rt::Map,
        arguments: &std::collections::BTreeMap<String, crate::Value>,
        doc_id: &str,
    ) -> ftd::p1::Result<ftd_rt::Condition> {
        let (variable, value) = match self {
            Self::Equal { left, right } => {
                let variable = match left {
                    ftd::PropertyValue::Reference { name, .. } => name.to_string(),
                    ftd::PropertyValue::Variable { name, .. } => {
                        if let Some(string_container) = all_locals.get(name) {
                            format!("@{}@{}", name, string_container)
                        } else if name.eq("MOUSE-IN") {
                            let string_container = all_locals.get("MOUSE-IN-TEMP").unwrap().clone();
                            all_locals.insert("MOUSE-IN".to_string(), string_container.to_string());
                            format!("@MOUSE-IN@{}", string_container)
                        } else {
                            return ftd::e2(
                                format!("Can't find the local variable {}", name),
                                doc_id,
                                line_number,
                            );
                        }
                    }
                    _ => {
                        return ftd::e2(
                            format!("{:?} must be variable or local variable", left),
                            doc_id,
                            line_number,
                        );
                    }
                };

                let value = match right {
                    ftd::PropertyValue::Value { value } => value.to_owned(),
                    ftd::PropertyValue::Variable { name, kind } => {
                        if let Some(arg) = arguments.get(name) {
                            if arg.kind().is_same_as(kind) {
                                arg.to_owned()
                            } else {
                                return ftd::e2(
                                    format!(
                                        "kind mismatch expected: {:?} found: {:?}",
                                        kind,
                                        arg.kind()
                                    ),
                                    doc_id,
                                    line_number,
                                );
                            }
                        } else {
                            return ftd::e2(
                                format!("argument not found {}", name),
                                doc_id,
                                line_number,
                            );
                        }
                    }
                    _ => {
                        return ftd::e2(
                            format!("{:?} must be value or argument", right),
                            doc_id,
                            line_number,
                        );
                    }
                };

                (variable, value)
            }
            _ => {
                return ftd::e2(
                    format!("{:?} must not happen", self),
                    doc_id,
                    line_number,
                )
            }
        };
        match value.to_string() {
            None => {
                return ftd::e2(
                    format!(
                        "expected value of type String, Integer, Decimal or Boolean, found: {:?}",
                        value
                    ),
                    doc_id,
                    line_number,
                )
            }
            Some(value) => Ok(ftd_rt::Condition { variable, value }),
        }
    }

    pub fn boolean_left_right(
        line_number: usize,
        expr: &str,
        doc_id: &str,
    ) -> ftd::p1::Result<(String, String, Option<String>)> {
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
                return ftd::e2(
                    format!("'{}' is not valid condition", rest),
                    doc_id,
                    line_number,
                )
            }
        })
    }

    pub fn from_expression(
        expr: &str,
        doc: &crate::p2::TDoc,
        arguments: &std::collections::BTreeMap<String, crate::p2::Kind>,
        left_right_resolved_property: (Option<crate::PropertyValue>, Option<crate::PropertyValue>),
        line_number: usize,
    ) -> ftd::p1::Result<Self> {
        let (boolean, left, right) =
            ftd::p2::Boolean::boolean_left_right(line_number, expr, doc.name)?;
        return Ok(match boolean.as_str() {
            "Literal" => Boolean::Literal {
                value: left == "true",
            },
            "IsNotNull" | "IsNull" => {
                let value = property_value(
                    &left,
                    None,
                    doc,
                    arguments,
                    left_right_resolved_property.0,
                    line_number,
                )?;
                if !value.kind().is_optional() {
                    return ftd::e2(
                        format!("'{}' is not to an optional", left),
                        doc.name,
                        line_number,
                    );
                }
                if boolean.as_str() == "IsNotNull" {
                    Boolean::IsNotNull { value }
                } else {
                    Boolean::IsNull { value }
                }
            }
            "IsNotEmpty" | "IsEmpty" => {
                let value = property_value(
                    &left,
                    None,
                    doc,
                    arguments,
                    left_right_resolved_property.0,
                    line_number,
                )?;
                if !value.kind().is_list() {
                    return ftd::e2(
                        format!("'{}' is not to a list", left),
                        doc.name,
                        line_number,
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
                    let left = property_value(
                        &left,
                        None,
                        doc,
                        arguments,
                        left_right_resolved_property.0,
                        line_number,
                    )?;
                    Boolean::Equal {
                        left: left.to_owned(),
                        right: property_value(
                            &right,
                            Some(left.kind()),
                            doc,
                            arguments,
                            left_right_resolved_property.1,
                            line_number,
                        )?,
                    }
                } else {
                    Boolean::Equal {
                        left: property_value(
                            &left,
                            Some(ftd::p2::Kind::boolean()),
                            doc,
                            arguments,
                            left_right_resolved_property.0,
                            line_number,
                        )?,
                        right: ftd::PropertyValue::Value {
                            value: ftd::Value::Boolean {
                                value: boolean.as_str() == "Equal",
                            },
                        },
                    }
                }
            }
            _ => {
                return ftd::e2(
                    format!("'{}' is not valid condition", expr),
                    doc.name,
                    line_number,
                )
            }
        });

        fn property_value(
            value: &str,
            expected_kind: Option<ftd::p2::Kind>,
            doc: &ftd::p2::TDoc,
            arguments: &std::collections::BTreeMap<String, ftd::p2::Kind>,
            loop_already_resolved_property: Option<crate::PropertyValue>,
            line_number: usize,
        ) -> ftd::p1::Result<ftd::PropertyValue> {
            Ok(
                match ftd::PropertyValue::resolve_value(
                    line_number,
                    value,
                    expected_kind,
                    doc,
                    arguments,
                    None,
                ) {
                    Ok(v) => v,
                    Err(e) => match &loop_already_resolved_property {
                        Some(crate::PropertyValue::Variable { .. }) => {
                            loop_already_resolved_property.clone().expect("")
                        }
                        _ => return Err(e),
                    },
                },
            )
        }
    }

    pub fn is_constant(&self) -> bool {
        let is_loop_constant = {
            let mut constant = false;
            if let ftd::p2::Boolean::Equal {
                left: ftd::PropertyValue::Variable { name, .. },
                right: ftd::PropertyValue::Value { .. },
            } = self
            {
                if name.starts_with("$loop$") {
                    constant = true;
                }
            }
            constant
        };
        (!matches!(
            self,
            Self::Equal {
                left: ftd::PropertyValue::Reference { .. },
                right: ftd::PropertyValue::Value { .. },
                ..
            }
        ) && !matches!(
            self,
            Self::Equal {
                left: ftd::PropertyValue::Variable { .. },
                right: ftd::PropertyValue::Value { .. },
                ..
            }
        )) || is_loop_constant
    }

    pub fn is_arg_constant(&self) -> bool {
        let is_loop_constant = {
            let mut constant = false;
            if let ftd::p2::Boolean::Equal {
                left: ftd::PropertyValue::Variable { name, .. },
                right: ftd::PropertyValue::Value { .. },
            } = self
            {
                if name.starts_with("$loop$") {
                    constant = true;
                }
            }
            constant
        };
        (!matches!(
            self,
            Self::Equal {
                left: ftd::PropertyValue::Reference { .. },
                right: ftd::PropertyValue::Value { .. },
                ..
            }
        ) && !matches!(
            self,
            Self::Equal {
                left: ftd::PropertyValue::Variable { .. },
                right: ftd::PropertyValue::Value { .. },
                ..
            }
        ) && !matches!(
            self,
            Self::Equal {
                left: ftd::PropertyValue::Reference { .. },
                right: ftd::PropertyValue::Variable { .. },
                ..
            }
        )) || is_loop_constant
    }

    pub fn eval(
        &self,
        line_number: usize,
        arguments: &std::collections::BTreeMap<String, crate::Value>,
        doc: &crate::p2::TDoc,
    ) -> crate::p1::Result<bool> {
        Ok(match self {
            Self::Literal { value } => *value,
            Self::IsNotNull { value } => !value.resolve(line_number, arguments, doc)?.is_null(),
            Self::IsNull { value } => value.resolve(line_number, arguments, doc)?.is_null(),
            Self::IsNotEmpty { value } => !value.resolve(line_number, arguments, doc)?.is_empty(),
            Self::IsEmpty { value } => value.resolve(line_number, arguments, doc)?.is_empty(),
            Self::Equal { left, right } => left
                .resolve(line_number, arguments, doc)?
                .is_equal(&right.resolve(line_number, arguments, doc)?),
            _ => {
                return ftd::e2(
                    format!("unknown Boolean found: {:?}", self),
                    doc.name,
                    line_number,
                )
            }
        })
    }

    pub fn set_null(&self, line_number: usize, doc_id: &str) -> crate::p1::Result<bool> {
        Ok(match self {
            Self::Literal { .. }
            | Self::IsNotNull { .. }
            | Self::IsNull { .. }
            | Self::IsNotEmpty { .. }
            | Self::IsEmpty { .. } => true,
            Self::Equal { left, right } => match (left, right) {
                (ftd::PropertyValue::Value { .. }, ftd::PropertyValue::Value { .. })
                | (ftd::PropertyValue::Value { .. }, ftd::PropertyValue::Variable { .. })
                | (ftd::PropertyValue::Variable { .. }, ftd::PropertyValue::Value { .. })
                | (ftd::PropertyValue::Variable { .. }, ftd::PropertyValue::Variable { .. }) => {
                    true
                }
                _ => false,
            },
            _ => {
                return ftd::e2(
                    format!("unimplemented for type: {:?}", self),
                    doc_id,
                    line_number,
                )
            }
        })
    }
}
