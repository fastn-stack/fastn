#[derive(Debug)]
pub enum Value {
    Data(ftd::interpreter::Value),
    Reference(String),
    Formula(Vec<ftd::interpreter::Property>),
}

impl Value {
    pub(crate) fn to_set_property_value_with_none(
        &self,
        doc: &ftd::interpreter::TDoc,
    ) -> fastn_js::SetPropertyValue {
        self.to_set_property_value(doc, &None, &None, fastn_js::INHERITED_VARIABLE, &None)
    }

    pub(crate) fn to_set_property_value(
        &self,
        doc: &ftd::interpreter::TDoc,
        component_definition_name: &Option<String>,
        loop_alias: &Option<String>,
        inherited_variable_name: &str,
        device: &Option<fastn_js::DeviceType>,
    ) -> fastn_js::SetPropertyValue {
        match self {
            Value::Data(value) => value.to_fastn_js_value(
                doc,
                component_definition_name,
                loop_alias,
                inherited_variable_name,
                device,
            ),
            Value::Reference(name) => {
                fastn_js::SetPropertyValue::Reference(ftd::js::utils::update_reference(
                    name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                ))
            }
            Value::Formula(formulas) => {
                fastn_js::SetPropertyValue::Formula(formulas_to_fastn_js_value(
                    doc,
                    formulas,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                    device,
                ))
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn to_set_property(
        &self,
        kind: fastn_js::PropertyKind,
        doc: &ftd::interpreter::TDoc,
        element_name: &str,
        component_definition_name: &Option<String>,
        loop_alias: &Option<String>,
        inherited_variable_name: &str,
        device: &Option<fastn_js::DeviceType>,
    ) -> fastn_js::SetProperty {
        fastn_js::SetProperty {
            kind,
            value: self.to_set_property_value(
                doc,
                component_definition_name,
                loop_alias,
                inherited_variable_name,
                device,
            ),
            element_name: element_name.to_string(),
            inherited: inherited_variable_name.to_string(),
        }
    }
}

fn formulas_to_fastn_js_value(
    doc: &ftd::interpreter::TDoc,
    properties: &[ftd::interpreter::Property],
    component_definition_name: &Option<String>,
    loop_alias: &Option<String>,
    inherited_variable_name: &str,
    device: &Option<fastn_js::DeviceType>,
) -> fastn_js::Formula {
    let mut deps = vec![];
    let mut conditional_values = vec![];
    for property in properties {
        deps.extend(property.value.get_deps(
            component_definition_name,
            loop_alias,
            inherited_variable_name,
        ));
        if let Some(ref condition) = property.condition {
            deps.extend(condition.get_deps(
                component_definition_name,
                loop_alias,
                inherited_variable_name,
            ));
        }

        conditional_values.push(fastn_js::ConditionalValue {
            condition: property.condition.as_ref().map(|condition| {
                condition.update_node_with_variable_reference_js(
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                )
            }),
            expression: property.value.to_fastn_js_value(
                doc,
                component_definition_name,
                loop_alias,
                inherited_variable_name,
                device,
            ),
        });
    }

    fastn_js::Formula {
        deps,
        conditional_values,
    }
}

impl ftd::interpreter::Expression {
    pub(crate) fn get_deps(
        &self,
        component_definition_name: &Option<String>,
        loop_alias: &Option<String>,
        inherited_variable_name: &str,
    ) -> Vec<String> {
        let mut deps = vec![];
        for property_value in self.references.values() {
            deps.extend(property_value.get_deps(
                component_definition_name,
                loop_alias,
                inherited_variable_name,
            ));
        }
        deps
    }

    pub fn update_node_with_variable_reference_js(
        &self,
        component_definition_name: &Option<String>,
        loop_alias: &Option<String>,
        inherited_variable_name: &str,
    ) -> fastn_grammar::evalexpr::ExprNode {
        return update_node_with_variable_reference_js_(
            &self.expression,
            &self.references,
            component_definition_name,
            loop_alias,
            inherited_variable_name,
        );

        fn update_node_with_variable_reference_js_(
            expr: &fastn_grammar::evalexpr::ExprNode,
            references: &ftd::Map<ftd::interpreter::PropertyValue>,
            component_definition_name: &Option<String>,
            loop_alias: &Option<String>,
            inherited_variable_name: &str,
        ) -> fastn_grammar::evalexpr::ExprNode {
            let mut operator = expr.operator().clone();
            if let fastn_grammar::evalexpr::Operator::VariableIdentifierRead { ref identifier } =
                operator
            {
                if format!("${}", ftd::interpreter::FTD_LOOP_COUNTER).eq(identifier) {
                    operator = fastn_grammar::evalexpr::Operator::VariableIdentifierRead {
                        identifier: "index".to_string(),
                    }
                } else if let Some(ftd::interpreter::PropertyValue::Reference { name, .. }) =
                    references.get(identifier)
                {
                    let name = ftd::js::utils::update_reference(
                        name,
                        component_definition_name,
                        loop_alias,
                        inherited_variable_name,
                    );
                    operator = fastn_grammar::evalexpr::Operator::VariableIdentifierRead {
                        identifier: fastn_js::utils::reference_to_js(name.as_str()),
                    }
                }
            }
            let mut children = vec![];
            for child in expr.children() {
                children.push(update_node_with_variable_reference_js_(
                    child,
                    references,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                ));
            }
            fastn_grammar::evalexpr::ExprNode::new(operator).add_children(children)
        }
    }
}

impl ftd::interpreter::PropertyValue {
    pub(crate) fn get_deps(
        &self,
        component_definition_name: &Option<String>,
        loop_alias: &Option<String>,
        inherited_variable_name: &str,
    ) -> Vec<String> {
        let mut deps = vec![];
        if let Some(reference) = self.get_reference_or_clone() {
            deps.push(ftd::js::utils::update_reference(
                reference,
                component_definition_name,
                loop_alias,
                inherited_variable_name,
            ));
        } else if let Some(function) = self.get_function() {
            for value in function.values.values() {
                deps.extend(value.get_deps(
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                ));
            }
        }
        deps
    }
}

impl ftd::interpreter::Argument {
    pub(crate) fn get_default_value(&self) -> Option<Value> {
        if let Some(ref value) = self.value {
            Some(value.to_value())
        } else if self.kind.is_list() {
            Some(Value::Data(ftd::interpreter::Value::List {
                data: vec![],
                kind: self.kind.clone(),
            }))
        } else if self.kind.is_optional() {
            Some(Value::Data(ftd::interpreter::Value::Optional {
                data: Box::new(None),
                kind: self.kind.clone(),
            }))
        } else {
            None
        }
    }
    pub(crate) fn get_value(&self, properties: &[ftd::interpreter::Property]) -> Value {
        if let Some(value) = self.get_optional_value(properties) {
            value
        } else if let Some(value) = self.get_default_value() {
            value
        } else {
            panic!("{}", format!("Expected value for argument: {:?}", &self))
        }
    }

    pub(crate) fn get_optional_value(
        &self,
        properties: &[ftd::interpreter::Property],
        // doc_name: &str,
        // line_number: usize
    ) -> Option<Value> {
        let sources = self.to_sources();
        let properties = ftd::interpreter::utils::find_properties_by_source(
            sources.as_slice(),
            properties,
            "", // doc_name
            self,
            0, // line_number
        )
        .unwrap();

        if properties.is_empty() {
            return None;
        }

        if properties.len() == 1 {
            let property = properties.first().unwrap();
            if property.condition.is_none() {
                return Some(property.value.to_value());
            }
        }

        Some(Value::Formula(properties))
    }
}

pub(crate) fn get_properties(
    key: &str,
    properties: &[ftd::interpreter::Property],
    arguments: &[ftd::interpreter::Argument],
) -> Option<Value> {
    arguments
        .iter()
        .find(|v| v.name.eq(key))
        .unwrap()
        .get_optional_value(properties)
}

impl ftd::interpreter::PropertyValue {
    pub(crate) fn to_fastn_js_value_with_none(
        &self,
        doc: &ftd::interpreter::TDoc,
    ) -> fastn_js::SetPropertyValue {
        self.to_fastn_js_value(doc, &None, &None, fastn_js::INHERITED_VARIABLE, &None)
    }

    pub(crate) fn to_fastn_js_value(
        &self,
        doc: &ftd::interpreter::TDoc,
        component_definition_name: &Option<String>,
        loop_alias: &Option<String>,
        inherited_variable_name: &str,
        device: &Option<fastn_js::DeviceType>,
    ) -> fastn_js::SetPropertyValue {
        match self {
            ftd::interpreter::PropertyValue::Value { ref value, .. } => value.to_fastn_js_value(
                doc,
                component_definition_name,
                loop_alias,
                inherited_variable_name,
                device,
            ),
            ftd::interpreter::PropertyValue::Reference { ref name, .. } => {
                fastn_js::SetPropertyValue::Reference(ftd::js::utils::update_reference(
                    name,
                    component_definition_name,
                    loop_alias,
                    inherited_variable_name,
                ))
            }
            _ => todo!(),
        }
    }

    pub(crate) fn to_value(&self) -> Value {
        match self {
            ftd::interpreter::PropertyValue::Value { ref value, .. } => {
                Value::Data(value.to_owned())
            }
            ftd::interpreter::PropertyValue::Reference { ref name, .. } => {
                Value::Reference(name.to_owned())
            }
            _ => todo!(),
        }
    }
}

impl ftd::interpreter::Value {
    pub(crate) fn to_fastn_js_value(
        &self,
        doc: &ftd::interpreter::TDoc,
        component_definition_name: &Option<String>,
        loop_alias: &Option<String>,
        inherited_variable_name: &str,
        device: &Option<fastn_js::DeviceType>,
    ) -> fastn_js::SetPropertyValue {
        use itertools::Itertools;

        match self {
            ftd::interpreter::Value::Boolean { value } => {
                fastn_js::SetPropertyValue::Value(fastn_js::Value::Boolean(*value))
            }
            ftd::interpreter::Value::String { text } => {
                fastn_js::SetPropertyValue::Value(fastn_js::Value::String(text.to_string()))
            }
            ftd::interpreter::Value::Integer { value } => {
                fastn_js::SetPropertyValue::Value(fastn_js::Value::Integer(*value))
            }
            ftd::interpreter::Value::Decimal { value } => {
                fastn_js::SetPropertyValue::Value(fastn_js::Value::Decimal(*value))
            }
            ftd::interpreter::Value::OrType {
                name,
                variant,
                value,
                ..
            } => {
                let (js_variant, has_value) = ftd_to_js_variant(name, variant);
                if has_value {
                    return fastn_js::SetPropertyValue::Value(fastn_js::Value::OrType {
                        variant: js_variant,
                        value: Some(Box::new(value.to_fastn_js_value(
                            doc,
                            component_definition_name,
                            loop_alias,
                            inherited_variable_name,
                            device,
                        ))),
                    });
                }
                fastn_js::SetPropertyValue::Value(fastn_js::Value::OrType {
                    variant: js_variant,
                    value: None,
                })
            }
            ftd::interpreter::Value::List { data, .. } => {
                fastn_js::SetPropertyValue::Value(fastn_js::Value::List {
                    value: data
                        .iter()
                        .map(|v| {
                            v.to_fastn_js_value(
                                doc,
                                component_definition_name,
                                loop_alias,
                                inherited_variable_name,
                                device,
                            )
                        })
                        .collect_vec(),
                })
            }
            ftd::interpreter::Value::Record { fields, .. } => {
                fastn_js::SetPropertyValue::Value(fastn_js::Value::Record {
                    fields: fields
                        .iter()
                        .map(|(k, v)| {
                            (
                                k.to_string(),
                                v.to_fastn_js_value(
                                    doc,
                                    component_definition_name,
                                    loop_alias,
                                    inherited_variable_name,
                                    device,
                                ),
                            )
                        })
                        .collect_vec(),
                })
            }
            ftd::interpreter::Value::UI { component, .. } => {
                fastn_js::SetPropertyValue::Value(fastn_js::Value::UI {
                    value: component.to_component_statements_(
                        fastn_js::FUNCTION_PARENT,
                        0,
                        doc,
                        component_definition_name,
                        loop_alias,
                        fastn_js::INHERITED_VARIABLE,
                        device,
                        false,
                    ),
                })
            }
            t => todo!("{:?}", t),
        }
    }
}

fn ftd_to_js_variant(name: &str, variant: &str) -> (String, bool) {
    // returns (JSVariant, has_value)
    let variant = variant.strip_prefix(format!("{}.", name).as_str()).unwrap();
    match name {
        "ftd#resizing" => {
            let js_variant = resizing_variants(variant);
            (format!("fastn_dom.Resizing.{}", js_variant.0), js_variant.1)
        }
        "ftd#length" => {
            let js_variant = length_variants(variant);
            (format!("fastn_dom.Length.{}", js_variant), true)
        }
        "ftd#border-style" => {
            let js_variant = border_style_variants(variant);
            (format!("fastn_dom.BorderStyle.{}", js_variant), false)
        }
        "ftd#background" => {
            let js_variant = background_variants(variant);
            (format!("fastn_dom.BackgroundStyle.{}", js_variant), true)
        }
        "ftd#font-size" => {
            let js_variant = font_size_variants(variant);
            (format!("fastn_dom.FontSize.{}", js_variant), true)
        }
        "ftd#overflow" => {
            let js_variant = overflow_variants(variant);
            (format!("fastn_dom.Overflow.{}", js_variant), false)
        }
        "ftd#display" => {
            let js_variant = display_variants(variant);
            (format!("fastn_dom.Display.{}", js_variant), false)
        }
        "ftd#spacing" => {
            let js_variant = spacing_variants(variant);
            (format!("fastn_dom.Spacing.{}", js_variant.0), js_variant.1)
        }
        "ftd#text-transform" => {
            let js_variant = text_transform_variants(variant);
            (format!("fastn_dom.TextTransform.{}", js_variant), false)
        }
        "ftd#text-align" => {
            let js_variant = text_align_variants(variant);
            (format!("fastn_dom.TextAlign.{}", js_variant), false)
        }
        "ftd#cursor" => {
            let js_variant = cursor_variants(variant);
            (format!("fastn_dom.Cursor.{}", js_variant), false)
        }
        "ftd#resize" => {
            let js_variant = resize_variants(variant);
            (format!("fastn_dom.Resize.{}", js_variant), false)
        }
        "ftd#white-space" => {
            let js_variant = whitespace_variants(variant);
            (format!("fastn_dom.WhiteSpace.{}", js_variant), false)
        }
        "ftd#align-self" => {
            let js_variant = align_self_variants(variant);
            (format!("fastn_dom.AlignSelf.{}", js_variant), false)
        }
        "ftd#anchor" => {
            let js_variant = anchor_variants(variant);
            (format!("fastn_dom.Anchor.{}", js_variant), false)
        }
        "ftd#text-style" => {
            let js_variant = text_style_variants(variant);
            (format!("fastn_dom.TextStyle.{}", js_variant), false)
        }
        "ftd#region" => {
            let js_variant = region_variants(variant);
            (format!("fastn_dom.Region.{}", js_variant), false)
        }
        "ftd#align" => {
            let js_variant = align_variants(variant);
            (format!("fastn_dom.AlignContent.{}", js_variant), false)
        }
        "ftd#text-input-type" => {
            let js_variant = text_input_type_variants(variant);
            (format!("fastn_dom.TextInputType.{}", js_variant), false)
        }
        "ftd#loading" => {
            let js_variant = loading_variants(variant);
            (format!("fastn_dom.Loading.{}", js_variant), false)
        }
        t => todo!("{} {}", t, variant),
    }
}

// Returns the corresponding js string and has_value
// Todo: Remove has_value flag
fn resizing_variants(name: &str) -> (&'static str, bool) {
    match name {
        "fixed" => ("Fixed", true),
        "fill-container" => ("FillContainer", false),
        "hug-content" => ("HugContent", false),
        t => todo!("invalid resizing variant {}", t),
    }
}

fn length_variants(name: &str) -> &'static str {
    match name {
        "px" => "Px",
        "em" => "Em",
        "rem" => "Rem",
        "percent" => "Percent",
        "vh" => "Vh",
        "vw" => "Vw",
        "vmin" => "Vmin",
        "vmax" => "Vmax",
        "calc" => "Calc",
        t => todo!("invalid length variant {}", t),
    }
}

fn border_style_variants(name: &str) -> &'static str {
    match name {
        "solid" => "Solid",
        "dashed" => "Dashed",
        "dotted" => "Dotted",
        "groove" => "Groove",
        "inset" => "Inset",
        "outset" => "Outset",
        "ridge" => "Ridge",
        "double" => "Double",
        t => todo!("invalid border-style variant {}", t),
    }
}

fn background_variants(name: &str) -> &'static str {
    match name {
        "solid" => "Solid",
        t => todo!("invalid background variant {}", t),
    }
}

