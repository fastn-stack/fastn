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
        doc: &ftd::ftd2021::p2::TDoc,
    ) -> ftd::ftd2021::p1::Result<ftd::Condition> {
        let (variable, value) = match self {
            Self::Equal { left, right } => {
                let variable = resolve_variable(left, line_number, doc)?;

                let value = match right {
                    ftd::PropertyValue::Value { value } => value.to_owned(),
                    ftd::PropertyValue::Variable { name, .. } => doc.get_value(0, name)?,
                    _ => {
                        return ftd::ftd2021::p2::utils::e2(
                            format!("{right:?} must be value or argument"),
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
                    ftd::Value::String {
                        text: "$IsNotNull$".to_string(),
                        source: ftd::TextSource::Header,
                    },
                )
            }
            Self::IsNull { value } => {
                let variable = resolve_variable(value, line_number, doc)?;
                (
                    variable,
                    ftd::Value::String {
                        text: "$IsNull$".to_string(),
                        source: ftd::TextSource::Header,
                    },
                )
            }
            _ => {
                return ftd::ftd2021::p2::utils::e2(
                    format!("{self:?} must not happen"),
                    doc.name,
                    line_number,
                );
            }
        };
        return match value.to_serde_value() {
            None => {
                return ftd::ftd2021::p2::utils::e2(
                    format!(
                        "expected value of type String, Integer, Decimal or Boolean, found: {value:?}"
                    ),
                    doc.name,
                    line_number,
                );
            }
            Some(value) => Ok(ftd::Condition { variable, value }),
        };

        fn resolve_variable(
            value: &ftd::PropertyValue,
            line_number: usize,
            doc: &ftd::ftd2021::p2::TDoc,
        ) -> ftd::ftd2021::p1::Result<String> {
            match value {
                ftd::PropertyValue::Variable { name, .. }
                | ftd::PropertyValue::Reference { name, .. } => Ok(name.to_string()),
                _ => ftd::ftd2021::p2::utils::e2(
                    format!("{value:?} must be variable or local variable"),
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
    ) -> ftd::ftd2021::p1::Result<(String, String, Option<String>)> {
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
                return ftd::ftd2021::p2::utils::e2(
                    format!("'{rest}' is not valid condition"),
                    doc_id,
                    line_number,
                );
            }
        })
    }

    pub fn from_expression(
        expr: &str,
        doc: &ftd::ftd2021::p2::TDoc,
        arguments: &ftd::Map<ftd::ftd2021::p2::Kind>,
        left_right_resolved_property: (Option<ftd::PropertyValue>, Option<ftd::PropertyValue>),
        line_number: usize,
    ) -> ftd::ftd2021::p1::Result<Self> {
        let (boolean, mut left, mut right) =
            ftd::ftd2021::p2::Boolean::boolean_left_right(line_number, expr, doc.name)?;
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
                        return ftd::ftd2021::p2::utils::e2(
                            format!("'{left}' is not to an optional"),
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
                    .unwrap_or(ftd::PropertyValue::Variable {
                        name: left.trim_start_matches('$').to_string(),
                        kind: ftd::ftd2021::p2::Kind::Element,
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
                    return ftd::ftd2021::p2::utils::e2(
                        format!("'{left}' is not to a list"),
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
                            Some(ftd::ftd2021::p2::Kind::boolean()),
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
                return ftd::ftd2021::p2::utils::e2(
                    format!("'{expr}' is not valid condition"),
                    doc.name,
                    line_number,
                );
            }
        });

        fn property_value(
            value: &str,
            expected_kind: Option<ftd::ftd2021::p2::Kind>,
            doc: &ftd::ftd2021::p2::TDoc,
            arguments: &ftd::Map<ftd::ftd2021::p2::Kind>,
            loop_already_resolved_property: Option<ftd::PropertyValue>,
            line_number: usize,
        ) -> ftd::ftd2021::p1::Result<ftd::PropertyValue> {
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
                        Some(ftd::PropertyValue::Variable { .. }) => {
                            loop_already_resolved_property.clone().expect("")
                        }
                        _ if value.starts_with("$PARENT") => ftd::PropertyValue::Variable {
                            name: value.trim_start_matches('$').to_string(),
                            kind: ftd::ftd2021::p2::Kind::Element,
                        },
                        _ => return Err(e),
                    },
                },
            )
        }
    }

    pub fn is_constant(&self) -> bool {
        let is_loop_constant = {
            let mut constant = false;
            if let ftd::ftd2021::p2::Boolean::Equal {
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
        ) && !matches!(self, Self::IsNotNull { .. })
            && !matches!(self, Self::IsNull { .. }))
            || is_loop_constant
    }

    pub fn is_arg_constant(&self) -> bool {
        let is_loop_constant = {
            let mut constant = false;
            if let ftd::ftd2021::p2::Boolean::Equal {
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
        ) && !matches!(
            self,
            Self::Equal {
                left: ftd::PropertyValue::Variable { .. },
                right: ftd::PropertyValue::Variable { .. },
                ..
            }
        ) && !matches!(self, Self::IsNotNull { .. })
            && !matches!(self, Self::IsNull { .. }))
            || is_loop_constant
    }

    pub fn eval(
        &self,
        line_number: usize,
        doc: &ftd::ftd2021::p2::TDoc,
    ) -> ftd::ftd2021::p1::Result<bool> {
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
                return ftd::ftd2021::p2::utils::e2(
                    format!("unknown Boolean found: {self:?}"),
                    doc.name,
                    line_number,
                );
            }
        })
    }

    pub fn set_null(&self, line_number: usize, doc_id: &str) -> ftd::ftd2021::p1::Result<bool> {
        Ok(match self {
            Self::Literal { .. } | Self::IsNotEmpty { .. } | Self::IsEmpty { .. } => true,
            Self::Equal { left, right } => matches!(
                (left, right),
                (
                    ftd::PropertyValue::Value { .. },
                    ftd::PropertyValue::Value { .. }
                ) | (
                    ftd::PropertyValue::Value { .. },
                    ftd::PropertyValue::Variable { .. }
                ) | (
                    ftd::PropertyValue::Variable { .. },
                    ftd::PropertyValue::Value { .. }
                ) | (
                    ftd::PropertyValue::Variable { .. },
                    ftd::PropertyValue::Variable { .. }
                )
            ),
            Self::IsNotNull { .. } | Self::IsNull { .. } => false,
            _ => {
                return ftd::ftd2021::p2::utils::e2(
                    format!("unimplemented for type: {self:?}"),
                    doc_id,
                    line_number,
                );
            }
        })
    }
}
