pub mod component;
mod kind;
mod or_type;
mod record;
pub(crate) mod variable;

pub use component::Component;
pub use component::Instruction;
pub use kind::Kind;
pub use or_type::OrType;
pub use record::Record;
pub use variable::Variable;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Thing {
    Component(ftd::interpreter::Component),
    Variable(ftd::interpreter::Variable),
    Record(ftd::interpreter::Record),
    OrType(ftd::interpreter::OrType),
    OrTypeWithVariant {
        e: ftd::interpreter::OrType,
        variant: String,
    },
    // Library -> Name of library successfully parsed
}

pub fn default_bag() -> ftd::Map<ftd::interpreter::Thing> {
    let record = |n: &str, r: &str| (n.to_string(), ftd::p2::Kind::record(r));
    let color = |n: &str| record(n, "ftd#color");
    std::iter::IntoIterator::into_iter([
        (
            "ftd#row".to_string(),
            ftd::p2::Thing::Component(ftd::p2::element::row_function()),
        ),
        (
            "ftd#column".to_string(),
            ftd::p2::Thing::Component(ftd::p2::element::column_function()),
        ),
        (
            "ftd#text-block".to_string(),
            ftd::p2::Thing::Component(ftd::p2::element::text_function()),
        ),
        (
            "ftd#code".to_string(),
            ftd::p2::Thing::Component(ftd::p2::element::code_function()),
        ),
        (
            "ftd#image".to_string(),
            ftd::p2::Thing::Component(ftd::p2::element::image_function()),
        ),
        (
            "ftd#iframe".to_string(),
            ftd::p2::Thing::Component(ftd::p2::element::iframe_function()),
        ),
        (
            "ftd#integer".to_string(),
            ftd::p2::Thing::Component(ftd::p2::element::integer_function()),
        ),
        (
            "ftd#decimal".to_string(),
            ftd::p2::Thing::Component(ftd::p2::element::decimal_function()),
        ),
        (
            "ftd#boolean".to_string(),
            ftd::p2::Thing::Component(ftd::p2::element::boolean_function()),
        ),
        (
            "ftd#scene".to_string(),
            ftd::p2::Thing::Component(ftd::p2::element::scene_function()),
        ),
        (
            "ftd#grid".to_string(),
            ftd::p2::Thing::Component(ftd::p2::element::grid_function()),
        ),
        (
            "ftd#text".to_string(),
            ftd::p2::Thing::Component(ftd::p2::element::markup_function()),
        ),
        (
            "ftd#input".to_string(),
            ftd::p2::Thing::Component(ftd::p2::element::input_function()),
        ),
        (
            "ftd#null".to_string(),
            ftd::p2::Thing::Component(ftd::p2::element::null()),
        ),
        (
            "ftd#dark-mode".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: "ftd#dark-mode".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Boolean { value: false },
                },
                conditions: vec![],
                flags: ftd::VariableFlags {
                    always_include: Some(true),
                },
            }),
        ),
        (
            "ftd#system-dark-mode".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: "ftd#system-dark-mode".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Boolean { value: false },
                },
                conditions: vec![],
                flags: ftd::VariableFlags {
                    always_include: Some(true),
                },
            }),
        ),
        (
            "ftd#follow-system-dark-mode".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: "ftd#follow-system-dark-mode".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Boolean { value: true },
                },
                conditions: vec![],
                flags: ftd::VariableFlags {
                    always_include: Some(true),
                },
            }),
        ),
        (
            "ftd#device".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: "ftd#device".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "desktop".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: ftd::VariableFlags {
                    always_include: Some(true),
                },
            }),
        ),
        (
            "ftd#mobile-breakpoint".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: "ftd#mobile-breakpoint".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 768 },
                },
                conditions: vec![],
                flags: ftd::VariableFlags {
                    always_include: Some(true),
                },
            }),
        ),
        (
            "ftd#desktop-breakpoint".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: "ftd#desktop-breakpoint".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 1440 },
                },
                conditions: vec![],
                flags: ftd::VariableFlags {
                    always_include: Some(true),
                },
            }),
        ),
        (
            "ftd#markdown-color-data".to_string(),
            ftd::p2::Thing::Record(ftd::p2::Record {
                name: "ftd#markdown-color-data".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    ("link".to_string(), ftd::p2::Kind::record("ftd#color")),
                    ("code".to_string(), ftd::p2::Kind::record("ftd#color")),
                    ("link-code".to_string(), ftd::p2::Kind::record("ftd#color")),
                    (
                        "link-visited".to_string(),
                        ftd::p2::Kind::record("ftd#color"),
                    ),
                    (
                        "link-visited-code".to_string(),
                        ftd::p2::Kind::record("ftd#color"),
                    ),
                    (
                        "ul-ol-li-before".to_string(),
                        ftd::p2::Kind::record("ftd#color"),
                    ),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![
                    "link".to_string(),
                    "code".to_string(),
                    "link-code".to_string(),
                    "link-visited".to_string(),
                    "link-visited-code".to_string(),
                    "ul-ol-li-before".to_string(),
                ],
            }),
        ),
        ("ftd#markdown-color".to_string(), markdown::color()),
        (
            "ftd#markdown-background-color-data".to_string(),
            ftd::p2::Thing::Record(ftd::p2::Record {
                name: "ftd#markdown-background-color-data".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    ("link".to_string(), ftd::p2::Kind::record("ftd#color")),
                    ("code".to_string(), ftd::p2::Kind::record("ftd#color")),
                    ("link-code".to_string(), ftd::p2::Kind::record("ftd#color")),
                    (
                        "link-visited".to_string(),
                        ftd::p2::Kind::record("ftd#color"),
                    ),
                    (
                        "link-visited-code".to_string(),
                        ftd::p2::Kind::record("ftd#color"),
                    ),
                    (
                        "ul-ol-li-before".to_string(),
                        ftd::p2::Kind::record("ftd#color"),
                    ),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![
                    "link".to_string(),
                    "code".to_string(),
                    "link-code".to_string(),
                    "link-visited".to_string(),
                    "link-visited-code".to_string(),
                    "ul-ol-li-before".to_string(),
                ],
            }),
        ),
        (
            "ftd#markdown-background-color".to_string(),
            markdown::background_color(),
        ),
        (
            "ftd#image-src".to_string(),
            ftd::p2::Thing::Record(ftd::p2::Record {
                name: "ftd#image-src".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    ("light".to_string(), ftd::p2::Kind::caption()),
                    ("dark".to_string(), ftd::p2::Kind::string()),
                ])
                .collect(),
                instances: Default::default(),
                order: vec!["light".to_string(), "dark".to_string()],
            }),
        ),
        (
            "ftd#color".to_string(),
            ftd::p2::Thing::Record(ftd::p2::Record {
                name: "ftd#color".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    ("light".to_string(), ftd::p2::Kind::caption()),
                    ("dark".to_string(), ftd::p2::Kind::string()),
                ])
                .collect(),
                instances: Default::default(),
                order: vec!["light".to_string(), "dark".to_string()],
            }),
        ),
        (
            "ftd#font-size".to_string(),
            ftd::p2::Thing::Record(ftd::p2::Record {
                name: "ftd#font-size".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    ("line-height".to_string(), ftd::p2::Kind::integer()),
                    ("size".to_string(), ftd::p2::Kind::integer()),
                    (
                        "letter-spacing".to_string(),
                        ftd::p2::Kind::integer().set_default(Some("0".to_string())),
                    ),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![
                    "line-height".to_string(),
                    "size".to_string(),
                    "letter-spacing".to_string(),
                ],
            }),
        ),
        (
            "ftd#type".to_string(),
            ftd::p2::Thing::Record(ftd::p2::Record {
                name: "ftd#type".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    ("font".to_string(), ftd::p2::Kind::caption()),
                    (
                        "desktop".to_string(),
                        ftd::p2::Kind::record("ftd#font-size"),
                    ),
                    ("mobile".to_string(), ftd::p2::Kind::record("ftd#font-size")),
                    ("xl".to_string(), ftd::p2::Kind::record("ftd#font-size")),
                    (
                        "weight".to_string(),
                        ftd::p2::Kind::integer().set_default(Some("400".to_string())),
                    ),
                    ("style".to_string(), ftd::p2::Kind::string().into_optional()),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![
                    "font".to_string(),
                    "desktop".to_string(),
                    "mobile".to_string(),
                    "xl".to_string(),
                    "weight".to_string(),
                    "style".to_string(),
                ],
            }),
        ),
        (
            "ftd#btb".to_string(),
            ftd::p2::Thing::Record(ftd::p2::Record {
                name: "ftd#btb".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    color("base"),
                    color("text"),
                    color("border"),
                ])
                .collect(),
                instances: Default::default(),
                order: vec!["base".to_string(), "text".to_string(), "border".to_string()],
            }),
        ),
        (
            "ftd#pst".to_string(),
            ftd::p2::Thing::Record(ftd::p2::Record {
                name: "ftd#pst".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    color("primary"),
                    color("secondary"),
                    color("tertiary"),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![
                    "primary".to_string(),
                    "secondary".to_string(),
                    "tertiary".to_string(),
                ],
            }),
        ),
        (
            "ftd#background-colors".to_string(),
            ftd::p2::Thing::Record(ftd::p2::Record {
                name: "ftd#background-colors".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    color("base"),
                    color("step-1"),
                    color("step-2"),
                    color("overlay"),
                    color("code"),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![
                    "base".to_string(),
                    "step-1".to_string(),
                    "step-2".to_string(),
                    "overlay".to_string(),
                    "code".to_string(),
                ],
            }),
        ),
        (
            "ftd#custom-colors".to_string(),
            ftd::p2::Thing::Record(ftd::p2::Record {
                name: "ftd#custom-colors".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    color("one"),
                    color("two"),
                    color("three"),
                    color("four"),
                    color("five"),
                    color("six"),
                    color("seven"),
                    color("eight"),
                    color("nine"),
                    color("ten"),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![
                    "one".to_string(),
                    "two".to_string(),
                    "three".to_string(),
                    "four".to_string(),
                    "five".to_string(),
                    "six".to_string(),
                    "seven".to_string(),
                    "eight".to_string(),
                    "nine".to_string(),
                    "ten".to_string(),
                ],
            }),
        ),
        (
            "ftd#cta-colors".to_string(),
            ftd::p2::Thing::Record(ftd::p2::Record {
                name: "ftd#cta-colors".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    color("base"),
                    color("hover"),
                    color("pressed"),
                    color("disabled"),
                    color("focused"),
                    color("border"),
                    color("text"),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![
                    "base".to_string(),
                    "hover".to_string(),
                    "pressed".to_string(),
                    "disabled".to_string(),
                    "focused".to_string(),
                    "border".to_string(),
                    "text".to_string(),
                ],
            }),
        ),
        (
            "ftd#color-scheme".to_string(),
            ftd::p2::Thing::Record(ftd::p2::Record {
                name: "ftd#color-scheme".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    record("background", "ftd#background-colors"),
                    color("border"),
                    color("border-strong"),
                    color("text"),
                    color("text-strong"),
                    color("shadow"),
                    color("scrim"),
                    record("cta-primary", "ftd#cta-colors"),
                    record("cta-secondary", "ftd#cta-colors"),
                    record("cta-tertiary", "ftd#cta-colors"),
                    record("cta-danger", "ftd#cta-colors"),
                    record("accent", "ftd#pst"),
                    record("error", "ftd#btb"),
                    record("success", "ftd#btb"),
                    record("info", "ftd#btb"),
                    record("warning", "ftd#btb"),
                    record("custom", "ftd#custom-colors"),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![
                    "background".to_string(),
                    "border".to_string(),
                    "border-strong".to_string(),
                    "text".to_string(),
                    "text-strong".to_string(),
                    "shadow".to_string(),
                    "scrim".to_string(),
                    "cta-primary".to_string(),
                    "cta-secondary".to_string(),
                    "cta-tertiary".to_string(),
                    "cta-danger".to_string(),
                    "accent".to_string(),
                    "error".to_string(),
                    "success".to_string(),
                    "info".to_string(),
                    "warning".to_string(),
                    "custom".to_string(),
                ],
            }),
        ),
    ])
    .collect()
}

