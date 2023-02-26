use ftd::evalexpr::ContextWithMutableFunctions;

/**
* The `default_aliases` function is intended to provide default aliases for the `ftd` module,
* with the only default alias being "ftd" itself. This allows users to reference the `ftd` module
* using this alias instead of the full module name.
**/
pub fn default_aliases() -> ftd::Map<String> {
    std::iter::IntoIterator::into_iter([
        ("ftd".to_string(), "ftd".to_string()),
        ("inherited".to_string(), "inherited".to_string()),
    ])
    .collect()
}

/**
The `default_functions` function returns a map of string keys to Function values. These functions
are built-in and available for use in the evaluation of an expression.

1. `is_empty` - This function takes an argument and returns a boolean value indicating whether or not
the argument is empty. It checks for empty values, strings, and tuples.

2. `enable_dark_mode` - This function takes no arguments and returns an empty value. It is used to
enable dark mode in the application.

3. `enable_light_mode` - This function takes no arguments and returns an empty value. It is used to
enable light mode in the application.

4. `enable_system_mode` - This function takes no arguments and returns an empty value. It is used to
enable system mode in the application, which means the application will use the system's default
color scheme.
**/
pub fn default_functions() -> ftd::Map<ftd::evalexpr::Function> {
    use ftd::evalexpr::*;

    std::iter::IntoIterator::into_iter([
        (
            "ftd.is_empty".to_string(),
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
            "ftd.append".to_string(),
            Function::new(|argument| {
                if let Ok(s) = argument.as_tuple() {
                    if s.len() != 2 {
                        Err(
                            ftd::evalexpr::error::EvalexprError::WrongOperatorArgumentAmount {
                                expected: 2,
                                actual: s.len(),
                            },
                        )
                    } else {
                        let mut argument = s.first().unwrap().as_tuple()?;
                        let value = s.last().unwrap();
                        argument.push(value.to_owned());
                        Ok(Value::Tuple(argument))
                    }
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

/**
The `default_bag` function is a public function that returns a `Map` of `Thing`s.

The `Map` is a data structure that stores key-value pairs in a hash table. In this case, the keys
are `String`s representing the names of different `Thing`s, and the values are the `Thing`s
themselves.
**/
pub fn default_bag() -> ftd::Map<ftd::interpreter2::Thing> {
    let record = |n: &str, r: &str| (n.to_string(), ftd::interpreter2::Kind::record(r));
    let _color = |n: &str| record(n, "ftd#color");
    std::iter::IntoIterator::into_iter([
        (
            "ftd#row".to_string(),
            ftd::interpreter2::Thing::Component(row_function()),
        ),
        (
            "ftd#code".to_string(),
            ftd::interpreter2::Thing::Component(code_function()),
        ),
        (
            "ftd#iframe".to_string(),
            ftd::interpreter2::Thing::Component(iframe_function()),
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
            "ftd#decimal".to_string(),
            ftd::interpreter2::Thing::Component(decimal_function()),
        ),
        (
            "ftd#boolean".to_string(),
            ftd::interpreter2::Thing::Component(boolean_function()),
        ),
        (
            "ftd#text-input".to_string(),
            ftd::interpreter2::Thing::Component(text_input_function()),
        ),
        (
            "ftd#checkbox".to_string(),
            ftd::interpreter2::Thing::Component(checkbox_function()),
        ),
        (
            "ftd#image".to_string(),
            ftd::interpreter2::Thing::Component(image_function()),
        ),
        (
            "ftd#toggle".to_string(),
            ftd::interpreter2::Thing::Function(ftd::interpreter2::Function {
                name: "ftd#toggle".to_string(),
                return_kind: ftd::interpreter2::KindData {
                    kind: ftd::interpreter2::Kind::void(),
                    caption: false,
                    body: false,
                },
                arguments: vec![
                    ftd::interpreter2::Argument {
                        name: "a".to_string(),
                        kind: ftd::interpreter2::KindData {
                            kind: ftd::interpreter2::Kind::boolean(),
                            caption: false,
                            body: false,
                        },
                        mutable: true,
                        value: None,
                        line_number: 0,
                    },
                ],
                expression: vec![
                    ftd::interpreter2::things::function::Expression {
                        expression: "a = !a".to_string(),
                        line_number: 0,
                    }
                ],
                js: None,
                line_number: 0,
            })
        ),
        (
            "ftd#increment".to_string(),
            ftd::interpreter2::Thing::Function(ftd::interpreter2::Function {
                name: "ftd#increment".to_string(),
                return_kind: ftd::interpreter2::KindData {
                    kind: ftd::interpreter2::Kind::void(),
                    caption: false,
                    body: false,
                },
                arguments: vec![
                    ftd::interpreter2::Argument {
                        name: "a".to_string(),
                        kind: ftd::interpreter2::KindData {
                            kind: ftd::interpreter2::Kind::integer(),
                            caption: false,
                            body: false,
                        },
                        mutable: true,
                        value: None,
                        line_number: 0,
                    },
                ],
                expression: vec![
                    ftd::interpreter2::things::function::Expression {
                        expression: "a += 1".to_string(),
                        line_number: 0,
                    }
                ],
                js: None,
                line_number: 0,
            })
        ),
        (
            "ftd#increment-by".to_string(),
            ftd::interpreter2::Thing::Function(ftd::interpreter2::Function {
                name: "ftd#increment-by".to_string(),
                return_kind: ftd::interpreter2::KindData {
                    kind: ftd::interpreter2::Kind::void(),
                    caption: false,
                    body: false,
                },
                arguments: vec![
                    ftd::interpreter2::Argument {
                        name: "a".to_string(),
                        kind: ftd::interpreter2::KindData {
                            kind: ftd::interpreter2::Kind::integer(),
                            caption: false,
                            body: false,
                        },
                        mutable: true,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Argument {
                        name: "v".to_string(),
                        kind: ftd::interpreter2::KindData {
                            kind: ftd::interpreter2::Kind::integer(),
                            caption: false,
                            body: false,
                        },
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                ],
                expression: vec![
                    ftd::interpreter2::things::function::Expression {
                        expression: "a += v".to_string(),
                        line_number: 0,
                    }
                ],
                js: None,
                line_number: 0,
            })
        ),
        (
            "ftd#enable-light-mode".to_string(),
            ftd::interpreter2::Thing::Function(ftd::interpreter2::Function {
                name: "ftd#enable-light-mode".to_string(),
                return_kind: ftd::interpreter2::KindData {
                    kind: ftd::interpreter2::Kind::void(),
                    caption: false,
                    body: false,
                },
                arguments: vec![
                ],
                expression: vec![
                    ftd::interpreter2::things::function::Expression {
                        expression: "enable_light_mode()".to_string(),
                        line_number: 0,
                    }
                ],
                js: None,
                line_number: 0,
            })
        ),
        (
            "ftd#enable-dark-mode".to_string(),
            ftd::interpreter2::Thing::Function(ftd::interpreter2::Function {
                name: "ftd#enable-dark-mode".to_string(),
                return_kind: ftd::interpreter2::KindData {
                    kind: ftd::interpreter2::Kind::void(),
                    caption: false,
                    body: false,
                },
                arguments: vec![
                ],
                expression: vec![
                    ftd::interpreter2::things::function::Expression {
                        expression: "enable_dark_mode()".to_string(),
                        line_number: 0,
                    }
                ],
                js: None,
                line_number: 0,
            })
        ),
        (
            "ftd#enable-system-mode".to_string(),
            ftd::interpreter2::Thing::Function(ftd::interpreter2::Function {
                name: "ftd#enable-system-mode".to_string(),
                return_kind: ftd::interpreter2::KindData {
                    kind: ftd::interpreter2::Kind::void(),
                    caption: false,
                    body: false,
                },
                arguments: vec![
                ],
                expression: vec![
                    ftd::interpreter2::things::function::Expression {
                        expression: "enable_system_mode()".to_string(),
                        line_number: 0,
                    }
                ],
                js: None,
                line_number: 0,
            })
        ),
        (
            "ftd#copy-to-clipboard".to_string(),
            ftd::interpreter2::Thing::Function(ftd::interpreter2::Function {
                name: "ftd#copy-to-clipboard".to_string(),
                return_kind: ftd::interpreter2::KindData {
                    kind: ftd::interpreter2::Kind::void(),
                    caption: false,
                    body: false,
                },
                arguments: vec![
                    ftd::interpreter2::Argument {
                        name: "a".to_string(),
                        kind: ftd::interpreter2::KindData {
                            kind: ftd::interpreter2::Kind::string(),
                            caption: false,
                            body: false,
                        },
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                ],
                expression: vec![
                    ftd::interpreter2::things::function::Expression {
                        expression: "ftd.copy_to_clipboard(a)".to_string(),
                        line_number: 0,
                    }
                ],
                js: None,
                line_number: 0,
            })
        ),
        (
            "ftd#set-bool".to_string(),
            ftd::interpreter2::Thing::Function(ftd::interpreter2::Function {
                name: "ftd#set-bool".to_string(),
                return_kind: ftd::interpreter2::KindData {
                    kind: ftd::interpreter2::Kind::void(),
                    caption: false,
                    body: false,
                },
                arguments: vec![
                    ftd::interpreter2::Argument {
                        name: "a".to_string(),
                        kind: ftd::interpreter2::KindData {
                            kind: ftd::interpreter2::Kind::boolean(),
                            caption: false,
                            body: false,
                        },
                        mutable: true,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Argument {
                        name: "v".to_string(),
                        kind: ftd::interpreter2::KindData {
                            kind: ftd::interpreter2::Kind::boolean(),
                            caption: false,
                            body: false,
                        },
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                ],
                expression: vec![
                    ftd::interpreter2::things::function::Expression {
                        expression: "a = v".to_string(),
                        line_number: 0,
                    }
                ],
                js: None,
                line_number: 0,
            })
        ),
        (
            "ftd#set-string".to_string(),
            ftd::interpreter2::Thing::Function(ftd::interpreter2::Function {
                name: "ftd#set-string".to_string(),
                return_kind: ftd::interpreter2::KindData {
                    kind: ftd::interpreter2::Kind::void(),
                    caption: false,
                    body: false,
                },
                arguments: vec![
                    ftd::interpreter2::Argument {
                        name: "a".to_string(),
                        kind: ftd::interpreter2::KindData {
                            kind: ftd::interpreter2::Kind::string(),
                            caption: false,
                            body: false,
                        },
                        mutable: true,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Argument {
                        name: "v".to_string(),
                        kind: ftd::interpreter2::KindData {
                            kind: ftd::interpreter2::Kind::string(),
                            caption: false,
                            body: false,
                        },
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                ],
                expression: vec![
                    ftd::interpreter2::things::function::Expression {
                        expression: "a = v".to_string(),
                        line_number: 0,
                    }
                ],
                js: None,
                line_number: 0,
            })
        ),
        (
            "ftd#set-integer".to_string(),
            ftd::interpreter2::Thing::Function(ftd::interpreter2::Function {
                name: "ftd#set-integer".to_string(),
                return_kind: ftd::interpreter2::KindData {
                    kind: ftd::interpreter2::Kind::void(),
                    caption: false,
                    body: false,
                },
                arguments: vec![
                    ftd::interpreter2::Argument {
                        name: "a".to_string(),
                        kind: ftd::interpreter2::KindData {
                            kind: ftd::interpreter2::Kind::integer(),
                            caption: false,
                            body: false,
                        },
                        mutable: true,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Argument {
                        name: "v".to_string(),
                        kind: ftd::interpreter2::KindData {
                            kind: ftd::interpreter2::Kind::integer(),
                            caption: false,
                            body: false,
                        },
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                ],
                expression: vec![
                    ftd::interpreter2::things::function::Expression {
                        expression: "a = v".to_string(),
                        line_number: 0,
                    }
                ],
                js: None,
                line_number: 0,
            })
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
            ftd::interpreter2::FTD_SPACING.to_string(),
            ftd::interpreter2::Thing::OrType(ftd::interpreter2::OrType {
                name: ftd::interpreter2::FTD_SPACING.to_string(),
                variants: vec![
                    ftd::interpreter2::OrTypeVariant::Regular(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_SPACING_FIXED,
                        ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                            .into_kind_data(),
                        false,
                        None,
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_SPACING_SPACE_BETWEEN,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("space-between")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_SPACING_SPACE_EVENLY,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("space-evenly")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_SPACING_SPACE_AROUND,
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
            ftd::interpreter2::FTD_ANCHOR.to_string(),
            ftd::interpreter2::Thing::OrType(ftd::interpreter2::OrType {
                name: ftd::interpreter2::FTD_ANCHOR.to_string(),
                variants: vec![
                    ftd::interpreter2::OrTypeVariant::Regular(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_ANCHOR_ID,
                        ftd::interpreter2::Kind::string()
                            .into_kind_data(),
                        false,
                        None,
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_ANCHOR_PARENT,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("absolute")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_ANCHOR_WINDOW,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("fixed")
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
            ftd::interpreter2::FTD_RESIZE.to_string(),
            ftd::interpreter2::Thing::OrType(ftd::interpreter2::OrType {
                name: ftd::interpreter2::FTD_RESIZE.to_string(),
                variants: vec![
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_RESIZE_HORIZONTAL,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("horizontal")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_RESIZE_VERTICAL,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("vertical")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_RESIZE_BOTH,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("both")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                ],
                line_number: 0,
            }),
        ),
        (
            ftd::interpreter2::FTD_CURSOR.to_string(),
            ftd::interpreter2::Thing::OrType(ftd::interpreter2::OrType {
                name: ftd::interpreter2::FTD_CURSOR.to_string(),
                variants: vec![
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_CURSOR_DEFAULT,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("default")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_CURSOR_NONE,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("none")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_CURSOR_CONTEXT_MENU,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("context-menu")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_CURSOR_HELP,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("help")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_CURSOR_POINTER,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("pointer")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_CURSOR_PROGRESS,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("progress")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_CURSOR_WAIT,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("wait")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_CURSOR_CELL,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("cell")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_CURSOR_CROSSHAIR,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("crosshair")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_CURSOR_TEXT,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("text")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_CURSOR_VERTICAL_TEXT,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("vertical-text")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_CURSOR_ALIAS,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("alias")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_CURSOR_COPY,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("copy")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_CURSOR_MOVE,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("move")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_CURSOR_NO_DROP,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("no-drop")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_CURSOR_NOT_ALLOWED,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("not-allowed")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_CURSOR_GRAB,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("grab")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_CURSOR_GRABBING,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("grabbing")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_CURSOR_E_RESIZE,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("e-resize")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_CURSOR_N_RESIZE,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("n-resize")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_CURSOR_NE_RESIZE,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("ne-resize")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_CURSOR_NW_RESIZE,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("nw-resize")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_CURSOR_S_RESIZE,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("s-resize")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_CURSOR_SE_RESIZE,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("se-resize")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_CURSOR_SW_RESIZE,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("sw-resize")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_CURSOR_W_RESIZE,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("w-resize")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_CURSOR_EW_RESIZE,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("ew-resize")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_CURSOR_NS_RESIZE,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("ns-resize")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_CURSOR_NESW_RESIZE,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("nesw-resize")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_CURSOR_NWSE_RESIZE,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("nwse-resize")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_CURSOR_COL_RESIZE,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("col-resize")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_CURSOR_ROW_RESIZE,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("row-resize")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_CURSOR_ALL_SCROLL,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("all-scroll")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_CURSOR_ZOOM_IN,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("zoom-in")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::new_constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_CURSOR_ZOOM_OUT,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string("zoom-out")
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
                        ftd::interpreter2::FTD_RESIZING_AUTO,
                        ftd::interpreter2::Kind::string().into_kind_data(),
                        false,
                        Some(
                            ftd::interpreter2::Value::new_string(
                                ftd::interpreter2::FTD_RESIZING_AUTO,
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
            ftd::interpreter2::FTD_WHITESPACE.to_string(),
            ftd::interpreter2::Thing::OrType(ftd::interpreter2::OrType {
                name: ftd::interpreter2::FTD_WHITESPACE.to_string(),
                variants: vec![
                    ftd::interpreter2::OrTypeVariant::Constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_WHITESPACE_NORMAL,
                        ftd::interpreter2::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(ftd::interpreter2::Value::new_string("normal")
                                  .into_property_value(false, 0),),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_WHITESPACE_NOWRAP,
                        ftd::interpreter2::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(ftd::interpreter2::Value::new_string("nowrap")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_WHITESPACE_PRE,
                        ftd::interpreter2::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(ftd::interpreter2::Value::new_string("pre")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_WHITESPACE_PREWRAP,
                        ftd::interpreter2::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(ftd::interpreter2::Value::new_string("pre-wrap")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_WHITESPACE_PRELINE,
                        ftd::interpreter2::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(ftd::interpreter2::Value::new_string("pre-line")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_WHITESPACE_BREAKSPACES,
                        ftd::interpreter2::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(ftd::interpreter2::Value::new_string("break-spaces")
                            .into_property_value(false, 0)),
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
                    ftd::interpreter2::OrTypeVariant::Regular(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_LENGTH_PX,
                        ftd::interpreter2::Kind::integer()
                            .into_kind_data()
                            .caption(),
                        false,
                        None,
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Regular(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_LENGTH_PERCENT,
                        ftd::interpreter2::Kind::decimal()
                            .into_kind_data()
                            .caption(),
                        false,
                        None,
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Regular(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_LENGTH_CALC,
                        ftd::interpreter2::Kind::string().into_kind_data().caption(),
                        false,
                        None,
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Regular(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_LENGTH_VH,
                        ftd::interpreter2::Kind::decimal()
                            .into_kind_data()
                            .caption(),
                        false,
                        None,
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Regular(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_LENGTH_VW,
                        ftd::interpreter2::Kind::decimal()
                            .into_kind_data()
                            .caption(),
                        false,
                        None,
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Regular(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_LENGTH_EM,
                        ftd::interpreter2::Kind::decimal()
                            .into_kind_data()
                            .caption(),
                        false,
                        None,
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Regular(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_LENGTH_REM,
                        ftd::interpreter2::Kind::decimal()
                            .into_kind_data()
                            .caption(),
                        false,
                        None,
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Regular(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_LENGTH_RESPONSIVE,
                        ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_RESPONSIVE_LENGTH)
                            .into_kind_data()
                            .caption(),
                        false,
                        None,
                        0,
                    )),
                ],
                line_number: 0,
            }),
        ),
        (
            ftd::interpreter2::FTD_RESPONSIVE_LENGTH.to_string(),
            ftd::interpreter2::Thing::Record(ftd::interpreter2::Record {
                name: ftd::interpreter2::FTD_RESPONSIVE_LENGTH.to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    ftd::interpreter2::Field {
                        name: "desktop".to_string(),
                        kind: ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                            .into_kind_data()
                            .caption(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "mobile".to_string(),
                        kind: ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                            .into_kind_data(),
                        mutable: false,
                        value: Some(ftd::interpreter2::PropertyValue::Reference {
                            name: ftd::interpreter2::FTD_RESPONSIVE_LENGTH_DESKTOP.to_string(),
                            kind: ftd::interpreter2::Kind::string().into_kind_data(),
                            source: ftd::interpreter2::PropertyValueSource::Local(
                                ftd::interpreter2::FTD_RESPONSIVE_LENGTH.to_string(),
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
            ftd::interpreter2::FTD_FONT_SIZE.to_string(),
            ftd::interpreter2::Thing::OrType(ftd::interpreter2::OrType {
                name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                variants: vec![
                    ftd::interpreter2::OrTypeVariant::Regular(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_FONT_SIZE_PX,
                        ftd::interpreter2::Kind::integer()
                            .into_kind_data()
                            .caption(),
                        false,
                        None,
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Regular(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_FONT_SIZE_EM,
                        ftd::interpreter2::Kind::decimal()
                            .into_kind_data()
                            .caption(),
                        false,
                        None,
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Regular(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_FONT_SIZE_REM,
                        ftd::interpreter2::Kind::decimal()
                            .into_kind_data()
                            .caption(),
                        false,
                        None,
                        0,
                    )),
                ],
                line_number: 0,
            }),
        ),
        (
            ftd::interpreter2::FTD_REGION.to_string(),
            ftd::interpreter2::Thing::OrType(ftd::interpreter2::OrType {
                name: ftd::interpreter2::FTD_REGION.to_string(),
                variants: vec![
                    ftd::interpreter2::OrTypeVariant::Constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_REGION_H1,
                        ftd::interpreter2::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(   ftd::interpreter2::Value::new_string("h1")
                                    .into_property_value(false, 0),),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_REGION_H2,
                        ftd::interpreter2::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(ftd::interpreter2::Value::new_string("h2")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_REGION_H3,
                        ftd::interpreter2::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(ftd::interpreter2::Value::new_string("h3")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_REGION_H4,
                        ftd::interpreter2::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(ftd::interpreter2::Value::new_string("h4")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_REGION_H5,
                        ftd::interpreter2::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(ftd::interpreter2::Value::new_string("h5")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_REGION_H6,
                        ftd::interpreter2::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(ftd::interpreter2::Value::new_string("h6")
                            .into_property_value(false, 0)),
                        0,
                    )),
                ],
                line_number: 0,
            }),
        ),
        (
            ftd::interpreter2::FTD_TEXT_INPUT_TYPE.to_string(),
            ftd::interpreter2::Thing::OrType(ftd::interpreter2::OrType {
                name: ftd::interpreter2::FTD_TEXT_INPUT_TYPE.to_string(),
                variants: vec![
                    ftd::interpreter2::OrTypeVariant::Constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_TEXT_INPUT_TYPE_TEXT,
                        ftd::interpreter2::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(   ftd::interpreter2::Value::new_string("text")
                                    .into_property_value(false, 0),),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_TEXT_INPUT_TYPE_EMAIL,
                        ftd::interpreter2::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(ftd::interpreter2::Value::new_string("email")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_TEXT_INPUT_TYPE_PASSWORD,
                        ftd::interpreter2::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(ftd::interpreter2::Value::new_string("password")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_TEXT_INPUT_TYPE_URL,
                        ftd::interpreter2::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(ftd::interpreter2::Value::new_string("url")
                            .into_property_value(false, 0)),
                        0,
                    )),
                ],
                line_number: 0,
            }),
        ),
        (
            ftd::interpreter2::FTD_LOADING.to_string(),
            ftd::interpreter2::Thing::OrType(ftd::interpreter2::OrType {
                name: ftd::interpreter2::FTD_LOADING.to_string(),
                variants: vec![
                    ftd::interpreter2::OrTypeVariant::Constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_LOADING_EAGER,
                        ftd::interpreter2::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(   ftd::interpreter2::Value::new_string("eager")
                                    .into_property_value(false, 0),),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_LOADING_LAZY,
                        ftd::interpreter2::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(ftd::interpreter2::Value::new_string("lazy")
                            .into_property_value(false, 0)),
                        0,
                    )),
                ],
                line_number: 0,
            }),
        ),
        (
            ftd::interpreter2::FTD_BORDER_STYLE.to_string(),
            ftd::interpreter2::Thing::OrType(ftd::interpreter2::OrType {
                name: ftd::interpreter2::FTD_BORDER_STYLE.to_string(),
                variants: vec![
                    ftd::interpreter2::OrTypeVariant::Constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_BORDER_STYLE_DASHED,
                        ftd::interpreter2::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(   ftd::interpreter2::Value::new_string("dashed")
                                    .into_property_value(false, 0),),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_BORDER_STYLE_DOTTED,
                        ftd::interpreter2::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(   ftd::interpreter2::Value::new_string("dotted")
                                    .into_property_value(false, 0),),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_BORDER_STYLE_DOUBLE,
                        ftd::interpreter2::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(   ftd::interpreter2::Value::new_string("double")
                                    .into_property_value(false, 0),),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_BORDER_STYLE_GROOVE,
                        ftd::interpreter2::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(   ftd::interpreter2::Value::new_string("groove")
                                    .into_property_value(false, 0),),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_BORDER_STYLE_INSET,
                        ftd::interpreter2::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(   ftd::interpreter2::Value::new_string("inset")
                                    .into_property_value(false, 0),),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_BORDER_STYLE_OUTSET,
                        ftd::interpreter2::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(   ftd::interpreter2::Value::new_string("outset")
                                    .into_property_value(false, 0),),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_BORDER_STYLE_RIDGE,
                        ftd::interpreter2::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(   ftd::interpreter2::Value::new_string("ridge")
                                    .into_property_value(false, 0),),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_BORDER_STYLE_SOLID,
                        ftd::interpreter2::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(   ftd::interpreter2::Value::new_string("solid")
                                    .into_property_value(false, 0),),
                        0,
                    )),
                ],
                line_number: 0,
            }),
        ),
        (
            ftd::interpreter2::FTD_TEXT_TRANSFORM.to_string(),
            ftd::interpreter2::Thing::OrType(ftd::interpreter2::OrType {
                name: ftd::interpreter2::FTD_TEXT_TRANSFORM.to_string(),
                variants: vec![
                    ftd::interpreter2::OrTypeVariant::Constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_TEXT_TRANSFORM_NONE,
                        ftd::interpreter2::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(   ftd::interpreter2::Value::new_string("none")
                                    .into_property_value(false, 0),),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_TEXT_TRANSFORM_CAPITALIZE,
                        ftd::interpreter2::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(ftd::interpreter2::Value::new_string("capitalize")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_TEXT_TRANSFORM_UPPERCASE,
                        ftd::interpreter2::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(ftd::interpreter2::Value::new_string("uppercase")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_TEXT_TRANSFORM_LOWERCASE,
                        ftd::interpreter2::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(ftd::interpreter2::Value::new_string("lowercase")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_TEXT_TRANSFORM_INITIAL,
                        ftd::interpreter2::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(ftd::interpreter2::Value::new_string("initial")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_TEXT_TRANSFORM_INHERIT,
                        ftd::interpreter2::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(ftd::interpreter2::Value::new_string("inherit")
                            .into_property_value(false, 0)),
                        0,
                    )),
                ],
                line_number: 0,
            }),
        ),
        (
            ftd::interpreter2::FTD_TYPE.to_string(),
            ftd::interpreter2::Thing::Record(ftd::interpreter2::Record {
                name: ftd::interpreter2::FTD_TYPE.to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    ftd::interpreter2::Field {
                        name: "size".to_string(),
                        kind: ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_FONT_SIZE)
                            .into_optional()
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "line-height".to_string(),
                        kind: ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_FONT_SIZE)
                            .into_optional()
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "letter-spacing".to_string(),
                        kind: ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_FONT_SIZE)
                            .into_optional()
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "weight".to_string(),
                        kind: ftd::interpreter2::Kind::integer()
                            .into_optional()
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "font-family".to_string(),
                        kind: ftd::interpreter2::Kind::string()
                            .into_optional()
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                ])
                .collect(),
                line_number: 0,
            }),
        ),
        (
            ftd::interpreter2::FTD_RESPONSIVE_TYPE.to_string(),
            ftd::interpreter2::Thing::Record(ftd::interpreter2::Record {
                name: ftd::interpreter2::FTD_RESPONSIVE_TYPE.to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    ftd::interpreter2::Field {
                        name: "desktop".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_TYPE)
                            .into_kind_data().caption(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "mobile".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_TYPE)
                            .into_kind_data(),
                        mutable: false,
                        value: Some(ftd::interpreter2::PropertyValue::Reference {
                            name: ftd::interpreter2::FTD_RESPONSIVE_TYPE_DESKTOP.to_string(),
                            kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_TYPE)
                                .into_kind_data(),
                            source: ftd::interpreter2::PropertyValueSource::Local(
                                ftd::interpreter2::FTD_RESPONSIVE_TYPE.to_string(),
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
            ftd::interpreter2::FTD_BACKGROUND_COLOR.to_string(),
            ftd::interpreter2::Thing::Record(ftd::interpreter2::Record {
                name: ftd::interpreter2::FTD_BACKGROUND_COLOR.to_string(),
                fields: vec![
                    ftd::interpreter2::Field {
                        name: "base".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "step-1".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "step-2".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "overlay".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "code".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                ],
                line_number: 0,
            }),
        ),
        (
            ftd::interpreter2::FTD_CTA_COLOR.to_string(),
            ftd::interpreter2::Thing::Record(ftd::interpreter2::Record {
                name: ftd::interpreter2::FTD_CTA_COLOR.to_string(),
                fields: vec![
                    ftd::interpreter2::Field {
                        name: "base".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "hover".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "pressed".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "disabled".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "focused".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "border".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "text".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                ],
                line_number: 0,
            }),
        ),
        (
            ftd::interpreter2::FTD_PST.to_string(),
            ftd::interpreter2::Thing::Record(ftd::interpreter2::Record {
                name: ftd::interpreter2::FTD_PST.to_string(),
                fields: vec![
                    ftd::interpreter2::Field {
                        name: "primary".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "secondary".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "tertiary".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                ],
                line_number: 0,
            }),
        ),
        (
            ftd::interpreter2::FTD_BTB.to_string(),
            ftd::interpreter2::Thing::Record(ftd::interpreter2::Record {
                name: ftd::interpreter2::FTD_BTB.to_string(),
                fields: vec![
                    ftd::interpreter2::Field {
                        name: "base".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "text".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "border".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                ],
                line_number: 0,
            }),
        ),
        (
            ftd::interpreter2::FTD_CUSTOM_COLORS.to_string(),
            ftd::interpreter2::Thing::Record(ftd::interpreter2::Record {
                name: ftd::interpreter2::FTD_CUSTOM_COLORS.to_string(),
                fields: vec![
                    ftd::interpreter2::Field {
                        name: "one".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "two".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "three".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "four".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "five".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "six".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "seven".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "eight".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "nine".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "ten".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                ],
                line_number: 0,
            }),
        ),
        (
            ftd::interpreter2::FTD_COLOR_SCHEME.to_string(),
            ftd::interpreter2::Thing::Record(ftd::interpreter2::Record {
                name: ftd::interpreter2::FTD_COLOR_SCHEME.to_string(),
                fields: vec![
                    ftd::interpreter2::Field {
                        name: "background".to_string(),
                        kind: ftd::interpreter2::Kind::record("ftd#background-colors")
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "border".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "border-strong".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "text".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "text-strong".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "shadow".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "scrim".to_string(),
                        kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "cta-primary".to_string(),
                        kind: ftd::interpreter2::Kind::record("ftd#cta-colors").into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "cta-secondary".to_string(),
                        kind: ftd::interpreter2::Kind::record("ftd#cta-colors").into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "cta-tertiary".to_string(),
                        kind: ftd::interpreter2::Kind::record("ftd#cta-colors").into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "cta-danger".to_string(),
                        kind: ftd::interpreter2::Kind::record("ftd#cta-colors").into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "accent".to_string(),
                        kind: ftd::interpreter2::Kind::record("ftd#pst").into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "error".to_string(),
                        kind: ftd::interpreter2::Kind::record("ftd#btb").into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "success".to_string(),
                        kind: ftd::interpreter2::Kind::record("ftd#btb").into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "info".to_string(),
                        kind: ftd::interpreter2::Kind::record("ftd#btb").into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "warning".to_string(),
                        kind: ftd::interpreter2::Kind::record("ftd#btb").into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                    ftd::interpreter2::Field {
                        name: "custom".to_string(),
                        kind: ftd::interpreter2::Kind::record("ftd#custom-colors").into_kind_data(),
                        mutable: false,
                        value: None,
                        line_number: 0,
                    },
                ],
                line_number: 0,
            }),
        ),
        (
            ftd::interpreter2::FTD_TYPE_DATA.to_string(),
            ftd::interpreter2::Thing::Record(ftd::interpreter2::Record {
                name: ftd::interpreter2::FTD_TYPE_DATA.to_string(),
                fields: vec![ftd::interpreter2::Field {
                    name: "heading-large".to_string(),
                    kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_RESPONSIVE_TYPE)
                        .into_kind_data(),
                    mutable: false,
                    value: None,
                    line_number: 0
                }, ftd::interpreter2::Field {
                    name: "heading-medium".to_string(),
                    kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_RESPONSIVE_TYPE)
                        .into_kind_data(),
                    mutable: false,
                    value: None,
                    line_number: 0
                }, ftd::interpreter2::Field {
                    name: "heading-small".to_string(),
                    kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_RESPONSIVE_TYPE)
                        .into_kind_data(),
                    mutable: false,
                    value: None,
                    line_number: 0
                },ftd::interpreter2::Field {
                    name: "heading-hero".to_string(),
                    kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_RESPONSIVE_TYPE)
                        .into_kind_data(),
                    mutable: false,
                    value: None,
                    line_number: 0
                },ftd::interpreter2::Field {
                    name: "heading-tiny".to_string(),
                    kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_RESPONSIVE_TYPE)
                        .into_kind_data(),
                    mutable: false,
                    value: None,
                    line_number: 0
                },ftd::interpreter2::Field {
                    name: "copy-small".to_string(),
                    kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_RESPONSIVE_TYPE)
                        .into_kind_data(),
                    mutable: false,
                    value: None,
                    line_number: 0
                },ftd::interpreter2::Field {
                    name: "copy-regular".to_string(),
                    kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_RESPONSIVE_TYPE)
                        .into_kind_data(),
                    mutable: false,
                    value: None,
                    line_number: 0
                },ftd::interpreter2::Field {
                    name: "copy-large".to_string(),
                    kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_RESPONSIVE_TYPE)
                        .into_kind_data(),
                    mutable: false,
                    value: None,
                    line_number: 0
                },ftd::interpreter2::Field {
                    name: "fine-print".to_string(),
                    kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_RESPONSIVE_TYPE)
                        .into_kind_data(),
                    mutable: false,
                    value: None,
                    line_number: 0
                },ftd::interpreter2::Field {
                    name: "blockquote".to_string(),
                    kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_RESPONSIVE_TYPE)
                        .into_kind_data(),
                    mutable: false,
                    value: None,
                    line_number: 0
                },ftd::interpreter2::Field {
                    name: "source-code".to_string(),
                    kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_RESPONSIVE_TYPE)
                        .into_kind_data(),
                    mutable: false,
                    value: None,
                    line_number: 0
                },ftd::interpreter2::Field {
                    name: "button-small".to_string(),
                    kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_RESPONSIVE_TYPE)
                        .into_kind_data(),
                    mutable: false,
                    value: None,
                    line_number: 0
                },ftd::interpreter2::Field {
                    name: "button-medium".to_string(),
                    kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_RESPONSIVE_TYPE)
                        .into_kind_data(),
                    mutable: false,
                    value: None,
                    line_number: 0
                },ftd::interpreter2::Field {
                    name: "button-large".to_string(),
                    kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_RESPONSIVE_TYPE)
                        .into_kind_data(),
                    mutable: false,
                    value: None,
                    line_number: 0
                },ftd::interpreter2::Field {
                    name: "link".to_string(),
                    kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_RESPONSIVE_TYPE)
                        .into_kind_data(),
                    mutable: false,
                    value: None,
                    line_number: 0
                },ftd::interpreter2::Field {
                    name: "label-large".to_string(),
                    kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_RESPONSIVE_TYPE)
                        .into_kind_data(),
                    mutable: false,
                    value: None,
                    line_number: 0
                },ftd::interpreter2::Field {
                    name: "label-small".to_string(),
                    kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_RESPONSIVE_TYPE)
                        .into_kind_data(),
                    mutable: false,
                    value: None,
                    line_number: 0
                },],
                line_number: 0
            })
        ),
        (
            "ftd#font-display".to_string(),
             ftd::interpreter2::Thing::Variable(ftd::interpreter2::Variable {
                 name: "ftd#font-display".to_string(),
                 kind: ftd::interpreter2::Kind::string().into_kind_data(),
                 mutable: true,
                 value: ftd::interpreter2::PropertyValue::Value {
                     value: ftd::interpreter2::Value::new_string("sans-serif"),
                     is_mutable: true,
                     line_number: 0
                 },
                 conditional_value: vec![],
                 line_number: 0,
                 is_static: false
             })
        ),
        (
            "ftd#font-copy".to_string(),
            ftd::interpreter2::Thing::Variable(ftd::interpreter2::Variable {
                name: "ftd#font-copy".to_string(),
                kind: ftd::interpreter2::Kind::string().into_kind_data(),
                mutable: true,
                value: ftd::interpreter2::PropertyValue::Value {
                    value: ftd::interpreter2::Value::new_string("sans-serif"),
                    is_mutable: true,
                    line_number: 0
                },
                conditional_value: vec![],
                line_number: 0,
                is_static: false
            })
        ),
        (
            "ftd#font-code".to_string(),
            ftd::interpreter2::Thing::Variable(ftd::interpreter2::Variable {
                name: "ftd#font-code".to_string(),
                kind: ftd::interpreter2::Kind::string().into_kind_data(),
                mutable: true,
                value: ftd::interpreter2::PropertyValue::Value {
                    value: ftd::interpreter2::Value::new_string("sans-serif"),
                    is_mutable: true,
                    line_number: 0
                },
                conditional_value: vec![],
                line_number: 0,
                is_static: false
            })
        ),
        (
            "ftd#default-types".to_string(),
            ftd::interpreter2::Thing::Variable(ftd::interpreter2::Variable {
                name: "ftd#default-types".to_string(),
                kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_TYPE_DATA).into_kind_data(),
                mutable: true,
                value: ftd::interpreter2::PropertyValue::Value {
                    value: ftd::interpreter2::Value::Record {
                        name: ftd::interpreter2::FTD_TYPE_DATA.to_string(),
                        fields: std::iter::IntoIterator::into_iter([
                            // HEADING TYPES -------------------------------------------
                            (
                                "heading-hero".to_string(),
                                ftd::interpreter2::PropertyValue::Value {
                                    value: ftd::interpreter2::Value::Record {
                                        name: ftd::interpreter2::FTD_RESPONSIVE_TYPE.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "desktop".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                ftd::interpreter2::PropertyValue::Reference {
                                                                    name: "ftd#font-display".to_string(),
                                                                    kind:
                                                                    ftd::interpreter2::Kind::string().into_kind_data(),
                                                                    source:
                                                                    ftd::interpreter2::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 80
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "line-height".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 104
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "weight".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value:
                                                                    ftd::interpreter2::Value::Integer {
                                                                        value: 400
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ), (
                                                "mobile".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                ftd::interpreter2::PropertyValue::Reference {
                                                                    name: "ftd#font-display".to_string(),
                                                                    kind:
                                                                    ftd::interpreter2::Kind::string().into_kind_data(),
                                                                    source:
                                                                    ftd::interpreter2::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 48
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "line-height".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 64
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "weight".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value:
                                                                    ftd::interpreter2::Value::Integer {
                                                                        value: 400
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            )
                                        ]).collect()
                                    },
                                    is_mutable: false,
                                    line_number: 0
                                }
                            ),
                            (
                                "heading-large".to_string(),
                                ftd::interpreter2::PropertyValue::Value {
                                    value: ftd::interpreter2::Value::Record {
                                        name: ftd::interpreter2::FTD_RESPONSIVE_TYPE.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "desktop".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                ftd::interpreter2::PropertyValue::Reference {
                                                                    name: "ftd#font-display".to_string(),
                                                                    kind:
                                                                    ftd::interpreter2::Kind::string().into_kind_data(),
                                                                    source:
                                                                    ftd::interpreter2::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 50
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "line-height".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 65
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "weight".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value:
                                                                    ftd::interpreter2::Value::Integer {
                                                                        value: 400
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ), (
                                                "mobile".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "size".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 36
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "line-height".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 54
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "weight".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value:
                                                                    ftd::interpreter2::Value::Integer {
                                                                        value: 400
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            )
                                        ]).collect()
                                    },
                                    is_mutable: false,
                                    line_number: 0
                                }
                            ),
                            (
                                "heading-medium".to_string(),
                                ftd::interpreter2::PropertyValue::Value {
                                    value: ftd::interpreter2::Value::Record {
                                        name: ftd::interpreter2::FTD_RESPONSIVE_TYPE.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "desktop".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                ftd::interpreter2::PropertyValue::Reference {
                                                                    name: "ftd#font-display".to_string(),
                                                                    kind:
                                                                    ftd::interpreter2::Kind::string().into_kind_data(),
                                                                    source:
                                                                    ftd::interpreter2::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 38
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "line-height".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 57
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "weight".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value:
                                                                    ftd::interpreter2::Value::Integer {
                                                                        value: 400
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ), (
                                                "mobile".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "size".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 26
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "line-height".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 40
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "weight".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value:
                                                                    ftd::interpreter2::Value::Integer {
                                                                        value: 400
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            )
                                        ]).collect()
                                    },
                                    is_mutable: false,
                                    line_number: 0
                                }
                            ),
                            (
                                "heading-small".to_string(),
                                ftd::interpreter2::PropertyValue::Value {
                                    value: ftd::interpreter2::Value::Record {
                                        name: ftd::interpreter2::FTD_RESPONSIVE_TYPE.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "desktop".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                ftd::interpreter2::PropertyValue::Reference {
                                                                    name: "ftd#font-display".to_string(),
                                                                    kind:
                                                                    ftd::interpreter2::Kind::string().into_kind_data(),
                                                                    source:
                                                                    ftd::interpreter2::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 24
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "line-height".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 31
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "weight".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value:
                                                                    ftd::interpreter2::Value::Integer {
                                                                        value: 400
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ), (
                                                "mobile".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "size".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 22
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "line-height".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 29
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "weight".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value:
                                                                    ftd::interpreter2::Value::Integer {
                                                                        value: 400
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            )
                                        ]).collect()
                                    },
                                    is_mutable: false,
                                    line_number: 0
                                }
                            ),
                            (
                                "heading-tiny".to_string(),
                                ftd::interpreter2::PropertyValue::Value {
                                    value: ftd::interpreter2::Value::Record {
                                        name: ftd::interpreter2::FTD_RESPONSIVE_TYPE.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "desktop".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                ftd::interpreter2::PropertyValue::Reference {
                                                                    name: "ftd#font-display".to_string(),
                                                                    kind:
                                                                    ftd::interpreter2::Kind::string().into_kind_data(),
                                                                    source:
                                                                    ftd::interpreter2::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 20
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "line-height".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 26
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "weight".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value:
                                                                    ftd::interpreter2::Value::Integer {
                                                                        value: 400
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ), (
                                                "mobile".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "size".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 18
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "line-height".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 24
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "weight".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value:
                                                                    ftd::interpreter2::Value::Integer {
                                                                        value: 400
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            )
                                        ]).collect()
                                    },
                                    is_mutable: false,
                                    line_number: 0
                                }
                            ),
                            // COPY TYPES -------------------------------------------
                            (
                                "copy-large".to_string(),
                                ftd::interpreter2::PropertyValue::Value {
                                    value: ftd::interpreter2::Value::Record {
                                        name: ftd::interpreter2::FTD_RESPONSIVE_TYPE.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "desktop".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                ftd::interpreter2::PropertyValue::Reference {
                                                                    name: "ftd#font-copy".to_string(),
                                                                    kind:
                                                                    ftd::interpreter2::Kind::string().into_kind_data(),
                                                                    source:
                                                                    ftd::interpreter2::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 22
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "line-height".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 34
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "weight".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value:
                                                                    ftd::interpreter2::Value::Integer {
                                                                        value: 400
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ), (
                                                "mobile".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "size".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 18
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "line-height".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 28
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "weight".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value:
                                                                    ftd::interpreter2::Value::Integer {
                                                                        value: 400
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            )
                                        ]).collect()
                                    },
                                    is_mutable: false,
                                    line_number: 0
                                }
                            ),
                            (
                                "copy-regular".to_string(),
                                ftd::interpreter2::PropertyValue::Value {
                                    value: ftd::interpreter2::Value::Record {
                                        name: ftd::interpreter2::FTD_RESPONSIVE_TYPE.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "desktop".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                ftd::interpreter2::PropertyValue::Reference {
                                                                    name: "ftd#font-copy".to_string(),
                                                                    kind:
                                                                    ftd::interpreter2::Kind::string().into_kind_data(),
                                                                    source:
                                                                    ftd::interpreter2::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 18
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "line-height".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 30
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "weight".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value:
                                                                    ftd::interpreter2::Value::Integer {
                                                                        value: 400
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ), (
                                                "mobile".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "size".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 16
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "line-height".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 24
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "weight".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value:
                                                                    ftd::interpreter2::Value::Integer {
                                                                        value: 400
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            )
                                        ]).collect()
                                    },
                                    is_mutable: false,
                                    line_number: 0
                                }
                            ),
                            (
                                "copy-small".to_string(),
                                ftd::interpreter2::PropertyValue::Value {
                                    value: ftd::interpreter2::Value::Record {
                                        name: ftd::interpreter2::FTD_RESPONSIVE_TYPE.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "desktop".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                ftd::interpreter2::PropertyValue::Reference {
                                                                    name: "ftd#font-copy".to_string(),
                                                                    kind:
                                                                    ftd::interpreter2::Kind::string().into_kind_data(),
                                                                    source:
                                                                    ftd::interpreter2::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 14
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "line-height".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 24
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "weight".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value:
                                                                    ftd::interpreter2::Value::Integer {
                                                                        value: 400
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ), (
                                                "mobile".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "size".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 12
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "line-height".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 16
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "weight".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value:
                                                                    ftd::interpreter2::Value::Integer {
                                                                        value: 400
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            )
                                        ]).collect()
                                    },
                                    is_mutable: false,
                                    line_number: 0
                                }
                            ),
                            // SPECIALIZED TEXT TYPES ---------------------------------
                            (
                                "fine-print".to_string(),
                                ftd::interpreter2::PropertyValue::Value {
                                    value: ftd::interpreter2::Value::Record {
                                        name: ftd::interpreter2::FTD_RESPONSIVE_TYPE.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "desktop".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                ftd::interpreter2::PropertyValue::Reference {
                                                                    name: "ftd#font-code".to_string(),
                                                                    kind:
                                                                    ftd::interpreter2::Kind::string().into_kind_data(),
                                                                    source:
                                                                    ftd::interpreter2::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 12
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "line-height".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 16
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "weight".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value:
                                                                    ftd::interpreter2::Value::Integer {
                                                                        value: 400
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ), (
                                                "mobile".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "size".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 12
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "line-height".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 16
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "weight".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value:
                                                                    ftd::interpreter2::Value::Integer {
                                                                        value: 400
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            )
                                        ]).collect()
                                    },
                                    is_mutable: false,
                                    line_number: 0
                                }
                            ),
                            (
                                "blockquote".to_string(),
                                ftd::interpreter2::PropertyValue::Value {
                                    value: ftd::interpreter2::Value::Record {
                                        name: ftd::interpreter2::FTD_RESPONSIVE_TYPE.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "desktop".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                ftd::interpreter2::PropertyValue::Reference {
                                                                    name: "ftd#font-code".to_string(),
                                                                    kind:
                                                                    ftd::interpreter2::Kind::string().into_kind_data(),
                                                                    source:
                                                                    ftd::interpreter2::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 16
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "line-height".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 21
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "weight".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value:
                                                                    ftd::interpreter2::Value::Integer {
                                                                        value: 400
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ), (
                                                "mobile".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "size".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 16
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "line-height".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 21
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "weight".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value:
                                                                    ftd::interpreter2::Value::Integer {
                                                                        value: 400
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            )
                                        ]).collect()
                                    },
                                    is_mutable: false,
                                    line_number: 0
                                }
                            ),
                            (
                                "source-code".to_string(),
                                ftd::interpreter2::PropertyValue::Value {
                                    value: ftd::interpreter2::Value::Record {
                                        name: ftd::interpreter2::FTD_RESPONSIVE_TYPE.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "desktop".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                ftd::interpreter2::PropertyValue::Reference {
                                                                    name: "ftd#font-code".to_string(),
                                                                    kind:
                                                                    ftd::interpreter2::Kind::string().into_kind_data(),
                                                                    source:
                                                                    ftd::interpreter2::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 18
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "line-height".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 30
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "weight".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value:
                                                                    ftd::interpreter2::Value::Integer {
                                                                        value: 400
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ), (
                                                "mobile".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "size".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 16
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "line-height".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 21
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "weight".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value:
                                                                    ftd::interpreter2::Value::Integer {
                                                                        value: 400
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            )
                                        ]).collect()
                                    },
                                    is_mutable: false,
                                    line_number: 0
                                }
                            ),
                            // LABEL TYPES -------------------------------------
                            (
                                "label-large".to_string(),
                                ftd::interpreter2::PropertyValue::Value {
                                    value: ftd::interpreter2::Value::Record {
                                        name: ftd::interpreter2::FTD_RESPONSIVE_TYPE.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "desktop".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                ftd::interpreter2::PropertyValue::Reference {
                                                                    name: "ftd#font-display".to_string(),
                                                                    kind:
                                                                    ftd::interpreter2::Kind::string().into_kind_data(),
                                                                    source:
                                                                    ftd::interpreter2::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 14
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "line-height".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 19
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "weight".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value:
                                                                    ftd::interpreter2::Value::Integer {
                                                                        value: 400
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ), (
                                                "mobile".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "size".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 14
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "line-height".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 19
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "weight".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value:
                                                                    ftd::interpreter2::Value::Integer {
                                                                        value: 400
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            )
                                        ]).collect()
                                    },
                                    is_mutable: false,
                                    line_number: 0
                                }
                            ),
                            (
                                "label-small".to_string(),
                                ftd::interpreter2::PropertyValue::Value {
                                    value: ftd::interpreter2::Value::Record {
                                        name: ftd::interpreter2::FTD_RESPONSIVE_TYPE.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "desktop".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                ftd::interpreter2::PropertyValue::Reference {
                                                                    name: "ftd#font-display".to_string(),
                                                                    kind:
                                                                    ftd::interpreter2::Kind::string().into_kind_data(),
                                                                    source:
                                                                    ftd::interpreter2::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 12
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "line-height".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 16
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "weight".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value:
                                                                    ftd::interpreter2::Value::Integer {
                                                                        value: 400
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ), (
                                                "mobile".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "size".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 12
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "line-height".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 16
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "weight".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value:
                                                                    ftd::interpreter2::Value::Integer {
                                                                        value: 400
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            )
                                        ]).collect()
                                    },
                                    is_mutable: false,
                                    line_number: 0
                                }
                            ),
                            // BUTTON TYPES -------------------------------------
                            (
                                "button-large".to_string(),
                                ftd::interpreter2::PropertyValue::Value {
                                    value: ftd::interpreter2::Value::Record {
                                        name: ftd::interpreter2::FTD_RESPONSIVE_TYPE.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "desktop".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                ftd::interpreter2::PropertyValue::Reference {
                                                                    name: "ftd#font-display".to_string(),
                                                                    kind:
                                                                    ftd::interpreter2::Kind::string().into_kind_data(),
                                                                    source:
                                                                    ftd::interpreter2::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 18
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "line-height".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 24
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "weight".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value:
                                                                    ftd::interpreter2::Value::Integer {
                                                                        value: 400
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ), (
                                                "mobile".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "size".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 18
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "line-height".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 24
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "weight".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value:
                                                                    ftd::interpreter2::Value::Integer {
                                                                        value: 400
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            )
                                        ]).collect()
                                    },
                                    is_mutable: false,
                                    line_number: 0
                                }
                            ),
                            (
                                "button-medium".to_string(),
                                ftd::interpreter2::PropertyValue::Value {
                                    value: ftd::interpreter2::Value::Record {
                                        name: ftd::interpreter2::FTD_RESPONSIVE_TYPE.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "desktop".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                ftd::interpreter2::PropertyValue::Reference {
                                                                    name: "ftd#font-display".to_string(),
                                                                    kind:
                                                                    ftd::interpreter2::Kind::string().into_kind_data(),
                                                                    source:
                                                                    ftd::interpreter2::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 16
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "line-height".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 21
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "weight".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value:
                                                                    ftd::interpreter2::Value::Integer {
                                                                        value: 400
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ), (
                                                "mobile".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "size".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 16
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "line-height".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 21
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "weight".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value:
                                                                    ftd::interpreter2::Value::Integer {
                                                                        value: 400
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            )
                                        ]).collect()
                                    },
                                    is_mutable: false,
                                    line_number: 0
                                }
                            ),
                            (
                                "button-small".to_string(),
                                ftd::interpreter2::PropertyValue::Value {
                                    value: ftd::interpreter2::Value::Record {
                                        name: ftd::interpreter2::FTD_RESPONSIVE_TYPE.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "desktop".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                ftd::interpreter2::PropertyValue::Reference {
                                                                    name: "ftd#font-display".to_string(),
                                                                    kind:
                                                                    ftd::interpreter2::Kind::string().into_kind_data(),
                                                                    source:
                                                                    ftd::interpreter2::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 14
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "line-height".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 19
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "weight".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value:
                                                                    ftd::interpreter2::Value::Integer {
                                                                        value: 400
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ), (
                                                "mobile".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "size".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 14
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "line-height".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 19
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "weight".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value:
                                                                    ftd::interpreter2::Value::Integer {
                                                                        value: 400
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            )
                                        ]).collect()
                                    },
                                    is_mutable: false,
                                    line_number: 0
                                }
                            ),
                            (
                                "link".to_string(),
                                ftd::interpreter2::PropertyValue::Value {
                                    value: ftd::interpreter2::Value::Record {
                                        name: ftd::interpreter2::FTD_RESPONSIVE_TYPE.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "desktop".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                ftd::interpreter2::PropertyValue::Reference {
                                                                    name: "ftd#font-display".to_string(),
                                                                    kind:
                                                                    ftd::interpreter2::Kind::string().into_kind_data(),
                                                                    source:
                                                                    ftd::interpreter2::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 14
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "line-height".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 19
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "weight".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value:
                                                                    ftd::interpreter2::Value::Integer {
                                                                        value: 400
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ), (
                                                "mobile".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "size".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 14
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "line-height".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::OrType {
                                                                        name: ftd::interpreter2::FTD_FONT_SIZE.to_string(),
                                                                        variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: ftd::interpreter2::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (ftd::interpreter2::PropertyValue::Value {
                                                                                value: ftd::interpreter2::Value::Integer {
                                                                                    value: 19
                                                                                },
                                                                                is_mutable: false,
                                                                                line_number: 0
                                                                            })
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "weight".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value:
                                                                    ftd::interpreter2::Value::Integer {
                                                                        value: 400
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            )
                                        ]).collect()
                                    },
                                    is_mutable: false,
                                    line_number: 0
                                }
                            ),
                        ]).collect()
                    },
                    is_mutable: false,
                    line_number: 0
                },
                conditional_value: vec![],
                line_number: 0,
                is_static: false
            })
        ),
        (
            "ftd#default-colors".to_string(),
            ftd::interpreter2::Thing::Variable(ftd::interpreter2::Variable {
                name: "ftd#default-colors".to_string(),
                kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR_SCHEME)
                    .into_kind_data(),
                mutable: true,
                value: ftd::interpreter2::PropertyValue::Value {
                    value: ftd::interpreter2::Value::Record {
                        name: ftd::interpreter2::FTD_COLOR_SCHEME.to_string(),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                "background".to_string(),
                                ftd::interpreter2::PropertyValue::Value {
                                    value: ftd::interpreter2::Value::Record {
                                        name: ftd::interpreter2::FTD_BACKGROUND_COLOR.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "base".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value:
                                                                ftd::interpreter2::Value::String {
                                                                    text: "#e7e7e4".to_string(),
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            }
                                                        ), (
                                                            "dark".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value:
                                                                ftd::interpreter2::Value::String {
                                                                    text: "#18181b".to_string(),
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            }
                                                        )])
                                                            .collect(),
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0,
                                                }
                                            ),
                                            (
                                                "step-1".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value:
                                                                ftd::interpreter2::Value::String {
                                                                    text: "#f3f3f3".to_string(),
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            }
                                                        ), (
                                                            "dark".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value:
                                                                ftd::interpreter2::Value::String {
                                                                    text: "#141414".to_string(),
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            }
                                                        )])
                                                            .collect(),
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0,
                                                }
                                            ),
                                            (
                                                "step-2".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value:
                                                                ftd::interpreter2::Value::String {
                                                                    text: "#c9cece".to_string(),
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            }
                                                        ), (
                                                            "dark".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value:
                                                                ftd::interpreter2::Value::String {
                                                                    text: "#585656".to_string(),
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            }
                                                        )])
                                                            .collect(),
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0,
                                                }
                                            ),
                                            (
                                                "overlay".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value:
                                                                ftd::interpreter2::Value::String {
                                                                    text: "rgba(0, 0, 0, 0.8)"
                                                                        .to_string(),
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            }
                                                        ), (
                                                            "dark".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value:
                                                                ftd::interpreter2::Value::String {
                                                                    text: "rgba(0, 0, 0, 0.8)"
                                                                        .to_string(),
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            }
                                                        )])
                                                            .collect(),
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0,
                                                }
                                            ),
                                            (
                                                "code".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value:
                                                                ftd::interpreter2::Value::String {
                                                                    text: "#2B303B".to_string(),
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            }
                                                        ), (
                                                            "dark".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value:
                                                                ftd::interpreter2::Value::String {
                                                                    text: "#2B303B".to_string(),
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            }
                                                        )])
                                                            .collect(),
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0,
                                                }
                                            ),
                                        ])
                                            .collect(),
                                    },
                                    is_mutable: false,
                                    line_number: 0,
                                }
                            ),
                            (
                                "border".to_string(),
                                ftd::interpreter2::PropertyValue::Value {
                                    value: ftd::interpreter2::Value::Record {
                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([(
                                            "light".to_string(),
                                            ftd::interpreter2::PropertyValue::Value {
                                                value:
                                                ftd::interpreter2::Value::String {
                                                    text: "#434547".to_string(),
                                                },
                                                is_mutable: false,
                                                line_number: 0,
                                            }
                                        ), (
                                            "dark".to_string(),
                                            ftd::interpreter2::PropertyValue::Value {
                                                value:
                                                ftd::interpreter2::Value::String {
                                                    text: "#434547".to_string(),
                                                },
                                                is_mutable: false,
                                                line_number: 0,
                                            }
                                        )])
                                            .collect()
                                    },
                                    is_mutable: false,
                                    line_number: 0
                                }
                            ),
                            (
                                "border-strong".to_string(),
                                ftd::interpreter2::PropertyValue::Value {
                                    value: ftd::interpreter2::Value::Record {
                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([(
                                            "light".to_string(),
                                            ftd::interpreter2::PropertyValue::Value {
                                                value:
                                                ftd::interpreter2::Value::String {
                                                    text: "#919192".to_string(),
                                                },
                                                is_mutable: false,
                                                line_number: 0,
                                            }
                                        ), (
                                            "dark".to_string(),
                                            ftd::interpreter2::PropertyValue::Value {
                                                value:
                                                ftd::interpreter2::Value::String {
                                                    text: "#919192".to_string(),
                                                },
                                                is_mutable: false,
                                                line_number: 0,
                                            }
                                        )])
                                            .collect()
                                    },
                                    is_mutable: false,
                                    line_number: 0
                                }
                            ),
                            (
                                "text".to_string(),
                                ftd::interpreter2::PropertyValue::Value {
                                    value: ftd::interpreter2::Value::Record {
                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([(
                                            "light".to_string(),
                                            ftd::interpreter2::PropertyValue::Value {
                                                value:
                                                ftd::interpreter2::Value::String {
                                                    text: "#584b42".to_string(),
                                                },
                                                is_mutable: false,
                                                line_number: 0,
                                            }
                                        ), (
                                            "dark".to_string(),
                                            ftd::interpreter2::PropertyValue::Value {
                                                value:
                                                ftd::interpreter2::Value::String {
                                                    text: "#a8a29e".to_string(),
                                                },
                                                is_mutable: false,
                                                line_number: 0,
                                            }
                                        )])
                                            .collect()
                                    },
                                    is_mutable: false,
                                    line_number: 0
                                }
                            ),
                            (
                                "text-strong".to_string(),
                                ftd::interpreter2::PropertyValue::Value {
                                    value: ftd::interpreter2::Value::Record {
                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([(
                                            "light".to_string(),
                                            ftd::interpreter2::PropertyValue::Value {
                                                value:
                                                ftd::interpreter2::Value::String {
                                                    text: "#141414".to_string(),
                                                },
                                                is_mutable: false,
                                                line_number: 0,
                                            }
                                        ), (
                                            "dark".to_string(),
                                            ftd::interpreter2::PropertyValue::Value {
                                                value:
                                                ftd::interpreter2::Value::String {
                                                    text: "#ffffff".to_string(),
                                                },
                                                is_mutable: false,
                                                line_number: 0,
                                            }
                                        )])
                                            .collect()
                                    },
                                    is_mutable: false,
                                    line_number: 0
                                }
                            ),
                            (
                                "shadow".to_string(),
                                ftd::interpreter2::PropertyValue::Value {
                                    value: ftd::interpreter2::Value::Record {
                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([(
                                            "light".to_string(),
                                            ftd::interpreter2::PropertyValue::Value {
                                                value:
                                                ftd::interpreter2::Value::String {
                                                    text: "#007f9b".to_string(),
                                                },
                                                is_mutable: false,
                                                line_number: 0,
                                            },
                                        ), (
                                            "dark".to_string(),
                                            ftd::interpreter2::PropertyValue::Value {
                                                value:
                                                ftd::interpreter2::Value::String {
                                                    text: "#007f9b".to_string(),
                                                },
                                                is_mutable: false,
                                                line_number: 0,
                                            },
                                        )])
                                            .collect()
                                    },
                                    is_mutable: false,
                                    line_number: 0
                                }
                            ),
                            (
                                "scrim".to_string(),
                                ftd::interpreter2::PropertyValue::Value {
                                    value: ftd::interpreter2::Value::Record {
                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([(
                                            "light".to_string(),
                                            ftd::interpreter2::PropertyValue::Value {
                                                value:
                                                ftd::interpreter2::Value::String {
                                                    text: "#007f9b".to_string(),
                                                },
                                                is_mutable: false,
                                                line_number: 0,
                                            },
                                        ), (
                                            "dark".to_string(),
                                            ftd::interpreter2::PropertyValue::Value {
                                                value:
                                                ftd::interpreter2::Value::String {
                                                    text: "#007f9b".to_string(),
                                                },
                                                is_mutable: false,
                                                line_number: 0,
                                            },
                                        )])
                                            .collect()
                                    },
                                    is_mutable: false,
                                    line_number: 0
                                }
                            ),
                            (
                                "cta-primary".to_string(),
                                ftd::interpreter2::PropertyValue::Value {
                                    value: ftd::interpreter2::Value::Record {
                                        name: ftd::interpreter2::FTD_CTA_COLOR.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "base".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#2dd4bf".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#2dd4bf".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        )]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0,
                                                },
                                            ),
                                            (
                                                "hover".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#2c9f90".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#2c9f90".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        )]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0,
                                                },
                                            ),
                                            (
                                                "pressed".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#2cc9b5".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#2cc9b5".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        )]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0,
                                                },
                                            ),
                                            (
                                                "disabled".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "rgba(44, 201, 181, 0.1)".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "rgba(44, 201, 181, 0.1)".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        )]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0,
                                                },
                                            ),
                                            (
                                                "focused".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#2cbfac".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#2cbfac".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        )]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0,
                                                },
                                            ),
                                            (
                                                "border".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#2b8074".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#2b8074".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        )]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0,
                                                },
                                            ),
                                            (
                                                "text".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#feffff".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#feffff".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        )]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0,
                                                },
                                            )
                                        ]).collect()
                                    },
                                    is_mutable: false,
                                    line_number: 0
                                }
                            ),
                            (
                                "cta-secondary".to_string(),
                                ftd::interpreter2::PropertyValue::Value {
                                    value: ftd::interpreter2::Value::Record {
                                        name: ftd::interpreter2::FTD_CTA_COLOR.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "base".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#4fb2df".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#4fb2df".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        )]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0,
                                                },
                                            ),
                                            (
                                                "hover".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#40afe1".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#40afe1".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        )]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0,
                                                },
                                            ),
                                            (
                                                "pressed".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#4fb2df".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#4fb2df".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        )]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0,
                                                },
                                            ),
                                            (
                                                "disabled".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "rgba(79, 178, 223, 0.1)".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "rgba(79, 178, 223, 0.1)".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        )]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0,
                                                },
                                            ),
                                            (
                                                "focused".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#4fb1df".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#4fb1df".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        )]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0,
                                                },
                                            ),
                                            (
                                                "border".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#209fdb".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#209fdb".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        )]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0,
                                                },
                                            ),
                                            (
                                                "text".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#584b42".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#ffffff".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        )]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0,
                                                },
                                            )
                                        ]).collect()
                                    },
                                    is_mutable: false,
                                    line_number: 0
                                }
                            ),
                            (
                                "cta-tertiary".to_string(),
                                ftd::interpreter2::PropertyValue::Value {
                                    value: ftd::interpreter2::Value::Record {
                                        name: ftd::interpreter2::FTD_CTA_COLOR.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "base".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#556375".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#556375".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        )]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0,
                                                },
                                            ),
                                            (
                                                "hover".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#c7cbd1".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#c7cbd1".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        )]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0,
                                                },
                                            ),
                                            (
                                                "pressed".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#3b4047".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#3b4047".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        )]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0,
                                                },
                                            ),
                                            (
                                                "disabled".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "rgba(85, 99, 117, 0.1)".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "rgba(85, 99, 117, 0.1)".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        )]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0,
                                                },
                                            ),
                                            (
                                                "focused".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#e0e2e6".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#e0e2e6".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        )]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0,
                                                },
                                            ),
                                            (
                                                "border".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#e2e4e7".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#e2e4e7".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        )]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0,
                                                },
                                            ),
                                            (
                                                "text".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#ffffff".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#ffffff".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        )]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0,
                                                },
                                            )
                                        ]).collect()
                                    },
                                    is_mutable: false,
                                    line_number: 0
                                }
                            ),
                            (
                                "cta-danger".to_string(),
                                ftd::interpreter2::PropertyValue::Value {
                                    value: ftd::interpreter2::Value::Record {
                                        name: ftd::interpreter2::FTD_CTA_COLOR.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "base".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#1C1B1F".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#1C1B1F".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        )]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0,
                                                },
                                            ),
                                            (
                                                "hover".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#1C1B1F".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#1C1B1F".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        )]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0,
                                                },
                                            ),
                                            (
                                                "pressed".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#1C1B1F".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#1C1B1F".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        )]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0,
                                                },
                                            ),
                                            (
                                                "disabled".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#1C1B1F".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#1C1B1F".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        )]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0,
                                                },
                                            ),
                                            (
                                                "focused".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#1C1B1F".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#1C1B1F".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        )]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0,
                                                },
                                            ),
                                            (
                                                "border".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#1C1B1F".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            ftd::interpreter2::PropertyValue::Value {
                                                                value: ftd::interpreter2::Value::String {
                                                                    text: "#1C1B1F".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        )]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0,
                                                },
                                            ),
                                            (
                                                "text".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#1C1B1F".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0,
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#1C1B1F".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0,
                                                                }
                                                            )
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0,
                                                },
                                            )
                                        ]).collect()
                                    },
                                    is_mutable: false,
                                    line_number: 0
                                }
                            ),
                            (
                                "accent".to_string(),
                                ftd::interpreter2::PropertyValue::Value {
                                    value: ftd::interpreter2::Value::Record {
                                        name: ftd::interpreter2::FTD_PST.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "primary".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#2dd4bf".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#2dd4bf".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            )
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ),
                                            (
                                                "secondary".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#4fb2df".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#4fb2df".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            )
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ),
                                            (
                                                "tertiary".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#c5cbd7".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#c5cbd7".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            )
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            )
                                        ]).collect()
                                    },
                                    is_mutable: false,
                                    line_number: 0
                                }
                            ),
                            (
                                "error".to_string(),
                                ftd::interpreter2::PropertyValue::Value {
                                    value: ftd::interpreter2::Value::Record {
                                        name: ftd::interpreter2::FTD_BTB.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "base".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#f5bdbb".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#530c0ab0".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            )
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ),
                                            (
                                                "text".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#c62a21".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#c62a21".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            )
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ),
                                            (
                                                "border".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#df2b2b".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#df2b2b".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            )
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            )
                                        ]).collect()
                                    },
                                    is_mutable: false,
                                    line_number: 0
                                }
                            ),
                            (
                                "success".to_string(),
                                ftd::interpreter2::PropertyValue::Value {
                                    value: ftd::interpreter2::Value::Record {
                                        name: ftd::interpreter2::FTD_BTB.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "base".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#e3f0c4".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#405508ad".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            )
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ),
                                            (
                                                "text".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#467b28".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#479f16".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            )
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ),
                                            (
                                                "border".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#3d741f".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#3d741f".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            )
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            )
                                        ]).collect()
                                    },
                                    is_mutable: false,
                                    line_number: 0
                                }
                            ),
                            (
                                "info".to_string(),
                                ftd::interpreter2::PropertyValue::Value {
                                    value: ftd::interpreter2::Value::Record {
                                        name: ftd::interpreter2::FTD_BTB.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "base".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#c4edfd".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#052a38de".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            )
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ),
                                            (
                                                "text".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#205694".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#0a60c2".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            )
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ),
                                            (
                                                "border".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#205694".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#205694".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            )
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ),
                                        ]).collect()
                                    },
                                    is_mutable: false,
                                    line_number: 0
                                }
                            ),
                            (
                                "warning".to_string(),
                                ftd::interpreter2::PropertyValue::Value {
                                    value: ftd::interpreter2::Value::Record {
                                        name: ftd::interpreter2::FTD_BTB.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "base".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#fbefba".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#544607a3".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            )
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ),
                                            (
                                                "text".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#966220".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#d07f19".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            )
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ),
                                            (
                                                "border".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#966220".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#966220".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            )
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ),
                                        ]).collect()
                                    },
                                    is_mutable: false,
                                    line_number: 0
                                }
                            ),
                            (
                                "custom".to_string(),
                                ftd::interpreter2::PropertyValue::Value {
                                    value: ftd::interpreter2::Value::Record {
                                        name: ftd::interpreter2::FTD_CUSTOM_COLORS.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "one".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#ed753a".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#ed753a".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            )
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ),
                                            (
                                                "two".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#f3db5f".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#f3db5f".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            )
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ),
                                            (
                                                "three".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#8fdcf8".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#8fdcf8".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            )
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ),
                                            (
                                                "four".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#7a65c7".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#7a65c7".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            )
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ),
                                            (
                                                "five".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#eb57be".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#eb57be".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            )
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ),
                                            (
                                                "six".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#ef8dd6".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#ef8dd6".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            )
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ),
                                            (
                                                "seven".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#7564be".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#7564be".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            )
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ),
                                            (
                                                "eight".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#d554b3".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#d554b3".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            )
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ),
                                            (
                                                "nine".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#ec8943".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#ec8943".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            )
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ),
                                            (
                                                "ten".to_string(),
                                                ftd::interpreter2::PropertyValue::Value {
                                                    value: ftd::interpreter2::Value::Record {
                                                        name: ftd::interpreter2::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#da7a4a".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                ftd::interpreter2::PropertyValue::Value {
                                                                    value: ftd::interpreter2::Value::String {
                                                                        text: "#da7a4a".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            )
                                                        ]).collect()
                                                    },
                                                    is_mutable: false,
                                                    line_number: 0
                                                }
                                            ),
                                        ]).collect()
                                    },
                                    is_mutable: false,
                                    line_number: 0
                                }
                            ),
                        ])
                            .collect(),
                    },
                    is_mutable: false,
                    line_number: 0,
                },
                conditional_value: vec![],
                line_number: 0,
                is_static: false,
            }),
        ),
        (
            ftd::interpreter2::FTD_BREAKPOINT_WIDTH_DATA.to_string(),
            ftd::interpreter2::Thing::Record(ftd::interpreter2::Record {
                name: ftd::interpreter2::FTD_BREAKPOINT_WIDTH_DATA.to_string(),
                fields: vec![ftd::interpreter2::Field {
                    name: "mobile".to_string(),
                    kind: ftd::interpreter2::Kind::integer().into_kind_data(),
                    mutable: false,
                    value: None,
                    line_number: 0
                }],
                line_number: 0
            })
        ),
        (
            ftd::interpreter2::FTD_BREAKPOINT_WIDTH.to_string(),
            ftd::interpreter2::Thing::Variable(ftd::interpreter2::Variable {
                name: ftd::interpreter2::FTD_BREAKPOINT_WIDTH.to_string(),
                kind: ftd::interpreter2::Kind::record
                    (ftd::interpreter2::FTD_BREAKPOINT_WIDTH_DATA).into_kind_data(),
                mutable: true,
                value: ftd::interpreter2::PropertyValue::Value {
                    value: ftd::interpreter2::Value::Record {
                        name: ftd::interpreter2::FTD_BREAKPOINT_WIDTH_DATA.to_string(),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                "mobile".to_string(),
                                ftd::interpreter2::PropertyValue::Value {
                                    value: ftd::interpreter2::Value::Integer {
                                        value: 768
                                    },
                                    is_mutable: false,
                                    line_number: 0
                                }
                            )
                        ]).collect()
                    },
                    is_mutable: true,
                    line_number: 0
                },
                conditional_value: vec![],
                line_number: 0,
                is_static: false
            })
        ),
        (
            ftd::interpreter2::FTD_DEVICE_DATA.to_string(),
            ftd::interpreter2::Thing::OrType(ftd::interpreter2::OrType {
                name: ftd::interpreter2::FTD_DEVICE_DATA.to_string(),
                variants: vec![
                    ftd::interpreter2::OrTypeVariant::Constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_DEVICE_DATA_MOBILE,
                        ftd::interpreter2::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(ftd::interpreter2::Value::new_string("mobile")
                                    .into_property_value(false, 0),),
                        0,
                    )),
                    ftd::interpreter2::OrTypeVariant::Constant(ftd::interpreter2::Field::new(
                        ftd::interpreter2::FTD_DEVICE_DATA_DESKTOP,
                        ftd::interpreter2::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(ftd::interpreter2::Value::new_string("desktop")
                                 .into_property_value(false, 0),),
                        0,
                    )),
                ],
                line_number: 0
            })
        ),
        (
            ftd::interpreter2::FTD_DEVICE.to_string(),
            ftd::interpreter2::Thing::Variable(ftd::interpreter2::Variable {
                name: ftd::interpreter2::FTD_DEVICE.to_string(),
                kind: ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_DEVICE_DATA).into_kind_data(),
                mutable: true,
                value: ftd::interpreter2::PropertyValue::Value {
                    value: ftd::interpreter2::Value::OrType {
                        name: ftd::interpreter2::FTD_DEVICE_DATA.to_string(),
                        variant: ftd::interpreter2::FTD_DEVICE_DATA_DESKTOP.to_string(),
                        full_variant: ftd::interpreter2::FTD_DEVICE_DATA_DESKTOP.to_string(),
                        value: Box::new(ftd::interpreter2::Value::new_string("desktop")
                            .into_property_value(false, 0))
                    },
                    is_mutable: true,
                    line_number: 0
                },
                conditional_value: vec![],
                line_number: 0,
                is_static: false
            })
        )
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
        css: None,
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
        css: None,
        line_number: 0,
    }
}

