#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Document {
    pub data: std::collections::BTreeMap<String, ftd::p2::Thing>,
    pub name: String,
    pub instructions: Vec<ftd::Instruction>,
    pub main: ftd::Column,
    pub p1: Vec<ftd::p1::Section>,
    pub aliases: std::collections::BTreeMap<String, String>,
}

impl ToString for Document {
    fn to_string(&self) -> String {
        ftd::p1::to_string(&self.p1)
    }
}

impl Document {
    fn get_data(&self) -> ftd::Map {
        let mut d: ftd::Map = Default::default();
        for (k, v) in self.data.iter() {
            if let ftd::p2::Thing::Variable(ftd::Variable { value, .. }) = v {
                let value = match value {
                    ftd::Value::Boolean { value } => value.to_string(),
                    ftd::Value::Integer { value } => value.to_string(),
                    ftd::Value::String { text: value, .. } => value.to_string(),
                    _ => continue,
                };
                d.insert(k.to_string(), value);
            }
        }
        d
    }

    fn get_locals(&self) -> ftd::Map {
        ftd::Element::get_locals(&self.main.container.children)
    }

    fn rt_data(&self) -> ftd::DataDependenciesMap {
        let mut d: ftd::Map = self.get_data();
        for (k, v) in self.get_locals() {
            d.insert(format!("@{}", k), v.to_string());
        }

        let mut data: ftd::DataDependenciesMap = Default::default();
        for (k, v) in d {
            data.insert(
                k.to_string(),
                ftd::Data {
                    value: v.to_string(),
                    dependencies: Default::default(),
                },
            );
        }
        ftd::Element::get_visible_event_dependencies(&self.main.container.children, &mut data);
        ftd::Element::get_value_event_dependencies(&self.main.container.children, &mut data);
        ftd::Element::get_style_event_dependencies(&self.main.container.children, &mut data);

        data
    }

    pub fn rerender(&mut self, id: &str, doc_id: &str) -> ftd::p1::Result<ftd::Document> {
        let mut rt = ftd::RT::from(
            self.name.as_str(),
            self.aliases.clone(),
            self.data.clone(),
            self.instructions.clone(),
        );
        self.main = rt.render()?;
        let data = self.rt_data();
        Ok(ftd::Document {
            data,
            html: self.html(id, doc_id),
            external_children: ftd::Element::get_external_children_dependencies(
                &self.main.container.children,
            ),
        })
    }

    pub fn to_rt(&self, id: &str, doc_id: &str) -> ftd::Document {
        let external_children =
            ftd::Element::get_external_children_dependencies(&self.main.container.children);

        ftd::Document {
            data: self.rt_data(),
            html: self.html(id, doc_id),
            external_children,
        }
    }

    pub fn html(&self, id: &str, doc_id: &str) -> String {
        self.main
            .to_node(doc_id)
            .to_html(&Default::default(), &self.rt_data(), id)
    }

    pub fn set_string(&mut self, name: &str, value: &str) {
        let thing = ftd::p2::Thing::Variable(ftd::Variable {
            name: name.to_string(),
            value: ftd::Value::String {
                text: value.to_string(),
                source: ftd::TextSource::Header,
            },
            conditions: vec![],
        });
        self.data.insert(name.to_string(), thing);
    }