pub fn default_aliases() -> ftd::Map<String> {
    std::iter::IntoIterator::into_iter([("ftd".to_string(), "ftd".to_string())]).collect()
}

pub fn default_column() -> ftd::Column {
    ftd::Column {
        common: ftd::Common {
            width: Some(ftd::Length::Fill),
            height: Some(ftd::Length::Fill),
            position: Some(ftd::Position::Center),
            ..Default::default()
        },
        spacing: None,
        ..Default::default()
    }
}

pub mod markdown {
    fn theme_color(light: &str, dark: &str) -> ftd::PropertyValue {
        ftd::PropertyValue::Value {
            value: ftd::Value::Record {
                name: "ftd#color".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    (
                        "light".to_string(),
                        ftd::PropertyValue::Value {
                            value: ftd::Value::String {
                                text: light.to_string(),
                                source: ftd::TextSource::Caption,
                            },
                        },
                    ),
                    (
                        "dark".to_string(),
                        ftd::PropertyValue::Value {
                            value: ftd::Value::String {
                                text: dark.to_string(),
                                source: ftd::TextSource::Header,
                            },
                        },
                    ),
                ])
                .collect(),
            },
        }
    }

    fn link(light: &str, dark: &str) -> (String, ftd::PropertyValue) {
        ("link".to_string(), theme_color(light, dark))
    }

    fn code(light: &str, dark: &str) -> (String, ftd::PropertyValue) {
        ("code".to_string(), theme_color(light, dark))
    }

    fn link_visited(light: &str, dark: &str) -> (String, ftd::PropertyValue) {
        ("link-visited".to_string(), theme_color(light, dark))
    }

    fn link_code(light: &str, dark: &str) -> (String, ftd::PropertyValue) {
        ("link-code".to_string(), theme_color(light, dark))
    }

    fn link_visited_code(light: &str, dark: &str) -> (String, ftd::PropertyValue) {
        ("link-visited-code".to_string(), theme_color(light, dark))
    }

    fn ul_ol_li_before(light: &str, dark: &str) -> (String, ftd::PropertyValue) {
        ("ul-ol-li-before".to_string(), theme_color(light, dark))
    }

    fn blockquote(light: &str, dark: &str) -> (String, ftd::PropertyValue) {
        ("blockquote".to_string(), theme_color(light, dark))
    }

    pub fn color() -> ftd::p2::Thing {
        ftd::p2::Thing::Variable(ftd::Variable {
            name: "ftd#markdown-color".to_string(),
            value: ftd::PropertyValue::Value {
                value: ftd::Value::Record {
                    name: "ftd#markdown-color-data".to_string(),
                    fields: std::iter::IntoIterator::into_iter([
                        link("#136351", "#25c19f"),
                        code("#000000", "#25c19f"),
                        link_visited("#7b3ee8", "#0f5750"),
                        link_code("#136351", "#25c19f"),
                        link_visited_code("#136351", "#0f5750"),
                        ul_ol_li_before("#000000", "#ffffff"),
                    ])
                    .collect(),
                },
            },
            conditions: vec![],
            flags: ftd::VariableFlags {
                always_include: Some(true),
            },
        })
    }

    pub fn background_color() -> ftd::p2::Thing {
        ftd::p2::Thing::Variable(ftd::Variable {
            name: "ftd#markdown-background-color".to_string(),
            value: ftd::PropertyValue::Value {
                value: ftd::Value::Record {
                    name: "ftd#markdown-background-color-data".to_string(),
                    fields: std::iter::IntoIterator::into_iter([
                        link("#136351", "#25c19f"),
                        code("#f6f7f8", "#ffffff"),
                        link_visited("#7b3ee8", "#0f5750"),
                        link_code("#136351", "#25c19f"),
                        link_visited_code("#136351", "#0f5750"),
                        ul_ol_li_before("#000000", "#ffffff"),
                        blockquote("#f6f7f8", "#f0f0f0"),
                    ])
                    .collect(),
                },
            },
            conditions: vec![],
            flags: ftd::VariableFlags {
                always_include: Some(true),
            },
        })
    }
}
