use crate::Value;

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
    pub fn to_condition(&self, all_locals: &ftd_rt::Map) -> ftd::p1::Result<ftd_rt::Condition> {
        Ok(match self {
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
                (
                    ftd::PropertyValue::LocalVariable { name, .. },
                    ftd::PropertyValue::Value {
                        value: ftd::Value::Boolean { value },
                    },
                ) => {
                    if let Some(string_container) = all_locals.get(name) {
                        ftd_rt::Condition {
                            variable: format!("@{}@{}", name, string_container),
                            value: value.to_string(),
                        }
                    } else {
                        return crate::e(format!("Can't find the local variable {}", name));
                    }
                }
                _ => unreachable!(
                    "{:?} must be boolean variable and {:?} exact value",
                    left, right
                ),
            },
            _ => unreachable!("{:?} must not happen", self),
        })
    }

    fn boolean_value(
        expr: &str,
        doc: &crate::p2::TDoc,
        component: &str,
        arguments: &std::collections::BTreeMap<String, crate::p2::Kind>,
        locals: &std::collections::BTreeMap<String, crate::p2::Kind>,
    ) -> ftd::p1::Result<ftd::PropertyValue> {
        Ok(if let Some(v) = expr.strip_prefix('$') {
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
        } else if let Some(v) = expr.strip_prefix('@') {
            let found_kind = match locals.get(v) {
                Some(k) => k,
                None => {
                    return crate::e(format!(
                        "'{}' is not an local variable of '{}'",
                        v, component
                    ));
                }
            };
            if !found_kind.is_boolean() {
                return crate::e(format!("'{}' is not to a boolean", expr));
            }
            crate::PropertyValue::LocalVariable {
                name: v.to_string(),
                kind: found_kind.to_owned(),
            }
        } else {
            let found_kind = doc.get_value(expr)?.kind();
            if !found_kind.is_boolean() {
                return crate::e(format!("'{}' is not to a boolean", expr));
            }
            crate::PropertyValue::Reference {
                name: doc.resolve_name(expr)?,
                kind: found_kind,
            }
        })
    }

    pub fn from_expression(
        expr: &str,
        doc: &crate::p2::TDoc,
        component: &str,
        arguments: &std::collections::BTreeMap<String, crate::p2::Kind>,
        locals: &std::collections::BTreeMap<String, crate::p2::Kind>,
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
                    left: Self::boolean_value(expr.as_str(), doc, component, arguments, locals)?,
                    right: ftd::PropertyValue::Value {
                        value: ftd::Value::Boolean { value: true },
                    },
                });
            }
        };

        if first == "not" {
            return Ok(Boolean::Equal {
                left: Self::boolean_value(rest, doc, component, arguments, locals)?,
                right: ftd::PropertyValue::Value {
                    value: ftd::Value::Boolean { value: false },
                },
            });
        }

        let value = if let Some(v) = first.strip_prefix('$') {
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
        } else if let Some(v) = first.strip_prefix('@') {
            let found_kind = match locals.get(v) {
                Some(k) => k,
                None => {
                    return crate::e(format!("'{}' is not an argument of '{}'", v, component));
                }
            };
            if !found_kind.is_optional() {
                return crate::e(format!("'{}' is not to an optional", first));
            }
            crate::PropertyValue::LocalVariable {
                name: v.to_string(),
                kind: found_kind.to_owned(),
            }
        } else {
            let found_kind = doc.get_value(first)?.kind();
            if !found_kind.is_optional() {
                return crate::e(format!("'{}' is not to an optional", first));
            }
            crate::PropertyValue::Reference {
                name: doc.resolve_name(first)?,
                kind: found_kind,
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
        ) && !matches!(
            self,
            Self::Equal {
                left: ftd::PropertyValue::LocalVariable { .. },
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

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Event {
    // $event-click$: toggle foo
    // will be parsed into this Event struct
    pub name: EventName, // click
    pub action: Action,
}

impl Event {
    pub fn get_events(
        events: &[Self],
        all_locals: &ftd_rt::Map,
    ) -> crate::p1::Result<Vec<ftd_rt::Event>> {
        let mut event: Vec<ftd_rt::Event> = vec![];
        for e in events {
            let target = match e.action.target.strip_prefix('@') {
                Some(value) => {
                    if let Some(val) = all_locals.get(value) {
                        format!("@{}@{}", value, val)
                    } else {
                        return crate::e(format!("Can't find the local variable {}", value));
                    }
                }
                None => e.action.target.to_string(),
            };
            event.push(ftd_rt::Event {
                name: e.name.to_str().to_string(),
                action: ftd_rt::Action {
                    action: e.action.action.to_str().to_string(),
                    target,
                },
            });
        }
        Ok(event)
    }
}

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub enum EventName {
    OnClick,
}

impl EventName {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::OnClick => "onclick",
        }
    }

    pub fn from_str(s: &str) -> ftd::p1::Result<Self> {
        match s {
            "click" => Ok(Self::OnClick),
            t => return crate::e(format!("{} is not a valid event", t)),
        }
    }
}

impl Event {
    pub fn to_event(
        event_name: &str,
        action: &str,
        doc: &crate::p2::TDoc,
        locals: &std::collections::BTreeMap<String, crate::p2::Kind>,
    ) -> ftd::p1::Result<Self> {
        let event_name = EventName::from_str(event_name)?;
        let action = Action::to_action(action, doc, locals)?;
        Ok(Self {
            name: event_name,
            action,
        })
    }
}

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Action {
    pub action: ActionKind, // toggle
    pub target: String,     // foo
}

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub enum ActionKind {
    Toggle,
}

impl ActionKind {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Toggle => "toggle",
        }
    }

    pub fn from_str(s: &str) -> ftd::p1::Result<Self> {
        match s {
            "toggle" => Ok(Self::Toggle),
            t => return crate::e(format!("{} is not a valid action kind", t)),
        }
    }
}

impl Action {
    fn to_action(
        a: &str,
        doc: &crate::p2::TDoc,
        locals: &std::collections::BTreeMap<String, crate::p2::Kind>,
    ) -> ftd::p1::Result<Self> {
        match a {
            _ if a.starts_with("toggle") => {
                let value = a.replace("toggle ", "");
                let target = match value.strip_prefix('@') {
                    Some(val) => match locals.get(val) {
                        Some(crate::p2::Kind::Boolean { .. }) => format!("@{}", val),
                        _ => {
                            return crate::e(format!(
                                "{} should be a local variable and of boolean type",
                                val
                            ))
                        }
                    },
                    None => match doc.get_value(&value)? {
                        Value::Boolean { .. } => doc.resolve_name(&value)?,
                        _ => return crate::e(format!("{} should be of boolean type", value)),
                    },
                };
                Ok(Self {
                    action: ActionKind::Toggle,
                    target,
                })
            }
            t => return crate::e(format!("{} is not a valid action", t)),
        }
    }
}
