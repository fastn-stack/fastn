use wasm_bindgen::prelude::wasm_bindgen;

impl ftd_rt::Node {
    pub fn to_dom(
        &self,
        style: &ftd_rt::Map,
        data: &ftd_rt::DataDependenciesMap,
        id: &str,
    ) -> Result<web_sys::Element, wasm_bindgen::JsValue> {
        self.to_dnode(style, data, &mut None, &None, &[], true, id, false)
            .to_dom(id)
    }
}

impl ftd_rt::dnode::DNode {
    fn to_dom(&self, id: &str) -> Result<web_sys::Element, wasm_bindgen::JsValue> {
        let doc = document();
        let e = doc.create_element(self.node.as_str())?;
        for (k, v) in self.attrs.iter() {
            e.set_attribute(k, v)?;
        }
        e.set_attribute("style", self.style_to_html(self.visible).as_str())?;
        e.set_attribute("class", self.class_to_html().as_str())?;

        let events = ftd_rt::event::group_by_js_event(&self.events);
        for (name, actions) in events {
            e.set_attribute(
                name.as_str(),
                format!(
                    "window.ftd.handle_event(event, \"{}\", \"{}\")",
                    id, actions
                )
                .as_str(),
            )?;
        }

        if let Some(ref v) = self.text {
            e.set_inner_html(v);
            return Ok(e);
        }
        for c in self.children.iter() {
            e.append_child(&web_sys::Node::from(c.to_dom(id)?))?;
        }
        Ok(e)
    }
}

fn document() -> web_sys::Document {
    web_sys::window()
        .expect("no global `window` exists")
        .document()
        .expect("should have a document on window")
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
#[derive(serde::Deserialize)]
pub struct Document {
    document: ftd_rt::Document,
    id: String,
}

impl Drop for Document {
    fn drop(&mut self) {
        if let Some(v) = document().get_element_by_id(self.id.as_str()) {
            console_log!("Document::drop: emptying {}", self.id.as_str());
            v.set_inner_html("")
        } else {
            console_log!("Document::drop: {} not found", self.id.as_str());
        }
    }
}

#[wasm_bindgen]
impl Document {
    pub fn set_bool(&mut self, variable: &str, value: bool) {
        console_log!("setting {} to {}", variable, value);
        let dependencies = if let Some(data) = self.document.data.get(variable) {
            data.dependencies.clone()
        } else {
            Default::default()
        };

        self.document.data.insert(
            variable.to_string(),
            ftd_rt::Data {
                value: value.to_string(),
                dependencies,
            },
        );
        self.render()
    }

    pub fn set_multi_value(&mut self, list: Vec<wasm_bindgen::JsValue>) {
        for items in list.iter() {
            let (variable, value) = {
                if let Ok((variable, value)) = items.into_serde::<(String, bool)>() {
                    (variable, value.to_string())
                } else if let Ok((variable, value)) = items.into_serde::<(String, i64)>() {
                    (variable, value.to_string())
                } else if let Ok((variable, value)) = items.into_serde::<(String, f64)>() {
                    (variable, value.to_string())
                } else {
                    items
                        .into_serde::<(String, String)>()
                        .expect("failed to parse variable and value")
                }
            };
            console_log!("setting {} to {}", variable, value);
            let dependencies = if let Some(data) = self.document.data.get(&variable) {
                data.dependencies.clone()
            } else {
                Default::default()
            };

            self.document.data.insert(
                variable.to_string(),
                ftd_rt::Data {
                    value: value.to_string(),
                    dependencies,
                },
            );
        }
        self.render()
    }

    pub fn render(&mut self) {
        let container = match document().get_element_by_id(self.id.as_str()) {
            Some(v) => {
                console_log!("Document::drop: emptying {}", self.id.as_str());
                v.set_inner_html("");
                v
            }
            None => {
                console_log!("no such container: {}", self.id.as_str());
                return;
            }
        };

        console_log!("rendering into {}", self.id.as_str());
        let rendered_dom = web_sys::Node::from(
            self.document
                .tree
                .to_dom(&Default::default(), &self.document.data, &self.id)
                .expect("failed to render dom"),
        );

        container.append_child(&rendered_dom).unwrap(); // why would append_child fail?
    }

    pub fn handle_event(&mut self, event: &str) {
        // $event-click$: toggle foo
        // value of event: "toggle foo"
        console_log!("event: {}", event);
        let actions = ftd_rt::event::Action::parse_js_event(event);
        for action in actions {
            action.handle_action(&mut self.document);
        }
        console_log!("rendering event: {}", event);
        self.render()
    }
}

#[wasm_bindgen]
pub fn create(id: &str, data: &wasm_bindgen::JsValue) -> Document {
    let document = match data.into_serde() {
        Ok(rt) => rt,
        Err(e) => {
            console_log!("failed to create: {:#?}", e);
            panic!()
        }
    };

    Document {
        id: id.to_string(),
        document,
    }
}
