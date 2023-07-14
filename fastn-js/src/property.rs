#[derive(Debug)]
pub struct SetProperty {
    pub kind: PropertyKind,
    pub value: SetPropertyValue,
    pub element_name: String,
    pub inherited: String,
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
                .map(|v| fastn_js::utils::reference_to_js(v))
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
    UI {
        value: Vec<fastn_js::ComponentStatement>,
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
            Value::UI { value } => format!(
                "function({}, {}){{{}}}",
                fastn_js::FUNCTION_PARENT,
                fastn_js::INHERITED_VARIABLE,
                value
                    .iter()
                    .map(|v| {
                        let mut w = Vec::new();
                        v.to_js().render(80, &mut w).unwrap();
                        String::from_utf8(w).unwrap()
                    })
                    .join("")
            ),
        }
    }
}

#[derive(Debug)]
pub enum PropertyKind {
    Children,
    StringValue,
    Id,
    Region,
    OpenInNewTab,
    Link,
    Anchor,
    Classes,
    AlignSelf,
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
    BorderTopWidth,
    BorderBottomWidth,
    BorderLeftWidth,
    BorderRightWidth,
    BorderRadius,
    BorderTopLeftRadius,
    BorderTopRightRadius,
    BorderBottomLeftRadius,
    BorderBottomRightRadius,
    BorderStyle,
    BorderStyleVertical,
    BorderStyleHorizontal,
    BorderLeftStyle,
    BorderRightStyle,
    BorderTopStyle,
    BorderBottomStyle,
    BorderColor,
    BorderLeftColor,
    BorderRightColor,
    BorderTopColor,
    BorderBottomColor,
    Color,
    Background,
    Role,
    ZIndex,
    Sticky,
    Top,
    Bottom,
    Left,
    Right,
    Overflow,
    OverflowX,
    OverflowY,
    Spacing,
    Wrap,
    TextTransform,
    TextIndent,
    TextAlign,
    LineClamp,
    Opacity,
    Cursor,
    Resize,
    MaxHeight,
    MinHeight,
    MaxWidth,
    MinWidth,
    WhiteSpace,
    TextStyle,
    AlignContent,
}

impl PropertyKind {
    pub(crate) fn to_js(&self) -> &'static str {
        match self {
            PropertyKind::Children => "fastn_dom.PropertyKind.Children",
            PropertyKind::Id => "fastn_dom.PropertyKind.Id",
            PropertyKind::AlignSelf => "fastn_dom.PropertyKind.AlignSelf",
            PropertyKind::Anchor => "fastn_dom.PropertyKind.Anchor",
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
            PropertyKind::BorderTopWidth => "fastn_dom.PropertyKind.BorderTopWidth",
            PropertyKind::BorderBottomWidth => "fastn_dom.PropertyKind.BorderBottomWidth",
            PropertyKind::BorderLeftWidth => "fastn_dom.PropertyKind.BorderLeftWidth",
            PropertyKind::BorderRightWidth => "fastn_dom.PropertyKind.BorderRightWidth",
            PropertyKind::BorderRadius => "fastn_dom.PropertyKind.BorderRadius",
            PropertyKind::BorderTopLeftRadius => "fastn_dom.PropertyKind.BorderTopLeftRadius",
            PropertyKind::BorderTopRightRadius => "fastn_dom.PropertyKind.BorderTopRightRadius",
            PropertyKind::BorderBottomLeftRadius => "fastn_dom.PropertyKind.BorderBottomLeftRadius",
            PropertyKind::BorderBottomRightRadius => {
                "fastn_dom.PropertyKind.BorderBottomRightRadius"
            }
            PropertyKind::BorderStyle => "fastn_dom.PropertyKind.BorderStyle",
            PropertyKind::BorderStyleVertical => "fastn_dom.PropertyKind.BorderStyleVertical",
            PropertyKind::BorderStyleHorizontal => "fastn_dom.PropertyKind.BorderStyleHorizontal",
            PropertyKind::BorderLeftStyle => "fastn_dom.PropertyKind.BorderLeftStyle",
            PropertyKind::BorderRightStyle => "fastn_dom.PropertyKind.BorderRightStyle",
            PropertyKind::BorderTopStyle => "fastn_dom.PropertyKind.BorderTopStyle",
            PropertyKind::BorderBottomStyle => "fastn_dom.PropertyKind.BorderBottomStyle",
            PropertyKind::BorderColor => "fastn_dom.PropertyKind.BorderColor",
            PropertyKind::BorderLeftColor => "fastn_dom.PropertyKind.BorderLeftColor",
            PropertyKind::BorderRightColor => "fastn_dom.PropertyKind.BorderRightColor",
            PropertyKind::BorderTopColor => "fastn_dom.PropertyKind.BorderTopColor",
            PropertyKind::BorderBottomColor => "fastn_dom.PropertyKind.BorderBottomColor",
            PropertyKind::Color => "fastn_dom.PropertyKind.Color",
            PropertyKind::Background => "fastn_dom.PropertyKind.Background",
            PropertyKind::Role => "fastn_dom.PropertyKind.Role",
            PropertyKind::ZIndex => "fastn_dom.PropertyKind.ZIndex",
            PropertyKind::Sticky => "fastn_dom.PropertyKind.Sticky",
            PropertyKind::Top => "fastn_dom.PropertyKind.Top",
            PropertyKind::Bottom => "fastn_dom.PropertyKind.Bottom",
            PropertyKind::Left => "fastn_dom.PropertyKind.Left",
            PropertyKind::Right => "fastn_dom.PropertyKind.Right",
            PropertyKind::Overflow => "fastn_dom.PropertyKind.Overflow",
            PropertyKind::OverflowX => "fastn_dom.PropertyKind.OverflowX",
            PropertyKind::OverflowY => "fastn_dom.PropertyKind.OverflowY",
            PropertyKind::Spacing => "fastn_dom.PropertyKind.Spacing",
            PropertyKind::Wrap => "fastn_dom.PropertyKind.Wrap",
            PropertyKind::TextTransform => "fastn_dom.PropertyKind.TextTransform",
            PropertyKind::TextIndent => "fastn_dom.PropertyKind.TextIndent",
            PropertyKind::TextAlign => "fastn_dom.PropertyKind.TextAlign",
            PropertyKind::LineClamp => "fastn_dom.PropertyKind.LineClamp",
            PropertyKind::Opacity => "fastn_dom.PropertyKind.Opacity",
            PropertyKind::Cursor => "fastn_dom.PropertyKind.Cursor",
            PropertyKind::Resize => "fastn_dom.PropertyKind.Resize",
            PropertyKind::MaxHeight => "fastn_dom.PropertyKind.MaxHeight",
            PropertyKind::MinHeight => "fastn_dom.PropertyKind.MinHeight",
            PropertyKind::MaxWidth => "fastn_dom.PropertyKind.MaxWidth",
            PropertyKind::MinWidth => "fastn_dom.PropertyKind.MinWidth",
            PropertyKind::WhiteSpace => "fastn_dom.PropertyKind.WhiteSpace",
            PropertyKind::Classes => "fastn_dom.PropertyKind.Classes",
            PropertyKind::Link => "fastn_dom.PropertyKind.Link",
            PropertyKind::OpenInNewTab => "fastn_dom.PropertyKind.OpenInNewTab",
            PropertyKind::TextStyle => "fastn_dom.PropertyKind.TextStyle",
            PropertyKind::Region => "fastn_dom.PropertyKind.Region",
            PropertyKind::AlignContent => "fastn_dom.PropertyKind.AlignContent",
        }
    }
}
