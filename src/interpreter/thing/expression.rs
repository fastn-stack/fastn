#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Boolean {
    // if: $caption is not null
    IsNotNull {
        value: ftd::interpreter::PropertyValue,
    },
    // if: $caption is null
    IsNull {
        value: ftd::interpreter::PropertyValue,
    },
    // if: $list is not empty
    IsNotEmpty {
        value: ftd::interpreter::PropertyValue,
    },
    // if: $list is empty
    IsEmpty {
        value: ftd::interpreter::PropertyValue,
    },
    // if: $caption == hello | if: $foo
    Equal {
        left: ftd::interpreter::PropertyValue,
        right: ftd::interpreter::PropertyValue,
    },
    // if: $caption != hello
    NotEqual {
        left: ftd::interpreter::PropertyValue,
        right: ftd::interpreter::PropertyValue,
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
        value: ftd::interpreter::PropertyValue,
    },
}

impl Boolean {
    pub fn to_condition(
        &self,
        line_number: usize,
        doc: &ftd::interpreter::TDoc,
    ) -> ftd::p11::Result<ftd::Condition> {
        let (variable, value) = match self {
            Self::Equal { left, right } => {
                let variable = resolve_variable(left, line_number, doc)?;

                let value = match right {
                    ftd::interpreter::PropertyValue::Value { value } => value.to_owned(),
                    ftd::interpreter::PropertyValue::Variable { name, .. } => {
                        doc.get_value(0, name)?
                    }
                    _ => {
                        return ftd::interpreter::utils::e2(
                            format!("{:?} must be value or argument", right),
                            doc.name,
                            line_number,
                        );
                    }
                };

                (variable, value)
            }
            Self::IsNotNull { value } => {
                let variable = resolve_variable(value, line_number, doc)?;
                (
                    variable,
                    ftd::interpreter::Value::String {
                        text: "$IsNotNull$".to_string(),
                        source: ftd::TextSource::Header,
                    },
                )
            }
            Self::IsNull { value } => {
                let variable = resolve_variable(value, line_number, doc)?;
                (
                    variable,
                    ftd::interpreter::Value::String {
                        text: "$IsNull$".to_string(),
                        source: ftd::TextSource::Header,
                    },
                )
            }
            _ => {
                return ftd::interpreter::utils::e2(
                    format!("{:?} must not happen", self),
                    doc.name,
                    line_number,
                )
            }
        };
        return match value.to_serde_value() {
            None => {
                return ftd::interpreter::utils::e2(
                    format!(
                        "expected value of type String, Integer, Decimal or Boolean, found: {:?}",
                        value
                    ),
                    doc.name,
                    line_number,
                )
            }
            Some(value) => Ok(ftd::Condition { variable, value }),
        };

        fn resolve_variable(
            value: &ftd::interpreter::PropertyValue,
            line_number: usize,
            doc: &ftd::interpreter::TDoc,
        ) -> ftd::p11::Result<String> {
            match value {
                ftd::interpreter::PropertyValue::Variable { name, .. }
                | ftd::interpreter::PropertyValue::Reference { name, .. } => Ok(name.to_string()),
                _ => ftd::interpreter::utils::e2(
                    format!("{:?} must be variable or local variable", value),
                    doc.name,
                    line_number,
                ),
            }
        }
    }

