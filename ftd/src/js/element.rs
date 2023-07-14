#![allow(unknown_lints)]
#![allow(renamed_and_removed_lints)]
#![allow(too_many_arguments)]

#[derive(Debug)]
pub enum Element {
    Text(Text),
    Integer(Integer),
    Decimal(Decimal),
    Boolean(Boolean),
    Column(Column),
    Row(Row),
    Device(Device),
}

impl Element {
    pub fn from_interpreter_component(component: &ftd::interpreter::Component) -> Element {
        match component.name.as_str() {
            "ftd#text" => Element::Text(Text::from(component)),
            "ftd#integer" => Element::Integer(Integer::from(component)),
            "ftd#decimal" => Element::Decimal(Decimal::from(component)),
            "ftd#boolean" => Element::Boolean(Boolean::from(component)),
            "ftd#column" => Element::Column(Column::from(component)),
            "ftd#row" => Element::Row(Row::from(component)),
            "ftd#desktop" | "ftd#mobile" => {
                Element::Device(Device::from(component, component.name.as_str()))
            }
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
        inherited_variable_name: &str,
        device: &Option<fastn_js::DeviceType>,
        should_return: bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        match self {
            Element::Text(text) => text.to_component_statements(
                parent,
                index,
                doc,
                component_definition_name,
                loop_alias,
                inherited_variable_name,
                device,
                should_return,
            ),
            Element::Integer(integer) => integer.to_component_statements(
                parent,
                index,
                doc,
                component_definition_name,
                loop_alias,
                inherited_variable_name,
                device,
                should_return,
            ),
            Element::Decimal(decimal) => decimal.to_component_statements(
                parent,
                index,
                doc,
                component_definition_name,
                loop_alias,
                inherited_variable_name,
                device,
                should_return,
            ),
            Element::Boolean(boolean) => boolean.to_component_statements(
                parent,
                index,
                doc,
                component_definition_name,
                loop_alias,
                inherited_variable_name,
                device,
                should_return,
            ),
            Element::Column(column) => column.to_component_statements(
                parent,
                index,
                doc,
                component_definition_name,
                loop_alias,
                inherited_variable_name,
                device,
                should_return,
            ),
            Element::Row(row) => row.to_component_statements(
                parent,
                index,
                doc,
                component_definition_name,
                loop_alias,
                inherited_variable_name,
                device,
                should_return,
            ),
            Element::Device(d) => d.to_component_statements(
                parent,
                index,
                doc,
                component_definition_name,
                loop_alias,
                inherited_variable_name,
                device,
                should_return,
            ),
        }
    }
}

#[derive(Debug)]
pub struct Text {
    pub text: ftd::js::Value,
    pub common: Common,
    pub text_common: TextCommon,
}

#[derive(Debug)]
pub struct Integer {
    pub value: ftd::js::Value,
    pub common: Common,
    pub text_common: TextCommon,
}

#[derive(Debug)]
pub struct Decimal {
    pub value: ftd::js::Value,
    pub common: Common,
    pub text_common: TextCommon,
}

#[derive(Debug)]
pub struct Boolean {
    pub value: ftd::js::Value,
    pub common: Common,
    pub text_common: TextCommon,
}

#[derive(Debug)]
pub struct Column {
    pub children: Option<ftd::interpreter::Property>,
    pub inherited: InheritedProperties,
    pub container: Container,
    pub common: Common,
}

#[derive(Debug)]
pub struct InheritedProperties {
    pub colors: Option<ftd::js::Value>,
    pub types: Option<ftd::js::Value>,
}

#[derive(Debug)]
pub struct Container {
    pub spacing: Option<ftd::js::Value>,
    pub wrap: Option<ftd::js::Value>,
    pub align_content: Option<ftd::js::Value>,
}

impl Container {
    pub fn from(
        properties: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
    ) -> Container {
        Container {
            spacing: ftd::js::value::get_properties("spacing", properties, arguments),
            wrap: ftd::js::value::get_properties("wrap", properties, arguments),
            align_content: ftd::js::value::get_properties("align-content", properties, arguments),
        }
    }

