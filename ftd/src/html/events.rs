#[derive(serde::Deserialize, Clone, Debug, serde::Serialize, PartialEq, Default)]
pub struct Action {
    pub name: String,
    pub values: Vec<(String, serde_json::Value)>,
}

impl ftd::html::Action {
    pub fn new(name: &str, values: Vec<(String, serde_json::Value)>) -> ftd::html::Action {
        ftd::html::Action {
            name: name.to_string(),
            values,
        }
    }

    pub(crate) fn from_function_call(
        function_call: &fastn_resolved::FunctionCall,
        id: &str,
        doc: &ftd::interpreter::TDoc,
    ) -> ftd::html::Result<ftd::html::Action> {
        let values = ftd::html::Action::from_values(function_call, doc)?;

        let function_name = ftd::html::utils::name_with_id(function_call.name.as_str(), id);
        Ok(ftd::html::Action::new(
            ftd::html::utils::function_name_to_js_function(function_name.as_str()).as_str(),
            values,
        ))
    }

    fn from_values(
        function_call: &fastn_resolved::FunctionCall,
        doc: &ftd::interpreter::TDoc,
    ) -> ftd::html::Result<Vec<(String, serde_json::Value)>> {
        function_call
            .order
            .iter()
            .filter_map(|k| {
                function_call.values.get(k).map(|v| {
                    ftd::html::Action::from_property_value(v, doc).map(|v| (k.to_string(), v))
                })
            })
            .collect()
    }

    fn from_property_value(
        value: &fastn_resolved::PropertyValue,
        doc: &ftd::interpreter::TDoc,
    ) -> ftd::html::Result<serde_json::Value> {
        use ftd::interpreter::PropertyValueExt;

        Ok(match value {
            fastn_resolved::PropertyValue::Value { value, .. } => {
                ftd::html::Action::from_value(value)
            }
            fastn_resolved::PropertyValue::Reference {
                name, is_mutable, ..
            } => {
                serde_json::json!({
                    "reference": name,
                    "mutable": is_mutable
                })
            }
            t @ fastn_resolved::PropertyValue::Clone { line_number, .. } => {
                let value = t.clone().resolve(doc, *line_number)?;
                ftd::html::Action::from_value(&value)
            }
            fastn_resolved::PropertyValue::FunctionCall(fnc) => unimplemented!("{:?}", fnc),
        })
    }

    fn from_value(value: &fastn_resolved::Value) -> serde_json::Value {
        match value {
            fastn_resolved::Value::String { text } => serde_json::json!(text),
            fastn_resolved::Value::Integer { value } => serde_json::json!(value),
            fastn_resolved::Value::Decimal { value } => serde_json::json!(value),
            fastn_resolved::Value::Boolean { value } => serde_json::json!(value),
            fastn_resolved::Value::Optional { data, .. } => {
                if let Some(data) = data.as_ref() {
                    ftd::html::Action::from_value(data)
                } else {
                    serde_json::Value::Null
                }
            }
            t => {
                unimplemented!("{:?}", t)
            }
        }
    }

    pub(crate) fn into_list(self) -> Vec<ftd::html::Action> {
        vec![self]
    }
}

impl ftd::html::main::HtmlGenerator<'_> {
    pub(crate) fn group_by_js_event(
        &self,
        evts: &[ftd::node::Event],
    ) -> ftd::html::Result<ftd::Map<String>> {
        pub fn clean_string(s: String) -> String {
            s.replace("\\\\", "/").replace('\\', "/")
        }

        // key: onclick
        // value: after group by for onclick find all actions, and call to_js_event()
        let mut events: ftd::Map<Vec<ftd::html::Action>> = Default::default();
        for event in evts {
            if let Some(actions) = events.get_mut(to_event_name(&event.name).as_str()) {
                actions.push(ftd::html::Action::from_function_call(
                    &event.action,
                    self.id.as_str(),
                    self.doc,
                )?);
            } else {
                events.insert(
                    to_event_name(&event.name),
                    ftd::html::Action::from_function_call(
                        &event.action,
                        self.id.as_str(),
                        self.doc,
                    )?
                    .into_list(),
                );
            }
        }
        let mut string_events: ftd::Map<String> = Default::default();
        for (k, v) in events {
            string_events.insert(k, clean_string(serde_json::to_string(&v).expect("")));
        }
        Ok(string_events)
    }
}

fn to_event_name(event_name: &fastn_resolved::EventName) -> String {
    match event_name {
        fastn_resolved::EventName::Click => "onclick".to_string(),
        fastn_resolved::EventName::MouseLeave => "onmouseleave".to_string(),
        fastn_resolved::EventName::MouseEnter => "onmouseenter".to_string(),
        fastn_resolved::EventName::ClickOutside => "onclickoutside".to_string(),
        fastn_resolved::EventName::GlobalKey(keys) => format!("onglobalkey[{}]", keys.join("-")),
        fastn_resolved::EventName::GlobalKeySeq(keys) => {
            format!("onglobalkeyseq[{}]", keys.join("-"))
        }
        fastn_resolved::EventName::Input => "oninput".to_string(),
        fastn_resolved::EventName::Change => "onchange".to_string(),
        fastn_resolved::EventName::Blur => "onblur".to_string(),
        fastn_resolved::EventName::Focus => "onfocus".to_string(),
        fastn_resolved::EventName::RivePlay(timeline) => format!("onriveplay[{timeline}]"),
        fastn_resolved::EventName::RiveStateChange(state_change) => {
            format!("onrivestatechange[{state_change}]")
        }
        fastn_resolved::EventName::RivePause(timeline) => {
            format!("onrivepause[{timeline}]")
        }
    }
}
