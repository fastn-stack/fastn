pub struct SetProperty {
    pub kind: PropertyKind,
    pub value: SetPropertyValue,
    pub element_name: String,
}

pub enum SetPropertyValue {
    Reference(String),
    Value(Value),
    Formula(Formula),
}

impl SetPropertyValue {
    pub fn to_js(&self, kind: &PropertyKind) -> String {
        match self {
            SetPropertyValue::Reference(name) => name.to_string(),
            SetPropertyValue::Value(v) => v.to_js(kind),
            SetPropertyValue::Formula(f) => f.to_js(kind),
        }
    }

    pub fn is_formula(&self) -> bool {
        matches!(&self, SetPropertyValue::Formula(_))
    }
}

pub struct Formula {
    pub deps: Vec<String>,
    pub conditional_values: Vec<ConditionalValue>,
}

impl Formula {
    pub(crate) fn to_js(&self, kind: &PropertyKind) -> String {
        format!(
            "[{}], {}",
            self.deps.join(", "),
            self.conditional_values_to_js(kind)
        )
    }

    pub(crate) fn conditional_values_to_js(&self, kind: &PropertyKind) -> String {
        let mut conditions = vec![];
        let mut default = None;
        for conditional_value in &self.conditional_values {
            if let Some(ref condition) = conditional_value.condition {
                conditions.push(format!(
                    indoc::indoc! {"
                        {if_exp}({condition}){{
                            return {expression};
                        }}
                    "},
                    if_exp = if conditions.is_empty() {
                        "if"
                    } else {
                        "else if"
                    },
                    condition = condition,
                    expression = conditional_value.expression.to_js(kind),
                ));
            } else {
                default = Some(conditional_value.expression.to_js(kind))
            }
        }

        let default = match default {
            Some(d) if conditions.is_empty() => d,
            Some(d) => format!("else {{ return {}; }}", d),
            None => format!(
                "else {{ return fastn_utils.defaultPropertyValue({}); }}",
                kind.to_js()
            ),
        };

        format!(
            indoc::indoc! {"
            {expressions}{default}
        "},
            expressions = conditions.join(" "),
            default = default,
        )
    }
}

pub struct ConditionalValue {
    pub condition: Option<String>,
    pub expression: SetPropertyValue,
}

pub enum Value {
    String(String),
    Integer(i64),
    Decimal(f64),
    OrType {
        variant: String,
        value: Option<Box<SetPropertyValue>>,
    },
}

impl Value {
    pub(crate) fn to_js(&self, kind: &PropertyKind) -> String {
        match self {
            Value::String(s) => format!("\"{s}\""),
            Value::Integer(i) => i.to_string(),
            Value::Decimal(f) => f.to_string(),
            Value::OrType { variant, value } => {
                if let Some(value) = value {
                    format!("{}({})", variant, value.to_js(kind))
                } else {
                    variant.to_owned()
                }
            }
        }
    }
}

pub enum PropertyKind {
    StringValue,
    Id,
    Width,
    Padding,
    Height,
}

impl PropertyKind {
    pub(crate) fn to_js(&self) -> &'static str {
        match self {
            PropertyKind::Id => "fastn_dom.PropertyKind.Id",
            PropertyKind::StringValue => "fastn_dom.PropertyKind.StringValue",
            PropertyKind::Width => "fastn_dom.PropertyKind.Width",
            PropertyKind::Padding => "fastn_dom.PropertyKind.Padding",
            PropertyKind::Height => "fastn_dom.PropertyKind.Height",
        }
    }
}
