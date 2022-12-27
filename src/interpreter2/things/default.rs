use ftd::evalexpr::ContextWithMutableFunctions;

pub fn default_aliases() -> ftd::Map<String> {
    std::iter::IntoIterator::into_iter([("ftd".to_string(), "ftd".to_string())]).collect()
}

pub fn default_functions() -> ftd::Map<ftd::evalexpr::Function> {
    use ftd::evalexpr::*;

    std::iter::IntoIterator::into_iter([
        (
            "isempty".to_string(),
            Function::new(|argument| {
                if argument.as_empty().is_ok() {
                    Ok(Value::Boolean(true))
                } else if let Ok(s) = argument.as_string() {
                    Ok(Value::Boolean(s.is_empty()))
                } else if let Ok(s) = argument.as_tuple() {
                    Ok(Value::Boolean(s.is_empty()))
                } else {
                    Ok(Value::Boolean(false)) //todo: throw error
                }
            }),
        ),
        (
            "enable_dark_mode".to_string(),
            Function::new(|_| Ok(Value::Empty)),
        ),
        (
            "enable_light_mode".to_string(),
            Function::new(|_| Ok(Value::Empty)),
        ),
        (
            "enable_system_mode".to_string(),
            Function::new(|_| Ok(Value::Empty)),
        ),
    ])
    .collect()
}

pub fn default_context() -> ftd::interpreter2::Result<ftd::evalexpr::HashMapContext> {
    let mut context = ftd::evalexpr::HashMapContext::new();
    for (key, function) in default_functions() {
        context.set_function(key, function)?;
    }
    Ok(context)
}