    pub fn to_set_properties(
        &self,
        element_name: &str,
        doc: &ftd::interpreter::TDoc,
        component_definition_name: &Option<String>,
        loop_alias: &Option<String>,
        inherited_variable_name: &str,
        device: &Option<fastn_js::DeviceType>,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        if let Some(ref spacing) = self.spacing {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                spacing.to_set_property(
                    fastn_js::PropertyKind::Spacing,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref wrap) = self.wrap {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                wrap.to_set_property(
                    fastn_js::PropertyKind::Wrap,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref align_content) = self.align_content {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                align_content.to_set_property(
                    fastn_js::PropertyKind::AlignContent,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        component_statements
    }
}

#[derive(Debug)]
pub struct Row {
    pub children: Option<ftd::interpreter::Property>,
    pub inherited: InheritedProperties,
    pub container: Container,
    pub common: Common,
}

impl InheritedProperties {
    pub fn from(
        properties: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
    ) -> InheritedProperties {
        InheritedProperties {
            colors: ftd::js::value::get_properties("colors", properties, arguments),
            types: ftd::js::value::get_properties("types", properties, arguments),
        }
    }

    pub(crate) fn get_inherited_variables(
        &self,
        doc: &ftd::interpreter::TDoc,
        component_definition_name: &Option<String>,
        loop_alias: &Option<String>,
        device: &Option<fastn_js::DeviceType>,
        component_name: &str,
    ) -> Option<fastn_js::StaticVariable> {
        let mut inherited_fields = vec![];

        if let Some(ref colors) = self.colors {
            inherited_fields.push((
                "colors".to_string(),
                colors.to_set_property_value(
                    doc,
                    component_definition_name,
                    loop_alias,
                    fastn_js::INHERITED_VARIABLE,
                    device,
                ),
            ));
        }

        if let Some(ref types) = self.types {
            inherited_fields.push((
                "types".to_string(),
                types.to_set_property_value(
                    doc,
                    component_definition_name,
                    loop_alias,
                    fastn_js::INHERITED_VARIABLE,
                    device,
                ),
            ));
        }

        if !inherited_fields.is_empty() {
            Some(fastn_js::StaticVariable {
                name: format!("{}{}", fastn_js::INHERITED_PREFIX, component_name),
                value: fastn_js::SetPropertyValue::Value(fastn_js::Value::Record {
                    fields: inherited_fields,
                }),
                prefix: None,
            })
        } else {
            None
        }
    }
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
            text_common: TextCommon::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
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
        inherited_variable_name: &str,
        device: &Option<fastn_js::DeviceType>,
        should_return: bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        let kernel = fastn_js::Kernel::from_component(fastn_js::ElementKind::Text, parent, index);
        component_statements.push(fastn_js::ComponentStatement::CreateKernel(kernel.clone()));
        component_statements.push(fastn_js::ComponentStatement::SetProperty(
            fastn_js::SetProperty {
                kind: fastn_js::PropertyKind::StringValue,
                value: self.text.to_set_property_value(
                    doc,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
                element_name: kernel.name.to_string(),
                inherited: inherited_variable_name.to_string(),
            },
        ));
        component_statements.extend(self.common.to_set_properties(
            kernel.name.as_str(),
            doc,
            component_definition_name,
            inherited_variable_name,
            loop_alias,
            device,
        ));
        component_statements.extend(self.text_common.to_set_properties(
            kernel.name.as_str(),
            doc,
            component_definition_name,
            inherited_variable_name,
            loop_alias,
            device,
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
            text_common: TextCommon::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
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
        inherited_variable_name: &str,
        device: &Option<fastn_js::DeviceType>,
        should_return: bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        let kernel =
            fastn_js::Kernel::from_component(fastn_js::ElementKind::Integer, parent, index);
        component_statements.push(fastn_js::ComponentStatement::CreateKernel(kernel.clone()));
        component_statements.push(fastn_js::ComponentStatement::SetProperty(
            fastn_js::SetProperty {
                kind: fastn_js::PropertyKind::StringValue,
                value: self.value.to_set_property_value(
                    doc,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
                element_name: kernel.name.to_string(),
                inherited: inherited_variable_name.to_string(),
            },
        ));
        component_statements.extend(self.common.to_set_properties(
            kernel.name.as_str(),
            doc,
            component_definition_name,
            inherited_variable_name,
            loop_alias,
            device,
        ));
        component_statements.extend(self.text_common.to_set_properties(
            kernel.name.as_str(),
            doc,
            component_definition_name,
            inherited_variable_name,
            loop_alias,
            device,
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
            text_common: TextCommon::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
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
        inherited_variable_name: &str,
        device: &Option<fastn_js::DeviceType>,
        should_return: bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        let kernel =
            fastn_js::Kernel::from_component(fastn_js::ElementKind::Decimal, parent, index);
        component_statements.push(fastn_js::ComponentStatement::CreateKernel(kernel.clone()));
        component_statements.push(fastn_js::ComponentStatement::SetProperty(
            fastn_js::SetProperty {
                kind: fastn_js::PropertyKind::StringValue,
                value: self.value.to_set_property_value(
                    doc,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
                element_name: kernel.name.to_string(),
                inherited: inherited_variable_name.to_string(),
            },
        ));
        component_statements.extend(self.common.to_set_properties(
            kernel.name.as_str(),
            doc,
            component_definition_name,
            inherited_variable_name,
            loop_alias,
            device,
        ));
        component_statements.extend(self.text_common.to_set_properties(
            kernel.name.as_str(),
            doc,
            component_definition_name,
            inherited_variable_name,
            loop_alias,
            device,
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
            text_common: TextCommon::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
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
        inherited_variable_name: &str,
        device: &Option<fastn_js::DeviceType>,
        should_return: bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        let kernel =
            fastn_js::Kernel::from_component(fastn_js::ElementKind::Boolean, parent, index);
        component_statements.push(fastn_js::ComponentStatement::CreateKernel(kernel.clone()));
        component_statements.push(fastn_js::ComponentStatement::SetProperty(
            fastn_js::SetProperty {
                kind: fastn_js::PropertyKind::StringValue,
                value: self.value.to_set_property_value(
                    doc,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
                element_name: kernel.name.to_string(),
                inherited: inherited_variable_name.to_string(),
            },
        ));
        component_statements.extend(self.common.to_set_properties(
            kernel.name.as_str(),
            doc,
            component_definition_name,
            inherited_variable_name,
            loop_alias,
            device,
        ));
        component_statements.extend(self.text_common.to_set_properties(
            kernel.name.as_str(),
            doc,
            component_definition_name,
            inherited_variable_name,
            loop_alias,
            device,
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
    pub fn from(component: &ftd::interpreter::Component) -> Column {
        let component_definition = ftd::interpreter::default::default_bag()
            .get("ftd#column")
            .unwrap()
            .clone()
            .component()
            .unwrap();

        Column {
            children: component.get_children_property(),
            inherited: InheritedProperties::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
            container: Container::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
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
        inherited_variable_name: &str,
        device: &Option<fastn_js::DeviceType>,
        should_return: bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        let kernel = fastn_js::Kernel::from_component(fastn_js::ElementKind::Column, parent, index);
        component_statements.push(fastn_js::ComponentStatement::CreateKernel(kernel.clone()));
        component_statements.extend(self.common.to_set_properties(
            kernel.name.as_str(),
            doc,
            component_definition_name,
            inherited_variable_name,
            loop_alias,
            device,
        ));

        component_statements.extend(self.container.to_set_properties(
            kernel.name.as_str(),
            doc,
            component_definition_name,
            loop_alias,
            inherited_variable_name,
            device,
        ));

        let inherited_variables = self.inherited.get_inherited_variables(
            doc,
            component_definition_name,
            loop_alias,
            device,
            kernel.name.as_str(),
        );

        let inherited_variable_name = inherited_variables
            .as_ref()
            .map(|v| v.name.clone())
            .unwrap_or_else(|| inherited_variable_name.to_string());

        if let Some(inherited_variables) = inherited_variables {
            component_statements.push(fastn_js::ComponentStatement::StaticVariable(
                inherited_variables,
            ));
        }

        component_statements.extend(self.children.iter().map(|v| {
            fastn_js::ComponentStatement::SetProperty(fastn_js::SetProperty {
                kind: fastn_js::PropertyKind::Children,
                value: v.value.to_fastn_js_value(
                    doc,
                    component_definition_name,
                    loop_alias,
                    &inherited_variable_name,
                    device,
                ),
                element_name: kernel.name.to_string(),
                inherited: inherited_variable_name.to_string(),
            })
        }));
        if should_return {
            component_statements.push(fastn_js::ComponentStatement::Return {
                component_name: kernel.name,
            });
        }
        component_statements
    }
}

impl Row {
    pub fn from(component: &ftd::interpreter::Component) -> Row {
        let component_definition = ftd::interpreter::default::default_bag()
            .get("ftd#row")
            .unwrap()
            .clone()
            .component()
            .unwrap();
        Row {
            children: component.get_children_property(),
            inherited: InheritedProperties::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
            container: Container::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
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
        inherited_variable_name: &str,
        device: &Option<fastn_js::DeviceType>,
        should_return: bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        let kernel = fastn_js::Kernel::from_component(fastn_js::ElementKind::Row, parent, index);
        component_statements.push(fastn_js::ComponentStatement::CreateKernel(kernel.clone()));

        component_statements.extend(self.common.to_set_properties(
            kernel.name.as_str(),
            doc,
            component_definition_name,
            inherited_variable_name,
            loop_alias,
            device,
        ));

        component_statements.extend(self.container.to_set_properties(
            kernel.name.as_str(),
            doc,
            component_definition_name,
            loop_alias,
            inherited_variable_name,
            device,
        ));

        let inherited_variables = self.inherited.get_inherited_variables(
            doc,
            component_definition_name,
            loop_alias,
            device,
            kernel.name.as_str(),
        );

        let inherited_variable_name = inherited_variables
            .as_ref()
            .map(|v| v.name.clone())
            .unwrap_or_else(|| inherited_variable_name.to_string());

        if let Some(inherited_variables) = inherited_variables {
            component_statements.push(fastn_js::ComponentStatement::StaticVariable(
                inherited_variables,
            ));
        }

        component_statements.extend(self.children.iter().map(|v| {
            fastn_js::ComponentStatement::SetProperty(fastn_js::SetProperty {
                kind: fastn_js::PropertyKind::Children,
                value: v.value.to_fastn_js_value(
                    doc,
                    component_definition_name,
                    loop_alias,
                    &inherited_variable_name,
                    device,
                ),
                element_name: kernel.name.to_string(),
                inherited: inherited_variable_name.to_string(),
            })
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
pub struct Device {
    pub children: Option<ftd::interpreter::Property>,
    pub container: InheritedProperties,
    pub device: fastn_js::DeviceType,
}

impl Device {
    pub fn from(component: &ftd::interpreter::Component, device: &str) -> Device {
        let component_definition = ftd::interpreter::default::default_bag()
            .get(device)
            .unwrap()
            .clone()
            .component()
            .unwrap();
        Device {
            children: component.get_children_property(),
            container: InheritedProperties::from(
                component.properties.as_slice(),
                component_definition.arguments.as_slice(),
            ),
            device: device.into(),
        }
    }

    pub fn to_component_statements(
        &self,
        parent: &str,
        index: usize,
        doc: &ftd::interpreter::TDoc,
        component_definition_name: &Option<String>,
        loop_alias: &Option<String>,
        inherited_variable_name: &str,
        device: &Option<fastn_js::DeviceType>,
        should_return: bool,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        if let Some(ref device) = device {
            if device.ne(&self.device) {
                return component_statements;
            }
        }

        let kernel = fastn_js::Kernel::from_component(fastn_js::ElementKind::Device, "root", index);
        component_statements.push(fastn_js::ComponentStatement::CreateKernel(kernel.clone()));

        let inherited_variables = self.container.get_inherited_variables(
            doc,
            component_definition_name,
            loop_alias,
            device,
            kernel.name.as_str(),
        );

        let inherited_variable_name = inherited_variables
            .as_ref()
            .map(|v| v.name.clone())
            .unwrap_or_else(|| inherited_variable_name.to_string());

        if let Some(inherited_variables) = inherited_variables {
            component_statements.push(fastn_js::ComponentStatement::StaticVariable(
                inherited_variables,
            ));
        }

        component_statements.extend(self.children.iter().map(|v| {
            fastn_js::ComponentStatement::SetProperty(fastn_js::SetProperty {
                kind: fastn_js::PropertyKind::Children,
                value: v.value.to_fastn_js_value(
                    doc,
                    component_definition_name,
                    loop_alias,
                    &inherited_variable_name,
                    device,
                ),
                element_name: kernel.name.to_string(),
                inherited: inherited_variable_name.to_string(),
            })
        }));
        component_statements.push(fastn_js::ComponentStatement::Return {
            component_name: kernel.name,
        });

        vec![fastn_js::ComponentStatement::DeviceBlock(
            fastn_js::DeviceBlock {
                device: self.device.to_owned(),
                statements: component_statements,
                parent: parent.to_string(),
                should_return,
            },
        )]
    }
}

#[derive(Debug)]
pub struct TextCommon {
    pub text_transform: Option<ftd::js::Value>,
    pub text_indent: Option<ftd::js::Value>,
    pub text_align: Option<ftd::js::Value>,
    pub line_clamp: Option<ftd::js::Value>,
    pub style: Option<ftd::js::Value>,
    pub display: Option<ftd::js::Value>,
}

impl TextCommon {
    pub fn from(
        properties: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
    ) -> TextCommon {
        TextCommon {
            text_transform: ftd::js::value::get_properties("text-transform", properties, arguments),
            text_indent: ftd::js::value::get_properties("text-indent", properties, arguments),
            text_align: ftd::js::value::get_properties("text-align", properties, arguments),
            line_clamp: ftd::js::value::get_properties("line-clamp", properties, arguments),
            style: ftd::js::value::get_properties("style", properties, arguments),
            display: ftd::js::value::get_properties("display", properties, arguments),
        }
    }

    pub fn to_set_properties(
        &self,
        element_name: &str,
        doc: &ftd::interpreter::TDoc,
        component_definition_name: &Option<String>,
        inherited_variable_name: &str,
        loop_alias: &Option<String>,
        device: &Option<fastn_js::DeviceType>,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        if let Some(ref transform) = self.text_transform {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                transform.to_set_property(
                    fastn_js::PropertyKind::TextTransform,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref indent) = self.text_indent {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                indent.to_set_property(
                    fastn_js::PropertyKind::TextIndent,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref align) = self.text_align {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                align.to_set_property(
                    fastn_js::PropertyKind::TextAlign,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref clamp) = self.line_clamp {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                clamp.to_set_property(
                    fastn_js::PropertyKind::LineClamp,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref style) = self.style {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                style.to_set_property(
                    fastn_js::PropertyKind::TextStyle,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref display) = self.display {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                display.to_set_property(
                    fastn_js::PropertyKind::Display,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        component_statements
    }
}

#[derive(Debug)]
pub struct Common {
    pub id: Option<ftd::js::Value>,
    pub region: Option<ftd::js::Value>,
    pub link: Option<ftd::js::Value>,
    pub open_in_new_tab: Option<ftd::js::Value>,
    pub align_self: Option<ftd::js::Value>,
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
    pub border_top_width: Option<ftd::js::Value>,
    pub border_bottom_width: Option<ftd::js::Value>,
    pub border_left_width: Option<ftd::js::Value>,
    pub border_right_width: Option<ftd::js::Value>,
    pub border_radius: Option<ftd::js::Value>,
    pub border_top_left_radius: Option<ftd::js::Value>,
    pub border_top_right_radius: Option<ftd::js::Value>,
    pub border_bottom_left_radius: Option<ftd::js::Value>,
    pub border_bottom_right_radius: Option<ftd::js::Value>,
    pub border_style: Option<ftd::js::Value>,
    pub border_style_vertical: Option<ftd::js::Value>,
    pub border_style_horizontal: Option<ftd::js::Value>,
    pub border_left_style: Option<ftd::js::Value>,
    pub border_right_style: Option<ftd::js::Value>,
    pub border_top_style: Option<ftd::js::Value>,
    pub border_bottom_style: Option<ftd::js::Value>,
    pub border_color: Option<ftd::js::Value>,
    pub border_left_color: Option<ftd::js::Value>,
    pub border_right_color: Option<ftd::js::Value>,
    pub border_top_color: Option<ftd::js::Value>,
    pub border_bottom_color: Option<ftd::js::Value>,
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
    pub opacity: Option<ftd::js::Value>,
    pub cursor: Option<ftd::js::Value>,
    pub resize: Option<ftd::js::Value>,
    pub max_height: Option<ftd::js::Value>,
    pub max_width: Option<ftd::js::Value>,
    pub min_height: Option<ftd::js::Value>,
    pub min_width: Option<ftd::js::Value>,
    pub whitespace: Option<ftd::js::Value>,
    pub classes: Option<ftd::js::Value>,
    pub anchor: Option<ftd::js::Value>,
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
            region: ftd::js::value::get_properties("region", properties, arguments),
            link: ftd::js::value::get_properties("link", properties, arguments),
            open_in_new_tab: ftd::js::value::get_properties(
                "open-in-new-tab",
                properties,
                arguments,
            ),
            anchor: ftd::js::value::get_properties("anchor", properties, arguments),
            classes: ftd::js::value::get_properties("classes", properties, arguments),
            align_self: ftd::js::value::get_properties("align-self", properties, arguments),
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
            border_top_width: ftd::js::value::get_properties(
                "border-top-width",
                properties,
                arguments,
            ),
            border_bottom_width: ftd::js::value::get_properties(
                "border-bottom-width",
                properties,
                arguments,
            ),
            border_left_width: ftd::js::value::get_properties(
                "border-left-width",
                properties,
                arguments,
            ),
            border_right_width: ftd::js::value::get_properties(
                "border-right-width",
                properties,
                arguments,
            ),
            border_radius: ftd::js::value::get_properties("border-radius", properties, arguments),
            border_top_left_radius: ftd::js::value::get_properties(
                "border-top-left-radius",
                properties,
                arguments,
            ),
            border_top_right_radius: ftd::js::value::get_properties(
                "border-top-right-radius",
                properties,
                arguments,
            ),
            border_bottom_left_radius: ftd::js::value::get_properties(
                "border-bottom-left-radius",
                properties,
                arguments,
            ),
            border_bottom_right_radius: ftd::js::value::get_properties(
                "border-bottom-right-radius",
                properties,
                arguments,
            ),
            border_style: ftd::js::value::get_properties("border-style", properties, arguments),
            border_style_vertical: ftd::js::value::get_properties(
                "border-style-vertical",
                properties,
                arguments,
            ),
            border_style_horizontal: ftd::js::value::get_properties(
                "border-style-horizontal",
                properties,
                arguments,
            ),
            border_left_style: ftd::js::value::get_properties(
                "border-style-left",
                properties,
                arguments,
            ),
            border_right_style: ftd::js::value::get_properties(
                "border-style-right",
                properties,
                arguments,
            ),
            border_top_style: ftd::js::value::get_properties(
                "border-style-top",
                properties,
                arguments,
            ),
            border_bottom_style: ftd::js::value::get_properties(
                "border-style-bottom",
                properties,
                arguments,
            ),
            border_color: ftd::js::value::get_properties("border-color", properties, arguments),
            border_left_color: ftd::js::value::get_properties(
                "border-left-color",
                properties,
                arguments,
            ),
            border_right_color: ftd::js::value::get_properties(
                "border-right-color",
                properties,
                arguments,
            ),
            border_top_color: ftd::js::value::get_properties(
                "border-top-color",
                properties,
                arguments,
            ),
            border_bottom_color: ftd::js::value::get_properties(
                "border-bottom-color",
                properties,
                arguments,
            ),
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
            opacity: ftd::js::value::get_properties("opacity", properties, arguments),
            cursor: ftd::js::value::get_properties("cursor", properties, arguments),
            resize: ftd::js::value::get_properties("resize", properties, arguments),
            max_height: ftd::js::value::get_properties("max-height", properties, arguments),
            max_width: ftd::js::value::get_properties("max-width", properties, arguments),
            min_height: ftd::js::value::get_properties("min-height", properties, arguments),
            min_width: ftd::js::value::get_properties("min-width", properties, arguments),
            whitespace: ftd::js::value::get_properties("white-space", properties, arguments),
            events: events.to_vec(),
        }
    }

    pub fn to_set_properties(
        &self,
        element_name: &str,
        doc: &ftd::interpreter::TDoc,
        component_definition_name: &Option<String>,
        inherited_variable_name: &str,
        loop_alias: &Option<String>,
        device: &Option<fastn_js::DeviceType>,
    ) -> Vec<fastn_js::ComponentStatement> {
        let mut component_statements = vec![];
        for event in self.events.iter() {
            component_statements.push(fastn_js::ComponentStatement::AddEventHandler(
                event.to_event_handler_js(
                    element_name,
                    doc,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref id) = self.id {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                id.to_set_property(
                    fastn_js::PropertyKind::Id,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref region) = self.region {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                region.to_set_property(
                    fastn_js::PropertyKind::Region,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref link) = self.link {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                link.to_set_property(
                    fastn_js::PropertyKind::Link,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref open_in_new_tab) = self.open_in_new_tab {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                open_in_new_tab.to_set_property(
                    fastn_js::PropertyKind::OpenInNewTab,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref align_self) = self.align_self {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                align_self.to_set_property(
                    fastn_js::PropertyKind::AlignSelf,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref classes) = self.classes {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                classes.to_set_property(
                    fastn_js::PropertyKind::Classes,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref anchor) = self.anchor {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                anchor.to_set_property(
                    fastn_js::PropertyKind::Anchor,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref width) = self.width {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                width.to_set_property(
                    fastn_js::PropertyKind::Width,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref height) = self.height {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                height.to_set_property(
                    fastn_js::PropertyKind::Height,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref padding) = self.padding {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                padding.to_set_property(
                    fastn_js::PropertyKind::Padding,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref padding_horizontal) = self.padding_horizontal {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                padding_horizontal.to_set_property(
                    fastn_js::PropertyKind::PaddingHorizontal,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref padding_vertical) = self.padding_vertical {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                padding_vertical.to_set_property(
                    fastn_js::PropertyKind::PaddingVertical,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref padding_left) = self.padding_left {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                padding_left.to_set_property(
                    fastn_js::PropertyKind::PaddingLeft,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref padding_right) = self.padding_right {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                padding_right.to_set_property(
                    fastn_js::PropertyKind::PaddingRight,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref padding_top) = self.padding_top {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                padding_top.to_set_property(
                    fastn_js::PropertyKind::PaddingTop,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref padding_bottom) = self.padding_bottom {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                padding_bottom.to_set_property(
                    fastn_js::PropertyKind::PaddingBottom,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref margin) = self.margin {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                margin.to_set_property(
                    fastn_js::PropertyKind::Margin,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref margin_horizontal) = self.margin_horizontal {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                margin_horizontal.to_set_property(
                    fastn_js::PropertyKind::MarginHorizontal,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref margin_vertical) = self.margin_vertical {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                margin_vertical.to_set_property(
                    fastn_js::PropertyKind::MarginVertical,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref margin_left) = self.margin_left {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                margin_left.to_set_property(
                    fastn_js::PropertyKind::MarginLeft,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref margin_right) = self.margin_right {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                margin_right.to_set_property(
                    fastn_js::PropertyKind::MarginRight,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref margin_top) = self.margin_top {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                margin_top.to_set_property(
                    fastn_js::PropertyKind::MarginTop,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref margin_bottom) = self.margin_bottom {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                margin_bottom.to_set_property(
                    fastn_js::PropertyKind::MarginBottom,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref border_width) = self.border_width {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_width.to_set_property(
                    fastn_js::PropertyKind::BorderWidth,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref border_top_width) = self.border_top_width {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_top_width.to_set_property(
                    fastn_js::PropertyKind::BorderTopWidth,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref border_bottom_width) = self.border_bottom_width {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_bottom_width.to_set_property(
                    fastn_js::PropertyKind::BorderBottomWidth,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref border_left_width) = self.border_left_width {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_left_width.to_set_property(
                    fastn_js::PropertyKind::BorderLeftWidth,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref border_right_width) = self.border_right_width {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_right_width.to_set_property(
                    fastn_js::PropertyKind::BorderRightWidth,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref border_radius) = self.border_radius {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_radius.to_set_property(
                    fastn_js::PropertyKind::BorderRadius,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref border_top_left_radius) = self.border_top_left_radius {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_top_left_radius.to_set_property(
                    fastn_js::PropertyKind::BorderTopLeftRadius,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref border_top_right_radius) = self.border_top_right_radius {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_top_right_radius.to_set_property(
                    fastn_js::PropertyKind::BorderTopRightRadius,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref border_bottom_left_radius) = self.border_bottom_left_radius {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_bottom_left_radius.to_set_property(
                    fastn_js::PropertyKind::BorderBottomLeftRadius,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref border_bottom_right_radius) = self.border_bottom_right_radius {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_bottom_right_radius.to_set_property(
                    fastn_js::PropertyKind::BorderBottomRightRadius,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref border_style) = self.border_style {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_style.to_set_property(
                    fastn_js::PropertyKind::BorderStyle,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref border_style_vertical) = self.border_style_vertical {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_style_vertical.to_set_property(
                    fastn_js::PropertyKind::BorderStyleVertical,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref border_style_horizontal) = self.border_style_horizontal {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_style_horizontal.to_set_property(
                    fastn_js::PropertyKind::BorderStyleHorizontal,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref border_left_style) = self.border_left_style {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_left_style.to_set_property(
                    fastn_js::PropertyKind::BorderLeftStyle,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref border_right_style) = self.border_right_style {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_right_style.to_set_property(
                    fastn_js::PropertyKind::BorderRightStyle,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref border_top_style) = self.border_top_style {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_top_style.to_set_property(
                    fastn_js::PropertyKind::BorderTopStyle,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref border_bottom_style) = self.border_bottom_style {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_bottom_style.to_set_property(
                    fastn_js::PropertyKind::BorderBottomStyle,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref border_color) = self.border_color {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_color.to_set_property(
                    fastn_js::PropertyKind::BorderColor,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref border_top_color) = self.border_top_color {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_top_color.to_set_property(
                    fastn_js::PropertyKind::BorderTopColor,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref border_bottom_color) = self.border_bottom_color {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_bottom_color.to_set_property(
                    fastn_js::PropertyKind::BorderBottomColor,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref border_left_color) = self.border_left_color {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_left_color.to_set_property(
                    fastn_js::PropertyKind::BorderLeftColor,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref border_right_color) = self.border_right_color {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                border_right_color.to_set_property(
                    fastn_js::PropertyKind::BorderRightColor,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref overflow) = self.overflow {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                overflow.to_set_property(
                    fastn_js::PropertyKind::Overflow,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref overflow_x) = self.overflow_x {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                overflow_x.to_set_property(
                    fastn_js::PropertyKind::OverflowX,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref overflow_y) = self.overflow_y {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                overflow_y.to_set_property(
                    fastn_js::PropertyKind::OverflowY,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref top) = self.top {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                top.to_set_property(
                    fastn_js::PropertyKind::Top,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref bottom) = self.bottom {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                bottom.to_set_property(
                    fastn_js::PropertyKind::Bottom,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref left) = self.left {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                left.to_set_property(
                    fastn_js::PropertyKind::Left,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref right) = self.right {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                right.to_set_property(
                    fastn_js::PropertyKind::Right,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref z_index) = self.z_index {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                z_index.to_set_property(
                    fastn_js::PropertyKind::ZIndex,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref sticky) = self.sticky {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                sticky.to_set_property(
                    fastn_js::PropertyKind::Sticky,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref color) = self.color {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                color.to_set_property(
                    fastn_js::PropertyKind::Color,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref background) = self.background {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                background.to_set_property(
                    fastn_js::PropertyKind::Background,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref background) = self.role {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                background.to_set_property(
                    fastn_js::PropertyKind::Role,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref opacity) = self.opacity {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                opacity.to_set_property(
                    fastn_js::PropertyKind::Opacity,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref cursor) = self.cursor {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                cursor.to_set_property(
                    fastn_js::PropertyKind::Cursor,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref resize) = self.resize {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                resize.to_set_property(
                    fastn_js::PropertyKind::Resize,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref max_height) = self.max_height {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                max_height.to_set_property(
                    fastn_js::PropertyKind::MaxHeight,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref min_height) = self.min_height {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                min_height.to_set_property(
                    fastn_js::PropertyKind::MinHeight,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref max_width) = self.max_width {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                max_width.to_set_property(
                    fastn_js::PropertyKind::MaxWidth,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref min_width) = self.min_width {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                min_width.to_set_property(
                    fastn_js::PropertyKind::MinWidth,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ),
            ));
        }
        if let Some(ref whitespace) = self.whitespace {
            component_statements.push(fastn_js::ComponentStatement::SetProperty(
                whitespace.to_set_property(
                    fastn_js::PropertyKind::WhiteSpace,
                    doc,
                    element_name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
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
        inherited_variable_name: &str,
        device: &Option<fastn_js::DeviceType>,
    ) -> fastn_js::EventHandler {
        fastn_js::EventHandler {
            event: self.name.to_js_event_name(),
            action: self.action.to_js_function(
                doc,
                component_definition_name,
                loop_alias,
                inherited_variable_name,
                device,
            ),
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
        inherited_variable_name: &str,
        device: &Option<fastn_js::DeviceType>,
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
            parameters.push(value.to_set_property_value(
                doc,
                component_definition_name,
                loop_alias,
                inherited_variable_name,
                device,
            ));
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
        "ftd#desktop",
        "ftd#mobile",
    ]
    .contains(&s)
}
