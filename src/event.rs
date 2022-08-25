#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct Event {
    // $on-click$: toggle foo
    // will be parsed into this Event struct
    pub name: String, // click
    pub action: Action,
}

pub(crate) fn group_by_js_event(evts: &[Event]) -> std::collections::HashMap<String, String> {
    // key: onclick
    // value: after group by for onclick find all actions, and call to_js_event()
    let mut events: std::collections::HashMap<String, Vec<Action>> = Default::default();
    for event in evts {
        if let Some(actions) = events.get_mut(&event.name) {
            actions.push(event.action.to_owned());
        } else {
            events.insert(event.name.to_string(), vec![event.action.to_owned()]);
        }
    }
    let mut string_events: std::collections::HashMap<String, String> = Default::default();
    for (k, v) in events {
        string_events.insert(k, serde_json::to_string(&v).expect(""));
    }
    string_events
}

#[derive(serde::Deserialize, Clone, Debug, serde::Serialize, PartialEq, Default)]
pub struct Action {
    pub action: String, // toggle
    pub target: String, // foo
    pub parameters: ftd::Map<Vec<ParameterData>>,
}

#[derive(serde::Deserialize, Clone, Debug, serde::Serialize, PartialEq, Default)]
pub struct ParameterData {
    pub value: serde_json::Value,
    pub reference: Option<String>,
}