fn font_size_variants(name: &str) -> &'static str {
    match name {
        "px" => "Px",
        "em" => "Em",
        "rem" => "Rem",
        t => todo!("invalid font-size variant {}", t),
    }
}

fn overflow_variants(name: &str) -> &'static str {
    match name {
        "scroll" => "Scroll",
        "visible" => "Visible",
        "hidden" => "Hidden",
        "auto" => "Auto",
        t => todo!("invalid overflow variant {}", t),
    }
}

fn display_variants(name: &str) -> &'static str {
    match name {
        "block" => "Block",
        "inline" => "Inline",
        "inline-block" => "InlineBlock",
        t => todo!("invalid display variant {}", t),
    }
}

fn spacing_variants(name: &str) -> (&'static str, bool) {
    match name {
        "space-evenly" => ("SpaceEvenly", false),
        "space-between" => ("SpaceBetween", false),
        "space-around" => ("SpaceAround", false),
        "fixed" => ("Fixed", true),
        t => todo!("invalid spacing variant {}", t),
    }
}

fn text_transform_variants(name: &str) -> &'static str {
    match name {
        "none" => "None",
        "capitalize" => "Capitalize",
        "uppercase" => "Uppercase",
        "lowercase" => "Lowercase",
        "inherit" => "Inherit",
        "initial" => "Initial",
        t => todo!("invalid text-transform variant {}", t),
    }
}