pub fn checkbox_function() -> ftd::interpreter2::ComponentDefinition {
    ftd::interpreter2::ComponentDefinition {
        name: "ftd#checkbox".to_string(),
        arguments: [
            common_arguments(),
            vec![
                ftd::interpreter2::Argument::default(
                    "checked",
                    ftd::interpreter2::Kind::boolean()
                        .into_optional()
                        .into_kind_data(),
                ),
                ftd::interpreter2::Argument::default(
                    "enabled",
                    ftd::interpreter2::Kind::boolean()
                        .into_optional()
                        .into_kind_data(),
                ),
            ],
        ]
        .concat()
        .into_iter()
        .collect(),
        definition: ftd::interpreter2::Component::from_name("ftd.kernel"),
        css: None,
        line_number: 0,
    }
}

pub fn text_input_function() -> ftd::interpreter2::ComponentDefinition {
    ftd::interpreter2::ComponentDefinition {
        name: "ftd#text-input".to_string(),
        arguments: [
            text_arguments(),
            common_arguments(),
            vec![
                ftd::interpreter2::Argument::default(
                    "placeholder",
                    ftd::interpreter2::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
                ftd::interpreter2::Argument::default(
                    "value",
                    ftd::interpreter2::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
                ftd::interpreter2::Argument::default(
                    "default-value",
                    ftd::interpreter2::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
                ftd::interpreter2::Argument::default(
                    "multiline",
                    ftd::interpreter2::Kind::boolean()
                        .into_optional()
                        .into_kind_data(),
                ),
                ftd::interpreter2::Argument::default(
                    "enabled",
                    ftd::interpreter2::Kind::boolean()
                        .into_optional()
                        .into_kind_data(),
                ),
                ftd::interpreter2::Argument::default(
                    "type",
                    ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_TEXT_INPUT_TYPE)
                        .into_optional()
                        .into_kind_data(),
                ),
            ],
        ]
        .concat()
        .into_iter()
        .collect(),
        definition: ftd::interpreter2::Component::from_name("ftd.kernel"),
        css: None,
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
        css: None,
        line_number: 0,
    }
}

pub fn decimal_function() -> ftd::interpreter2::ComponentDefinition {
    ftd::interpreter2::ComponentDefinition {
        name: "ftd#decimal".to_string(),
        arguments: [
            text_arguments(),
            common_arguments(),
            vec![
                ftd::interpreter2::Argument::default(
                    "value",
                    ftd::interpreter2::Kind::decimal()
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
            ],
        ]
        .concat()
        .into_iter()
        .collect(),
        definition: ftd::interpreter2::Component::from_name("ftd.kernel"),
        css: None,
        line_number: 0,
    }
}

pub fn markup_function() -> ftd::interpreter2::ComponentDefinition {
    ftd::interpreter2::ComponentDefinition {
        name: "ftd#text".to_string(),
        arguments: [
            text_arguments(),
            common_arguments(),
            vec![ftd::interpreter2::Argument::default(
                "text",
                ftd::interpreter2::Kind::string()
                    .into_kind_data()
                    .caption_or_body(),
            )],
        ]
        .concat()
        .into_iter()
        .collect(),
        definition: ftd::interpreter2::Component::from_name("ftd.kernel"),
        css: None,
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
        css: None,
        line_number: 0,
    }
}

pub fn code_function() -> ftd::interpreter2::ComponentDefinition {
    ftd::interpreter2::ComponentDefinition {
        name: "ftd#code".to_string(),
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
                    "lang",
                    ftd::interpreter2::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
                ftd::interpreter2::Argument::default(
                    "theme",
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
        css: None,
        line_number: 0,
    }
}

pub fn iframe_function() -> ftd::interpreter2::ComponentDefinition {
    ftd::interpreter2::ComponentDefinition {
        name: "ftd#iframe".to_string(),
        arguments: [
            common_arguments(),
            vec![
                ftd::interpreter2::Argument::default(
                    "src",
                    ftd::interpreter2::Kind::string()
                        .into_optional()
                        .into_kind_data()
                        .caption(),
                ),
                ftd::interpreter2::Argument::default(
                    "youtube",
                    ftd::interpreter2::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
                ftd::interpreter2::Argument::default(
                    "srcdoc",
                    ftd::interpreter2::Kind::string()
                        .into_optional()
                        .into_kind_data()
                        .body(),
                ),
                ftd::interpreter2::Argument::default(
                    "loading",
                    ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LOADING)
                        .into_optional()
                        .into_kind_data(),
                ),
            ],
        ]
        .concat()
        .into_iter()
        .collect(),
        definition: ftd::interpreter2::Component::from_name("ftd.kernel"),
        css: None,
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
        css: None,
        line_number: 0,
    }
}

fn container_arguments() -> Vec<ftd::interpreter2::Argument> {
    vec![
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
            "spacing",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_SPACING)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "children",
            ftd::interpreter2::Kind::subsection_ui()
                .into_list()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "colors",
            ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_COLOR_SCHEME)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "types",
            ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_TYPE_DATA)
                .into_optional()
                .into_kind_data(),
        ),
    ]
}

