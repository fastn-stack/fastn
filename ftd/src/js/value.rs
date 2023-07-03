#[derive(Debug)]
pub enum Value {
    Data(ftd::interpreter::Value),
    Reference(String),
    Formula(Vec<ftd::interpreter::Property>),
}

impl Value {
    pub(crate) fn to_set_property_value_with_none(&self) -> fastn_js::SetPropertyValue {
        self.to_set_property_value(&None, &None)
    }

    pub(crate) fn to_set_property_value(
        &self,
        component_definition_name: &Option<String>,
        loop_alias: &Option<String>,
    ) -> fastn_js::SetPropertyValue {
        match self {
            Value::Data(value) => value.to_fastn_js_value(),
            Value::Reference(name) => fastn_js::SetPropertyValue::Reference(
                ftd::js::utils::update_reference(name, component_definition_name, loop_alias),
            ),
            Value::Formula(formulas) => fastn_js::SetPropertyValue::Formula(
                formulas_to_fastn_js_value(formulas, component_definition_name, loop_alias),
            ),
        }
    }

    pub(crate) fn to_set_property(
        &self,
        kind: fastn_js::PropertyKind,
        element_name: &str,
        component_definition_name: &Option<String>,
        loop_alias: &Option<String>,
    ) -> fastn_js::SetProperty {
        fastn_js::SetProperty {
            kind,
            value: self.to_set_property_value(component_definition_name, loop_alias),
            element_name: element_name.to_string(),
        }
    }
}

fn formulas_to_fastn_js_value(
    properties: &[ftd::interpreter::Property],
    component_definition_name: &Option<String>,
    loop_alias: &Option<String>,
) -> fastn_js::Formula {
    let mut deps = vec![];
    let mut conditional_values = vec![];
    for property in properties {
        deps.extend(
            property
                .value
                .get_deps(component_definition_name, loop_alias),
        );
        if let Some(ref condition) = property.condition {
            deps.extend(condition.get_deps(component_definition_name, loop_alias));
        }

        conditional_values.push(fastn_js::ConditionalValue {
            condition: property.condition.as_ref().map(|condition| {
                condition
                    .update_node_with_variable_reference_js(component_definition_name, loop_alias)
            }),
            expression: property.value.to_fastn_js_value(),
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
    ) -> Vec<String> {
        let mut deps = vec![];
        for property_value in self.references.values() {
            deps.extend(property_value.get_deps(component_definition_name, loop_alias));
        }
        deps
    }

    pub fn update_node_with_variable_reference_js(
        &self,
        component_definition_name: &Option<String>,
        loop_alias: &Option<String>,
    ) -> fastn_grammar::evalexpr::ExprNode {
        return update_node_with_variable_reference_js_(
            &self.expression,
            &self.references,
            component_definition_name,
            loop_alias,
        );

        fn update_node_with_variable_reference_js_(
            expr: &fastn_grammar::evalexpr::ExprNode,
            references: &ftd::Map<ftd::interpreter::PropertyValue>,
            component_definition_name: &Option<String>,
            loop_alias: &Option<String>,
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
                    );
                    operator = fastn_grammar::evalexpr::Operator::VariableIdentifierRead {
                        identifier: fastn_js::utils::name_to_js(name.as_str()),
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
    ) -> Vec<String> {
        let mut deps = vec![];
        if let Some(reference) = self.get_reference_or_clone() {
            deps.push(ftd::js::utils::update_reference(
                reference,
                component_definition_name,
                loop_alias,
            ));
        } else if let Some(function) = self.get_function() {
            for value in function.values.values() {
                deps.extend(value.get_deps(component_definition_name, loop_alias));
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
    pub(crate) fn to_fastn_js_value(&self) -> fastn_js::SetPropertyValue {
        match self {
            ftd::interpreter::PropertyValue::Value { ref value, .. } => value.to_fastn_js_value(),
            ftd::interpreter::PropertyValue::Reference { ref name, .. } => {
                fastn_js::SetPropertyValue::Reference(name.to_string())
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
    pub(crate) fn to_fastn_js_value(&self) -> fastn_js::SetPropertyValue {
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
                        value: Some(Box::new(value.to_fastn_js_value())),
                    });
                }
                fastn_js::SetPropertyValue::Value(fastn_js::Value::OrType {
                    variant: js_variant,
                    value: None,
                })
            }
            ftd::interpreter::Value::List { data, .. } => {
                fastn_js::SetPropertyValue::Value(fastn_js::Value::List {
                    value: data.iter().map(|v| v.to_fastn_js_value()).collect_vec(),
                })
            }
            ftd::interpreter::Value::Record { fields, .. } => {
                fastn_js::SetPropertyValue::Value(fastn_js::Value::Record {
                    fields: fields
                        .iter()
                        .map(|(k, v)| (k.to_string(), v.to_fastn_js_value()))
                        .collect_vec(),
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
            (format!("fastn_dom.Overflow.{}", js_variant.0), js_variant.1)
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
        _ => todo!(),
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
        _ => todo!(),
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
        _ => todo!(),
    }
}

fn background_variants(name: &str) -> &'static str {
    match name {
        "solid" => "Solid",
        t => todo!("{}", t),
    }
}

fn font_size_variants(name: &str) -> &'static str {
    match name {
        "px" => "Px",
        "em" => "Em",
        "rem" => "Rem",
        _ => todo!(),
    }
}

fn overflow_variants(name: &str) -> (&'static str, bool) {
    match name {
        "scroll" => ("Scroll", false),
        "visible" => ("Visible", false),
        "hidden" => ("Hidden", false),
        "auto" => ("Auto", false),
        _ => todo!(),
    }
}