fn text_align_variants(name: &str) -> &'static str {
    match name {
        "start" => "Start",
        "center" => "Center",
        "end" => "End",
        "justify" => "Justify",
        t => todo!("invalid text-align variant {}", t),
    }
}

fn cursor_variants(name: &str) -> &'static str {
    match name {
        "none" => "None",
        "default" => "Default",
        "context-menu" => "ContextMenu",
        "help" => "Help",
        "pointer" => "Pointer",
        "progress" => "Progress",
        "wait" => "Wait",
        "cell" => "Cell",
        "crosshair" => "CrossHair",
        "text" => "Text",
        "vertical-text" => "VerticalText",
        "alias" => "Alias",
        "copy" => "Copy",
        "move" => "Move",
        "no-drop" => "NoDrop",
        "not-allowed" => "NotAllowed",
        "grab" => "Grab",
        "grabbing" => "Grabbing",
        "e-resize" => "EResize",
        "n-resize" => "NResize",
        "ne-resize" => "NeResize",
        "s-resize" => "SResize",
        "se-resize" => "SeResize",
        "sw-resize" => "SwResize",
        "w-resize" => "Wresize",
        "ew-resize" => "Ewresize",
        "ns-resize" => "NsResize",
        "nesw-resize" => "NeswResize",
        "nwse-resize" => "NwseResize",
        "col-resize" => "ColResize",
        "row-resize" => "RowResize",
        "all-scroll" => "AllScroll",
        "zoom-in" => "ZoomIn",
        "zoom-out" => "ZoomOut",
        t => todo!("invalid cursor variant {}", t),
    }
}