    pub fn boolean_left_right(
        line_number: usize,
        expr: &str,
        doc_id: &str,
    ) -> ftd::p11::Result<(String, String, Option<String>)> {
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
                return ftd::interpreter::utils::e2(
                    format!("'{}' is not valid condition", rest),
                    doc_id,
                    line_number,
                )
            }
        })
    }

    pub fn from_expression(
        expr: &str,
        doc: &ftd::interpreter::TDoc,
        arguments: &ftd::Map<ftd::interpreter::Kind>,
        left_right_resolved_property: (
            Option<ftd::interpreter::PropertyValue>,
            Option<ftd::interpreter::PropertyValue>,
        ),
        line_number: usize,
    ) -> ftd::p11::Result<Self> {
        let (boolean, mut left, mut right) =
            ftd::interpreter::Boolean::boolean_left_right(line_number, expr, doc.name)?;
        left = doc.resolve_reference_name(line_number, left.as_str(), arguments)?;
        if let Some(ref r) = right {
            right = doc.resolve_reference_name(line_number, r, arguments).ok();
        }
        return Ok(match boolean.as_str() {
            "Literal" => Boolean::Literal {
                value: left == "true",
            },
            "IsNotNull" | "IsNull" => {
                let value = if !left.starts_with("$PARENT") {
                    let value = property_value(
                        &left,
                        None,
                        doc,
                        arguments,
                        left_right_resolved_property.0,
                        line_number,
                    )?;
                    if !value.kind().is_optional() {
                        return ftd::interpreter::utils::e2(
                            format!("'{}' is not to an optional", left),
                            doc.name,
                            line_number,
                        );
                    }
                    value
                } else {
                    property_value(
                        &left,
                        None,
                        doc,
                        arguments,
                        left_right_resolved_property.0,
                        line_number,
                    )
                    .unwrap_or(ftd::interpreter::PropertyValue::Variable {
                        name: left.trim_start_matches('$').to_string(),
                        kind: ftd::interpreter::Kind::Element,
                    })
                };
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
                    return ftd::interpreter::utils::e2(
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
                            Some(ftd::interpreter::Kind::boolean()),
                            doc,
                            arguments,
                            left_right_resolved_property.0,
                            line_number,
                        )?,
                        right: ftd::interpreter::PropertyValue::Value {
                            value: ftd::interpreter::Value::Boolean {
                                value: boolean.as_str() == "Equal",
                            },
                        },
                    }
                }
            }
            _ => {
                return ftd::interpreter::utils::e2(
                    format!("'{}' is not valid condition", expr),
                    doc.name,
                    line_number,
                )
            }
        });

        fn property_value(
            value: &str,
            expected_kind: Option<ftd::interpreter::Kind>,
            doc: &ftd::interpreter::TDoc,
            arguments: &ftd::Map<ftd::interpreter::Kind>,
            loop_already_resolved_property: Option<ftd::interpreter::PropertyValue>,
            line_number: usize,
        ) -> ftd::p11::Result<ftd::interpreter::PropertyValue> {
            Ok(
                match ftd::interpreter::PropertyValue::resolve_value(
                    line_number,
                    value,
                    expected_kind,
                    doc,
                    arguments,
                    None,
                ) {
                    Ok(v) => v,
                    Err(e) => match &loop_already_resolved_property {
                        Some(ftd::interpreter::PropertyValue::Variable { .. }) => {
                            loop_already_resolved_property.clone().expect("")
                        }
                        _ if value.starts_with("$PARENT") => {
                            ftd::interpreter::PropertyValue::Variable {
                                name: value.trim_start_matches('$').to_string(),
                                kind: ftd::interpreter::Kind::Element,
                            }
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
            if let ftd::interpreter::Boolean::Equal {
                left: ftd::interpreter::PropertyValue::Variable { name, .. },
                right: ftd::interpreter::PropertyValue::Value { .. },
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
                left: ftd::interpreter::PropertyValue::Reference { .. },
                right: ftd::interpreter::PropertyValue::Value { .. },
                ..
            }
        ) && !matches!(
            self,
            Self::Equal {
                left: ftd::interpreter::PropertyValue::Variable { .. },
                right: ftd::interpreter::PropertyValue::Value { .. },
                ..
            }
        ) && !matches!(self, Self::IsNotNull { .. })
            && !matches!(self, Self::IsNull { .. }))
            || is_loop_constant
    }

    pub fn is_arg_constant(&self) -> bool {
        let is_loop_constant = {
            let mut constant = false;
            if let ftd::interpreter::Boolean::Equal {
                left: ftd::interpreter::PropertyValue::Variable { name, .. },
                right: ftd::interpreter::PropertyValue::Value { .. },
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
                left: ftd::interpreter::PropertyValue::Reference { .. },
                right: ftd::interpreter::PropertyValue::Value { .. },
                ..
            }
        ) && !matches!(
            self,
            Self::Equal {
                left: ftd::interpreter::PropertyValue::Variable { .. },
                right: ftd::interpreter::PropertyValue::Value { .. },
                ..
            }
        ) && !matches!(
            self,
            Self::Equal {
                left: ftd::interpreter::PropertyValue::Reference { .. },
                right: ftd::interpreter::PropertyValue::Variable { .. },
                ..
            }
        ) && !matches!(
            self,
            Self::Equal {
                left: ftd::interpreter::PropertyValue::Variable { .. },
                right: ftd::interpreter::PropertyValue::Variable { .. },
                ..
            }
        ) && !matches!(self, Self::IsNotNull { .. })
            && !matches!(self, Self::IsNull { .. }))
            || is_loop_constant
    }

    pub fn eval(&self, line_number: usize, doc: &ftd::interpreter::TDoc) -> ftd::p11::Result<bool> {
        Ok(match self {
            Self::Literal { value } => *value,
            Self::IsNotNull { value } => !value.resolve(line_number, doc)?.is_null(),
            Self::IsNull { value } => value.resolve(line_number, doc)?.is_null(),
            Self::IsNotEmpty { value } => !value.resolve(line_number, doc)?.is_empty(),
            Self::IsEmpty { value } => value.resolve(line_number, doc)?.is_empty(),
            Self::Equal { left, right } => left
                .resolve(line_number, doc)?
                .is_equal(&right.resolve(line_number, doc)?),
            _ => {
                return ftd::interpreter::utils::e2(
                    format!("unknown Boolean found: {:?}", self),
                    doc.name,
                    line_number,
                )
            }
        })
    }

    pub fn set_null(&self, line_number: usize, doc_id: &str) -> ftd::p11::Result<bool> {
        Ok(match self {
            Self::Literal { .. } | Self::IsNotEmpty { .. } | Self::IsEmpty { .. } => true,
            Self::Equal { left, right } => match (left, right) {
                (
                    ftd::interpreter::PropertyValue::Value { .. },
                    ftd::interpreter::PropertyValue::Value { .. },
                )
                | (
                    ftd::interpreter::PropertyValue::Value { .. },
                    ftd::interpreter::PropertyValue::Variable { .. },
                )
                | (
                    ftd::interpreter::PropertyValue::Variable { .. },
                    ftd::interpreter::PropertyValue::Value { .. },
                )
                | (
                    ftd::interpreter::PropertyValue::Variable { .. },
                    ftd::interpreter::PropertyValue::Variable { .. },
                ) => true,
                _ => false,
            },
            Self::IsNotNull { .. } | Self::IsNull { .. } => false,
            _ => {
                return ftd::interpreter::utils::e2(
                    format!("unimplemented for type: {:?}", self),
                    doc_id,
                    line_number,
                )
            }
        })
    }
}
