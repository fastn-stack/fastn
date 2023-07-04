#[derive(Debug)]
pub enum Element {
    Text(Text),
    Integer(Integer),
    Decimal(Decimal),
    Boolean(Boolean),
    Column(Column),
    // Row(Row),
}

impl Element {
    pub fn from_interpreter_component(
        component: &ftd::interpreter::Component,
        doc: &ftd::interpreter::TDoc,
    ) -> Element {
        match component.name.as_str() {
            "ftd#text" => Element::Text(Text::from(component)),
            "ftd#column" => Element::Column(Column::from(component, doc)),
            "ftd#integer" => Element::Integer(Integer::from(component)),
            "ftd#decimal" => Element::Decimal(Decimal::from(component)),
            "ftd#boolean" => Element::Boolean(Boolean::from(component)),
            _ => todo!("{}", component.name.as_str()),
        }
    }

    pub fn to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        component_definition_name: &Option<String>,
        loop_alias: &Option<String>,
        should_return: bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        match self {
            Element::Text(text) => text.to_component_statements(
                parent,
                index,
                doc,
                component_definition_name,
                loop_alias,
                should_return,
            ),
            Element::Integer(integer) => integer.to_component_statements(
                parent,
                index,
                doc,
                component_definition_name,
                loop_alias,
                should_return,
            ),
            Element::Decimal(decimal) => decimal.to_component_statements(
                parent,
                index,
                doc,
                component_definition_name,
                loop_alias,
                should_return,
            ),
            Element::Boolean(boolean) => boolean.to_component_statements(
                parent,
                index,
                doc,
                component_definition_name,
                loop_alias,
                should_return,
            ),
            Element::Column(column) => column.to_component_statements(
                parent,
                index,
                doc,
                component_definition_name,
                loop_alias,
                should_return,
            ),
        }
    }
}

#[derive(Debug)]
pub struct Text {
    pub text: ftd::js::Value,
    pub common: Common,
}

#[derive(Debug)]
pub struct Integer {
    pub value: ftd::js::Value,
    pub common: Common,
}

#[derive(Debug)]
pub struct Decimal {
    pub value: ftd::js::Value,
    pub common: Common,
}

#[derive(Debug)]
pub struct Boolean {
    pub value: ftd::js::Value,
    pub common: Common,
}

#[derive(Debug)]
pub struct Column {
    pub children: Vec<ftd::interpreter::Component>,
    pub common: Common,
}

#[derive(Debug)]
pub struct Row {
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
            text: ftd::js::value::get_properties(
                "text",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            )
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
        component_definition_name: &Option<String>,
        loop_alias: &Option<String>,
        should_return: bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        let kernel = fastn_js::Kernel::from_component("ftd#text", parent, index);
        component_statements.push(fastn_js::ComponentStatement::CreateKernel(kernel.clone()));
        component_statements.push(fastn_js::ComponentStatement::SetProperty(
            fastn_js::SetProperty {
                kind: fastn_js::PropertyKind::StringValue,
                value: self
                    .text
                    .to_set_property_value(component_definition_name, loop_alias),
                element_name: kernel.name.to_string(),
            },
        ));
        component_statements.extend(self.common.to_set_properties(
            kernel.name.as_str(),
            doc,
            component_definition_name,
            loop_alias,
        ));
        if should_return {
            component_statements.push(fastn_js::ComponentStatement::Return {
                component_name: kernel.name,
            });
        }
        component_statements
    }
}

