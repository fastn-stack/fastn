use crate::node::Value;

pub trait CheckMap {
    fn check_and_insert(&mut self, key: &str, value: ftd::node::Value);
    // only `insert if value is null or not present, and update properties always`
    fn insert_if_not_contains(&mut self, key: &str, value: ftd::node::Value);
}

impl CheckMap for ftd::Map<ftd::node::Value> {
    fn check_and_insert(&mut self, key: &str, value: ftd::node::Value) {
        let value = if let Some(old_value) = self.get(key) {
            let mut new_value = old_value.to_owned();
            if let Some(default) = value.value {
                new_value.value = Some(default);
                new_value.line_number = value.line_number.or(old_value.line_number);
            }
            new_value.properties.extend(value.properties);
            new_value
        } else {
            value
        };

        if value.value.is_some() || !value.properties.is_empty() {
            self.insert(key.to_string(), value);
        }
    }

    fn insert_if_not_contains(&mut self, key: &str, value: Value) {
        if let Some(old_value) = self.get_mut(key) {
            // if old value present so extend the props only
            if old_value.value.is_some() {
                old_value.properties.extend(value.properties);
                return;
            }
        }
        self.insert(key.to_string(), value);
    }
}

pub(crate) fn wrap_to_css(wrap: bool) -> String {
    if wrap {
        "wrap".to_string()
    } else {
        "nowrap".to_string()
    }
}

pub(crate) fn escape(s: &str) -> String {
    let s = s.replace('>', "\\u003E");
    let s = s.replace('<', "\\u003C");
    s.replace('&', "\\u0026")
}

pub(crate) fn count_children_with_absolute_parent(children: &[ftd::executor::Element]) -> usize {
    children
        .iter()
        .filter(|v| {
            let mut bool = false;
            if let Some(common) = v.get_common() {
                if Some(ftd::executor::Anchor::Parent) == common.anchor.value {
                    bool = true;
                }
            }
            bool
        })
        .count()
}

pub(crate) fn has_click_event(events: &[ftd::executor::Event]) -> bool {
    events
        .iter()
        .any(|f| f.name.eq(&fastn_resolved::EventName::Click))
}
