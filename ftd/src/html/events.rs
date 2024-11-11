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
        function_call: &fastn_type::FunctionCall,
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
        function_call: &fastn_type::FunctionCall,
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
        value: &fastn_type::PropertyValue,
        doc: &ftd::interpreter::TDoc,
    ) -> ftd::html::Result<serde_json::Value> {
        use ftd::interpreter::PropertyValueExt;

        Ok(match value {
            fastn_type::PropertyValue::Value { value, .. } => ftd::html::Action::from_value(value),
            fastn_type::PropertyValue::Reference {
                name, is_mutable, ..
            } => {
                serde_json::json!({
                    "reference": name,
                    "mutable": is_mutable
                })
            }
            t @ fastn_type::PropertyValue::Clone { line_number, .. } => {
                let value = t.clone().resolve(doc, *line_number)?;
                ftd::html::Action::from_value(&value)
            }
            fastn_type::PropertyValue::FunctionCall(fnc) => unimplemented!("{:?}", fnc),
        })
    }

    fn from_value(value: &fastn_type::Value) -> serde_json::Value {
        match value {
            fastn_type::Value::String { text } => serde_json::json!(text),
            fastn_type::Value::Integer { value } => serde_json::json!(value),
            fastn_type::Value::Decimal { value } => serde_json::json!(value),
            fastn_type::Value::Boolean { value } => serde_json::json!(value),
            fastn_type::Value::Optional { data, .. } => {
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

fn to_event_name(event_name: &ftd::interpreter::EventName) -> String {
    match event_name {
        ftd::interpreter::EventName::Click => "onclick".to_string(),
        ftd::interpreter::EventName::MouseLeave => "onmouseleave".to_string(),
        ftd::interpreter::EventName::MouseEnter => "onmouseenter".to_string(),
        ftd::interpreter::EventName::ClickOutside => "onclickoutside".to_string(),
        ftd::interpreter::EventName::GlobalKey(keys) => format!("onglobalkey[{}]", keys.join("-")),
        ftd::interpreter::EventName::GlobalKeySeq(keys) => {
            format!("onglobalkeyseq[{}]", keys.join("-"))
        }
        ftd::interpreter::EventName::Input => "oninput".to_string(),
        ftd::interpreter::EventName::Change => "onchange".to_string(),
        ftd::interpreter::EventName::Blur => "onblur".to_string(),
        ftd::interpreter::EventName::Focus => "onfocus".to_string(),
        ftd::interpreter::EventName::RivePlay(timeline) => format!("onriveplay[{}]", timeline),
        ftd::interpreter::EventName::RiveStateChange(state_change) => {
            format!("onrivestatechange[{}]", state_change)
        }
        ftd::interpreter::EventName::RivePause(timeline) => {
            format!("onrivepause[{}]", timeline)
        }
    }
}
