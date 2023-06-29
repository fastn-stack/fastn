#[derive(Debug)]
pub enum Element {
    Text(Text),
    Column(Column),
}

impl Element {
    pub fn from_interpreter_component(
        component: &ftd::interpreter::Component,
        doc: &ftd::interpreter::TDoc,
    ) -> Element {
        match component.name.as_str() {
            "ftd#text" => Element::Text(Text::from(component)),
            "ftd#column" => Element::Column(Column::from(component, doc)),
            _ => todo!(),
        }
    }

    pub fn to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        component_definition_name: Option<String>,
    ) -> Vec<fastn_js::ComponentStatement> {
        match self {
            Element::Text(text) => {
                text.to_component_statements(parent, index, doc, component_definition_name)
            }
            Element::Column(column) => {
                column.to_component_statements(parent, index, doc, component_definition_name)
            }
        }
    }
}

#[derive(Debug)]
pub struct Text {
    pub text: ftd::js::Value,
    pub common: Common,
}

#[derive(Debug)]
pub struct Column {
    pub children: Vec<ftd::interpreter::Component>,
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
            text: dbg!(ftd::js::value::get_properties(
                "text",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ))
            .unwrap(),
            common: Common::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
                component.events.as_slice(),
            ),
        }
    }

    pub fn to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        component_definition_name: Option<String>,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        let kernel = fastn_js::Kernel::from_component("ftd#text", parent, index);
        component_statements.push(fastn_js::ComponentStatement::CreateKernel(kernel.clone()));
        component_statements.push(fastn_js::ComponentStatement::SetProperty(
            fastn_js::SetProperty {
                kind: fastn_js::PropertyKind::StringValue,
                value: self
                    .text
                    .to_set_property_value(component_definition_name.clone()),
                element_name: kernel.name.to_string(),
            },
        ));
        component_statements.extend(self.common.to_set_properties(
            kernel.name.as_str(),
            doc,
            component_definition_name,
        ));
        component_statements.push(fastn_js::ComponentStatement::Done {
            component_name: kernel.name,
        });
        component_statements
    }
}

impl Column {
    pub fn from(component: &ftd::interpreter::Component, doc: &ftd::interpreter::TDoc) -> Column {
        let component_definition = ftd::interpreter::default::default_bag()
            .get("ftd#column")
            .unwrap()
            .clone()
            .component()
            .unwrap();
        Column {
            children: component.get_children(doc).unwrap(),
            common: Common::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
                component.events.as_slice(),
            ),
        }
    }

    pub fn to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        component_definition_name: Option<String>,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        let kernel = fastn_js::Kernel::from_component("ftd#column", parent, index);
        component_statements.push(fastn_js::ComponentStatement::CreateKernel(kernel.clone()));
        component_statements.extend(self.common.to_set_properties(
            kernel.name.as_str(),
            doc,
            component_definition_name.clone(),
        ));

        component_statements.extend(self.children.iter().enumerate().flat_map(|(index, v)| {
            v.to_component_statements(
                kernel.name.as_str(),
                index,
                doc,
                component_definition_name.clone(),
            )
        }));
        component_statements.push(fastn_js::ComponentStatement::Done {
            component_name: kernel.name,
        });
        component_statements
    }
}

#[derive(Debug)]
pub struct Common {
    pub id: Option<ftd::js::Value>,
    pub width: Option<ftd::js::Value>,
    pub height: Option<ftd::js::Value>,
    pub padding: Option<ftd::js::Value>,
    pub margin: Option<ftd::js::Value>,
    pub border_width: Option<ftd::js::Value>,
    pub border_style: Option<ftd::js::Value>,
    pub events: Vec<ftd::interpreter::Event>,
}