    pub fn set_bool(&mut self, name: &str, value: bool) {
        let thing = ftd::p2::Thing::Variable(ftd::Variable {
            name: name.to_string(),
            value: ftd::Value::Boolean { value },
            conditions: vec![],
        });
        self.data.insert(name.to_string(), thing);
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
                    ftd::Element::Text(_) => {
                        if let Some(v) = f(e) {
                            return Some(v);
                        }
                    }
                    ftd::Element::TextBlock(_) => {
                        if let Some(v) = f(e) {
                            return Some(v);
                        }
                    }
                    ftd::Element::Code(_) => {
                        if let Some(v) = f(e) {
                            return Some(v);
                        }
                    }
                    ftd::Element::Input(_) => {
                        if let Some(v) = f(e) {
                            return Some(v);
                        }
                    }
                    ftd::Element::Column(c) => {
                        if let Some(v) = f(e) {
                            return Some(v);
                        }

                        if let Some(t) = finder(&c.container.children, f) {
                            return Some(t);
                        }
                    }
                    ftd::Element::Row(c) => {
                        if let Some(v) = f(e) {
                            return Some(v);
                        }

                        if let Some(t) = finder(&c.container.children, f) {
                            return Some(t);
                        }
                    }
                    ftd::Element::Scene(c) => {
                        if let Some(v) = f(e) {
                            return Some(v);
                        }

                        if let Some(t) = finder(&c.container.children, f) {
                            return Some(t);
                        }
                    }
                    ftd::Element::Grid(c) => {
                        if let Some(v) = f(e) {
                            return Some(v);
                        }

                        if let Some(t) = finder(&c.container.children, f) {
                            return Some(t);
                        }
                    }
                    ftd::Element::Image(_) => {
                        if let Some(v) = f(e) {
                            return Some(v);
                        }
                    }
                    ftd::Element::Markup(_) => {
                        if let Some(v) = f(e) {
                            return Some(v);
                        }
                    }
                    ftd::Element::IFrame(_) => {
                        if let Some(v) = f(e) {
                            return Some(v);
                        }
                    }
                    ftd::Element::Decimal(_) => {
                        if let Some(v) = f(e) {
                            return Some(v);
                        }
                    }
                    ftd::Element::Integer(_) => {
                        if let Some(v) = f(e) {
                            return Some(v);
                        }
                    }
                    ftd::Element::Boolean(_) => {
                        if let Some(v) = f(e) {
                            return Some(v);
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
                ftd::Element::Text(t) => f(t),
                _ => None,
            }
        })
    }

    pub fn without_render(
        name: &str,
        source: &str,
        lib: &dyn ftd::p2::Library,
    ) -> ftd::p1::Result<Document> {
        let mut interpreter = ftd::p2::interpreter::Interpreter::new(lib);
        let instructions = interpreter.interpret(name, source)?;
        // dbg!(&instructions);
        let rt = ftd::RT::from(name, interpreter.aliases, interpreter.bag, instructions);

        Ok(Document {
            main: Default::default(),
            data: rt.bag,
            instructions: rt.instructions,
            p1: interpreter.p1,
            aliases: rt.aliases,
            name: name.to_string(),
        })
    }

    pub fn from(name: &str, source: &str, lib: &dyn ftd::p2::Library) -> ftd::p1::Result<Document> {
        let mut d = Self::without_render(name, source, lib)?;

        let mut rt = ftd::RT::from(
            d.name.as_str(),
            d.aliases.clone(),
            d.data.clone(),
            d.instructions.clone(),
        );

        d.main = rt.render()?;
        dbg!(&d.main);
        Ok(d)
    }

    pub fn get_heading<F>(children: &[ftd::Element], f: &F) -> Option<ftd::Rendered>
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

    pub fn title(&self) -> Option<ftd::Rendered> {
        // find the text of first primary heading
        for i in vec![
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
            if t.line {
                Some(t.text.clone())
            } else {
                None
            }
        }) {
            return Some(t);
        }

        None
    }

    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> ftd::p1::Result<T> {
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

    pub fn only_instance<T>(&self, record: &str) -> ftd::p1::Result<Option<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let v = self.instances::<T>(record)?;
        if v.is_empty() {
            return Ok(None);
        }
        if v.len() > 1 {
            return ftd::e2(
                format!("more than one instances({}) of {} found", v.len(), record),
                self.name.as_str(),
                0,
            );
        }
        Ok(Some(v.into_iter().next().unwrap())) // unwrap okay because v not empty
    }

    pub fn instances<T>(&self, record: &str) -> ftd::p1::Result<Vec<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let name = self.name(record);
        let thing = match self.data.get(name.as_str()) {
            Some(t) => t,
            None => return Ok(vec![]),
        };

        let json = match thing {
            ftd::p2::Thing::Record(r) => {
                let mut a = vec![];
                for c in match r.instances.get(self.name.as_str()) {
                    Some(v) => v.iter(),
                    None => return Ok(vec![]),
                } {
                    a.push(self.object_to_json(None, c)?);
                }
                serde_json::Value::Array(a)
            }
            t => return ftd::e2(format!("not a record: {:?}", t), self.name.as_str(), 0),
        };

        Ok(serde_json::from_value(json)?)
    }

    #[cfg(calls)]
    pub fn calls<T: serde::de::DeserializeOwned>(
        &self,
        component: &str,
    ) -> ftd::p1::Result<Vec<T>> {
        let component = self.name(component);
        let thing = match self.data.get(component.as_str()) {
            Some(t) => t,
            None => return Ok(vec![]),
        };

        let json = match thing {
            ftd::p2::Thing::Component(c) => {
                let mut a = vec![];
                for c in c.invocations.iter() {
                    a.push(self.object2_to_json(c)?);
                }
                serde_json::Value::Array(a)
            }
            t => panic!("{:?} is not a component", t),
        };

        Ok(serde_json::from_value(json)?)
    }

    pub fn json(&self, key: &str) -> ftd::p1::Result<serde_json::Value> {
        let key = self.name(key);
        let thing = match self.data.get(key.as_str()) {
            Some(v) => v,
            None => {
                return Err(ftd::p1::Error::NotFound {
                    doc_id: "".to_string(),
                    line_number: 0,
                    key: key.to_string(),
                })
            }
        };

        match thing {
            ftd::p2::Thing::Variable(v) => self.value_to_json(&v.value),
            t => panic!("{:?} is not a variable", t),
        }
    }

    fn value_to_json(&self, v: &ftd::Value) -> ftd::p1::Result<serde_json::Value> {
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
            ftd::Value::List { data, .. } => self.list_to_json(data)?,
            ftd::Value::None { .. } => serde_json::Value::Null,
            _ => {
                return ftd::e2(
                    format!("unhandled value found(value_to_json): {:?}", v),
                    self.name.as_str(),
                    0,
                )
            }
        })
    }

    fn list_to_json(&self, data: &[ftd::Value]) -> ftd::p1::Result<serde_json::Value> {
        let mut list = vec![];
        for item in data.iter() {
            list.push(self.value_to_json(item)?)
        }
        Ok(serde_json::Value::Array(list))
    }

    #[cfg(calls)]
    fn object2_to_json(
        &self,
        fields: &std::collections::BTreeMap<String, ftd::Value>,
    ) -> ftd::p1::Result<serde_json::Value> {
        let mut map = serde_json::Map::new();
        for (k, v) in fields.iter() {
            map.insert(k.to_string(), self.value_to_json(v)?);
        }
        Ok(serde_json::Value::Object(map))
    }

    fn object_to_json(
        &self,
        variant: Option<&String>,
        fields: &std::collections::BTreeMap<String, ftd::PropertyValue>,
    ) -> ftd::p1::Result<serde_json::Value> {
        let mut map = serde_json::Map::new();
        if let Some(v) = variant {
            map.insert("type".to_string(), serde_json::Value::String(v.to_owned()));
        }
        for (k, v) in fields.iter() {
            map.insert(k.to_string(), self.property_value_to_json(v)?);
        }
        Ok(serde_json::Value::Object(map))
    }

    fn property_value_to_json(&self, v: &ftd::PropertyValue) -> ftd::p1::Result<serde_json::Value> {
        match v {
            ftd::PropertyValue::Value { value, .. } => self.value_to_json(value),
            ftd::PropertyValue::Reference { name, .. } => self.json(name),
            _ => unreachable!(),
        }
    }
}

