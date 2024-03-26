use itertools::Itertools;

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Event {
    // $on-click$: toggle foo
    // will be parsed into this Event struct
    pub name: EventName, // click
    pub action: Action,
}

impl Event {
    fn to_value(
        line_number: usize,
        property: &ftd::Map<Vec<ftd::PropertyValue>>,
        doc: &ftd::ftd2021::p2::TDoc,
    ) -> ftd::ftd2021::p1::Result<ftd::Map<Vec<ftd::ftd2021::event::ParameterData>>> {
        let mut property_string: ftd::Map<Vec<ftd::ftd2021::event::ParameterData>> =
            Default::default();
        for (s, property_values) in property {
            let mut property_values_string = vec![];
            for property_value in property_values {
                let value = property_value.resolve(line_number, doc)?;
                let reference = get_reference(property_value, doc, line_number)?;
                if let Some(value) = value.to_serde_value() {
                    property_values_string
                        .push(ftd::ftd2021::event::ParameterData { value, reference });
                } else {
                    return ftd::ftd2021::p2::utils::e2(
                        format!("Can't convert value to string {:?}", value),
                        doc.name,
                        line_number,
                    );
                }
            }
            if !property_values_string.is_empty() {
                property_string.insert(s.to_string(), property_values_string);
            }
        }
        return Ok(property_string);

        fn get_reference(
            property_value: &ftd::PropertyValue,
            doc: &ftd::ftd2021::p2::TDoc,
            line_number: usize,
        ) -> ftd::ftd2021::p1::Result<Option<String>> {
            Ok(match property_value {
                ftd::PropertyValue::Reference { name, .. }
                | ftd::PropertyValue::Variable { name, .. } => {
                    match doc.get_value(line_number, name)? {
                        ftd::Value::Object { values } => {
                            let mut val: ftd::Map<String> = Default::default();
                            for (k, v) in values.iter() {
                                if let Some(reference) = get_reference(v, doc, line_number)? {
                                    val.insert(k.to_string(), reference);
                                }
                            }
                            serde_json::to_string(&val).ok()
                        }
                        _ => Some(name.to_owned()),
                    }
                }
                _ => None,
            })
        }
    }

    pub fn get_events(
        line_number: usize,
        events: &[Self],
        doc: &ftd::ftd2021::p2::TDoc,
    ) -> ftd::ftd2021::p1::Result<Vec<ftd::Event>> {
        let mut event: Vec<ftd::Event> = vec![];
        for e in events {
            let target = match &e.action.target {
                ftd::PropertyValue::Value { value } => value.to_string().unwrap_or_default(),
                ftd::PropertyValue::Reference { name, .. }
                | ftd::PropertyValue::Variable { name, .. } => name.to_string(),
            };

            event.push(ftd::Event {
                name: e.name.to_string(),
                action: ftd::Action {
                    action: e.action.action.to_str().to_string(),
                    target,
                    parameters: ftd::ftd2021::p2::Event::to_value(
                        line_number,
                        &e.action.parameters,
                        doc,
                    )?,
                },
            });
        }
        Ok(event)
    }

    pub fn mouse_event(val: &str) -> Vec<ftd::Event> {
        vec![
            ftd::Event {
                name: "onmouseenter".to_string(),
                action: ftd::Action {
                    action: "set-value".to_string(),
                    target: val.to_string(),
                    parameters: std::iter::IntoIterator::into_iter([(
                        "value".to_string(),
                        vec![
                            ftd::ftd2021::event::ParameterData {
                                value: serde_json::Value::Bool(true),
                                reference: None,
                            },
                            ftd::ftd2021::event::ParameterData {
                                value: serde_json::json!("boolean"),
                                reference: None,
                            },
                        ],
                    )])
                    .collect(),
                },
            },
            ftd::Event {
                name: "onmouseleave".to_string(),
                action: ftd::Action {
                    action: "set-value".to_string(),
                    target: val.to_string(),
                    parameters: std::iter::IntoIterator::into_iter([(
                        "value".to_string(),
                        vec![
                            ftd::ftd2021::event::ParameterData {
                                value: serde_json::Value::Bool(false),
                                reference: None,
                            },
                            ftd::ftd2021::event::ParameterData {
                                value: serde_json::json!("boolean"),
                                reference: None,
                            },
                        ],
                    )])
                    .collect(),
                },
            },
        ]
    }
}

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub enum EventName {
    OnClick,
    OnChange,
    OnInput,
    OnMouseEnter,
    OnMouseLeave,
    OnClickOutside,
    OnFocus,
    OnBlur,
    OnGlobalKey(Vec<String>),
    OnGlobalKeySeq(Vec<String>),
}