pub fn default_bag() -> ftd::Map<ftd::interpreter2::Thing> {
    let record = |n: &str, r: &str| (n.to_string(), ftd::interpreter2::Kind::record(r));
    let _color = |n: &str| record(n, "ftd#color");
    std::iter::IntoIterator::into_iter([
        (
            "ftd#row".to_string(),
            ftd::interpreter2::Thing::Component(row_function()),
        ),
        (
            "ftd#column".to_string(),
            ftd::interpreter2::Thing::Component(column_function()),
        ),
        (
            "ftd#text".to_string(),
            ftd::interpreter2::Thing::Component(markup_function()),
        ),
        (
            "ftd#integer".to_string(),
            ftd::interpreter2::Thing::Component(integer_function()),
        ),
        (
            "ftd#boolean".to_string(),
            ftd::interpreter2::Thing::Component(boolean_function()),
        ),
        (
            "ftd#image".to_string(),
            ftd::interpreter2::Thing::Component(image_function()),
        ),
        (
            ftd::interpreter2::FTD_IMAGE_SRC.to_string(),
            ftd::interpreter2::Thing::Record(ftd::interpreter2::Record {
                name: ftd::interpreter2::FTD_IMAGE_SRC.to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    ftd::interpreter2::Field {
                        name: "light".to_string(),
                        kind: ftd::interpreter2::Kind::string().into_kind_data().caption(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "dark".to_string(),
                        kind: ftd::interpreter2::Kind::string().into_kind_data(),
                        mutable: false,
                        value: Some(ftd::interpreter2::PropertyValue::Reference {
                            name: ftd::interpreter2::FTD_IMAGE_SRC_LIGHT.to_string(),
                            kind: ftd::interpreter2::Kind::string().into_kind_data(),
                            source: ftd::interpreter2::PropertyValueSource::Local(
                                ftd::interpreter2::FTD_IMAGE_SRC.to_string(),
                            ),
                            is_mutable: false,
                            line_number: 0,
                        }),
                        line_number: 0,
                    },
                ])
                .collect(),
                line_number: 0,
            }),
        ),
        (
            ftd::interpreter2::FTD_COLOR.to_string(),
            ftd::interpreter2::Thing::Record(ftd::interpreter2::Record {
                name: ftd::interpreter2::FTD_COLOR.to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    ftd::interpreter2::Field {
                        name: "light".to_string(),
                        kind: ftd::interpreter2::Kind::string().into_kind_data().caption(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "dark".to_string(),
                        kind: ftd::interpreter2::Kind::string().into_kind_data(),
                        mutable: false,
                        value: Some(ftd::interpreter2::PropertyValue::Reference {
                            name: ftd::interpreter2::FTD_COLOR_LIGHT.to_string(),
                            kind: ftd::interpreter2::Kind::string().into_kind_data(),
                            source: ftd::interpreter2::PropertyValueSource::Local(
                                ftd::interpreter2::FTD_COLOR.to_string(),
                            ),
                            is_mutable: false,
                            line_number: 0,
                        }),
                        line_number: 0,
                    },
                ])
                .collect(),
                line_number: 0,
            }),
        ),
        (
            ftd::interpreter2::FTD_BACKGROUND.to_string(),
            ftd::interpreter2::Thing::OrType(ftd::interpreter2::OrType {
                name: ftd::interpreter2::FTD_BACKGROUND.to_string(),
                variants: vec![ftd::interpreter2::OrTypeVariant::Regular(
                    ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_BACKGROUND_SOLID,
                        ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                            .into_kind_data(),
                        false,
                        None,
                        0,
                    ),
                )],
                line_number: 0,
            }),
        ),
        (
            ftd::interpreter2::FTD_ALIGN.to_string(),
            ftd::interpreter2::Thing::OrType(ftd::interpreter2::OrType {
                name: ftd::interpreter2::FTD_ALIGN.to_string(),
                variants: vec![
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_ALIGN_TOP_LEFT,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string(
                                ftd::interpreter2::FTD_ALIGN_TOP_LEFT,
                            )
                            .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_ALIGN_TOP_CENTER,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string(
                                ftd::interpreter2::FTD_ALIGN_TOP_CENTER,
                            )
                            .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_ALIGN_TOP_RIGHT,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string(
                                ftd::interpreter2::FTD_ALIGN_TOP_RIGHT,
                            )
                            .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_ALIGN_LEFT,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string(ftd::interpreter2::FTD_ALIGN_LEFT)
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_ALIGN_CENTER,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string(
                                ftd::interpreter2::FTD_ALIGN_CENTER,
                            )
                            .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_ALIGN_RIGHT,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string(
                                ftd::interpreter2::FTD_ALIGN_RIGHT,
                            )
                            .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_ALIGN_BOTTOM_LEFT,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string(
                                ftd::interpreter2::FTD_ALIGN_BOTTOM_LEFT,
                            )
                            .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_ALIGN_BOTTOM_CENTER,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string(
                                ftd::interpreter2::FTD_ALIGN_BOTTOM_CENTER,
                            )
                            .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_ALIGN_BOTTOM_RIGHT,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string(
                                ftd::interpreter2::FTD_ALIGN_BOTTOM_RIGHT,
                            )
                            .into_property_value(false, 0),
                        ),
                        0,
                    )),
                ],
                line_number: 0,
            }),
        ),
        (
            ftd::interpreter2::FTD_SPACING_MODE.to_string(),
            ftd::interpreter2::Thing::OrType(ftd::interpreter2::OrType {
                name: ftd::interpreter2::FTD_SPACING_MODE.to_string(),
                variants: vec![
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_SPACING_MODE_SPACE_BETWEEN,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("space-between")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_SPACING_MODE_SPACE_EVENLY,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("space-evenly")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_SPACING_MODE_SPACE_AROUND,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("space-around")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                ],
                line_number: 0,
            }),
        ),
        (
            ftd::interpreter2::FTD_OVERFLOW.to_string(),
            ftd::interpreter2::Thing::OrType(ftd::interpreter2::OrType {
                name: ftd::interpreter2::FTD_OVERFLOW.to_string(),
                variants: vec![
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_OVERFLOW_SCROLL,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("scroll")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_OVERFLOW_VISIBLE,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("visible")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_OVERFLOW_HIDDEN,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("hidden")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_OVERFLOW_AUTO,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("auto")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                ],
                line_number: 0,
            }),
        ),
        (
            ftd::interpreter2::FTD_ALIGN_SELF.to_string(),
            ftd::interpreter2::Thing::OrType(ftd::interpreter2::OrType {
                name: ftd::interpreter2::FTD_ALIGN_SELF.to_string(),
                variants: vec![
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_ALIGN_SELF_START,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("start")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_ALIGN_SELF_CENTER,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("center")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_ALIGN_SELF_END,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("end")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                ],
                line_number: 0,
            }),
        ),
        (
            ftd::interpreter2::FTD_TEXT_ALIGN.to_string(),
            ftd::interpreter2::Thing::OrType(ftd::interpreter2::OrType {
                name: ftd::interpreter2::FTD_TEXT_ALIGN.to_string(),
                variants: vec![
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_TEXT_ALIGN_START,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("start")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_TEXT_ALIGN_CENTER,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("center")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_TEXT_ALIGN_END,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("end")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_TEXT_ALIGN_JUSTIFY,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("justify")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                ],
                line_number: 0,
            }),
        ),
        (
            ftd::interpreter2::FTD_RESIZING.to_string(),
            ftd::interpreter2::Thing::OrType(ftd::interpreter2::OrType {
                name: ftd::interpreter2::FTD_RESIZING.to_string(),
                variants: vec![
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_RESIZING_HUG_CONTENT,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string(
                                ftd::interpreter2::FTD_RESIZING_HUG_CONTENT,
                            )
                            .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_RESIZING_FILL_CONTAINER,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string(
                                ftd::interpreter2::FTD_RESIZING_FILL_CONTAINER,
                            )
                            .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Regular(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_RESIZING_FIXED,
                        ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                            .into_kind_data(),
                        false,
                        None,
                        0,
                    )),
                ],
                line_number: 0,
            }),
        ),
        (
            ftd::interpreter2::FTD_LENGTH.to_string(),
            ftd::interpreter2::Thing::OrType(ftd::interpreter2::OrType {
                name: ftd::interpreter2::FTD_LENGTH.to_string(),
                variants: vec![
                    ftd::interpreter2::OrTypeVariant::new_record(ftd::interpreter2::Record {
                        name: ftd::interpreter2::FTD_LENGTH_PX.to_string(),
                        fields: std::iter::IntoIterator::into_iter([ftd::interpreter2::Field {
                            name: ftd::interpreter2::FTD_LENGTH_VALUE.to_string(),
                            kind: ftd::interpreter2::Kind::integer()
                                .into_kind_data()
                                .caption(),
                            mutable: false,
                            value: None,
                            line_number: 0,
                        }])
                        .collect(),
                        line_number: 0,
                    }),
                    ftd::interpreter2::OrTypeVariant::new_record(ftd::interpreter2::Record {
                        name: ftd::interpreter2::FTD_LENGTH_PERCENT.to_string(),
                        fields: std::iter::IntoIterator::into_iter([ftd::interpreter2::Field {
                            name: ftd::interpreter2::FTD_LENGTH_VALUE.to_string(),
                            kind: ftd::interpreter2::Kind::decimal()
                                .into_kind_data()
                                .caption(),
                            mutable: false,
                            value: None,
                            line_number: 0,
                        }])
                        .collect(),
                        line_number: 0,
                    }),
                    ftd::interpreter2::OrTypeVariant::new_record(ftd::interpreter2::Record {
                        name: ftd::interpreter2::FTD_LENGTH_CALC.to_string(),
                        fields: std::iter::IntoIterator::into_iter([ftd::interpreter2::Field {
                            name: ftd::interpreter2::FTD_LENGTH_VALUE.to_string(),
                            kind: ftd::interpreter2::Kind::string().into_kind_data().caption(),
                            mutable: false,
                            value: None,
                            line_number: 0,
                        }])
                        .collect(),
                        line_number: 0,
                    }),
                    ftd::interpreter2::OrTypeVariant::new_record(ftd::interpreter2::Record {
                        name: ftd::interpreter2::FTD_LENGTH_VH.to_string(),
                        fields: std::iter::IntoIterator::into_iter([ftd::interpreter2::Field {
                            name: ftd::interpreter2::FTD_LENGTH_VALUE.to_string(),
                            kind: ftd::interpreter2::Kind::decimal()
                                .into_kind_data()
                                .caption(),
                            mutable: false,
                            value: None,
                            line_number: 0,
                        }])
                        .collect(),
                        line_number: 0,
                    }),
                    ftd::interpreter2::OrTypeVariant::new_record(ftd::interpreter2::Record {
                        name: ftd::interpreter2::FTD_LENGTH_VW.to_string(),
                        fields: std::iter::IntoIterator::into_iter([ftd::interpreter2::Field {
                            name: ftd::interpreter2::FTD_LENGTH_VALUE.to_string(),
                            kind: ftd::interpreter2::Kind::decimal()
                                .into_kind_data()
                                .caption(),
                            mutable: false,
                            value: None,
                            line_number: 0,
                        }])
                        .collect(),
                        line_number: 0,
                    }),
                    ftd::interpreter2::OrTypeVariant::new_record(ftd::interpreter2::Record {
                        name: ftd::interpreter2::FTD_LENGTH_EM.to_string(),
                        fields: std::iter::IntoIterator::into_iter([ftd::interpreter2::Field {
                            name: ftd::interpreter2::FTD_LENGTH_VALUE.to_string(),
                            kind: ftd::interpreter2::Kind::decimal()
                                .into_kind_data()
                                .caption(),
                            mutable: false,
                            value: None,
                            line_number: 0,
                        }])
                        .collect(),
                        line_number: 0,
                    }),
                    ftd::interpreter2::OrTypeVariant::new_record(ftd::interpreter2::Record {
                        name: ftd::interpreter2::FTD_LENGTH_REM.to_string(),
                        fields: std::iter::IntoIterator::into_iter([ftd::interpreter2::Field {
                            name: ftd::interpreter2::FTD_LENGTH_VALUE.to_string(),
                            kind: ftd::interpreter2::Kind::decimal()
                                .into_kind_data()
                                .caption(),
                            mutable: false,
                            value: None,
                            line_number: 0,
                        }])
                        .collect(),
                        line_number: 0,
                    }),
                ],
                line_number: 0,
            }),
        ),
        (
            "ftd#dark-mode".to_string(),
            ftd::interpreter2::Thing::Variable(ftd::interpreter2::Variable {
                name: "ftd#dark-mode".to_string(),
                kind: ftd::interpreter2::Kind::boolean().into_kind_data(),
                mutable: true,
                value: ftd::interpreter2::PropertyValue::Value {
                    value: ftd::interpreter2::Value::Boolean { value: false },
                    is_mutable: true,
                    line_number: 0,
                },
                conditional_value: vec![],
                line_number: 0,
                is_static: false,
            }),
        ),
        (
            "ftd#system-dark-mode".to_string(),
            ftd::interpreter2::Thing::Variable(ftd::interpreter2::Variable {
                name: "ftd#system-dark-mode".to_string(),
                kind: ftd::interpreter2::Kind::boolean().into_kind_data(),
                mutable: true,
                value: ftd::interpreter2::PropertyValue::Value {
                    value: ftd::interpreter2::Value::Boolean { value: false },
                    is_mutable: true,
                    line_number: 0,
                },
                conditional_value: vec![],
                line_number: 0,
                is_static: false,
            }),
        ),
        (
            "ftd#follow-system-dark-mode".to_string(),
            ftd::interpreter2::Thing::Variable(ftd::interpreter2::Variable {
                name: "ftd#follow-system-dark-mode".to_string(),
                kind: ftd::interpreter2::Kind::boolean().into_kind_data(),
                mutable: true,
                value: ftd::interpreter2::PropertyValue::Value {
                    value: ftd::interpreter2::Value::Boolean { value: true },
                    is_mutable: true,
                    line_number: 0,
                },
                conditional_value: vec![],
                line_number: 0,
                is_static: false,
            }),
        ),
        (
            "ftd#enable-dark-mode".to_string(),
            ftd::interpreter2::Thing::Function(ftd::interpreter2::Function {
                name: "ftd#enable-dark-mode".to_string(),
                return_kind: ftd::interpreter2::Kind::void().into_kind_data(),
                arguments: vec![],
                expression: vec![],
                line_number: 0,
            }),
        ),
    ])
    .collect()
}