pub fn set_region_id(elements: &mut Vec<ftd::Element>) {
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
                {
                    if let Some(h) =
                        ftd::p2::Document::get_heading(vec![element.clone()].as_slice(), &|r| {
                            r.is_heading()
                        })
                    {
                        map.insert(idx, slug::slugify(h.original));
                    }
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

#[cfg(test)]
mod test {
    use ftd::test::*;

    #[test]
    fn variable_from_other_doc() {
        let bag = super::Document::from(
            "foo/bar",
            indoc::indoc!(
                "
            -- import: fifthtry/ft

            -- ft.toc:

            foo is the toc
            "
            ),
            &ftd::p2::TestLibrary {},
        )
        .unwrap();

        pretty_assertions::assert_eq!(
            bag.get::<String>("fifthtry/ft#toc").unwrap(),
            "foo is the toc"
        );
    }

    #[test]
    fn meta() {
        #[derive(Debug, PartialEq, serde::Deserialize)]
        #[serde(tag = "type")]
        enum Someone {
            Username { username: String },
            Who { who: String },
        }

        #[derive(Debug, PartialEq, serde::Deserialize)]
        struct Meta {
            license: String,
            reader: Vec<Someone>,
        }

        let bag = super::Document::from(
            "foo/bar",
            indoc::indoc!(
                "
                -- or-type someone:

                --- Username:
                caption username:

                --- Who:
                caption who:

                -- record meta_type:
                string license:
                someone list reader:

                -- meta_type list meta:

                -- meta:
                license: BSD

                --- reader.Username: foo
                --- reader.Who: everyone
            "
            ),
            &ftd::p2::TestLibrary {},
        )
        .unwrap();

        pretty_assertions::assert_eq!(
            bag.get::<Vec<Meta>>("meta").unwrap(),
            vec![Meta {
                license: s("BSD"),
                reader: vec![
                    Someone::Username { username: s("foo") },
                    Someone::Who { who: s("everyone") }
                ],
            }]
        )
    }

    #[test]
    #[cfg(calls)]
    #[ignore] // TODO: this is buggy
    fn calls() {
        #[derive(Debug, PartialEq, serde::Deserialize)]
        struct PR {
            number: i64,
            title: String,
        }

        let bag = super::Document::from(
            "foo/bar",
            indoc::indoc!(
                "
                -- component pr:
                $number: integer
                $title: caption
                component: ftd.text
                text: ref $title

                -- pr: some pr
                number: 24

                -- pr: some other pr
                number: 224
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .unwrap();

        pretty_assertions::assert_eq!(
            bag.instances::<PR>("pr").unwrap(),
            vec![
                PR {
                    number: 24,
                    title: s("some pr")
                },
                PR {
                    number: 224,
                    title: s("some other pr")
                }
            ]
        )
    }

    #[test]
    fn instances() {
        #[derive(Debug, PartialEq, serde::Deserialize)]
        struct PR {
            number: i64,
            title: String,
        }

        let bag = super::Document::from(
            "foo/bar",
            indoc::indoc!(
                "
                -- record pr:
                integer number:
                caption title:

                -- pr: some pr
                number: 24

                -- pr: some other pr
                number: 224
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .unwrap();

        pretty_assertions::assert_eq!(
            bag.instances::<PR>("pr").unwrap(),
            vec![
                PR {
                    number: 24,
                    title: s("some pr")
                },
                PR {
                    number: 224,
                    title: s("some other pr")
                }
            ]
        )
    }
}
