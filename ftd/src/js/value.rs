#[derive(Debug)]
pub enum Value {
    Data(fastn_type::Value),
    Reference(ReferenceData),
    ConditionalFormula(Vec<fastn_type::Property>),
    FunctionCall(fastn_type::FunctionCall),
    Clone(String),
}

#[derive(Debug)]
pub struct ReferenceData {
    pub name: String,
    pub value: Option<fastn_type::PropertyValue>,
}

impl Value {
    pub(crate) fn to_set_property_value_with_none(
        &self,
        doc: &ftd::interpreter::TDoc,
        has_rive_components: &mut bool,
    ) -> fastn_js::SetPropertyValue {
        self.to_set_property_value_with_ui(
            doc,
            &ftd::js::ResolverData::none(),
            has_rive_components,
            false,
        )
    }

    pub(crate) fn to_set_property_value(
        &self,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
    ) -> fastn_js::SetPropertyValue {
        self.to_set_property_value_with_ui(doc, rdata, &mut false, false)
    }

    pub(crate) fn to_set_property_value_with_ui(
        &self,
        doc: &ftd::interpreter::TDoc,
        rdata: &ftd::js::ResolverData,
        has_rive_components: &mut bool,
        should_return: bool,
    ) -> fastn_js::SetPropertyValue {
        use ftd::js::fastn_type_functions::ValueExt;

        match self {
            Value::Data(value) => {
                value.to_fastn_js_value(doc, rdata, has_rive_components, should_return)
            }
            Value::Reference(data) => {
                if let Some(value) = &data.value {
                    if let fastn_type::Kind::OrType {
                        name,
                        variant: Some(variant),
                        full_variant: Some(full_variant),
                    } = value.kind().inner()
                    {
                        let (js_variant, has_value) = ftd_to_js_variant(
                            name.as_str(),
                            variant.as_str(),
                            full_variant.as_str(),
                            value,
                            doc.name,
                            value.line_number(),
                        );

                        // return or-type value with reference
                        if has_value {
                            return fastn_js::SetPropertyValue::Value(fastn_js::Value::OrType {
                                variant: js_variant,
                                value: Some(Box::new(fastn_js::SetPropertyValue::Reference(
                                    ftd::js::utils::update_reference(data.name.as_str(), rdata),
                                ))),
                            });
                        }

                        // return or-type value
                        return fastn_js::SetPropertyValue::Value(fastn_js::Value::OrType {
                            variant: js_variant,
                            value: None,
                        });
                    }
                }

                // for other datatypes, simply return a reference
                fastn_js::SetPropertyValue::Reference(ftd::js::utils::update_reference(
                    data.name.as_str(),
                    rdata,
                ))
            }
            Value::ConditionalFormula(formulas) => fastn_js::SetPropertyValue::Formula(
                properties_to_js_conditional_formula(doc, formulas, rdata),
            ),
            Value::FunctionCall(function_call) => fastn_js::SetPropertyValue::Formula(
                ftd::js::utils::function_call_to_js_formula(function_call, doc, rdata),
            ),
            Value::Clone(name) => {
                fastn_js::SetPropertyValue::Clone(ftd::js::utils::update_reference(name, rdata))
            }
        }
    }

    pub(crate) fn to_set_property(
        &self,
        kind: fastn_js::PropertyKind,
        doc: &ftd::interpreter::TDoc,
        element_name: &str,
        rdata: &ftd::js::ResolverData,
    ) -> fastn_js::SetProperty {
        fastn_js::SetProperty {
            kind,
            value: self.to_set_property_value(doc, rdata),
            element_name: element_name.to_string(),
            inherited: rdata.inherited_variable_name.to_string(),
        }
    }

    pub fn from_str_value(s: &str) -> Value {
        Value::Data(fastn_type::Value::String {
            text: s.to_string(),
        })
    }

    pub fn get_string_data(&self) -> Option<String> {
        if let Value::Data(fastn_type::Value::String { text }) = self {
            return Some(text.to_string());
        }
        None
    }
}

fn properties_to_js_conditional_formula(
    doc: &ftd::interpreter::TDoc,
    properties: &[fastn_type::Property],
    rdata: &ftd::js::ResolverData,
) -> fastn_js::Formula {
    use ftd::js::fastn_type_functions::PropertyValueExt;

    let mut deps = vec![];
    let mut conditional_values = vec![];
    for property in properties {
        deps.extend(property.value.get_deps(rdata));
        if let Some(ref condition) = property.condition {
            deps.extend(condition.get_deps(rdata));
        }

        conditional_values.push(fastn_js::ConditionalValue {
            condition: property
                .condition
                .as_ref()
                .map(|condition| condition.update_node_with_variable_reference_js(rdata)),
            expression: property.value.to_fastn_js_value(doc, rdata, false),
        });
    }

    fastn_js::Formula {
        deps,
        type_: fastn_js::FormulaType::Conditional(conditional_values),
    }
}

