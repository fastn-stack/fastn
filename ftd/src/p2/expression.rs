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
    pub fn to_condition(&self) -> ftd_rt::Condition {
        match self {
            Self::Equal { left, right } => match (left, right) {
                (
                    ftd::PropertyValue::Reference { name, .. },
                    ftd::PropertyValue::Value {
                        value: ftd::Value::Boolean { value },
                    },
                ) => ftd_rt::Condition {
                    variable: name.to_string(),
                    value: value.to_string(),
                },
                _ => unreachable!(
                    "{:?} must be boolean variable and {:?} exact value",
                    left, right
                ),
            },
            _ => unreachable!("{:?} must not happen", self),
        }
    }

    fn boolean_value(
        expr: &str,
        doc: &crate::p2::TDoc,
        component: &str,
        arguments: &std::collections::BTreeMap<String, crate::p2::Kind>,
    ) -> ftd::p1::Result<ftd::PropertyValue> {
        Ok(match expr.strip_prefix('$') {
            Some(v) => {
                let found_kind = match arguments.get(v) {
                    Some(k) => k,
                    None => {
                        return crate::e(format!("'{}' is not an argument of '{}'", v, component));
                    }
                };
                if !found_kind.is_boolean() {
                    return crate::e(format!("'{}' is not to a boolean", expr));
                }
                crate::PropertyValue::Argument {
                    name: v.to_string(),
                    kind: found_kind.to_owned(),
                }
            }
            None => {
                let found_kind = doc.get_value(expr)?.kind();
                if !found_kind.is_boolean() {
                    return crate::e(format!("'{}' is not to a boolean", expr));
                }
                crate::PropertyValue::Reference {
                    name: doc.resolve_name(expr)?,
                    kind: found_kind,
                }
            }
        })
    }

    pub fn from_expression(
        expr: &str,
        doc: &crate::p2::TDoc,
        component: &str,
        arguments: &std::collections::BTreeMap<String, crate::p2::Kind>,
    ) -> ftd::p1::Result<Self> {
        let expr = expr.split_whitespace().collect::<Vec<&str>>().join(" ");
        if expr == "true" {
            return Ok(Boolean::Literal { value: true });
        }
        if expr == "false" {
            return Ok(Boolean::Literal { value: false });
        }
        let (first, rest) = match expr.split_once(' ') {
            Some(v) => v,
            None => {
                return Ok(Boolean::Equal {
                    left: Self::boolean_value(expr.as_str(), doc, component, arguments)?,
                    right: ftd::PropertyValue::Value {
                        value: ftd::Value::Boolean { value: true },
                    },
                });
            }
        };

        if first == "not" {
            return Ok(Boolean::Equal {
                left: Self::boolean_value(rest, doc, component, arguments)?,
                right: ftd::PropertyValue::Value {
                    value: ftd::Value::Boolean { value: false },
                },
            });
        }

        let value = match first.strip_prefix('$') {
            Some(v) => {
                let found_kind = match arguments.get(v) {
                    Some(k) => k,
                    None => {
                        return crate::e(format!("'{}' is not an argument of '{}'", v, component));
                    }
                };
                if !found_kind.is_optional() {
                    return crate::e(format!("'{}' is not to an optional", first));
                }
                crate::PropertyValue::Argument {
                    name: v.to_string(),
                    kind: found_kind.to_owned(),
                }
            }
            None => {
                let found_kind = doc.get_value(first)?.kind();
                if !found_kind.is_optional() {
                    return crate::e(format!("'{}' is not to an optional", first));
                }
                crate::PropertyValue::Reference {
                    name: doc.resolve_name(first)?,
                    kind: found_kind,
                }
            }
        };

        Ok(match rest {
            "is not null" => Boolean::IsNotNull { value },
            "is null" => Boolean::IsNull { value },
            _ => return crate::e(format!("'{}' is not valid condition", rest)),
        })
    }

    pub fn is_constant(&self) -> bool {
        !matches!(
            self,
            Self::Equal {
                left: ftd::PropertyValue::Reference { .. },
                right: ftd::PropertyValue::Value {
                    value: ftd::Value::Boolean { .. }
                },
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
            Self::Equal { left, right } => {
                left.resolve(arguments, doc)? == right.resolve(arguments, doc)?
            }
            _ => todo!(),
        })
    }
}