impl Common {
    pub fn from(
        properties: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
        events: &[ftd::interpreter::Event],
    ) -> Common {
        Common {
            id: ftd::js::value::get_properties("id", properties, arguments),
            width: ftd::js::value::get_properties("width", properties, arguments),
            height: ftd::js::value::get_properties("height", properties, arguments),
            padding: ftd::js::value::get_properties("padding", properties, arguments),
            margin: ftd::js::value::get_properties("margin", properties, arguments),
            border_width: ftd::js::value::get_properties("border-width", properties, arguments),
            border_style: ftd::js::value::get_properties("border-style", properties, arguments),
            events: events.to_vec(),
        }
    }

    pub fn to_set_properties(
        &self,
        element_name: &str,
        doc: &ftd::interpreter::TDoc,
        component_definition_name: Option<String>,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        for event in self.events.iter() {
            component_statements.push(fastn_js::ComponentStatement::AddEventHandler(
                event.to_event_handler_js(element_name, doc, component_definition_name.clone()),
            ));
        }
        if let Some(ref id) = self.id {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                id.to_set_property(
                    fastn_js::PropertyKind::Id,
                    element_name,
                    component_definition_name.clone(),
                ),
            ));
        }
        if let Some(ref width) = self.width {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                width.to_set_property(
                    fastn_js::PropertyKind::Width,
                    element_name,
                    component_definition_name.clone(),
                ),
            ));
        }
        if let Some(ref height) = self.height {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                height.to_set_property(
                    fastn_js::PropertyKind::Height,
                    element_name,
                    component_definition_name.clone(),
                ),
            ));
        }
        if let Some(ref padding) = self.padding {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                padding.to_set_property(
                    fastn_js::PropertyKind::Padding,
                    element_name,
                    component_definition_name.clone(),
                ),
            ));
        }
        if let Some(ref margin) = self.margin {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                margin.to_set_property(
                    fastn_js::PropertyKind::Margin,
                    element_name,
                    component_definition_name.clone(),
                ),
            ));
        }
        if let Some(ref border_width) = self.border_width {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_width.to_set_property(
                    fastn_js::PropertyKind::BorderWidth,
                    element_name,
                    component_definition_name.clone(),
                ),
            ));
        }
        if let Some(ref border_style) = self.border_style {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_style.to_set_property(
                    fastn_js::PropertyKind::BorderStyle,
                    element_name,
                    component_definition_name.clone(),
                ),
            ));
        }
        component_statements
    }
}

impl ftd::interpreter::Event {
    fn to_event_handler_js(
        &self,
        element_name: &str,
        doc: &ftd::interpreter::TDoc,
        component_definition_name: Option<String>,
    ) -> fastn_js::EventHandler {
        fastn_js::EventHandler {
            event: self.name.to_js_event_name(),
            action: self.action.to_js_function(doc, component_definition_name),
            element_name: element_name.to_string(),
        }
    }
}

impl ftd::interpreter::FunctionCall {
    fn to_js_function(
        &self,
        doc: &ftd::interpreter::TDoc,
        component_definition_name: Option<String>,
    ) -> fastn_js::Function {
        let mut parameters = vec![];
        let function = doc
            .get_function(self.name.as_str(), self.line_number)
            .unwrap();
        for argument in function.arguments {
            let value = if let Some(value) = self.values.get(argument.name.as_str()) {
                value.to_value()
            } else if let Some(value) = argument.get_default_value() {
                value
            } else {
                panic!("Argument value not found {:?}", argument)
            };
            parameters.push(value.to_set_property_value(component_definition_name.clone()));
        }
        fastn_js::Function {
            name: self.name.to_string(),
            parameters,
        }
    }
}

impl ftd::interpreter::EventName {
    fn to_js_event_name(&self) -> fastn_js::Event {
        match self {
            ftd::interpreter::EventName::Click => fastn_js::Event::OnClick,
            _ => todo!(),
        }
    }
}

pub fn is_kernel(s: &str) -> bool {
    ["ftd#text", "ftd#row", "ftd#column"].contains(&s)
}