impl std::fmt::Display for EventName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::OnClick => "onclick".to_string(),
            Self::OnChange => "onchange".to_string(),
            Self::OnInput => "oninput".to_string(),
            Self::OnMouseEnter => "onmouseenter".to_string(),
            Self::OnMouseLeave => "onmouseleave".to_string(),
            Self::OnClickOutside => "onclickoutside".to_string(),
            Self::OnFocus => "onfocus".to_string(),
            Self::OnBlur => "onblur".to_string(),
            Self::OnGlobalKey(keys) => format!("onglobalkey[{}]", keys.join("-")),
            Self::OnGlobalKeySeq(keys) => format!("onglobalkeyseq[{}]", keys.join("-")),
        };
        write!(f, "{}", value)
    }
}

impl EventName {
    pub fn from_string(s: &str, doc_id: &str) -> ftd::ftd2021::p1::Result<Self> {
        match s {
            "click" => Ok(Self::OnClick),
            "change" => Ok(Self::OnChange),
            "input" => Ok(Self::OnInput),
            "mouse-enter" => Ok(Self::OnMouseEnter),
            "mouse-leave" => Ok(Self::OnMouseLeave),
            "click-outside" => Ok(Self::OnClickOutside),
            "focus" => Ok(Self::OnFocus),
            "blur" => Ok(Self::OnBlur),
            t if t.starts_with("global-key[") && t.ends_with(']') => {
                let keys = t
                    .trim_start_matches("global-key[")
                    .trim_end_matches(']')
                    .split('-')
                    .map(|v| v.to_string())
                    .collect_vec();
                Ok(Self::OnGlobalKey(keys))
            }
            t if t.starts_with("global-key-seq[") && t.ends_with(']') => {
                let keys = t
                    .trim_start_matches("global-key-seq[")
                    .trim_end_matches(']')
                    .split('-')
                    .map(|v| v.to_string())
                    .collect_vec();
                Ok(Self::OnGlobalKeySeq(keys))
            }
            t => ftd::ftd2021::p2::utils::e2(format!("{} is not a valid event", t), doc_id, 0),
        }
    }
}

impl Event {
    pub fn to_event(
        line_number: usize,
        event_name: &str,
        action: &str,
        doc: &ftd::ftd2021::p2::TDoc,
        arguments: &ftd::Map<ftd::ftd2021::p2::Kind>,
    ) -> ftd::ftd2021::p1::Result<Self> {
        let event_name = EventName::from_string(event_name, doc.name)?;
        let action = Action::to_action(line_number, action, doc, arguments)?;
        Ok(Self {
            name: event_name,
            action,
        })
    }
}

pub struct Parameter {
    pub min: usize,
    pub max: usize,
    pub ptype: Vec<ftd::ftd2021::p2::Kind>,
}

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Action {
    pub action: ActionKind,         // toggle
    pub target: ftd::PropertyValue, // foo
    pub parameters: ftd::Map<Vec<ftd::PropertyValue>>,
}

#[derive(Debug, PartialEq, Clone, serde::Deserialize)]
pub enum ActionKind {
    Toggle,
    Insert,
    Clear,
    Increment,
    Decrement,
    StopPropagation,
    PreventDefault,
    SetValue,
    MessageHost,
}

impl serde::Serialize for ActionKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_str())
    }
}

