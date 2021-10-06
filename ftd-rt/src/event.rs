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
    let mut events: std::collections::HashMap<String, String> = Default::default();
    for event in evts {
        if let Some(actions) = events.get_mut(&event.name) {
            actions.push(' ');
            actions.push_str(&event.action.from_action());
        } else {
            events.insert(event.name.to_string(), event.action.from_action());
        }
    }
    events
}

#[derive(serde::Deserialize, Clone, Debug)]
#[cfg_attr(not(feature = "wasm"), derive(serde::Serialize, PartialEq, Default))]
pub struct Action {
    pub action: String, // toggle
    pub target: String, // foo
}

impl Action {
    #[cfg(feature = "wasm")]
    fn to_action(a: &str) -> crate::Result<Self> {
        match a {
            _ if a.starts_with("toggle") => {
                let target = a.replace("toggle ", "");
                Ok(Self {
                    action: "toggle".to_string(),
                    target,
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

    #[cfg(feature = "wasm")]
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

    #[cfg(feature = "wasm")]
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
