#[derive(Debug)]
pub enum Element {
    Text(Text),
}

impl Element {
    pub fn from_interpreter_component(component: &ftd::interpreter::Component) -> Element {
        match component.name.as_str() {
            "ftd#text" => Element::Text(Text::from(component)),
            _ => todo!(),
        }
    }

    pub fn to_component_statements(
        &self,
        parent: &str,
        index: usize,
    ) -> Vec<fastn_js::ComponentStatement> {
        match self {
            Element::Text(text) => text.to_component_statements(parent, index),
        }
    }
}

#[derive(Debug)]
pub struct Text {
    pub text: ftd::js::Value,
    pub common: Common,
}

impl Text {
    pub fn from(component: &ftd::interpreter::Component) -> Text {
        let component_definition = ftd::interpreter::default::default_bag()
            .get("ftd#text")
            .unwrap()
            .clone()
            .component()
            .unwrap();
        Text {
            text: ftd::js::value::get_properties(
                "text",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            )
            .unwrap(),
            common: Common::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
        }
    }

    pub fn to_component_statements(
        &self,
        parent: &str,
        index: usize,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        let kernel = fastn_js::Kernel::from_component("ftd#text", parent, index);
        component_statements.push(fastn_js::ComponentStatement::CreateKernel(kernel.clone()));
        component_statements.push(fastn_js::ComponentStatement::SetProperty(
            fastn_js::SetProperty {
                kind: fastn_js::PropertyKind::StringValue,
                value: self.text.to_set_property_value(),
                element_name: kernel.name.to_string(),
            },
        ));
        component_statements.extend(self.common.to_set_properties(kernel.name.as_str()));
        component_statements.push(fastn_js::ComponentStatement::Done {
            component_name: kernel.name,
        });
        component_statements
    }
}

#[derive(Debug)]
pub struct Common {
    pub width: Option<ftd::js::Value>,
}

impl Common {
    pub fn from(
        property: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
    ) -> Common {
        Common {
            width: ftd::js::value::get_properties("width", property, arguments),
        }
    }

    pub fn to_set_properties(&self, element_name: &str) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        if let Some(ref width) = self.width {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                width.to_set_property(fastn_js::PropertyKind::Width, element_name),
            ));
        }
        component_statements
    }
}

pub fn is_kernel(s: &str) -> bool {
    ["ftd#text", "ftd#row", "ftd#column"].contains(&s)
}