impl ActionKind {
    pub fn to_str(&self) -> &'static str {
        match self {
            ftd::ftd2021::p2::ActionKind::Toggle => "toggle",
            ftd::ftd2021::p2::ActionKind::Increment => "increment",
            ftd::ftd2021::p2::ActionKind::Decrement => "decrement",
            ftd::ftd2021::p2::ActionKind::Insert => "insert",
            ftd::ftd2021::p2::ActionKind::StopPropagation => "stop-propagation",
            ftd::ftd2021::p2::ActionKind::PreventDefault => "prevent-default",
            ftd::ftd2021::p2::ActionKind::SetValue => "set-value",
            ftd::ftd2021::p2::ActionKind::MessageHost => "message-host",
            ftd::ftd2021::p2::ActionKind::Clear => "clear",
        }
    }

    // pub fn from_string(s: &str, doc_id: &str) -> ftd_p1::Result<Self> {
    //     match s {
    //         "toggle" => Ok(Self::Toggle),
    //         "increment" => Ok(Self::Increment),
    //         "decrement" => Ok(Self::Decrement),
    //         "stop-propagation" => Ok(Self::StopPropagation),
    //         "prevent-default" => Ok(Self::PreventDefault),
    //         "set-value" => Ok(Self::SetValue),
    //         t => return ftd::p2::utils::e2(format!("{} is not a valid action kind", t), doc_id),
    //     }
    // }

    pub fn parameters(&self) -> ftd::Map<ftd::ftd2021::p2::event::Parameter> {
        let mut parameters: ftd::Map<ftd::ftd2021::p2::event::Parameter> = Default::default();
        match self {
            ftd::ftd2021::p2::ActionKind::Toggle
            | ftd::ftd2021::p2::ActionKind::StopPropagation
            | ftd::ftd2021::p2::ActionKind::PreventDefault
            | ftd::ftd2021::p2::ActionKind::Clear
            | ftd::ftd2021::p2::ActionKind::SetValue => {}
            ftd::ftd2021::p2::ActionKind::MessageHost => {
                parameters.insert(
                    "data".to_string(),
                    ftd::ftd2021::p2::event::Parameter {
                        min: 1,
                        max: 1,
                        ptype: vec![ftd::ftd2021::p2::Kind::object()],
                    },
                );
            }
            ftd::ftd2021::p2::ActionKind::Increment | ftd::ftd2021::p2::ActionKind::Decrement => {
                parameters.insert(
                    "by".to_string(),
                    ftd::ftd2021::p2::event::Parameter {
                        min: 1,
                        max: 1,
                        ptype: vec![ftd::ftd2021::p2::Kind::integer()],
                    },
                );
                parameters.insert(
                    "clamp".to_string(),
                    ftd::ftd2021::p2::event::Parameter {
                        min: 1,
                        max: 2,
                        ptype: vec![
                            ftd::ftd2021::p2::Kind::integer(),
                            ftd::ftd2021::p2::Kind::integer(),
                        ],
                    },
                );
            }
            ftd::ftd2021::p2::ActionKind::Insert => {
                parameters.insert(
                    "value".to_string(),
                    ftd::ftd2021::p2::event::Parameter {
                        min: 1,
                        max: 1,
                        ptype: vec![],
                    },
                );
                parameters.insert(
                    "at".to_string(),
                    ftd::ftd2021::p2::event::Parameter {
                        min: 1,
                        max: 1,
                        ptype: vec![ftd::ftd2021::p2::Kind::string()],
                    },
                );
            }
        }
        parameters
    }
}

