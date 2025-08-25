use itertools::Itertools;

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Document {
    pub data: ftd::Map<ftd::ftd2021::p2::Thing>,
    pub name: String,
    pub instructions: Vec<ftd::Instruction>,
    pub main: ftd::Column,
    pub aliases: ftd::Map<String>,
}

impl Document {
    fn get_data(&self) -> (ftd::Map<serde_json::Value>, Vec<String>) {
        let mut d: ftd::Map<serde_json::Value> = Default::default();
        let mut always_include = vec![];
        let doc = ftd::ftd2021::p2::TDoc {
            name: self.name.as_str(),
            aliases: &self.aliases,
            bag: &self.data,
            local_variables: &mut Default::default(),
            referenced_local_variables: &mut Default::default(),
        };
        for (k, v) in self.data.iter() {
            if let ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                value, flags: flag, ..
            }) = v
            {
                let val = if let Ok(val) = value.resolve(0, &doc) {
                    val
                } else {
                    continue;
                };
                if let Some(value) = get_value(&val, &doc) {
                    d.insert(k.to_string(), value);
                }
                if let ftd::ftd2021::variable::VariableFlags {
                    always_include: Some(f),
                } = flag
                    && *f
                {
                    always_include.push(k.to_string());
                }
            }
        }
        return (d, always_include);

        fn get_value(
            value: &ftd::Value,
            doc: &ftd::ftd2021::p2::TDoc,
        ) -> Option<serde_json::Value> {
            if let ftd::Value::List { data, .. } = value {
                let mut list_data = vec![];
                for val in data {
                    let val = if let Ok(val) = val.resolve(0, doc) {
                        val
                    } else {
                        continue;
                    };
                    if let Some(val) = get_value(&val, doc) {
                        list_data.push(val);
                    }
                }
                return serde_json::to_value(&list_data).ok();
            }
            let value = if let ftd::Value::Optional { data, kind } = value {
                match data.as_ref() {
                    None => ftd::Value::None {
                        kind: kind.to_owned(),
                    },
                    Some(v) => v.to_owned(),
                }
            } else {
                value.to_owned()
            };

            match value {
                ftd::Value::None { .. } => Some(serde_json::Value::Null),
                ftd::Value::Boolean { value } => serde_json::to_value(value).ok(),
                ftd::Value::Integer { value } => serde_json::to_value(value).ok(),
                ftd::Value::String { text: value, .. } => serde_json::to_value(value).ok(),
                ftd::Value::Record { fields, name } => {
                    let mut value_fields = ftd::Map::new();
                    if ["ftd#image-src", "ftd#color"].contains(&name.as_str()) {
                        value_fields
                            .insert("$kind$".to_string(), serde_json::to_value("light").unwrap());
                    }
                    if ["ftd#type"].contains(&name.as_str()) {
                        value_fields.insert(
                            "$kind$".to_string(),
                            serde_json::to_value("desktop").unwrap(),
                        );
                    }
                    for (k, v) in fields {
                        if let Ok(val) = v.resolve(0, doc)
                            && let Some(val) = get_value(&val, doc)
                        {
                            value_fields.insert(
                                if k.eq("size") {
                                    "font-size".to_string()
                                } else {
                                    k
                                },
                                val,
                            );
                        }
                    }
                    if let Some(val) = value_fields.get_mut("font-size") {
                        let size = serde_json::to_string(val).unwrap();
                        *val = serde_json::to_value(format!("{size}px")).unwrap();
                    }
                    if let Some(val) = value_fields.get_mut("line-height") {
                        let size = serde_json::to_string(val).unwrap();
                        *val = serde_json::to_value(format!("{size}px")).unwrap();
                    }
                    serde_json::to_value(value_fields).ok()
                }
                _ => None,
            }
        }
    }

    fn rt_data(&self) -> ftd::DataDependenciesMap {
        let (d, always_include) = self.get_data();

        let mut data: ftd::DataDependenciesMap = Default::default();
        for (k, v) in d {
            data.insert(
                k.to_string(),
                ftd::Data {
                    value: v,
                    dependencies: Default::default(),
                },
            );
        }
        ftd::Element::get_variable_dependencies(self, &mut data);
        ftd::Element::get_event_dependencies(&self.main.container.children, &mut data);
        let mut data_dependencies = data
            .into_iter()
            .filter(|(k, v)| (!v.dependencies.is_empty() || always_include.contains(k)))
            .collect::<ftd::DataDependenciesMap>();
        ftd::Element::get_dark_mode_dependencies(self, &mut data_dependencies);
        // ftd::Element::get_device_dependencies(self, &mut data_dependencies);

        data_dependencies
    }

    pub fn rerender(&mut self, id: &str, doc_id: &str) -> ftd::ftd2021::p1::Result<ftd::Document> {
        let mut rt = ftd::ftd2021::RT::from(
            self.name.as_str(),
            self.aliases.clone(),
            self.data.clone(),
            self.instructions.clone(),
        );
        self.main = rt.render()?;
        self.data.extend(rt.bag);
        let data = self.rt_data();
        let mut collector = ftd::Collector::new();
        Ok(ftd::Document {
            html: self.html(id, doc_id, &data, &mut collector),
            data,
            external_children: ftd::Element::get_external_children_dependencies(
                &self.main.container.children,
            ),
            body_events: self.body_events(id),
            css_collector: collector.to_css(),
        })
    }

    pub fn to_rt(&self, id: &str, doc_id: &str) -> ftd::Document {
        let external_children =
            ftd::Element::get_external_children_dependencies(&self.main.container.children);
        let rt_data = self.rt_data();
        let mut collector = ftd::Collector::new();
        ftd::Document {
            html: self.html(id, doc_id, &rt_data, &mut collector),
            data: rt_data,
            external_children,
            body_events: self.body_events(id),
            css_collector: collector.to_css(),
        }
    }

    pub fn body_events(&self, id: &str) -> String {
        let mut events = vec![];
        body_events_(self.main.container.children.as_slice(), &mut events, id);

        return events_to_string(events);

        #[derive(Debug)]
        struct EventData {
            name: String,
            oid: String,
            action: String,
        }

        fn to_key(key: &str) -> String {
            match key {
                "ctrl" => "Control",
                "alt" => "Alt",
                "shift" => "Shift",
                "up" => "ArrowUp",
                "down" => "ArrowDown",
                "right" => "ArrowRight",
                "left" => "ArrowLeft",
                "esc" => "Escape",
                "dash" => "-",
                "space" => " ",
                t => t,
            }
            .to_string()
        }

        fn events_to_string(events: Vec<EventData>) -> String {
            if events.is_empty() {
                return "".to_string();
            }

            let global_variables =
                "let global_keys = {};\nlet buffer = [];\nlet lastKeyTime = Date.now();"
                    .to_string();
            let mut keydown_seq_event = "".to_string();
            let mut keydown_events = indoc::indoc! {"
                document.addEventListener(\"keydown\", function(event) {
                    global_keys[event.key] = true;
                    const currentTime = Date.now();
                    if (currentTime - lastKeyTime > 1000) {{
                        buffer = [];
                    }}
                    lastKeyTime = currentTime;
                    if (event.target.nodeName === \"INPUT\" || event.target.nodeName === \"TEXTAREA\") {
                        return;
                    }          
                    buffer.push(event.key);
            "}.to_string();

            for (keys, actions) in events.iter().filter_map(|e| {
                if let Some(keys) = e.name.strip_prefix("onglobalkeyseq[") {
                    let keys = keys
                        .trim_end_matches(']')
                        .split('-')
                        .map(to_key)
                        .collect_vec();
                    Some((keys, e.action.clone()))
                } else {
                    None
                }
            }) {
                keydown_seq_event = format!(
                    indoc::indoc! {"
                        {string}
                        if (buffer.join(',').includes(\"{sequence}\")) {{
                           {actions}
                            buffer = [];
                            global_keys[event.key] = false;
                            return;
                        }}
                    "},
                    string = keydown_seq_event,
                    sequence = keys.join(","),
                    actions = actions,
                );
            }

            let keyup_events =
                "document.addEventListener(\"keyup\", function(event) { global_keys[event.key] = false; })".to_string();

            for (keys, actions) in events.iter().filter_map(|e| {
                if let Some(keys) = e.name.strip_prefix("onglobalkey[") {
                    let keys = keys
                        .trim_end_matches(']')
                        .split('-')
                        .map(to_key)
                        .collect_vec();
                    Some((keys, e.action.clone()))
                } else {
                    None
                }
            }) {
                let all_keys = keys
                    .iter()
                    .map(|v| format!("global_keys[\"{v}\"]"))
                    .join(" && ");
                keydown_seq_event = format!(
                    indoc::indoc! {"
                        {string}
                        if ({all_keys} && buffer.join(',').includes(\"{sequence}\")) {{
                            {actions}
                            buffer = [];
                            global_keys[event.key] = false;
                            return;
                        }}
                    "},
                    string = keydown_seq_event,
                    all_keys = all_keys,
                    sequence = keys.join(","),
                    actions = actions,
                );
            }

            if !keydown_seq_event.is_empty() {
                keydown_events = format!("{keydown_events}\n\n{keydown_seq_event}}});");
            }

            let mut string = "document.addEventListener(\"click\", function(event) {".to_string();
            for event in events.iter().filter(|e| e.name.eq("onclickoutside")) {
                string = format!(
                    indoc::indoc! {"
                        {string}
                        if (document.querySelector(`[data-id=\"{data_id}\"]`).style.display !== \"none\" && !document.querySelector(`[data-id=\"{data_id}\"]`).contains(event.target)) {{
                            {event}
                        }}
                    "},
                    string = string,
                    data_id = event.oid,
                    event = event.action,
                );
            }
            string = format!("{string}}});");

            if keydown_seq_event.is_empty() {
                string
            } else {
                format!(
                    "{string}\n\n\n{global_variables}\n\n\n{keydown_events}\n\n\n{keyup_events}"
                )
            }
        }

        fn body_events_(children: &[ftd::Element], event_string: &mut Vec<EventData>, id: &str) {
            for child in children {
                let (events, data_id) = match child {
                    ftd::Element::Column(ftd::Column {
                        common, container, ..
                    })
                    | ftd::Element::Row(ftd::Row {
                        common, container, ..
                    })
                    | ftd::Element::Scene(ftd::Scene {
                        common, container, ..
                    })
                    | ftd::Element::Grid(ftd::Grid {
                        common, container, ..
                    }) => {
                        body_events_(&container.children, event_string, id);
                        if let Some((_, _, external_children)) = &container.external_children {
                            body_events_(external_children, event_string, id);
                        }
                        (common.events.as_slice(), &common.data_id)
                    }
                    ftd::Element::Markup(ftd::Markups {
                        common, children, ..
                    }) => {
                        markup_body_events_(children, event_string, id);
                        (common.events.as_slice(), &common.data_id)
                    }
                    ftd::Element::Image(ftd::Image { common, .. })
                    | ftd::Element::TextBlock(ftd::TextBlock { common, .. })
                    | ftd::Element::Code(ftd::Code { common, .. })
                    | ftd::Element::IFrame(ftd::IFrame { common, .. })
                    | ftd::Element::Input(ftd::Input { common, .. })
                    | ftd::Element::Integer(ftd::Text { common, .. })
                    | ftd::Element::Boolean(ftd::Text { common, .. })
                    | ftd::Element::Decimal(ftd::Text { common, .. }) => {
                        (common.events.as_slice(), &common.data_id)
                    }
                    ftd::Element::Null => continue,
                };
                get_events(event_string, events, id, data_id);
            }
        }

        fn markup_body_events_(
            children: &[ftd::Markup],
            event_string: &mut Vec<EventData>,
            id: &str,
        ) {
            for child in children {
                let (events, data_id) = match child.itext {
                    ftd::IText::Text(ref t)
                    | ftd::IText::Integer(ref t)
                    | ftd::IText::Boolean(ref t)
                    | ftd::IText::Decimal(ref t) => (t.common.events.as_slice(), &t.common.data_id),
                    ftd::IText::TextBlock(ref t) => (t.common.events.as_slice(), &t.common.data_id),
                    ftd::IText::Markup(ref t) => {
                        markup_body_events_(&t.children, event_string, id);
                        (t.common.events.as_slice(), &t.common.data_id)
                    }
                };
                markup_body_events_(&child.children, event_string, id);
                get_events(event_string, events, id, data_id);
            }
        }

        fn get_events(
            event_string: &mut Vec<EventData>,
            events: &[ftd::Event],
            id: &str,
            data_id: &Option<String>,
        ) {
            let events = ftd::ftd2021::event::group_by_js_event(events);
            for (name, actions) in events {
                let action = format!("window.ftd.handle_event(event, '{id}', '{actions}', this);");
                if name != "onclickoutside" && !name.starts_with("onglobalkey") {
                    continue;
                }
                let oid = if let Some(oid) = data_id {
                    format!("{oid}:{id}")
                } else {
                    format!("{id}:root")
                };
                event_string.push(EventData { oid, name, action });
            }
        }
    }

    pub fn html(
        &self,
        id: &str,
        doc_id: &str,
        rt_data: &ftd::DataDependenciesMap,
        collector: &mut ftd::Collector,
    ) -> String {
        let mut node = self.main.to_node(doc_id, false, collector);
        node.children = {
            let mut children = vec![];
            for child in self.main.container.children.iter() {
                let mut child_node = child.to_node(doc_id, collector);
                let common = if let Some(common) = child.get_common() {
                    common
                } else {
                    children.push(child_node);
                    continue;
                };
                if common.anchor.is_some() {
                    children.push(child_node);
                    continue;
                }
                if let Some(ref position) = common.position {
                    for (key, value) in ftd::ftd2021::html::column_align(position) {
                        child_node.style.insert(key.as_str().to_string(), value);
                    }
                }
                children.push(child_node);
            }
            children
        };
        node.to_html(&Default::default(), rt_data, id)
    }

    pub fn alias(&self, doc: &str) -> Option<&str> {
        for (k, v) in self.aliases.iter() {
            if v == doc {
                return Some(k);
            }
        }

        None
    }

    pub fn find<T, F>(children: &[ftd::Element], f: &F) -> Option<T>
    where
        F: Fn(&ftd::Element) -> Option<T>,
    {
        fn finder<T2, F2>(elements: &[ftd::Element], f: &F2) -> Option<T2>
        where
            F2: Fn(&ftd::Element) -> Option<T2>,
        {
            for e in elements.iter() {
                match e {
                    ftd::Element::TextBlock(_)
                    | ftd::Element::Code(_)
                    | ftd::Element::Input(_)
                    | ftd::Element::Image(_)
                    | ftd::Element::Markup(_)
                    | ftd::Element::IFrame(_)
                    | ftd::Element::Decimal(_)
                    | ftd::Element::Integer(_)
                    | ftd::Element::Boolean(_) => {
                        if let Some(v) = f(e) {
                            return Some(v);
                        }
                    }
                    ftd::Element::Column(ftd::Column { container, .. })
                    | ftd::Element::Row(ftd::Row { container, .. })
                    | ftd::Element::Scene(ftd::Scene { container, .. })
                    | ftd::Element::Grid(ftd::Grid { container, .. }) => {
                        if let Some(v) = f(e) {
                            return Some(v);
                        }

                        if let Some(t) = finder(&container.children, f) {
                            return Some(t);
                        }

                        if let Some((_, _, ref external_children)) = container.external_children
                            && let Some(t) = finder(external_children, f)
                        {
                            return Some(t);
                        }
                    }
                    ftd::Element::Null => {}
                }
            }
            None
        }

        finder(children, f)
    }

    pub fn find_text<T, F>(children: &[ftd::Element], f: F) -> Option<T>
    where
        F: Fn(&ftd::Text) -> Option<T>,
    {
        Self::find(children, &|e: &ftd::Element| -> Option<T> {
            match e {
                ftd::Element::Markup(t) => f(&t.to_owned().to_text()),
                _ => None,
            }
        })
    }

    pub fn get_heading<F>(children: &[ftd::Element], f: &F) -> Option<ftd::ftd2021::Rendered>
    where
        F: Fn(&ftd::Region) -> bool,
    {
        if let Some(t) = Self::find_text(children, |t| {
            if t.common.region.as_ref().map(f).unwrap_or(false) {
                Some(t.text.clone())
            } else {
                None
            }
        }) {
            return Some(t);
        }
        if let Some(t) = Self::find(children, &|e| match e {
            ftd::Element::Column(t) => {
                if t.common.region.as_ref().map(f).unwrap_or(false) {
                    Some(t.container.children.clone())
                } else {
                    None
                }
            }
            ftd::Element::Row(t) => {
                if t.common.region.as_ref().map(f).unwrap_or(false) {
                    Some(t.container.children.clone())
                } else {
                    None
                }
            }
            _ => None,
        }) {
            if let Some(t) = Self::find_text(&t, |t| {
                if t.common
                    .region
                    .as_ref()
                    .map(|r| r.is_title())
                    .unwrap_or(false)
                {
                    Some(t.text.clone())
                } else {
                    None
                }
            }) {
                return Some(t);
            };
            return Self::find_text(&t, |t| if t.line { Some(t.text.clone()) } else { None });
        }
        None
    }

    pub fn title(&self) -> Option<ftd::ftd2021::Rendered> {
        // find the text of first primary heading
        for i in [
            ftd::Region::H0,
            ftd::Region::H1,
            ftd::Region::H2,
            ftd::Region::H3,
            ftd::Region::H4,
            ftd::Region::H5,
            ftd::Region::H6,
            ftd::Region::H7,
        ] {
            if let Some(t) = Self::get_heading(
                &self.main.container.children,
                &|r| matches!(r, r if r == &i),
            ) {
                return Some(t);
            }
        }

        // find any text with caption
        if let Some(t) = Self::find_text(&self.main.container.children, |t| {
            if t.line { Some(t.text.clone()) } else { None }
        }) {
            return Some(t);
        }

        None
    }

    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> ftd::ftd2021::p1::Result<T> {
        let v = self.json(key)?;
        Ok(serde_json::from_value(v)?)
    }

    pub fn name(&self, k: &str) -> String {
        if k.contains('#') {
            k.to_string()
        } else {
            format!("{}#{}", self.name.as_str(), k)
        }
    }

    pub fn only_instance<T>(&self, record: &str) -> ftd::ftd2021::p1::Result<Option<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let v = self.instances::<T>(record)?;
        if v.is_empty() {
            return Ok(None);
        }
        if v.len() > 1 {
            return ftd::ftd2021::p2::utils::e2(
                format!("more than one instances({}) of {} found", v.len(), record),
                self.name.as_str(),
                0,
            );
        }
        Ok(Some(v.into_iter().next().unwrap())) // unwrap okay because v not empty
    }

    pub fn instances<T>(&self, record: &str) -> ftd::ftd2021::p1::Result<Vec<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let name = self.name(record);
        let thing = match self.data.get(name.as_str()) {
            Some(t) => t,
            None => return Ok(vec![]),
        };

        let json = match thing {
            ftd::ftd2021::p2::Thing::Record(r) => {
                let mut a = vec![];
                for c in match r.instances.get(self.name.as_str()) {
                    Some(v) => v.iter(),
                    None => return Ok(vec![]),
                } {
                    a.push(self.object_to_json(None, c)?);
                }
                serde_json::Value::Array(a)
            }
            t => {
                return ftd::ftd2021::p2::utils::e2(
                    format!("not a record: {t:?}"),
                    self.name.as_str(),
                    0,
                );
            }
        };

        Ok(serde_json::from_value(json)?)
    }

    pub fn json(&self, key: &str) -> ftd::ftd2021::p1::Result<serde_json::Value> {
        let key = self.name(key);
        let thing = match self.data.get(key.as_str()) {
            Some(v) => v,
            None => {
                return Err(ftd::ftd2021::p1::Error::NotFound {
                    doc_id: "".to_string(),
                    line_number: 0,
                    key: key.to_string(),
                });
            }
        };
        let doc = ftd::ftd2021::p2::TDoc {
            name: self.name.as_str(),
            aliases: &self.aliases,
            bag: &self.data,
            local_variables: &mut Default::default(),
            referenced_local_variables: &mut Default::default(),
        };

        match thing {
            ftd::ftd2021::p2::Thing::Variable(v) => {
                let mut property_value = &v.value;
                for (boolean, pv) in v.conditions.iter() {
                    if boolean.eval(0, &doc)? {
                        property_value = pv;
                    }
                }
                self.value_to_json(&property_value.resolve(0, &doc)?)
            }
            t => panic!("{t:?} is not a variable"),
        }
    }

    fn value_to_json(&self, v: &ftd::Value) -> ftd::ftd2021::p1::Result<serde_json::Value> {
        let doc = ftd::ftd2021::p2::TDoc {
            name: self.name.as_str(),
            aliases: &self.aliases,
            bag: &self.data,
            local_variables: &mut Default::default(),
            referenced_local_variables: &mut Default::default(),
        };
        Ok(match v {
            ftd::Value::Integer { value } => {
                serde_json::Value::Number(serde_json::Number::from(*value))
            }
            ftd::Value::Boolean { value } => serde_json::Value::Bool(*value),
            ftd::Value::Decimal { value } => {
                serde_json::Value::Number(serde_json::Number::from_f64(*value).unwrap())
                // TODO: remove unwrap
            }
            ftd::Value::String { text, .. } => serde_json::Value::String(text.to_owned()),
            ftd::Value::Record { fields, .. } => self.object_to_json(None, fields)?,
            ftd::Value::OrType {
                variant, fields, ..
            } => self.object_to_json(Some(variant), fields)?,
            ftd::Value::List { data, .. } => self.list_to_json(
                data.iter()
                    .filter_map(|v| v.resolve(0, &doc).ok())
                    .collect::<Vec<ftd::Value>>()
                    .as_slice(),
            )?,
            ftd::Value::None { .. } => serde_json::Value::Null,
            ftd::Value::Optional { data, .. } => match data.as_ref() {
                Some(v) => self.value_to_json(v)?,
                None => serde_json::Value::Null,
            },
            _ => {
                return ftd::ftd2021::p2::utils::e2(
                    format!("unhandled value found(value_to_json): {v:?}"),
                    self.name.as_str(),
                    0,
                );
            }
        })
    }

    fn list_to_json(&self, data: &[ftd::Value]) -> ftd::ftd2021::p1::Result<serde_json::Value> {
        let mut list = vec![];
        for item in data.iter() {
            list.push(self.value_to_json(item)?)
        }
        Ok(serde_json::Value::Array(list))
    }

    fn object_to_json(
        &self,
        variant: Option<&String>,
        fields: &ftd::Map<ftd::PropertyValue>,
    ) -> ftd::ftd2021::p1::Result<serde_json::Value> {
        let mut map = serde_json::Map::new();
        if let Some(v) = variant {
            map.insert("type".to_string(), serde_json::Value::String(v.to_owned()));
        }
        for (k, v) in fields.iter() {
            map.insert(k.to_string(), self.property_value_to_json(v)?);
        }
        Ok(serde_json::Value::Object(map))
    }

    fn property_value_to_json(
        &self,
        v: &ftd::PropertyValue,
    ) -> ftd::ftd2021::p1::Result<serde_json::Value> {
        match v {
            ftd::PropertyValue::Value { value, .. } => self.value_to_json(value),
            ftd::PropertyValue::Reference { name, .. } => self.json(name),
            _ => unreachable!(),
        }
    }
}

pub fn set_region_id(elements: &mut [ftd::Element]) {
    let mut map: std::collections::BTreeMap<usize, String> = Default::default();
    for element in elements.iter_mut() {
        match element {
            ftd::Element::Column(ftd::Column { container, .. })
            | ftd::Element::Row(ftd::Row { container, .. }) => {
                set_region_id(&mut container.children);
                if let Some((_, _, ref mut e)) = container.external_children {
                    set_region_id(e);
                }
            }
            _ => continue,
        }
    }

    for (idx, element) in elements.iter().enumerate() {
        match element {
            ftd::Element::Column(ftd::Column { common, .. })
            | ftd::Element::Row(ftd::Row { common, .. }) => {
                if common.region.as_ref().filter(|v| v.is_heading()).is_some()
                    && common.data_id.is_none()
                    && let Some(h) = ftd::ftd2021::p2::Document::get_heading(
                        vec![element.clone()].as_slice(),
                        &|r| r.is_heading(),
                    )
                {
                    map.insert(idx, slug::slugify(h.original));
                }
            }
            _ => continue,
        }
    }
    for (idx, s) in map {
        elements[idx].get_mut_common().unwrap().id = Some(s);
    }
}

pub fn default_scene_children_position(elements: &mut Vec<ftd::Element>) {
    for element in elements {
        if let ftd::Element::Scene(scene) = element {
            for child in &mut scene.container.children {
                check_and_set_default_position(child);
            }
            if let Some((_, _, ref mut ext_children)) = scene.container.external_children {
                for child in ext_children {
                    check_and_set_default_position(child);
                }
            }
        }
        match element {
            ftd::Element::Scene(ftd::Scene { container, .. })
            | ftd::Element::Row(ftd::Row { container, .. })
            | ftd::Element::Column(ftd::Column { container, .. })
            | ftd::Element::Grid(ftd::Grid { container, .. }) => {
                default_scene_children_position(&mut container.children);
                if let Some((_, _, ref mut ext_children)) = container.external_children {
                    default_scene_children_position(ext_children);
                }
            }
            _ => {}
        }
    }

    fn check_and_set_default_position(child: &mut ftd::Element) {
        if let Some(common) = child.get_mut_common() {
            if common.top.is_none() && common.bottom.is_none() {
                common.top = Some(0);
            }
            if common.left.is_none() && common.right.is_none() {
                common.left = Some(0);
            }
        }
    }
}
