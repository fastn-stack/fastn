#[derive(serde::Deserialize, Clone)]
#[cfg_attr(
    not(feature = "wasm"),
    derive(serde::Serialize, PartialEq, Debug, Default)
)]
pub struct Event {
    // $event-click$: toggle foo
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

#[derive(serde::Deserialize, Clone, Debug, serde::Serialize)]
#[cfg_attr(not(feature = "wasm"), derive(PartialEq, Default))]
pub struct Action {
    pub action: String, // toggle
    pub target: String, // foo
    pub parameters: std::collections::BTreeMap<String, Vec<String>>,
}

#[cfg(feature = "wasm")]
impl Action {
    fn to_action(a: &str) -> crate::Result<Self> {
        match a {
            _ if a.starts_with("toggle") => {
                let target = a.replace("toggle ", "");
                Ok(Self {
                    action: "toggle".to_string(),
                    target,
                    parameters: Default::default(),
                })
            }
            t => return crate::e(format!("{} is not a valid action", t)),
        }
    }

    fn from_action(&self) -> String {
        // input: { action: toggle, target: x }
        // output: "toggle x;"
        format!("{} {};", self.action, self.target)
    }

    pub(crate) fn parse_js_event(s: &str) -> Vec<Action> {
        // input: "toggle x; set-true y"
        // output: { action: toggle, target: x }, { action: set-true, target: y }
        let actions_string: Vec<_> = s.split(";").collect();
        let mut actions = vec![];
        for action in actions_string {
            let a = action.trim();
            if !a.is_empty() {
                actions.push(Action::to_action(action).expect("Can't convert to action"));
            }
        }
        actions
    }

    pub(crate) fn handle_action(&self, doc: &mut ftd_rt::Document) {
        match self.action.as_str() {
            "toggle" => {
                let data = doc
                    .data
                    .get(&self.target)
                    .expect(format!("{} should be present", self.target).as_str());
                let value = !data
                    .value
                    .parse::<bool>()
                    .expect(format!("Can't parse value for {} into bool", self.target).as_str());
                let dependencies = data.dependencies.to_owned();

                doc.data.insert(
                    self.target.to_string(),
                    ftd_rt::Data {
                        value: value.to_string(),
                        dependencies,
                    },
                );
            }
            _ => unimplemented!(),
        }
    }
}