impl Action {
    fn to_action(
        line_number: usize,
        a: &str,
        doc: &ftd::ftd2021::p2::TDoc,
        arguments: &ftd::Map<ftd::ftd2021::p2::Kind>,
    ) -> ftd::ftd2021::p1::Result<Self> {
        let a: String = a.split_whitespace().collect::<Vec<&str>>().join(" ");
        return match a {
            _ if a.starts_with("toggle ") => {
                let value = a.replace("toggle ", "");
                let target = get_target(
                    line_number,
                    value,
                    doc,
                    arguments,
                    Some(ftd::ftd2021::p2::Kind::boolean()),
                )?;
                Ok(Self {
                    action: ActionKind::Toggle,
                    target,
                    parameters: Default::default(),
                })
            }
            _ if a.starts_with("clear ") => {
                let value = a.replace("clear ", "");
                let target = get_target(line_number, value, doc, arguments, None)?;
                let kind = target.kind();
                if !kind.is_list() && !kind.is_optional() {
                    return ftd::ftd2021::p2::utils::e2(
                        format!(
                            "clear should have target of kind: `list` or `optional`, found: {:?}",
                            kind
                        ),
                        doc.name,
                        line_number,
                    );
                }
                Ok(Self {
                    action: ActionKind::Clear,
                    target,
                    parameters: Default::default(),
                })
            }
            _ if a.starts_with("message-host") => {
                let value = a.replace("message-host", "").trim().to_string();
                let parameters = if value.starts_with('$') {
                    let mut parameters: ftd::Map<Vec<ftd::PropertyValue>> = Default::default();
                    if let Some(p) = ActionKind::MessageHost.parameters().get("data") {
                        parameters.insert(
                            "data".to_string(),
                            vec![ftd::PropertyValue::resolve_value(
                                line_number,
                                value.as_str(),
                                p.ptype.get(0).map(|k| k.to_owned()),
                                doc,
                                arguments,
                                None,
                            )?],
                        );
                    }
                    parameters
                } else {
                    Default::default()
                };

                let target = ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: if value.is_empty() {
                            "ftd_message".to_string()
                        } else {
                            value
                        },
                        source: ftd::TextSource::Header,
                    },
                };

                Ok(Self {
                    action: ActionKind::MessageHost,
                    target,
                    parameters,
                })
            }
            _ if a.starts_with("increment ") || a.starts_with("decrement ") => {
                let (action_kind, action_string) = if a.starts_with("increment ") {
                    (ActionKind::Increment, "increment")
                } else {
                    (ActionKind::Decrement, "decrement")
                };

                let vector: Vec<&str> = a.split(' ').filter(|x| !x.is_empty()).collect();
                let value = if let Some(val) = vector.get(1) {
                    val.to_string()
                } else {
                    return ftd::ftd2021::p2::utils::e2(
                        format!(
                            "target not found, expected `{} something` found: {}",
                            action_string, a
                        ),
                        doc.name,
                        line_number,
                    );
                };
                let target = get_target(
                    line_number,
                    value,
                    doc,
                    arguments,
                    Some(ftd::ftd2021::p2::Kind::integer()),
                )?;

                let parameters = {
                    let mut parameters: ftd::Map<Vec<ftd::PropertyValue>> = Default::default();
                    let mut current_parameter = "".to_string();
                    let (mut min, mut max, mut idx) = (0, 0, 0);
                    let mut pkind = vec![];
                    for parameter in vector[2..].iter() {
                        if let Some(p) = action_kind.parameters().get(*parameter) {
                            if min > idx {
                                return ftd::ftd2021::p2::utils::e2(
                                    format!(
                                        "minumum number of arguments for {} are {}, found: {}",
                                        current_parameter, min, idx
                                    ),
                                    doc.name,
                                    line_number,
                                );
                            }
                            current_parameter = parameter.to_string();
                            min = p.min;
                            max = p.max;
                            pkind = p.ptype.to_vec();
                            idx = 0;
                            parameters.insert(current_parameter.to_string(), vec![]);
                        } else if let Some(p) = parameters.get_mut(&current_parameter) {
                            if idx >= max {
                                return ftd::ftd2021::p2::utils::e2(
                                    format!(
                                        "maximum number of arguments for {} are {}, found: {}",
                                        current_parameter,
                                        max,
                                        max + 1
                                    ),
                                    doc.name,
                                    line_number,
                                );
                            }
                            p.push(ftd::PropertyValue::resolve_value(
                                line_number,
                                parameter,
                                pkind.get(idx).map(|k| k.to_owned()),
                                doc,
                                arguments,
                                None,
                            )?);
                            idx += 1;
                        }
                    }
                    parameters
                };

                Ok(Self {
                    action: action_kind,
                    target,
                    parameters,
                })
            }
            _ if a.starts_with("insert into ") => {
                let vector: Vec<&str> = a.split(' ').filter(|x| !x.is_empty()).collect();
                let value = if let Some(val) = vector.get(2) {
                    val.to_string()
                } else {
                    return ftd::ftd2021::p2::utils::e2(
                        format!(
                            "target not found, expected `insert into <something>` found: {}",
                            a
                        ),
                        doc.name,
                        line_number,
                    );
                };
                let target = get_target(line_number, value.clone(), doc, arguments, None)?;
                let kind = target.kind();
                let expected_value_kind = if let ftd::ftd2021::p2::Kind::List { kind, .. } = kind {
                    kind.as_ref().to_owned()
                } else {
                    return ftd::ftd2021::p2::utils::e2(
                        format!(
                            "expected target `{}` kind is list found: `{:?}`",
                            value, kind
                        ),
                        doc.name,
                        line_number,
                    );
                };
                let parameters = {
                    let mut parameters: ftd::Map<Vec<ftd::PropertyValue>> = Default::default();
                    let mut current_parameter = "".to_string();
                    let (mut min, mut max, mut idx) = (0, 0, 0);
                    let mut pkind = vec![];
                    for parameter in vector[3..].iter() {
                        if let Some(p) = ActionKind::Insert.parameters().get(*parameter) {
                            if min > idx {
                                return ftd::ftd2021::p2::utils::e2(
                                    format!(
                                        "minumum number of arguments for {} are {}, found: {}",
                                        current_parameter, min, idx
                                    ),
                                    doc.name,
                                    line_number,
                                );
                            }
                            current_parameter = parameter.to_string();
                            min = p.min;
                            max = p.max;
                            pkind = p.ptype.to_vec();
                            idx = 0;
                            parameters.insert(current_parameter.to_string(), vec![]);
                        } else if let Some(p) = parameters.get_mut(&current_parameter) {
                            if idx >= max {
                                return ftd::ftd2021::p2::utils::e2(
                                    format!(
                                        "maximum number of arguments for {} are {}, found: {}",
                                        current_parameter,
                                        max,
                                        max + 1
                                    ),
                                    doc.name,
                                    line_number,
                                );
                            }
                            let value = if parameter.eq(&"$VALUE") {
                                ftd::PropertyValue::Value {
                                    value: ftd::ftd2021::variable::Value::String {
                                        text: parameter.to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                }
                            } else {
                                ftd::PropertyValue::resolve_value(
                                    line_number,
                                    parameter,
                                    pkind.get(idx).map(|k| k.to_owned()),
                                    doc,
                                    arguments,
                                    None,
                                )?
                            };
                            if !value.kind().inner().eq(&expected_value_kind) {
                                return ftd::ftd2021::p2::utils::e2(
                                    format!(
                                        "expected value kind: `{:?}` found: `{:?}`",
                                        value.kind(),
                                        expected_value_kind
                                    ),
                                    doc.name,
                                    line_number,
                                );
                            }
                            p.push(value);
                            idx += 1;
                        }
                    }
                    parameters
                };

                Ok(Self {
                    action: ActionKind::Insert,
                    target,
                    parameters,
                })
            }
            _ if a.eq("stop-propagation") => Ok(Self {
                action: ActionKind::StopPropagation,
                target: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
                parameters: Default::default(),
            }),
            _ if a.eq("prevent-default") => Ok(Self {
                action: ActionKind::PreventDefault,
                target: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
                parameters: Default::default(),
            }),
            _ if a.contains('=') => {
                let (part_1, part_2) = ftd::ftd2021::p2::utils::split(a, "=")?;
                let target = get_target(line_number, part_1, doc, arguments, None)?;
                let kind = target.kind();
                let mut parameters: ftd::Map<Vec<ftd::PropertyValue>> = Default::default();

                let value = {
                    if part_2.eq("$VALUE") || part_2.eq("$MOUSE-IN") {
                        ftd::PropertyValue::Value {
                            value: ftd::ftd2021::variable::Value::String {
                                text: part_2,
                                source: ftd::TextSource::Header,
                            },
                        }
                    } else {
                        ftd::PropertyValue::resolve_value(
                            line_number,
                            &part_2,
                            Some(kind.clone()),
                            doc,
                            arguments,
                            None,
                        )?
                    }
                };
                let kind = ftd::PropertyValue::Value {
                    value: ftd::ftd2021::variable::Value::String {
                        text: kind.to_string(line_number, doc.name)?,
                        source: ftd::TextSource::Header,
                    },
                };

                parameters.insert("value".to_string(), vec![value, kind]);
                Ok(Self {
                    action: ActionKind::SetValue,
                    target,
                    parameters,
                })
            }
            t => {
                return ftd::ftd2021::p2::utils::e2(
                    format!("{} is not a valid action", t),
                    doc.name,
                    line_number,
                )
            }
        };

        fn get_target(
            line_number: usize,
            value: String,
            doc: &ftd::ftd2021::p2::TDoc,
            arguments: &ftd::Map<ftd::ftd2021::p2::Kind>,
            kind: Option<ftd::ftd2021::p2::Kind>,
        ) -> ftd::ftd2021::p1::Result<ftd::PropertyValue> {
            ftd::PropertyValue::resolve_value(line_number, &value, kind, doc, arguments, None)
        }
    }
}
