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
        all_locals: &mut ftd_rt::Map,
        arguments: &std::collections::BTreeMap<String, crate::Value>,
    ) -> ftd::p1::Result<ftd_rt::Condition> {
        let (variable, value) = match self {
            Self::Equal { left, right } => {
                let variable = match left {
                    ftd::PropertyValue::Reference { name, .. } => name.to_string(),
                    ftd::PropertyValue::LocalVariable { name, .. } => {
                        if let Some(string_container) = all_locals.get(name) {
                            format!("@{}@{}", name, string_container)
                        } else if name.eq("mouse-in") {
                            let string_container = all_locals.get("mouse-in-temp").unwrap().clone();
                            all_locals.insert("mouse-in".to_string(), string_container.to_string());
                            format!("@mouse-in@{}", string_container)
                        } else {
                            return crate::e(format!("Can't find the local variable {}", name));
                        }
                    }
                    _ => {
                        return crate::e(format!("{:?} must be variable or local variable", left));
                    }
                };

                let value = match right {
                    ftd::PropertyValue::Value { value } => value.to_owned(),
                    ftd::PropertyValue::Argument { name, kind } => {
                        if let Some(arg) = arguments.get(name) {
                            if arg.kind().is_same_as(kind) {
                                arg.to_owned()
                            } else {
                                return crate::e(format!(
                                    "kind mismatch expected: {:?} found: {:?}",
                                    kind,
                                    arg.kind()
                                ));
                            }
                        } else {
                            return crate::e(format!("argument not found {}", name));
                        }
                    }
                    _ => {
                        return crate::e(format!("{:?} must be value or argument", right));
                    }
                };

                (variable, value)
            }
            _ => return crate::e(format!("{:?} must not happen", self)),
        };
        match value.to_string() {
            None => {
                return crate::e(format!(
                    "expected value of type String, Integer, Decimal or Boolean, found: {:?}",
                    value
                ))
            }
            Some(value) => Ok(ftd_rt::Condition { variable, value }),
        }
    }

    pub fn boolean_left_right(expr: &str) -> ftd::p1::Result<(String, String, Option<String>)> {
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
            _ => return crate::e(format!("'{}' is not valid condition", rest)),
        })
    }

    pub fn from_expression(
        expr: &str,
        doc: &crate::p2::TDoc,
        arguments: &std::collections::BTreeMap<String, crate::p2::Kind>,
        locals: &std::collections::BTreeMap<String, crate::p2::Kind>,
        left_right_resolved_property: (Option<crate::PropertyValue>, Option<crate::PropertyValue>),
    ) -> ftd::p1::Result<Self> {
        let (boolean, left, right) = ftd::p2::Boolean::boolean_left_right(expr)?;
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
                    locals,
                    left_right_resolved_property.0,
                )?;
                if !value.kind().is_optional() {
                    return crate::e(format!("'{}' is not to an optional", left));
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
                    locals,
                    left_right_resolved_property.0,
                )?;
                if !value.kind().is_list() {
                    return crate::e(format!("'{}' is not to a list", left));
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
                        locals,
                        left_right_resolved_property.0,
                    )?;
                    Boolean::Equal {
                        left: left.to_owned(),
                        right: property_value(
                            &right,
                            Some(left.kind()),
                            doc,
                            arguments,
                            locals,
                            left_right_resolved_property.1,
                        )?,
                    }
                } else {
                    Boolean::Equal {
                        left: property_value(
                            &left,
                            Some(ftd::p2::Kind::boolean()),
                            doc,
                            arguments,
                            locals,
                            left_right_resolved_property.0,
                        )?,
                        right: ftd::PropertyValue::Value {
                            value: ftd::Value::Boolean {
                                value: boolean.as_str() == "Equal",
                            },
                        },
                    }
                }
            }
            _ => return crate::e(format!("'{}' is not valid condition", expr)),
        });

        fn property_value(
            value: &str,
            expected_kind: Option<ftd::p2::Kind>,
            doc: &ftd::p2::TDoc,
            arguments: &std::collections::BTreeMap<String, ftd::p2::Kind>,
            locals: &std::collections::BTreeMap<String, ftd::p2::Kind>,
            loop_already_resolved_property: Option<crate::PropertyValue>,
        ) -> ftd::p1::Result<ftd::PropertyValue> {
            Ok(
                match ftd::PropertyValue::resolve_value(
                    value,
                    expected_kind,
                    doc,
                    arguments,
                    locals,
                    None,
                    false,
                ) {
                    Ok(v) => v,
                    Err(e) => match &loop_already_resolved_property {
                        Some(crate::PropertyValue::Argument { .. }) => {
                            loop_already_resolved_property.clone().expect("")
                        }
                        _ => return Err(e),
                    },
                },
            )
        }
    }

    pub fn is_constant(&self) -> bool {
        !matches!(
            self,
            Self::Equal {
                left: ftd::PropertyValue::Reference { .. },
                right: ftd::PropertyValue::Value { .. },
                ..
            }
        ) && !matches!(
            self,
            Self::Equal {
                left: ftd::PropertyValue::LocalVariable { .. },
                right: ftd::PropertyValue::Value { .. },
                ..
            }
        )
    }

    pub fn is_arg_constant(&self) -> bool {
        !matches!(
            self,
            Self::Equal {
                left: ftd::PropertyValue::Reference { .. },
                right: ftd::PropertyValue::Value { .. },
                ..
            }
        ) && !matches!(
            self,
            Self::Equal {
                left: ftd::PropertyValue::LocalVariable { .. },
                right: ftd::PropertyValue::Value { .. },
                ..
            }
        ) && !matches!(
            self,
            Self::Equal {
                left: ftd::PropertyValue::Reference { .. },
                right: ftd::PropertyValue::Argument { .. },
                ..
            }
        ) && !matches!(
            self,
            Self::Equal {
                left: ftd::PropertyValue::LocalVariable { .. },
                right: ftd::PropertyValue::Argument { .. },
                ..
            }
        )
    }

    pub fn eval(
        &self,
        arguments: &std::collections::BTreeMap<String, crate::Value>,
        doc: &crate::p2::TDoc,
    ) -> crate::p1::Result<bool> {
        Ok(match self {
            Self::Literal { value } => *value,
            Self::IsNotNull { value } => !value.resolve(arguments, doc)?.is_null(),
            Self::IsNull { value } => value.resolve(arguments, doc)?.is_null(),
            Self::IsNotEmpty { value } => !value.resolve(arguments, doc)?.is_empty(),
            Self::IsEmpty { value } => value.resolve(arguments, doc)?.is_empty(),
            Self::Equal { left, right } => left
                .resolve(arguments, doc)?
                .is_equal(&right.resolve(arguments, doc)?),
            _ => return ftd::e2("unknown Boolean found", self),
        })
    }

    pub fn set_null(&self) -> crate::p1::Result<bool> {
        Ok(match self {
            Self::Literal { .. }
            | Self::IsNotNull { .. }
            | Self::IsNull { .. }
            | Self::IsNotEmpty { .. }
            | Self::IsEmpty { .. } => true,
            Self::Equal { left, right } => match (left, right) {
                (ftd::PropertyValue::Value { .. }, ftd::PropertyValue::Value { .. })
                | (ftd::PropertyValue::Value { .. }, ftd::PropertyValue::Argument { .. })
                | (ftd::PropertyValue::Argument { .. }, ftd::PropertyValue::Value { .. })
                | (ftd::PropertyValue::Argument { .. }, ftd::PropertyValue::Argument { .. }) => {
                    true
                }
                _ => false,
            },
            _ => return crate::e(format!("unimplemented for type: {:?}", self)),
        })
    }
}