fn resize_variants(name: &str) -> &'static str {
    match name {
        "vertical" => "Vertical",
        "horizontal" => "Horizontal",
        "both" => "Both",
        t => todo!("invalid resize variant {}", t),
    }
}

fn whitespace_variants(name: &str) -> &'static str {
    match name {
        "normal" => "Normal",
        "nowrap" => "NoWrap",
        "pre" => "Pre",
        "pre-line" => "PreLine",
        "pre-wrap" => "PreWrap",
        "break-spaces" => "BreakSpaces",
        t => todo!("invalid resize variant {}", t),
    }
}

fn align_self_variants(name: &str) -> &'static str {
    match name {
        "start" => "Start",
        "center" => "Center",
        "end" => "End",
        t => todo!("invalid align-self variant {}", t),
    }
}

fn anchor_variants(name: &str) -> &'static str {
    match name {
        "window" => "Window",
        "parent" => "Parent",
        "id" => "Id",
        t => todo!("invalid anchor variant {}", t),
    }
}

fn text_style_variants(name: &str) -> &'static str {
    match name {
        "underline" => "Underline",
        "italic" => "Italic",
        "strike" => "Strike",
        "heavy" => "Heavy",
        "extra-bold" => "Extrabold",
        "bold" => "Bold",
        "semi-bold" => "SemiBold",
        "medium" => "Medium",
        "regular" => "Regular",
        "light" => "Light",
        "extra-light" => "ExtraLight",
        "hairline" => "Hairline",
        t => todo!("invalid text-style variant {}", t),
    }
}

