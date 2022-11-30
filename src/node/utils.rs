pub trait CheckMap {
    fn check_and_insert(&mut self, key: &str, value: ftd::node::Value);
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
}