impl Integer {
    pub fn from(component: &ftd::interpreter::Component) -> Integer {
        let component_definition = ftd::interpreter::default::default_bag()
            .get("ftd#integer")
            .unwrap()
            .clone()
            .component()
            .unwrap();
        Integer {
            value: ftd::js::value::get_properties(
                "value",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            )
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
        component_definition_name: &Option<String>,
        loop_alias: &Option<String>,
        should_return: bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        let kernel = fastn_js::Kernel::from_component("ftd#integer", parent, index);
        component_statements.push(fastn_js::ComponentStatement::CreateKernel(kernel.clone()));
        component_statements.push(fastn_js::ComponentStatement::SetProperty(
            fastn_js::SetProperty {
                kind: fastn_js::PropertyKind::StringValue,
                value: self
                    .value
                    .to_set_property_value(component_definition_name, loop_alias),
                element_name: kernel.name.to_string(),
            },
        ));
        component_statements.extend(self.common.to_set_properties(
            kernel.name.as_str(),
            doc,
            component_definition_name,
            loop_alias,
        ));
        if should_return {
            component_statements.push(fastn_js::ComponentStatement::Return {
                component_name: kernel.name,
            });
        }
        component_statements
    }
}

impl Decimal {
    pub fn from(component: &ftd::interpreter::Component) -> Decimal {
        let component_definition = ftd::interpreter::default::default_bag()
            .get("ftd#decimal")
            .unwrap()
            .clone()
            .component()
            .unwrap();
        Decimal {
            value: ftd::js::value::get_properties(
                "value",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            )
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
        component_definition_name: &Option<String>,
        loop_alias: &Option<String>,
        should_return: bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        let kernel = fastn_js::Kernel::from_component("ftd#decimal", parent, index);
        component_statements.push(fastn_js::ComponentStatement::CreateKernel(kernel.clone()));
        component_statements.push(fastn_js::ComponentStatement::SetProperty(
            fastn_js::SetProperty {
                kind: fastn_js::PropertyKind::StringValue,
                value: self
                    .value
                    .to_set_property_value(component_definition_name, loop_alias),
                element_name: kernel.name.to_string(),
            },
        ));
        component_statements.extend(self.common.to_set_properties(
            kernel.name.as_str(),
            doc,
            component_definition_name,
            loop_alias,
        ));
        if should_return {
            component_statements.push(fastn_js::ComponentStatement::Return {
                component_name: kernel.name,
            });
        }
        component_statements
    }
}

impl Boolean {
    pub fn from(component: &ftd::interpreter::Component) -> Boolean {
        let component_definition = ftd::interpreter::default::default_bag()
            .get("ftd#boolean")
            .unwrap()
            .clone()
            .component()
            .unwrap();
        Boolean {
            value: ftd::js::value::get_properties(
                "value",
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            )
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
        component_definition_name: &Option<String>,
        loop_alias: &Option<String>,
        should_return: bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        let kernel = fastn_js::Kernel::from_component("ftd#boolean", parent, index);
        component_statements.push(fastn_js::ComponentStatement::CreateKernel(kernel.clone()));
        component_statements.push(fastn_js::ComponentStatement::SetProperty(
            fastn_js::SetProperty {
                kind: fastn_js::PropertyKind::StringValue,
                value: self
                    .value
                    .to_set_property_value(component_definition_name, loop_alias),
                element_name: kernel.name.to_string(),
            },
        ));
        component_statements.extend(self.common.to_set_properties(
            kernel.name.as_str(),
            doc,
            component_definition_name,
            loop_alias,
        ));
        if should_return {
            component_statements.push(fastn_js::ComponentStatement::Return {
                component_name: kernel.name,
            });
        }
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
        component_definition_name: &Option<String>,
        loop_alias: &Option<String>,
        should_return: bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        let kernel = fastn_js::Kernel::from_component("ftd#column", parent, index);
        component_statements.push(fastn_js::ComponentStatement::CreateKernel(kernel.clone()));
        component_statements.extend(self.common.to_set_properties(
            kernel.name.as_str(),
            doc,
            component_definition_name,
            loop_alias,
        ));

        component_statements.extend(self.children.iter().enumerate().flat_map(|(index, v)| {
            v.to_component_statements(
                kernel.name.as_str(),
                index,
                doc,
                component_definition_name,
                false,
            )
        }));
        if should_return {
            component_statements.push(fastn_js::ComponentStatement::Return {
                component_name: kernel.name,
            });
        }
        component_statements
    }
}

#[derive(Debug)]
pub struct Common {
    pub id: Option<ftd::js::Value>,
    pub width: Option<ftd::js::Value>,
    pub height: Option<ftd::js::Value>,
    pub padding: Option<ftd::js::Value>,
    pub padding_horizontal: Option<ftd::js::Value>,
    pub padding_vertical: Option<ftd::js::Value>,
    pub padding_left: Option<ftd::js::Value>,
    pub padding_right: Option<ftd::js::Value>,
    pub padding_top: Option<ftd::js::Value>,
    pub padding_bottom: Option<ftd::js::Value>,
    pub margin: Option<ftd::js::Value>,
    pub margin_horizontal: Option<ftd::js::Value>,
    pub margin_vertical: Option<ftd::js::Value>,
    pub margin_left: Option<ftd::js::Value>,
    pub margin_right: Option<ftd::js::Value>,
    pub margin_top: Option<ftd::js::Value>,
    pub margin_bottom: Option<ftd::js::Value>,
    pub border_width: Option<ftd::js::Value>,
    pub border_style: Option<ftd::js::Value>,
    pub color: Option<ftd::js::Value>,
    pub background: Option<ftd::js::Value>,
    pub role: Option<ftd::js::Value>,
    pub z_index: Option<ftd::js::Value>,
    pub sticky: Option<ftd::js::Value>,
    pub top: Option<ftd::js::Value>,
    pub bottom: Option<ftd::js::Value>,
    pub left: Option<ftd::js::Value>,
    pub right: Option<ftd::js::Value>,
    pub overflow: Option<ftd::js::Value>,
    pub overflow_x: Option<ftd::js::Value>,
    pub overflow_y: Option<ftd::js::Value>,
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
            padding_horizontal: ftd::js::value::get_properties(
                "padding-horizontal",
                properties,
                arguments,
            ),
            padding_vertical: ftd::js::value::get_properties(
                "padding-vertical",
                properties,
                arguments,
            ),
            padding_left: ftd::js::value::get_properties("padding-left", properties, arguments),
            padding_right: ftd::js::value::get_properties("padding-right", properties, arguments),
            padding_top: ftd::js::value::get_properties("padding-top", properties, arguments),
            padding_bottom: ftd::js::value::get_properties("padding-bottom", properties, arguments),
            margin: ftd::js::value::get_properties("margin", properties, arguments),
            margin_horizontal: ftd::js::value::get_properties(
                "margin-horizontal",
                properties,
                arguments,
            ),
            margin_vertical: ftd::js::value::get_properties(
                "margin-vertical",
                properties,
                arguments,
            ),
            margin_left: ftd::js::value::get_properties("margin-left", properties, arguments),
            margin_right: ftd::js::value::get_properties("margin-right", properties, arguments),
            margin_top: ftd::js::value::get_properties("margin-top", properties, arguments),
            margin_bottom: ftd::js::value::get_properties("margin-bottom", properties, arguments),
            border_width: ftd::js::value::get_properties("border-width", properties, arguments),
            border_style: ftd::js::value::get_properties("border-style", properties, arguments),
            color: ftd::js::value::get_properties("color", properties, arguments),
            background: ftd::js::value::get_properties("background", properties, arguments),
            role: ftd::js::value::get_properties("role", properties, arguments),
            z_index: ftd::js::value::get_properties("z-index", properties, arguments),
            sticky: ftd::js::value::get_properties("sticky", properties, arguments),
            top: ftd::js::value::get_properties("top", properties, arguments),
            bottom: ftd::js::value::get_properties("bottom", properties, arguments),
            left: ftd::js::value::get_properties("left", properties, arguments),
            right: ftd::js::value::get_properties("right", properties, arguments),
            overflow: ftd::js::value::get_properties("overflow", properties, arguments),
            overflow_x: ftd::js::value::get_properties("overflow-x", properties, arguments),
            overflow_y: ftd::js::value::get_properties("overflow-y", properties, arguments),
            events: events.to_vec(),
        }
    }