impl ftd::interpreter::Expression {
    pub(crate) fn get_deps(&self, rdata: &ftd::js::ResolverData) -> Vec<String> {
        use ftd::js::fastn_type_functions::PropertyValueExt;

        let mut deps = vec![];
        for property_value in self.references.values() {
            deps.extend(property_value.get_deps(rdata));
        }
        deps
    }

    pub fn update_node_with_variable_reference_js(
        &self,
        rdata: &ftd::js::ResolverData,
    ) -> fastn_grammar::evalexpr::ExprNode {
        return update_node_with_variable_reference_js_(&self.expression, &self.references, rdata);

        fn update_node_with_variable_reference_js_(
            expr: &fastn_grammar::evalexpr::ExprNode,
            references: &ftd::Map<fastn_type::PropertyValue>,
            rdata: &ftd::js::ResolverData,
        ) -> fastn_grammar::evalexpr::ExprNode {
            let mut operator = expr.operator().clone();
            if let fastn_grammar::evalexpr::Operator::VariableIdentifierRead { ref identifier } =
                operator
            {
                if format!("${}", ftd::interpreter::FTD_LOOP_COUNTER).eq(identifier) {
                    operator = fastn_grammar::evalexpr::Operator::VariableIdentifierRead {
                        identifier: "index".to_string(),
                    }
                } else if let Some(loop_counter_alias) = rdata.loop_counter_alias {
                    if loop_counter_alias.eq(identifier.trim_start_matches('$')) {
                        operator = fastn_grammar::evalexpr::Operator::VariableIdentifierRead {
                            identifier: "index".to_string(),
                        }
                    }
                } else if let Some(fastn_type::PropertyValue::Reference { name, .. }) =
                    references.get(identifier)
                {
                    let name = ftd::js::utils::update_reference(name, rdata);
                    operator = fastn_grammar::evalexpr::Operator::VariableIdentifierRead {
                        identifier: fastn_js::utils::reference_to_js(name.as_str()),
                    }
                }
            }
            let mut children = vec![];
            for child in expr.children() {
                children.push(update_node_with_variable_reference_js_(
                    child, references, rdata,
                ));
            }
            fastn_grammar::evalexpr::ExprNode::new(operator).add_children(children)
        }
    }
}

impl ftd::interpreter::Argument {
    pub(crate) fn get_default_value(&self) -> Option<ftd::js::Value> {
        use ftd::js::fastn_type_functions::PropertyValueExt;

        if let Some(ref value) = self.value {
            Some(value.to_value())
        } else if self.kind.is_list() {
            Some(ftd::js::Value::Data(fastn_type::Value::List {
                data: vec![],
                kind: self.kind.clone(),
            }))
        } else if self.kind.is_optional() {
            Some(ftd::js::Value::Data(fastn_type::Value::Optional {
                data: Box::new(None),
                kind: self.kind.clone(),
            }))
        } else {
            None
        }
    }
    pub(crate) fn get_value(&self, properties: &[fastn_type::Property]) -> ftd::js::Value {
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
        properties: &[fastn_type::Property],
        // doc_name: &str,
        // line_number: usize
    ) -> Option<ftd::js::Value> {
        let sources = self.to_sources();

        let properties = ftd::interpreter::utils::find_properties_by_source_without_default(
            sources.as_slice(),
            properties,
        );

        ftd::js::utils::get_js_value_from_properties(properties.as_slice()) /* .map(|v|
                                                                            if let Some(fastn_type::Value::Module {}) = self.value.and_then(|v| v.value_optional()) {

                                                                             }*/
    }
}

pub(crate) fn get_optional_js_value(
    key: &str,
    properties: &[fastn_type::Property],
    arguments: &[ftd::interpreter::Argument],
) -> Option<ftd::js::Value> {
    let argument = arguments.iter().find(|v| v.name.eq(key)).unwrap();
    argument.get_optional_value(properties)
}

pub(crate) fn get_optional_js_value_with_default(
    key: &str,
    properties: &[fastn_type::Property],
    arguments: &[ftd::interpreter::Argument],
) -> Option<ftd::js::Value> {
    let argument = arguments.iter().find(|v| v.name.eq(key)).unwrap();
    argument
        .get_optional_value(properties)
        .or(argument.get_default_value())
}

pub(crate) fn get_js_value_with_default(
    key: &str,
    properties: &[fastn_type::Property],
    arguments: &[ftd::interpreter::Argument],
    default: ftd::js::Value,
) -> ftd::js::Value {
    ftd::js::value::get_optional_js_value(key, properties, arguments).unwrap_or(default)
}

