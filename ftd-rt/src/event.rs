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

#[cfg(feature = "wasm")]
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

#[cfg(feature = "wasm")]
impl Action {
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
            "toggle" => match self.target.strip_prefix('@') {
                None => {
                    let value = !doc
                        .data
                        .get(&self.target)
                        .expect(format!("{} should be present", self.target).as_str())
                        .parse::<bool>()
                        .expect(
                            format!("Can't parse value for {} into bool", self.target).as_str(),
                        );

                    doc.data.insert(self.target.to_string(), value.to_string());
                }
                Some(target) => {
                    let mut part = target.splitn(2, '@');
                    let part_1 = part.next().unwrap().trim();
                    let part_2 = part.next().unwrap().trim();
                    let container: Vec<_> = part_2
                        .split(',')
                        .map(|v| v.parse::<usize>().expect(""))
                        .collect();
                    let target_doc = doc.tree.get_target_node(container);
                    let value = !target_doc
                        .locals
                        .get(target)
                        .expect(format!("{} should be present", part_1).as_str())
                        .parse::<bool>()
                        .expect(format!("Can't parse value for {} into bool", part_1).as_str());
                    target_doc
                        .locals
                        .insert(target.to_string(), value.to_string());
                }
            },
            _ => unimplemented!(),
        }
    }
}