    pub fn to_set_properties(
        &self,
        element_name: &str,
        doc: &ftd::interpreter::TDoc,
        component_definition_name: &Option<String>,
        loop_alias: &Option<String>,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        for event in self.events.iter() {
            component_statements.push(fastn_js::ComponentStatement::AddEventHandler(
                event.to_event_handler_js(element_name, doc, component_definition_name, loop_alias),
            ));
        }
        if let Some(ref id) = self.id {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                id.to_set_property(
                    fastn_js::PropertyKind::Id,
                    element_name,
                    component_definition_name,
                    loop_alias,
                ),
            ));
        }
        if let Some(ref width) = self.width {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                width.to_set_property(
                    fastn_js::PropertyKind::Width,
                    element_name,
                    component_definition_name,
                    loop_alias,
                ),
            ));
        }
        if let Some(ref height) = self.height {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                height.to_set_property(
                    fastn_js::PropertyKind::Height,
                    element_name,
                    component_definition_name,
                    loop_alias,
                ),
            ));
        }
        if let Some(ref padding) = self.padding {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                padding.to_set_property(
                    fastn_js::PropertyKind::Padding,
                    element_name,
                    component_definition_name,
                    loop_alias,
                ),
            ));
        }
        if let Some(ref padding_horizontal) = self.padding_horizontal {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                padding_horizontal.to_set_property(
                    fastn_js::PropertyKind::PaddingHorizontal,
                    element_name,
                    component_definition_name,
                    loop_alias,
                ),
            ));
        }
        if let Some(ref padding_vertical) = self.padding_vertical {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                padding_vertical.to_set_property(
                    fastn_js::PropertyKind::PaddingVertical,
                    element_name,
                    component_definition_name,
                    loop_alias,
                ),
            ));
        }
        if let Some(ref padding_left) = self.padding_left {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                padding_left.to_set_property(
                    fastn_js::PropertyKind::PaddingLeft,
                    element_name,
                    component_definition_name,
                    loop_alias,
                ),
            ));
        }
        if let Some(ref padding_right) = self.padding_right {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                padding_right.to_set_property(
                    fastn_js::PropertyKind::PaddingRight,
                    element_name,
                    component_definition_name,
                    loop_alias,
                ),
            ));
        }
        if let Some(ref padding_top) = self.padding_top {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                padding_top.to_set_property(
                    fastn_js::PropertyKind::PaddingTop,
                    element_name,
                    component_definition_name,
                    loop_alias,
                ),
            ));
        }
        if let Some(ref padding_bottom) = self.padding_bottom {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                padding_bottom.to_set_property(
                    fastn_js::PropertyKind::PaddingBottom,
                    element_name,
                    component_definition_name,
                    loop_alias,
                ),
            ));
        }
        if let Some(ref margin) = self.margin {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                margin.to_set_property(
                    fastn_js::PropertyKind::Margin,
                    element_name,
                    component_definition_name,
                    loop_alias,
                ),
            ));
        }
        if let Some(ref margin_horizontal) = self.margin_horizontal {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                margin_horizontal.to_set_property(
                    fastn_js::PropertyKind::MarginHorizontal,
                    element_name,
                    component_definition_name,
                    loop_alias,
                ),
            ));
        }
        if let Some(ref margin_vertical) = self.margin_vertical {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                margin_vertical.to_set_property(
                    fastn_js::PropertyKind::MarginVertical,
                    element_name,
                    component_definition_name,
                    loop_alias,
                ),
            ));
        }
        if let Some(ref margin_left) = self.margin_left {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                margin_left.to_set_property(
                    fastn_js::PropertyKind::MarginLeft,
                    element_name,
                    component_definition_name,
                    loop_alias,
                ),
            ));
        }
        if let Some(ref margin_right) = self.margin_right {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                margin_right.to_set_property(
                    fastn_js::PropertyKind::MarginRight,
                    element_name,
                    component_definition_name,
                    loop_alias,
                ),
            ));
        }
        if let Some(ref margin_top) = self.margin_top {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                margin_top.to_set_property(
                    fastn_js::PropertyKind::MarginTop,
                    element_name,
                    component_definition_name,
                    loop_alias,
                ),
            ));
        }
        if let Some(ref margin_bottom) = self.margin_bottom {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                margin_bottom.to_set_property(
                    fastn_js::PropertyKind::MarginBottom,
                    element_name,
                    component_definition_name,
                    loop_alias,
                ),
            ));
        }
        if let Some(ref border_width) = self.border_width {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_width.to_set_property(
                    fastn_js::PropertyKind::BorderWidth,
                    element_name,
                    component_definition_name,
                    loop_alias,
                ),
            ));
        }
        if let Some(ref border_style) = self.border_style {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_style.to_set_property(
                    fastn_js::PropertyKind::BorderStyle,
                    element_name,
                    component_definition_name,
                    loop_alias,
                ),
            ));
        }
        if let Some(ref overflow) = self.overflow {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                overflow.to_set_property(
                    fastn_js::PropertyKind::Overflow,
                    element_name,
                    component_definition_name,
                    loop_alias,
                ),
            ));
        }
        if let Some(ref overflow_x) = self.overflow_x {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                overflow_x.to_set_property(
                    fastn_js::PropertyKind::OverflowX,
                    element_name,
                    component_definition_name,
                    loop_alias,
                ),
            ));
        }
        if let Some(ref overflow_y) = self.overflow_y {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                overflow_y.to_set_property(
                    fastn_js::PropertyKind::OverflowY,
                    element_name,
                    component_definition_name,
                    loop_alias,
                ),
            ));
        }
        if let Some(ref top) = self.top {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                top.to_set_property(
                    fastn_js::PropertyKind::Top,
                    element_name,
                    component_definition_name,
                    loop_alias,
                ),
            ));
        }
        if let Some(ref bottom) = self.bottom {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                bottom.to_set_property(
                    fastn_js::PropertyKind::Bottom,
                    element_name,
                    component_definition_name,
                    loop_alias,
                ),
            ));
        }
        if let Some(ref left) = self.left {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                left.to_set_property(
                    fastn_js::PropertyKind::Left,
                    element_name,
                    component_definition_name,
                    loop_alias,
                ),
            ));
        }
        if let Some(ref right) = self.right {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                right.to_set_property(
                    fastn_js::PropertyKind::Right,
                    element_name,
                    component_definition_name,
                    loop_alias,
                ),
            ));
        }
        if let Some(ref z_index) = self.z_index {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                z_index.to_set_property(
                    fastn_js::PropertyKind::ZIndex,
                    element_name,
                    component_definition_name,
                    loop_alias,
                ),
            ));
        }
        if let Some(ref sticky) = self.sticky {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                sticky.to_set_property(
                    fastn_js::PropertyKind::Sticky,
                    element_name,
                    component_definition_name,
                    loop_alias,
                ),
            ));
        }
        if let Some(ref color) = self.color {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                color.to_set_property(
                    fastn_js::PropertyKind::Color,
                    element_name,
                    component_definition_name,
                    loop_alias,
                ),
            ));
        }
        if let Some(ref background) = self.background {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                background.to_set_property(
                    fastn_js::PropertyKind::Background,
                    element_name,
                    component_definition_name,
                    loop_alias,
                ),
            ));
        }
        if let Some(ref background) = self.role {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                background.to_set_property(
                    fastn_js::PropertyKind::Role,
                    element_name,
                    component_definition_name,
                    loop_alias,
                ),
            ));
        }
        component_statements
    }
}

impl ftd::interpreter::Event {
    pub(crate) fn to_event_handler_js(
        &self,
        element_name: &str,
        doc: &ftd::interpreter::TDoc,
        component_definition_name: &Option<String>,
        loop_alias: &Option<String>,
    ) -> fastn_js::EventHandler {
        fastn_js::EventHandler {
            event: self.name.to_js_event_name(),
            action: self
                .action
                .to_js_function(doc, component_definition_name, loop_alias),
            element_name: element_name.to_string(),
        }
    }
}

impl ftd::interpreter::FunctionCall {
    fn to_js_function(
        &self,
        doc: &ftd::interpreter::TDoc,
        component_definition_name: &Option<String>,
        loop_alias: &Option<String>,
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
            parameters.push(value.to_set_property_value(component_definition_name, loop_alias));
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
    [
        "ftd#text",
        "ftd#row",
        "ftd#column",
        "ftd#integer",
        "ftd#decimal",
        "ftd#boolean",
    ]
    .contains(&s)
}