pub(crate) fn ftd_to_js_variant(
    name: &str,
    variant: &str,
    full_variant: &str,
    value: &fastn_type::PropertyValue,
    doc_id: &str,
    line_number: usize,
) -> (String, bool) {
    // returns (JSVariant, has_value)
    let variant = variant
        .strip_prefix(format!("{}.", name).as_str())
        .unwrap_or(full_variant);
    match name {
        "ftd#resizing" => {
            let js_variant = resizing_variants(variant);
            (format!("fastn_dom.Resizing.{}", js_variant.0), js_variant.1)
        }
        "ftd#link-rel" => {
            let js_variant = link_rel_variants(variant);
            (format!("fastn_dom.LinkRel.{}", js_variant), false)
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
        "ftd#background-repeat" => {
            let js_variant = background_repeat_variants(variant);
            (format!("fastn_dom.BackgroundRepeat.{}", js_variant), false)
        }
        "ftd#background-size" => {
            let js_variant = background_size_variants(variant);
            (
                format!("fastn_dom.BackgroundSize.{}", js_variant.0),
                js_variant.1,
            )
        }
        "ftd#linear-gradient-directions" => {
            let js_variant = linear_gradient_direction_variants(variant);
            (
                format!("fastn_dom.LinearGradientDirection.{}", js_variant.0),
                js_variant.1,
            )
        }
        "ftd#background-position" => {
            let js_variant = background_position_variants(variant);
            (
                format!("fastn_dom.BackgroundPosition.{}", js_variant.0),
                js_variant.1,
            )
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
            (format!("fastn_dom.Anchor.{}", js_variant.0), js_variant.1)
        }
        "ftd#device-data" => {
            let js_variant = device_data_variants(variant);
            (format!("fastn_dom.DeviceData.{}", js_variant), false)
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
        "ftd#image-fit" => {
            let js_variant = object_fit_variants(variant);
            (format!("fastn_dom.Fit.{}", js_variant), false)
        }
        "ftd#image-fetch-priority" => {
            let js_variant = object_fetch_priority_variants(variant);
            (format!("fastn_dom.FetchPriority.{}", js_variant), false)
        }
        "ftd#backdrop-filter" => {
            let js_variant = backdrop_filter_variants(variant);
            (format!("fastn_dom.BackdropFilter.{}", js_variant), true)
        }
        "ftd#mask" => {
            let js_variant = mask_variants(variant);
            (format!("fastn_dom.Mask.{}", js_variant), true)
        }
        "ftd#mask-size" => {
            let js_variant = mask_size_variants(variant);
            (format!("fastn_dom.MaskSize.{}", js_variant.0), js_variant.1)
        }
        "ftd#mask-repeat" => {
            let js_variant = mask_repeat_variants(variant);
            (format!("fastn_dom.MaskRepeat.{}", js_variant), false)
        }
        "ftd#mask-position" => {
            let js_variant = mask_position_variants(variant);
            (
                format!("fastn_dom.MaskPosition.{}", js_variant.0),
                js_variant.1,
            )
        }
        t => {
            use ftd::interpreter::PropertyValueExt;

            if let Ok(value) = value.value(doc_id, line_number) {
                return match value {
                    fastn_type::Value::Integer { value } => (value.to_string(), false),
                    fastn_type::Value::Decimal { value } => (value.to_string(), false),
                    fastn_type::Value::String { text } => (format!("\"{}\"", text), false),
                    fastn_type::Value::Boolean { value } => (value.to_string(), false),
                    _ => todo!("{} {}", t, variant),
                };
            }

            todo!("{} {}", t, variant)
        }
    }
}

// Returns the corresponding js string and has_value
// Todo: Remove has_value flag
fn resizing_variants(name: &str) -> (&'static str, bool) {
    match name {
        "fixed" => ("Fixed", true),
        "fill-container" => ("FillContainer", false),
        "hug-content" => ("HugContent", false),
        "auto" => ("Auto", false),
        t => panic!("invalid resizing variant {}", t),
    }
}

fn link_rel_variants(name: &str) -> &'static str {
    match name {
        "no-follow" => "NoFollow",
        "sponsored" => "Sponsored",
        "ugc" => "Ugc",
        t => panic!("invalid link rel variant {}", t),
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
        "dvh" => "Dvh",
        "lvh" => "Lvh",
        "svh" => "Svh",
        "calc" => "Calc",
        "responsive" => "Responsive",
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
        "image" => "Image",
        "linear-gradient" => "LinearGradient",
        t => todo!("invalid background variant {}", t),
    }
}

fn background_repeat_variants(name: &str) -> &'static str {
    match name {
        "repeat" => "Repeat",
        "repeat-x" => "RepeatX",
        "repeat-y" => "RepeatY",
        "no-repeat" => "NoRepeat",
        "space" => "Space",
        "round" => "Round",
        t => todo!("invalid background repeat variant {}", t),
    }
}

