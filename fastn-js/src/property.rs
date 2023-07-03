pub struct SetProperty {
    pub kind: PropertyKind,
    pub value: SetPropertyValue,
    pub element_name: String,
}

#[derive(Debug)]
pub enum SetPropertyValue {
    Reference(String),
    Value(Value),
    Formula(Formula),
}

impl SetPropertyValue {
    pub fn to_js(&self) -> String {
        match self {
            SetPropertyValue::Reference(name) => fastn_js::utils::reference_to_js(name),
            SetPropertyValue::Value(v) => v.to_js(),
            SetPropertyValue::Formula(f) => f.to_js(),
        }
    }

    pub fn is_formula(&self) -> bool {
        matches!(&self, SetPropertyValue::Formula(_))
    }
}

#[derive(Debug)]
pub struct Formula {
    pub deps: Vec<String>,
    pub conditional_values: Vec<ConditionalValue>,
}

impl Formula {
    pub(crate) fn to_js(&self) -> String {
        use itertools::Itertools;

        format!(
            "fastn.formula([{}], {})",
            self.deps
                .iter()
                .map(|v| fastn_js::utils::name_to_js(v))
                .collect_vec()
                .join(", "),
            self.conditional_values_to_js()
        )
    }

    pub(crate) fn conditional_values_to_js(&self) -> String {
        let mut conditions = vec![];
        let mut default = None;
        for conditional_value in &self.conditional_values {
            if let Some(ref condition) = conditional_value.condition {
                let condition = format!(
                    indoc::indoc! {"
                        function(){{
                            {expression}
                        }}()"
                    },
                    expression = fastn_js::to_js::ExpressionGenerator.to_js(condition).trim(),
                );
                conditions.push(format!(
                    indoc::indoc! {
                        "{if_exp}({condition}){{
                            return {expression};
                        }}"
                    },
                    if_exp = if conditions.is_empty() {
                        "if"
                    } else {
                        "else if"
                    },
                    condition = condition,
                    expression = conditional_value.expression.to_js(),
                ));
            } else {
                default = Some(conditional_value.expression.to_js())
            }
        }

        let default = match default {
            Some(d) if conditions.is_empty() => d,
            Some(d) => format!("else {{ return {}; }}", d),
            None => "".to_string(),
        };

        format!(
            indoc::indoc! {"
            function() {{
                {expressions}{default}
            }}
        "},
            expressions = conditions.join(" "),
            default = default,
        )
    }
}

#[derive(Debug)]
pub struct ConditionalValue {
    pub condition: Option<fastn_grammar::evalexpr::ExprNode>,
    pub expression: SetPropertyValue,
}

#[derive(Debug)]
pub enum Value {
    String(String),
    Integer(i64),
    Decimal(f64),
    Boolean(bool),
    OrType {
        variant: String,
        value: Option<Box<SetPropertyValue>>,
    },
    List {
        value: Vec<SetPropertyValue>,
    },
    Record {
        fields: Vec<(String, SetPropertyValue)>,
    },
}

impl Value {
    pub(crate) fn to_js(&self) -> String {
        use itertools::Itertools;
        match self {
            Value::String(s) => format!("\"{}\"", s.replace('\n', "\\n")),
            Value::Integer(i) => i.to_string(),
            Value::Decimal(f) => f.to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::OrType { variant, value } => {
                if let Some(value) = value {
                    format!("{}({})", variant, value.to_js())
                } else {
                    variant.to_owned()
                }
            }
            Value::List { value } => format!(
                "fastn.mutableList([{}])",
                value.iter().map(|v| v.to_js()).join(", ")
            ),
            Value::Record { fields } => format!(
                "fastn.recordInstance({{{}}})",
                fields
                    .iter()
                    .map(|(k, v)| format!(
                        "{}: {}",
                        fastn_js::utils::kebab_to_snake_case(k),
                        v.to_js()
                    ))
                    .join(", ")
            ),
        }
    }
}

pub enum PropertyKind {
    StringValue,
    Id,
    Width,
    Padding,
    PaddingHorizontal,
    PaddingVertical,
    PaddingLeft,
    PaddingRight,
    PaddingTop,
    PaddingBottom,
    Margin,
    MarginHorizontal,
    MarginVertical,
    MarginTop,
    MarginBottom,
    MarginLeft,
    MarginRight,
    Height,
    BorderWidth,
    BorderStyle,
    Color,
    Background,
    Role,
}

impl PropertyKind {
    pub(crate) fn to_js(&self) -> &'static str {
        match self {
            PropertyKind::Id => "fastn_dom.PropertyKind.Id",
            PropertyKind::StringValue => "fastn_dom.PropertyKind.StringValue",
            PropertyKind::Width => "fastn_dom.PropertyKind.Width",
            PropertyKind::Padding => "fastn_dom.PropertyKind.Padding",
            PropertyKind::PaddingHorizontal => "fastn_dom.PropertyKind.PaddingHorizontal",
            PropertyKind::PaddingVertical => "fastn_dom.PropertyKind.PaddingVertical",
            PropertyKind::PaddingLeft => "fastn_dom.PropertyKind.PaddingLeft",
            PropertyKind::PaddingRight => "fastn_dom.PropertyKind.PaddingRight",
            PropertyKind::PaddingTop => "fastn_dom.PropertyKind.PaddingTop",
            PropertyKind::PaddingBottom => "fastn_dom.PropertyKind.PaddingBottom",
            PropertyKind::Margin => "fastn_dom.PropertyKind.Margin",
            PropertyKind::MarginHorizontal => "fastn_dom.PropertyKind.MarginHorizontal",
            PropertyKind::MarginVertical => "fastn_dom.PropertyKind.MarginVertical",
            PropertyKind::MarginLeft => "fastn_dom.PropertyKind.MarginLeft",
            PropertyKind::MarginRight => "fastn_dom.PropertyKind.MarginRight",
            PropertyKind::MarginTop => "fastn_dom.PropertyKind.MarginTop",
            PropertyKind::MarginBottom => "fastn_dom.PropertyKind.MarginBottom",
            PropertyKind::Height => "fastn_dom.PropertyKind.Height",
            PropertyKind::BorderWidth => "fastn_dom.PropertyKind.BorderWidth",
            PropertyKind::BorderStyle => "fastn_dom.PropertyKind.BorderStyle",
            PropertyKind::Color => "fastn_dom.PropertyKind.Color",
            PropertyKind::Background => "fastn_dom.PropertyKind.Background",
            PropertyKind::Role => "fastn_dom.PropertyKind.Role",
        }
    }
}
