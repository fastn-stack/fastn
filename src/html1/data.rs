pub struct DataGenerator<'a> {
    pub name: &'a str,
    pub bag: &'a ftd::Map<ftd::interpreter2::Thing>,
}

impl<'a> DataGenerator<'a> {
    pub(crate) fn new(
        name: &'a str,
        bag: &'a ftd::Map<ftd::interpreter2::Thing>,
    ) -> DataGenerator<'a> {
        DataGenerator { name, bag }
    }

    pub(crate) fn get_data(&self) -> ftd::Map<serde_json::Value> {
        let mut d: ftd::Map<serde_json::Value> = Default::default();
        for (k, v) in self.bag.iter() {
            if let ftd::interpreter2::Thing::Variable(ftd::interpreter2::Variable {
                value, ..
            }) = v
            {
                let value = match value {
                    ftd::interpreter2::PropertyValue::Value { value, .. } => value,
                    _ => continue, //todo
                };

                if let Some(value) = get_value(&value) {
                    d.insert(k.to_string(), value);
                }
            }
        }
        return d;

        fn get_value(value: &ftd::interpreter2::Value) -> Option<serde_json::Value> {
            if let ftd::interpreter2::Value::List { data, .. } = value {
                let mut list_data = vec![];
                for val in data {
                    let value = match val {
                        ftd::interpreter2::PropertyValue::Value { value, .. } => value,
                        _ => continue, //todo
                    };
                    if let Some(val) = get_value(&value) {
                        list_data.push(val);
                    }
                }
                return serde_json::to_value(&list_data).ok();
            }
            let value = value.inner();

            match value {
                None => Some(serde_json::Value::Null),
                Some(ftd::interpreter2::Value::Boolean { value }) => {
                    serde_json::to_value(value).ok()
                }
                Some(ftd::interpreter2::Value::Integer { value }) => {
                    serde_json::to_value(value).ok()
                }
                Some(ftd::interpreter2::Value::String { text: value, .. }) => {
                    serde_json::to_value(value).ok()
                }
                Some(ftd::interpreter2::Value::Record { fields, .. }) => {
                    let mut value_fields = ftd::Map::new();
                    for (k, v) in fields {
                        let value = match v {
                            ftd::interpreter2::PropertyValue::Value { value, .. } => value,
                            _ => continue, //todo
                        };
                        value_fields.insert(k, value);
                    }
                    serde_json::to_value(value_fields).ok()
                }
                _ => None,
            }
        }
    }
}
