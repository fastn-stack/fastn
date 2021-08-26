use wasm_bindgen::prelude::wasm_bindgen;

impl ftd_rt::Node {
    pub fn to_dom(
        &self,
        style: &ftd_rt::Map,
        data: &ftd_rt::Map,
    ) -> Result<web_sys::Element, wasm_bindgen::JsValue> {
        let doc = document();
        let e = doc.create_element(self.node.as_str())?;
        for (k, v) in self.attrs.iter() {
            e.set_attribute(k, v)?;
        }
        e.set_attribute("style", self.style_to_html(style).as_str())?;
        e.set_attribute("class", self.class_to_html().as_str())?;
        if let Some(ref v) = self.text {
            e.set_inner_html(v);
            return Ok(e);
        }
        for (i, c) in self.children.iter().enumerate() {
            if !c.is_visible(data) {
                continue;
            }

            e.append_child(&web_sys::Node::from(
                c.to_dom(&self.fixed_children_style(i), data)?,
            ))?;
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
        self.document
            .data
            .insert(variable.to_string(), value.to_string());
        self.render()
    }

    pub fn render(&mut self) {
        let container = match document().get_element_by_id(self.id.as_str()) {
            Some(v) => v,
            None => {
                console_log!("no such container: {}", self.id.as_str());
                return;
            }
        };

        console_log!("rendering into {}", self.id.as_str());
        container
            .append_child(&web_sys::Node::from(
                self.document
                    .tree
                    .to_dom(&Default::default(), &self.document.data)
                    .expect("failed to render dom"),
            ))
            .unwrap(); // why would append_child fail?
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
