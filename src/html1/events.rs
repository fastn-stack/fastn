#[derive(serde::Deserialize, Clone, Debug, serde::Serialize, PartialEq, Default)]
pub struct Action {
    pub name: String,
    pub values: ftd::Map<serde_json::Value>,
}

impl ftd::html1::Action {
    pub fn new(name: &str, values: ftd::Map<serde_json::Value>) -> ftd::html1::Action {
        ftd::html1::Action {
            name: name.to_string(),
            values,
        }
    }

    pub(crate) fn from_function_call(
        function_call: &ftd::interpreter2::FunctionCall,
        id: &str,
    ) -> ftd::html1::Action {
        let values = ftd::html1::Action::from_values(function_call);
        let function_name = ftd::html1::utils::name_with_id(function_call.name.as_str(), id);
        ftd::html1::Action::new(
            ftd::html1::utils::function_name_to_js_function(function_name.as_str()).as_str(),
            values,
        )
    }

    fn from_values(function_call: &ftd::interpreter2::FunctionCall) -> ftd::Map<serde_json::Value> {
        function_call
            .values
            .iter()
            .map(|(k, v)| (k.to_string(), ftd::html1::Action::from_property_value(v)))
            .collect()
    }

    fn from_property_value(value: &ftd::interpreter2::PropertyValue) -> serde_json::Value {
        match value {
            ftd::interpreter2::PropertyValue::Value { value, .. } => {
                ftd::html1::Action::from_value(value)
            }
            ftd::interpreter2::PropertyValue::Reference {
                name, is_mutable, ..
            } => {
                serde_json::json!({
                    "reference": name,
                    "mutable": is_mutable
                })
            }
            ftd::interpreter2::PropertyValue::Clone {
                name, is_mutable, ..
            } => {
                serde_json::json!({
                    "clone": name,
                    "mutable": is_mutable
                })
            }
            ftd::interpreter2::PropertyValue::FunctionCall(fnc) => unimplemented!("{:?}", fnc),
        }
    }

    fn from_value(value: &ftd::interpreter2::Value) -> serde_json::Value {
        match value {
            ftd::interpreter2::Value::String { text } => serde_json::json!(text),
            ftd::interpreter2::Value::Integer { value } => serde_json::json!(value),
            ftd::interpreter2::Value::Decimal { value } => serde_json::json!(value),
            ftd::interpreter2::Value::Boolean { value } => serde_json::json!(value),
            t => {
                unimplemented!("{:?}", t)
            }
        }
    }

    fn into_list(self) -> Vec<ftd::html1::Action> {
        vec![self]
    }
}

impl ftd::html1::main::HtmlGenerator {
    pub(crate) fn group_by_js_event(
        &self,
        evts: &[ftd::node::Event],
    ) -> std::collections::HashMap<String, String> {
        // key: onclick
        // value: after group by for onclick find all actions, and call to_js_event()
        let mut events: ftd::Map<Vec<ftd::html1::Action>> = Default::default();
        for event in evts {
            if let Some(actions) = events.get_mut(to_event_name(&event.name)) {
                actions.push(ftd::html1::Action::from_function_call(
                    &event.action,
                    self.id.as_str(),
                ));
            } else {
                events.insert(
                    to_event_name(&event.name).to_string(),
                    ftd::html1::Action::from_function_call(&event.action, self.id.as_str())
                        .into_list(),
                );
            }
        }
        let mut string_events: std::collections::HashMap<String, String> = Default::default();
        for (k, v) in events {
            string_events.insert(k, serde_json::to_string(&v).expect(""));
        }
        string_events
    }
}

fn to_event_name(event_name: &ftd::interpreter2::EventName) -> &'static str {
    match event_name {
        ftd::interpreter2::EventName::Click => "onclick",
    }
}