fn background_size_variants(name: &str) -> (&'static str, bool) {
    match name {
        "auto" => ("Auto", false),
        "cover" => ("Cover", false),
        "contain" => ("Contain", false),
        "length" => ("Length", true),
        t => todo!("invalid background size variant {}", t),
    }
}

fn background_position_variants(name: &str) -> (&'static str, bool) {
    match name {
        "left" => ("Left", false),
        "right" => ("Right", false),
        "center" => ("Center", false),
        "left-top" => ("LeftTop", false),
        "left-center" => ("LeftCenter", false),
        "left-bottom" => ("LeftBottom", false),
        "center-top" => ("CenterTop", false),
        "center-center" => ("CenterCenter", false),
        "center-bottom" => ("CenterBottom", false),
        "right-top" => ("RightTop", false),
        "right-center" => ("RightCenter", false),
        "right-bottom" => ("RightBottom", false),
        "length" => ("Length", true),
        t => todo!("invalid background position variant {}", t),
    }
}

fn linear_gradient_direction_variants(name: &str) -> (&'static str, bool) {
    match name {
        "angle" => ("Angle", true),
        "turn" => ("Turn", true),
        "left" => ("Left", false),
        "right" => ("Right", false),
        "top" => ("Top", false),
        "bottom" => ("Bottom", false),
        "top-left" => ("TopLeft", false),
        "top-right" => ("TopRight", false),
        "bottom-left" => ("BottomLeft", false),
        "bottom-right" => ("BottomRight", false),
        t => todo!("invalid linear-gradient direction variant {}", t),
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

fn anchor_variants(name: &str) -> (&'static str, bool) {
    match name {
        "window" => ("Window", false),
        "parent" => ("Parent", false),
        "id" => ("Id", true),
        t => todo!("invalid anchor variant {}", t),
    }
}

fn device_data_variants(name: &str) -> &'static str {
    match name {
        "desktop" => "Desktop",
        "mobile" => "Mobile",
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

fn object_fit_variants(name: &str) -> &'static str {
    match name {
        "none" => "none",
        "fill" => "fill",
        "contain" => "contain",
        "cover" => "cover",
        "scale-down" => "scaleDown",
        t => todo!("invalid object fit variant {}", t),
    }
}

fn object_fetch_priority_variants(name: &str) -> &'static str {
    match name {
        "auto" => "auto",
        "high" => "high",
        "low" => "low",
        t => todo!("invalid object fetchPriority variant {}", t),
    }
}

fn backdrop_filter_variants(name: &str) -> &'static str {
    match name {
        "blur" => "Blur",
        "brightness" => "Brightness",
        "contrast" => "Contrast",
        "grayscale" => "Grayscale",
        "invert" => "Invert",
        "opacity" => "Opacity",
        "sepia" => "Sepia",
        "saturate" => "Saturate",
        "multi" => "Multi",
        t => unimplemented!("invalid backdrop filter variant {}", t),
    }
}

fn mask_variants(name: &str) -> &'static str {
    match name {
        "image" => "Image",
        "multi" => "Multi",
        t => todo!("invalid mask variant {}", t),
    }
}

fn mask_size_variants(name: &str) -> (&'static str, bool) {
    match name {
        "auto" => ("Auto", false),
        "cover" => ("Cover", false),
        "contain" => ("Contain", false),
        "fixed" => ("Fixed", true),
        t => todo!("invalid mask variant {}", t),
    }
}

fn mask_repeat_variants(name: &str) -> &'static str {
    match name {
        "repeat" => "Repeat",
        "repeat-x" => "RepeatX",
        "repeat-y" => "RepeatY",
        "no-repeat" => "NoRepeat",
        "space" => "Space",
        "round" => "Round",
        t => todo!("invalid mask repeat variant {}", t),
    }
}

fn mask_position_variants(name: &str) -> (&'static str, bool) {
    match name {
        "left" => ("Left", false),
        "right" => ("Right", false),
        "center" => ("Center", false),
        "left-top" => ("LeftTop", false),
        "left-center" => ("LeftCenter", false),
        "left-bottom" => ("LeftBottom", false),
        "center-top" => ("CenterTop", false),
        "center-center" => ("CenterCenter", false),
        "center-bottom" => ("CenterBottom", false),
        "right-top" => ("RightTop", false),
        "right-center" => ("RightCenter", false),
        "right-bottom" => ("RightBottom", false),
        "length" => ("Length", true),
        t => todo!("invalid mask position variant {}", t),
    }
}