fn region_variants(name: &str) -> &'static str {
    match name {
        "h1" => "H1",
        "h2" => "H2",
        "h3" => "H3",
        "h4" => "H4",
        "h5" => "H5",
        "h6" => "H6",
        t => todo!("invalid region variant {}", t),
    }
}

fn align_variants(name: &str) -> &'static str {
    match name {
        "top-left" => "TopLeft",
        "top-center" => "TopCenter",
        "top-right" => "TopRight",
        "right" => "Right",
        "left" => "Left",
        "center" => "Center",
        "bottom-left" => "BottomLeft",
        "bottom-right" => "BottomRight",
        "bottom-center" => "BottomCenter",
        t => todo!("invalid align-content variant {}", t),
    }
}

fn text_input_type_variants(name: &str) -> &'static str {
    match name {
        "text" => "Text",
        "email" => "Email",
        "password" => "Password",
        "url" => "Url",
        "datetime" => "DateTime",
        "date" => "Date",
        "time" => "Time",
        "month" => "Month",
        "week" => "Week",
        "color" => "Color",
        "file" => "File",
        t => todo!("invalid text-input-type variant {}", t),
    }
}

fn loading_variants(name: &str) -> &'static str {
    match name {
        "lazy" => "Lazy",
        "eager" => "Eager",
        t => todo!("invalid loading variant {}", t),
    }
}