pub fn image_function() -> ftd::interpreter2::ComponentDefinition {
    ftd::interpreter2::ComponentDefinition {
        name: "ftd#image".to_string(),
        arguments: [
            common_arguments(),
            vec![ftd::interpreter2::Argument::default(
                "src",
                ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_IMAGE_SRC)
                    .into_kind_data()
                    .caption(),
            )],
        ]
        .concat()
        .into_iter()
        .collect(),
        definition: ftd::interpreter2::Component::from_name("ftd.kernel"),
        line_number: 0,
    }
}

pub fn boolean_function() -> ftd::interpreter2::ComponentDefinition {
    ftd::interpreter2::ComponentDefinition {
        name: "ftd#boolean".to_string(),
        arguments: [
            text_arguments(),
            common_arguments(),
            vec![
                ftd::interpreter2::Argument::default(
                    "value",
                    ftd::interpreter2::Kind::boolean()
                        .into_kind_data()
                        .caption_or_body(),
                ),
                ftd::interpreter2::Argument::default(
                    "style",
                    ftd::interpreter2::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
                ftd::interpreter2::Argument::default(
                    "role",
                    ftd::interpreter2::Kind::record("ftd#type")
                        .into_optional()
                        .into_kind_data(),
                ),
                ftd::interpreter2::Argument::default(
                    "format",
                    ftd::interpreter2::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
                ftd::interpreter2::Argument::default(
                    "text-indent",
                    ftd::interpreter2::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
                ftd::interpreter2::Argument::default(
                    "text-align",
                    ftd::interpreter2::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
            ],
        ]
        .concat()
        .into_iter()
        .collect(),
        definition: ftd::interpreter2::Component::from_name("ftd.kernel"),
        line_number: 0,
    }
}

pub fn integer_function() -> ftd::interpreter2::ComponentDefinition {
    ftd::interpreter2::ComponentDefinition {
        name: "ftd#integer".to_string(),
        arguments: [
            text_arguments(),
            common_arguments(),
            vec![
                ftd::interpreter2::Argument::default(
                    "value",
                    ftd::interpreter2::Kind::integer()
                        .into_kind_data()
                        .caption_or_body(),
                ),
                ftd::interpreter2::Argument::default(
                    "style",
                    ftd::interpreter2::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
                ftd::interpreter2::Argument::default(
                    "role",
                    ftd::interpreter2::Kind::record("ftd#type")
                        .into_optional()
                        .into_kind_data(),
                ),
                ftd::interpreter2::Argument::default(
                    "format",
                    ftd::interpreter2::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
                ftd::interpreter2::Argument::default(
                    "text-indent",
                    ftd::interpreter2::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
                ftd::interpreter2::Argument::default(
                    "text-align",
                    ftd::interpreter2::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
            ],
        ]
        .concat()
        .into_iter()
        .collect(),
        definition: ftd::interpreter2::Component::from_name("ftd.kernel"),
        line_number: 0,
    }
}

pub fn markup_function() -> ftd::interpreter2::ComponentDefinition {
    ftd::interpreter2::ComponentDefinition {
        name: "ftd#text".to_string(),
        arguments: [
            text_arguments(),
            common_arguments(),
            vec![
                ftd::interpreter2::Argument::default(
                    "text",
                    ftd::interpreter2::Kind::string()
                        .into_kind_data()
                        .caption_or_body(),
                ),
                ftd::interpreter2::Argument::default(
                    "style",
                    ftd::interpreter2::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
                ftd::interpreter2::Argument::default(
                    "role",
                    ftd::interpreter2::Kind::record("ftd#type")
                        .into_optional()
                        .into_kind_data(),
                ),
                ftd::interpreter2::Argument::default(
                    "line-clamp",
                    ftd::interpreter2::Kind::integer()
                        .into_optional()
                        .into_kind_data(),
                ),
                ftd::interpreter2::Argument::default(
                    "text-indent",
                    ftd::interpreter2::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
                ftd::interpreter2::Argument::default(
                    "text-align",
                    ftd::interpreter2::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
            ],
        ]
        .concat()
        .into_iter()
        .collect(),
        definition: ftd::interpreter2::Component::from_name("ftd.kernel"),
        line_number: 0,
    }
}

pub fn row_function() -> ftd::interpreter2::ComponentDefinition {
    ftd::interpreter2::ComponentDefinition {
        name: "ftd#row".to_string(),
        arguments: [container_arguments(), common_arguments()]
            .concat()
            .into_iter()
            .collect(),
        definition: ftd::interpreter2::Component::from_name("ftd.kernel"),
        line_number: 0,
    }
}

pub fn column_function() -> ftd::interpreter2::ComponentDefinition {
    ftd::interpreter2::ComponentDefinition {
        name: "ftd#column".to_string(),
        arguments: [container_arguments(), common_arguments()]
            .concat()
            .into_iter()
            .collect(),
        definition: ftd::interpreter2::Component::from_name("ftd.kernel"),
        line_number: 0,
    }
}

fn container_arguments() -> Vec<ftd::interpreter2::Argument> {
    vec![
        ftd::interpreter2::Argument::default(
            "spacing",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "wrap",
            ftd::interpreter2::Kind::boolean()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "align-content",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_ALIGN)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "spacing-mode",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_SPACING_MODE)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "children",
            ftd::interpreter2::Kind::subsection_ui()
                .into_list()
                .into_kind_data(),
        ),
    ]
}

fn common_arguments() -> Vec<ftd::interpreter2::Argument> {
    vec![
        ftd::interpreter2::Argument::default(
            "open-in-new-tab",
            ftd::interpreter2::Kind::boolean()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "overflow",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_OVERFLOW)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "overflow-x",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_OVERFLOW)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "overflow-y",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_OVERFLOW)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "align-self",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_ALIGN_SELF)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "background",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_BACKGROUND)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-color",
            ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "color",
            ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "max-width",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "min-width",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "min-height",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "max-height",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "width",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_RESIZING)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "height",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_RESIZING)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "padding",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "padding-vertical",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "padding-horizontal",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "padding-left",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "padding-right",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "padding-top",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "padding-bottom",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "margin",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "margin-vertical",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "margin-horizontal",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "margin-left",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "margin-right",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "margin-top",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "margin-bottom",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-width",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-bottom-width",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-bottom-color",
            ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-top-width",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-top-color",
            ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-left-width",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-left-color",
            ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-right-width",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-right-color",
            ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-radius",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-top-left-radius",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-top-right-radius",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-bottom-left-radius",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-bottom-right-radius",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "link",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "width",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "min-width",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "max-width",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "height",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "min-height",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "max-height",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "region",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "color",
            ftd::interpreter2::Kind::record("ftd#color")
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "background-color",
            ftd::interpreter2::Kind::Record {
                name: "ftd#color".to_string(),
            }
            .into_optional()
            .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-color",
            ftd::interpreter2::Kind::record("ftd#color")
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "id",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "overflow-x",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "overflow-y",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-top",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-bottom",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-left",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-right",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-top-color",
            ftd::interpreter2::Kind::record("ftd#color")
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-left-color",
            ftd::interpreter2::Kind::record("ftd#color")
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-right-color",
            ftd::interpreter2::Kind::record("ftd#color")
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-bottom-color",
            ftd::interpreter2::Kind::record("ftd#color")
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "margin-top",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "margin-bottom",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "margin-left",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "margin-right",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "submit",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "open-in-new-tab",
            ftd::interpreter2::Kind::boolean()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "sticky",
            ftd::interpreter2::Kind::boolean()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "top",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "bottom",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "left",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "right",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "cursor",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "anchor",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "gradient-direction",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "gradient-colors",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "shadow-offset-x",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "shadow-offset-y",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "shadow-blur",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "shadow-size",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "shadow-color",
            ftd::interpreter2::Kind::record("ftd#color")
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "background-image",
            ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_IMAGE_SRC)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "background-repeat",
            ftd::interpreter2::Kind::boolean()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "background-parallax",
            ftd::interpreter2::Kind::boolean()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "scale",
            ftd::interpreter2::Kind::decimal()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "scale-x",
            ftd::interpreter2::Kind::decimal()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "scale-y",
            ftd::interpreter2::Kind::decimal()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "rotate",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "move-up",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "move-down",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "move-left",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "move-right",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "position",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "z-index",
            ftd::interpreter2::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "slot",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "white-space",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "border-style",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "text-transform",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        /*ftd::interpreter2::Argument::default(
            "grid-column".to_string(),
            ftd::interpreter2::Kind::string().into_optional().into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "grid-row".to_string(),
            ftd::interpreter2::Kind::string().into_optional().into_kind_data(),
        ),*/
    ]
}

fn text_arguments() -> Vec<ftd::interpreter2::Argument> {
    vec![ftd::interpreter2::Argument::default(
        "text-align",
        ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_TEXT_ALIGN)
            .into_optional()
            .into_kind_data(),
    )]
}

/*fn kernel_component() -> ftd::interpreter2::ComponentDefinition {
    ftd::interpreter2::ComponentDefinition {
        name: "ftd.kernel".to_string(),
        arguments: vec![],
        definition: ftd::interpreter2::Component {
            name: "ftd.kernel".to_string(),
            properties: vec![],
            iteration: Box::new(None),
            condition: Box::new(None),
            events: vec![],
            children: vec![],
            line_number: 0,
        },
        line_number: 0,
    }
}*/