fn common_arguments() -> Vec<ftd::interpreter2::Argument> {
    vec![
        ftd::interpreter2::Argument::default(
            "sticky",
            ftd::interpreter2::Kind::boolean()
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
            "border-style",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_BORDER_STYLE)
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
            "white-space",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_WHITESPACE)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "text-transform",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_TEXT_TRANSFORM)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "region",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_REGION)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "left",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "right",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "top",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "bottom",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "anchor",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_ANCHOR)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "role",
            ftd::interpreter2::Kind::record(ftd::interpreter2::FTD_RESPONSIVE_TYPE)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "cursor",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_CURSOR)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "classes",
            ftd::interpreter2::Kind::string()
                .into_list()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "js-list",
            ftd::interpreter2::Kind::string()
                .into_list()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "js",
            ftd::interpreter2::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "css-list",
            ftd::interpreter2::Kind::string()
                .into_list()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "css",
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
            "resize",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_RESIZE)
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
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_RESIZING)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "min-width",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_RESIZING)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "min-height",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_RESIZING)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "max-height",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_RESIZING)
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
    ]
}

fn text_arguments() -> Vec<ftd::interpreter2::Argument> {
    vec![
        ftd::interpreter2::Argument::default(
            "text-align",
            ftd::interpreter2::Kind::or_type(ftd::interpreter2::FTD_TEXT_ALIGN)
                .into_optional()
                .into_kind_data(),
        ),
        ftd::interpreter2::Argument::default(
            "line-clamp",
            ftd::interpreter2::Kind::integer()
                .into_kind_data()
                .into_optional(),
        ),
    ]
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
