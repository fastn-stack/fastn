#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn_builtins;

pub mod constants;

pub type Map<T> = std::collections::BTreeMap<String, T>;
use fastn_resolved::evalexpr::ContextWithMutableFunctions;

/**
* The `default_aliases` function is intended to provide default aliases for the `ftd` module,
* with the only default alias being "ftd" itself. This allows users to reference the `ftd` module
* using this alias instead of the full module name.
**/
pub fn default_aliases() -> Map<String> {
    std::iter::IntoIterator::into_iter([
        ("ftd".to_string(), "ftd".to_string()),
        ("inherited".to_string(), "inherited".to_string()),
    ])
    .collect()
}

/*
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
*/
pub fn default_functions() -> Map<fastn_resolved::evalexpr::Function> {
    use fastn_resolved::evalexpr::*;

    std::iter::IntoIterator::into_iter([
        (
            "ftd.clean_code".to_string(),
            Function::new(|argument| {
                if argument.as_empty().is_ok() {
                    Ok(Value::String("".to_string()))
                } else if let Ok(s) = argument.as_string() {
                    let mut new_string = vec![];
                    for line in s.split('\n') {
                        new_string.push(
                            fastn_builtins::constants::FTD_HIGHLIGHTER.replace(line, regex::NoExpand("")),
                        );
                    }
                    Ok(Value::String(new_string.join("\n")))
                } else if let Ok(tuple) = argument.as_tuple() {
                    if tuple.len().ne(&2) {
                        Err(
                            fastn_resolved::evalexpr::error::EvalexprError::WrongFunctionArgumentAmount {
                                expected: 2,
                                actual: tuple.len(),
                            },
                        )
                    } else {
                        let s = tuple.first().unwrap().as_string()?;
                        let lang = tuple.last().unwrap().as_string()?;
                        if lang.eq("ftd") {
                            let mut new_string = vec![];
                            for line in s.split('\n') {
                                new_string.push(
                                    fastn_builtins::constants::FTD_HIGHLIGHTER
                                        .replace(line, regex::NoExpand("")),
                                );
                            }
                            Ok(Value::String(new_string.join("\n")))
                        } else {
                            Ok(Value::String(s))
                        }
                    }
                } else {
                    Err(fastn_resolved::evalexpr::error::EvalexprError::ExpectedString {
                        actual: argument.clone(),
                    })
                }
            }),
        ),
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
                            fastn_resolved::evalexpr::error::EvalexprError::WrongFunctionArgumentAmount {
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

pub fn default_context(
) -> Result<fastn_resolved::evalexpr::HashMapContext, fastn_resolved::evalexpr::EvalexprError> {
    let mut context = fastn_resolved::evalexpr::HashMapContext::new();
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
pub fn default_bag() -> indexmap::IndexMap<String, fastn_resolved::Definition> {
    let record = |n: &str, r: &str| (n.to_string(), fastn_resolved::Kind::record(r));
    let _color = |n: &str| record(n, "ftd#color");
    let things = vec![
        (
            "ftd#row".to_string(),
            fastn_resolved::Definition::Component(row_function()),
        ),
        (
            "ftd#rive".to_string(),
            fastn_resolved::Definition::Component(rive_function()),
        ),
        (
            "ftd#container".to_string(),
            fastn_resolved::Definition::Component(container_function()),
        ),
        (
            "ftd#desktop".to_string(),
            fastn_resolved::Definition::Component(desktop_function()),
        ),
        (
            "ftd#mobile".to_string(),
            fastn_resolved::Definition::Component(mobile_function()),
        ),
        (
            "ftd#code".to_string(),
            fastn_resolved::Definition::Component(code_function()),
        ),
        (
            "ftd#iframe".to_string(),
            fastn_resolved::Definition::Component(iframe_function()),
        ),
        (
            "ftd#column".to_string(),
            fastn_resolved::Definition::Component(column_function()),
        ),
        (
            "ftd#document".to_string(),
            fastn_resolved::Definition::Component(document_function()),
        ),
        (
            "ftd#text".to_string(),
            fastn_resolved::Definition::Component(markup_function()),
        ),
        (
            "ftd#integer".to_string(),
            fastn_resolved::Definition::Component(integer_function()),
        ),
        (
            "ftd#decimal".to_string(),
            fastn_resolved::Definition::Component(decimal_function()),
        ),
        (
            "ftd#boolean".to_string(),
            fastn_resolved::Definition::Component(boolean_function()),
        ),
        (
            "ftd#text-input".to_string(),
            fastn_resolved::Definition::Component(text_input_function()),
        ),
        (
            "ftd#checkbox".to_string(),
            fastn_resolved::Definition::Component(checkbox_function()),
        ),
        (
            "ftd#image".to_string(),
            fastn_resolved::Definition::Component(image_function()),
        ),

        (
            "ftd#audio".to_string(),
            fastn_resolved::Definition::Component(audio_function()),
        ),
        (
            "ftd#video".to_string(),
            fastn_resolved::Definition::Component(video_function()),
        ),
        (
            "ftd#set-rive-boolean".to_string(),
            fastn_resolved::Definition::Function(fastn_resolved::Function {
                name: "ftd#set-rive-boolean".to_string(),
                return_kind: fastn_resolved::KindData {
                    kind: fastn_resolved::Kind::void(),
                    caption: false,
                    body: false,
                },
                arguments: vec![
                    fastn_resolved::Argument {
                        name: "rive".to_string(),
                        kind: fastn_resolved::Kind::string().into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Argument {
                        name: "input".to_string(),
                        kind: fastn_resolved::Kind::string().into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Argument {
                        name: "value".to_string(),
                        kind: fastn_resolved::Kind::boolean().into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ],
                expression: vec![
                    fastn_resolved::FunctionExpression {
                        expression: "ftd.set_rive_boolean(rive, input, value)".to_string(),
                        line_number: 0,
                    }
                ],
                js: None,
                line_number: 0,
                external_implementation: true,
            })
        ),
        (
            "ftd#toggle-rive-boolean".to_string(),
            fastn_resolved::Definition::Function(fastn_resolved::Function {
                name: "ftd#toggle-rive-boolean".to_string(),
                return_kind: fastn_resolved::KindData {
                    kind: fastn_resolved::Kind::void(),
                    caption: false,
                    body: false,
                },
                arguments: vec![
                    fastn_resolved::Argument {
                        name: "rive".to_string(),
                        kind: fastn_resolved::Kind::string().into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Argument {
                        name: "input".to_string(),
                        kind: fastn_resolved::Kind::string().into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ],
                expression: vec![
                    fastn_resolved::FunctionExpression {
                        expression: "ftd.toggle_rive_boolean(rive, input)".to_string(),
                        line_number: 0,
                    }
                ],
                js: None,
                line_number: 0,
                external_implementation: true
            })
        ),
        (
            "ftd#set-rive-integer".to_string(),
            fastn_resolved::Definition::Function(fastn_resolved::Function {
                name: "ftd#set-rive-integer".to_string(),
                return_kind: fastn_resolved::KindData {
                    kind: fastn_resolved::Kind::void(),
                    caption: false,
                    body: false,
                },
                arguments: vec![
                    fastn_resolved::Argument {
                        name: "rive".to_string(),
                        kind: fastn_resolved::Kind::string().into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Argument {
                        name: "input".to_string(),
                        kind: fastn_resolved::Kind::string().into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Argument {
                        name: "value".to_string(),
                        kind: fastn_resolved::Kind::integer().into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ],
                expression: vec![
                    fastn_resolved::FunctionExpression {
                        expression: "ftd.set_rive_integer(rive, input, value)".to_string(),
                        line_number: 0,
                    }
                ],
                js: None,
                line_number: 0,
                external_implementation: true
            })
        ),
        (
            "ftd#fire-rive".to_string(),
            fastn_resolved::Definition::Function(fastn_resolved::Function {
                name: "ftd#fire-rive".to_string(),
                return_kind: fastn_resolved::KindData {
                    kind: fastn_resolved::Kind::void(),
                    caption: false,
                    body: false,
                },
                arguments: vec![
                    fastn_resolved::Argument {
                        name: "rive".to_string(),
                        kind: fastn_resolved::Kind::string().into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Argument {
                        name: "input".to_string(),
                        kind: fastn_resolved::Kind::string().into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ],
                expression: vec![
                    fastn_resolved::FunctionExpression {
                        expression: "ftd.fire_rive(rive, input)".to_string(),
                        line_number: 0,
                    }
                ],
                js: None,
                line_number: 0,
                external_implementation: true
            })
        ),
        (
            "ftd#play-rive".to_string(),
            fastn_resolved::Definition::Function(fastn_resolved::Function {
                name: "ftd#play-rive".to_string(),
                return_kind: fastn_resolved::KindData {
                    kind: fastn_resolved::Kind::void(),
                    caption: false,
                    body: false,
                },
                arguments: vec![
                    fastn_resolved::Argument {
                        name: "rive".to_string(),
                        kind: fastn_resolved::Kind::string().into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Argument {
                        name: "input".to_string(),
                        kind: fastn_resolved::Kind::string().into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ],
                expression: vec![
                    fastn_resolved::FunctionExpression {
                        expression: "ftd.play_rive(rive, input)".to_string(),
                        line_number: 0,
                    }
                ],
                js: None,
                line_number: 0,
                external_implementation: true
            })
        ),
        (
            "ftd#pause-rive".to_string(),
            fastn_resolved::Definition::Function(fastn_resolved::Function {
                name: "ftd#pause-rive".to_string(),
                return_kind: fastn_resolved::KindData {
                    kind: fastn_resolved::Kind::void(),
                    caption: false,
                    body: false,
                },
                arguments: vec![
                    fastn_resolved::Argument {
                        name: "rive".to_string(),
                        kind: fastn_resolved::Kind::string().into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Argument {
                        name: "input".to_string(),
                        kind: fastn_resolved::Kind::string().into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ],
                expression: vec![
                    fastn_resolved::FunctionExpression {
                        expression: "ftd.pause_rive(rive, input)".to_string(),
                        line_number: 0,
                    }
                ],
                js: None,
                line_number: 0,
                external_implementation: true
            })
        ),
        (
            "ftd#toggle-play-rive".to_string(),
            fastn_resolved::Definition::Function(fastn_resolved::Function {
                name: "ftd#toggle-play-rive".to_string(),
                return_kind: fastn_resolved::KindData {
                    kind: fastn_resolved::Kind::void(),
                    caption: false,
                    body: false,
                },
                arguments: vec![
                    fastn_resolved::Argument {
                        name: "rive".to_string(),
                        kind: fastn_resolved::Kind::string().into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Argument {
                        name: "input".to_string(),
                        kind: fastn_resolved::Kind::string().into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ],
                expression: vec![
                    fastn_resolved::FunctionExpression {
                        expression: "ftd.toggle_play_rive(rive, input)".to_string(),
                        line_number: 0,
                    }
                ],
                js: None,
                line_number: 0,
                external_implementation: true
            })
        ),
        (
            "ftd#toggle".to_string(),
            fastn_resolved::Definition::Function(fastn_resolved::Function {
                name: "ftd#toggle".to_string(),
                return_kind: fastn_resolved::KindData {
                    kind: fastn_resolved::Kind::void(),
                    caption: false,
                    body: false,
                },
                arguments: vec![
                    fastn_resolved::Argument {
                        name: "a".to_string(),
                        kind: fastn_resolved::KindData {
                            kind: fastn_resolved::Kind::boolean(),
                            caption: false,
                            body: false,
                        },
                        mutable: true,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ],
                expression: vec![
                    fastn_resolved::FunctionExpression {
                        expression: "a = !a".to_string(),
                        line_number: 0,
                    }
                ],
                js: None,
                line_number: 0,
                external_implementation: false
            })
        ),
        (
            "ftd#integer-field-with-default".to_string(),
            fastn_resolved::Definition::Function(fastn_resolved::Function {
                name: "ftd#integer-field-with-default".to_string(),
                return_kind: fastn_resolved::KindData {
                    kind: fastn_resolved::Kind::record("ftd#integer-field"),
                    caption: false,
                    body: false,
                },
                arguments: vec![
                    fastn_resolved::Argument {
                        name: "name".to_string(),
                        kind: fastn_resolved::KindData {
                            kind: fastn_resolved::Kind::string(),
                            caption: false,
                            body: false,
                        },
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Argument {
                        name: "default".to_string(),
                        kind: fastn_resolved::KindData {
                            kind: fastn_resolved::Kind::integer(),
                            caption: false,
                            body: false,
                        },
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ],
                expression: vec![
                    fastn_resolved::FunctionExpression {
                        expression: "ftd.field_with_default_js(name, default)".to_string(),
                        line_number: 0,
                    }
                ],
                js: None,
                line_number: 0,
                external_implementation: false
            })
        ),
        (
            "ftd#decimal-field-with-default".to_string(),
            fastn_resolved::Definition::Function(fastn_resolved::Function {
                name: "ftd#decimal-field-with-default".to_string(),
                return_kind: fastn_resolved::KindData {
                    kind: fastn_resolved::Kind::record("ftd#decimal-field"),
                    caption: false,
                    body: false,
                },
                arguments: vec![
                    fastn_resolved::Argument {
                        name: "name".to_string(),
                        kind: fastn_resolved::KindData {
                            kind: fastn_resolved::Kind::string(),
                            caption: false,
                            body: false,
                        },
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Argument {
                        name: "default".to_string(),
                        kind: fastn_resolved::KindData {
                            kind: fastn_resolved::Kind::decimal(),
                            caption: false,
                            body: false,
                        },
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ],
                expression: vec![
                    fastn_resolved::FunctionExpression {
                        expression: "ftd.field_with_default_js(name, default)".to_string(),
                        line_number: 0,
                    }
                ],
                js: None,
                line_number: 0,
                external_implementation: false
            })
        ),
        (
            "ftd#boolean-field-with-default".to_string(),
            fastn_resolved::Definition::Function(fastn_resolved::Function {
                name: "ftd#boolean-field-with-default".to_string(),
                return_kind: fastn_resolved::KindData {
                    kind: fastn_resolved::Kind::record("ftd#boolean-field"),
                    caption: false,
                    body: false,
                },
                arguments: vec![
                    fastn_resolved::Argument {
                        name: "name".to_string(),
                        kind: fastn_resolved::KindData {
                            kind: fastn_resolved::Kind::string(),
                            caption: false,
                            body: false,
                        },
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Argument {
                        name: "default".to_string(),
                        kind: fastn_resolved::KindData {
                            kind: fastn_resolved::Kind::boolean(),
                            caption: false,
                            body: false,
                        },
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ],
                expression: vec![
                    fastn_resolved::FunctionExpression {
                        expression: "ftd.field_with_default_js(name, default)".to_string(),
                        line_number: 0,
                    }
                ],
                js: None,
                line_number: 0,
                external_implementation: false
            })
        ),        (
            "ftd#string-field-with-default".to_string(),
            fastn_resolved::Definition::Function(fastn_resolved::Function {
                name: "ftd#string-field-with-default".to_string(),
                return_kind: fastn_resolved::KindData {
                    kind: fastn_resolved::Kind::record("ftd#string-field"),
                    caption: false,
                    body: false,
                },
                arguments: vec![
                    fastn_resolved::Argument {
                        name: "name".to_string(),
                        kind: fastn_resolved::KindData {
                            kind: fastn_resolved::Kind::string(),
                            caption: false,
                            body: false,
                        },
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Argument {
                        name: "default".to_string(),
                        kind: fastn_resolved::KindData {
                            kind: fastn_resolved::Kind::string(),
                            caption: false,
                            body: false,
                        },
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ],
                expression: vec![
                    fastn_resolved::FunctionExpression {
                        expression: "ftd.field_with_default_js(name, default)".to_string(),
                        line_number: 0,
                    }
                ],
                js: None,
                line_number: 0,
                external_implementation: false
            })
        ),
        (
            "ftd#increment".to_string(),
            fastn_resolved::Definition::Function(fastn_resolved::Function {
                name: "ftd#increment".to_string(),
                return_kind: fastn_resolved::KindData {
                    kind: fastn_resolved::Kind::void(),
                    caption: false,
                    body: false,
                },
                arguments: vec![
                    fastn_resolved::Argument {
                        name: "a".to_string(),
                        kind: fastn_resolved::KindData {
                            kind: fastn_resolved::Kind::integer(),
                            caption: false,
                            body: false,
                        },
                        mutable: true,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ],
                expression: vec![
                    fastn_resolved::FunctionExpression {
                        expression: "a = a + 1".to_string(),
                        line_number: 0,
                    }
                ],
                js: None,
                line_number: 0,
                external_implementation: false
            })
        ),
        (
            "ftd#increment-by".to_string(),
            fastn_resolved::Definition::Function(fastn_resolved::Function {
                name: "ftd#increment-by".to_string(),
                return_kind: fastn_resolved::KindData {
                    kind: fastn_resolved::Kind::void(),
                    caption: false,
                    body: false,
                },
                arguments: vec![
                    fastn_resolved::Argument {
                        name: "a".to_string(),
                        kind: fastn_resolved::KindData {
                            kind: fastn_resolved::Kind::integer(),
                            caption: false,
                            body: false,
                        },
                        mutable: true,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Argument {
                        name: "v".to_string(),
                        kind: fastn_resolved::KindData {
                            kind: fastn_resolved::Kind::integer(),
                            caption: false,
                            body: false,
                        },
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ],
                expression: vec![
                    fastn_resolved::FunctionExpression {
                        expression: "a = a + v".to_string(),
                        line_number: 0,
                    }
                ],
                js: None,
                line_number: 0,
                external_implementation: false
            })
        ),
        (
            "ftd#decrement".to_string(),
            fastn_resolved::Definition::Function(fastn_resolved::Function {
                name: "ftd#decrement".to_string(),
                return_kind: fastn_resolved::KindData {
                    kind: fastn_resolved::Kind::void(),
                    caption: false,
                    body: false,
                },
                arguments: vec![
                    fastn_resolved::Argument {
                        name: "a".to_string(),
                        kind: fastn_resolved::KindData {
                            kind: fastn_resolved::Kind::integer(),
                            caption: false,
                            body: false,
                        },
                        mutable: true,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ],
                expression: vec![
                    fastn_resolved::FunctionExpression {
                        expression: "a = a - 1".to_string(),
                        line_number: 0,
                    }
                ],
                js: None,
                line_number: 0,
                external_implementation: false
            })
        ),
        (
            "ftd#decrement-by".to_string(),
            fastn_resolved::Definition::Function(fastn_resolved::Function {
                name: "ftd#decrement-by".to_string(),
                return_kind: fastn_resolved::KindData {
                    kind: fastn_resolved::Kind::void(),
                    caption: false,
                    body: false,
                },
                arguments: vec![
                    fastn_resolved::Argument {
                        name: "a".to_string(),
                        kind: fastn_resolved::KindData {
                            kind: fastn_resolved::Kind::integer(),
                            caption: false,
                            body: false,
                        },
                        mutable: true,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Argument {
                        name: "v".to_string(),
                        kind: fastn_resolved::KindData {
                            kind: fastn_resolved::Kind::integer(),
                            caption: false,
                            body: false,
                        },
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ],
                expression: vec![
                    fastn_resolved::FunctionExpression {
                        expression: "a = a - v".to_string(),
                        line_number: 0,
                    }
                ],
                js: None,
                line_number: 0,
                external_implementation: false
            })
        ),
        (
            "ftd#enable-light-mode".to_string(),
            fastn_resolved::Definition::Function(fastn_resolved::Function {
                name: "ftd#enable-light-mode".to_string(),
                return_kind: fastn_resolved::KindData {
                    kind: fastn_resolved::Kind::void(),
                    caption: false,
                    body: false,
                },
                arguments: vec![
                ],
                expression: vec![
                    fastn_resolved::FunctionExpression {
                        expression: "enable_light_mode()".to_string(),
                        line_number: 0,
                    }
                ],
                js: None,
                line_number: 0,
                external_implementation: false
            })
        ),
        (
            "ftd#enable-dark-mode".to_string(),
            fastn_resolved::Definition::Function(fastn_resolved::Function {
                name: "ftd#enable-dark-mode".to_string(),
                return_kind: fastn_resolved::KindData {
                    kind: fastn_resolved::Kind::void(),
                    caption: false,
                    body: false,
                },
                arguments: vec![
                ],
                expression: vec![
                    fastn_resolved::FunctionExpression {
                        expression: "enable_dark_mode()".to_string(),
                        line_number: 0,
                    }
                ],
                js: None,
                line_number: 0,
                external_implementation: false
            })
        ),
        (
            "ftd#enable-system-mode".to_string(),
            fastn_resolved::Definition::Function(fastn_resolved::Function {
                name: "ftd#enable-system-mode".to_string(),
                return_kind: fastn_resolved::KindData {
                    kind: fastn_resolved::Kind::void(),
                    caption: false,
                    body: false,
                },
                arguments: vec![
                ],
                expression: vec![
                    fastn_resolved::FunctionExpression {
                        expression: "enable_system_mode()".to_string(),
                        line_number: 0,
                    }
                ],
                js: None,
                line_number: 0,
                external_implementation: false
            })
        ),
        (
            "ftd#clean-code".to_string(),
            fastn_resolved::Definition::Function(fastn_resolved::Function {
                name: "ftd#clean-code".to_string(),
                return_kind: fastn_resolved::KindData {
                    kind: fastn_resolved::Kind::string(),
                    caption: false,
                    body: false,
                },
                arguments: vec![
                    fastn_resolved::Argument {
                        name: "a".to_string(),
                        kind: fastn_resolved::KindData {
                            kind: fastn_resolved::Kind::string(),
                            caption: false,
                            body: false,
                        },
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Argument {
                        name: "lang".to_string(),
                        kind: fastn_resolved::KindData {
                            kind: fastn_resolved::Kind::string(),
                            caption: false,
                            body: false,
                        },
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ],
                expression: vec![
                    fastn_resolved::FunctionExpression {
                        expression: "ftd.clean_code(a, lang)".to_string(),
                        line_number: 0,
                    }
                ],
                js: None,
                line_number: 0,
                external_implementation: true
            })
        ),
        (
            "ftd#copy-to-clipboard".to_string(),
            fastn_resolved::Definition::Function(fastn_resolved::Function {
                name: "ftd#copy-to-clipboard".to_string(),
                return_kind: fastn_resolved::KindData {
                    kind: fastn_resolved::Kind::void(),
                    caption: false,
                    body: false,
                },
                arguments: vec![
                    fastn_resolved::Argument {
                        name: "a".to_string(),
                        kind: fastn_resolved::KindData {
                            kind: fastn_resolved::Kind::string(),
                            caption: false,
                            body: false,
                        },
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ],
                expression: vec![
                    fastn_resolved::FunctionExpression {
                        expression: "ftd.copy_to_clipboard(a)".to_string(),
                        line_number: 0,
                    }
                ],
                js: None,
                line_number: 0,
                external_implementation: true
            })
        ),
        (
            "ftd#set-bool".to_string(),
            fastn_resolved::Definition::Function(fastn_resolved::Function {
                name: "ftd#set-bool".to_string(),
                return_kind: fastn_resolved::KindData {
                    kind: fastn_resolved::Kind::void(),
                    caption: false,
                    body: false,
                },
                arguments: vec![
                    fastn_resolved::Argument {
                        name: "a".to_string(),
                        kind: fastn_resolved::KindData {
                            kind: fastn_resolved::Kind::boolean(),
                            caption: false,
                            body: false,
                        },
                        mutable: true,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Argument {
                        name: "v".to_string(),
                        kind: fastn_resolved::KindData {
                            kind: fastn_resolved::Kind::boolean(),
                            caption: false,
                            body: false,
                        },
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ],
                expression: vec![
                    fastn_resolved::FunctionExpression {
                        expression: "a = v".to_string(),
                        line_number: 0,
                    }
                ],
                js: None,
                line_number: 0,
                external_implementation: false
            })
        ),
        (
            "ftd#set-boolean".to_string(),
            fastn_resolved::Definition::Function(fastn_resolved::Function {
                name: "ftd#set-boolean".to_string(),
                return_kind: fastn_resolved::KindData {
                    kind: fastn_resolved::Kind::void(),
                    caption: false,
                    body: false,
                },
                arguments: vec![
                    fastn_resolved::Argument {
                        name: "a".to_string(),
                        kind: fastn_resolved::KindData {
                            kind: fastn_resolved::Kind::boolean(),
                            caption: false,
                            body: false,
                        },
                        mutable: true,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Argument {
                        name: "v".to_string(),
                        kind: fastn_resolved::KindData {
                            kind: fastn_resolved::Kind::boolean(),
                            caption: false,
                            body: false,
                        },
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ],
                expression: vec![
                    fastn_resolved::FunctionExpression {
                        expression: "a = v".to_string(),
                        line_number: 0,
                    }
                ],
                js: None,
                line_number: 0,
                external_implementation: false
            })
        ),
        (
            "ftd#set-string".to_string(),
            fastn_resolved::Definition::Function(fastn_resolved::Function {
                name: "ftd#set-string".to_string(),
                return_kind: fastn_resolved::KindData {
                    kind: fastn_resolved::Kind::void(),
                    caption: false,
                    body: false,
                },
                arguments: vec![
                    fastn_resolved::Argument {
                        name: "a".to_string(),
                        kind: fastn_resolved::KindData {
                            kind: fastn_resolved::Kind::string(),
                            caption: false,
                            body: false,
                        },
                        mutable: true,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Argument {
                        name: "v".to_string(),
                        kind: fastn_resolved::KindData {
                            kind: fastn_resolved::Kind::string(),
                            caption: false,
                            body: false,
                        },
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ],
                expression: vec![
                    fastn_resolved::FunctionExpression {
                        expression: "a = v".to_string(),
                        line_number: 0,
                    }
                ],
                js: None,
                line_number: 0,
                external_implementation: false
            })
        ),
        (
            "ftd#set-integer".to_string(),
            fastn_resolved::Definition::Function(fastn_resolved::Function {
                name: "ftd#set-integer".to_string(),
                return_kind: fastn_resolved::KindData {
                    kind: fastn_resolved::Kind::void(),
                    caption: false,
                    body: false,
                },
                arguments: vec![
                    fastn_resolved::Argument {
                        name: "a".to_string(),
                        kind: fastn_resolved::KindData {
                            kind: fastn_resolved::Kind::integer(),
                            caption: false,
                            body: false,
                        },
                        mutable: true,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Argument {
                        name: "v".to_string(),
                        kind: fastn_resolved::KindData {
                            kind: fastn_resolved::Kind::integer(),
                            caption: false,
                            body: false,
                        },
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ],
                expression: vec![
                    fastn_resolved::FunctionExpression {
                        expression: "a = v".to_string(),
                        line_number: 0,
                    }
                ],
                js: None,
                line_number: 0,
                external_implementation: false
            })
        ),
        (
            fastn_builtins::constants::FTD_IMAGE_SRC.to_string(),
            fastn_resolved::Definition::Record(fastn_resolved::Record {
                name: fastn_builtins::constants::FTD_IMAGE_SRC.to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    fastn_resolved::Field {
                        name: "light".to_string(),
                        kind: fastn_resolved::Kind::string().into_kind_data().caption(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "dark".to_string(),
                        kind: fastn_resolved::Kind::string().into_kind_data(),
                        mutable: false,
                        default: Some(fastn_resolved::PropertyValue::Reference {
                            name: fastn_builtins::constants::FTD_IMAGE_SRC_LIGHT.to_string(),
                            kind: fastn_resolved::Kind::string().into_kind_data(),
                            source: fastn_resolved::PropertyValueSource::Local(
                                fastn_builtins::constants::FTD_IMAGE_SRC.to_string(),
                            ),
                            is_mutable: false,
                            line_number: 0,
                        }),
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ])
                    .collect(),
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_VIDEO_SRC.to_string(),
            fastn_resolved::Definition::Record(fastn_resolved::Record {
                name: fastn_builtins::constants::FTD_VIDEO_SRC.to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    fastn_resolved::Field {
                        name: "light".to_string(),
                        kind: fastn_resolved::Kind::string().into_kind_data().caption(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "dark".to_string(),
                        kind: fastn_resolved::Kind::string().into_kind_data(),
                        mutable: false,
                        default: Some(fastn_resolved::PropertyValue::Reference {
                            name: fastn_builtins::constants::FTD_VIDEO_SRC_LIGHT.to_string(),
                            kind: fastn_resolved::Kind::string().into_kind_data(),
                            source: fastn_resolved::PropertyValueSource::Local(
                                fastn_builtins::constants::FTD_VIDEO_SRC.to_string(),
                            ),
                            is_mutable: false,
                            line_number: 0,
                        }),
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ])
                    .collect(),
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_RAW_IMAGE_SRC.to_string(),
            fastn_resolved::Definition::Record(fastn_resolved::Record {
                name: fastn_builtins::constants::FTD_RAW_IMAGE_SRC.to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    fastn_resolved::Field {
                        name: "src".to_string(),
                        kind: fastn_resolved::Kind::string().into_kind_data().caption(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ])
                    .collect(),
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_COLOR.to_string(),
            fastn_resolved::Definition::Record(fastn_resolved::Record {
                name: fastn_builtins::constants::FTD_COLOR.to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    fastn_resolved::Field {
                        name: "light".to_string(),
                        kind: fastn_resolved::Kind::string().into_kind_data().caption(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "dark".to_string(),
                        kind: fastn_resolved::Kind::string().into_kind_data(),
                        mutable: false,
                        default: Some(fastn_resolved::PropertyValue::Reference {
                            name: fastn_builtins::constants::FTD_COLOR_LIGHT.to_string(),
                            kind: fastn_resolved::Kind::string().into_kind_data(),
                            source: fastn_resolved::PropertyValueSource::Local(
                                fastn_builtins::constants::FTD_COLOR.to_string(),
                            ),
                            is_mutable: false,
                            line_number: 0,
                        }),
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ])
                    .collect(),
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_SHADOW.to_string(),
            fastn_resolved::Definition::Record(fastn_resolved::Record {
                name: fastn_builtins::constants::FTD_SHADOW.to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    fastn_resolved::Field {
                        name: "x-offset".to_string(),
                        kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                            .into_kind_data(),
                        mutable: false,
                        default: Some(fastn_resolved::PropertyValue::Value {
                            value: fastn_resolved::Value::OrType {
                                name: fastn_builtins::constants::FTD_LENGTH.to_string(),
                                variant: fastn_builtins::constants::FTD_LENGTH_PX.to_string(),
                                full_variant: fastn_builtins::constants::FTD_LENGTH_PX.to_string(),
                                value: Box::new
                                    (fastn_resolved::PropertyValue::Value {
                                        value: fastn_resolved::Value::Integer {
                                            value: 0
                                        },
                                        is_mutable: false,
                                        line_number: 0
                                    }),
                            },
                            is_mutable: false,
                            line_number: 0,
                        }),
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "y-offset".to_string(),
                        kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                            .into_kind_data(),
                        mutable: false,
                        default: Some(fastn_resolved::PropertyValue::Value {
                            value: fastn_resolved::Value::OrType {
                                name: fastn_builtins::constants::FTD_LENGTH.to_string(),
                                variant: fastn_builtins::constants::FTD_LENGTH_PX.to_string(),
                                full_variant: fastn_builtins::constants::FTD_LENGTH_PX.to_string(),
                                value: Box::new
                                    (fastn_resolved::PropertyValue::Value {
                                        value: fastn_resolved::Value::Integer {
                                            value: 0
                                        },
                                        is_mutable: false,
                                        line_number: 0
                                    }),
                            },
                            is_mutable: false,
                            line_number: 0,
                        }),
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "blur".to_string(),
                        kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                            .into_kind_data(),
                        mutable: false,
                        default: Some(fastn_resolved::PropertyValue::Value {
                            value: fastn_resolved::Value::OrType {
                                name: fastn_builtins::constants::FTD_LENGTH.to_string(),
                                variant: fastn_builtins::constants::FTD_LENGTH_PX.to_string(),
                                full_variant: fastn_builtins::constants::FTD_LENGTH_PX.to_string(),
                                value: Box::new
                                    (fastn_resolved::PropertyValue::Value {
                                        value: fastn_resolved::Value::Integer {
                                            value: 0
                                        },
                                        is_mutable: false,
                                        line_number: 0
                                    }),
                            },
                            is_mutable: false,
                            line_number: 0,
                        }),
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "spread".to_string(),
                        kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                            .into_kind_data(),
                        mutable: false,
                        default: Some(fastn_resolved::PropertyValue::Value {
                            value: fastn_resolved::Value::OrType {
                                name: fastn_builtins::constants::FTD_LENGTH.to_string(),
                                variant: fastn_builtins::constants::FTD_LENGTH_PX.to_string(),
                                full_variant: fastn_builtins::constants::FTD_LENGTH_PX.to_string(),
                                value: Box::new
                                    (fastn_resolved::PropertyValue::Value {
                                        value: fastn_resolved::Value::Integer {
                                            value: 0
                                        },
                                        is_mutable: false,
                                        line_number: 0
                                    }),
                            },
                            is_mutable: false,
                            line_number: 0,
                        }),
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "color".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        access_modifier: Default::default(),
                        default: Some(fastn_resolved::PropertyValue::Value {
                            value: fastn_resolved::Value::Record {
                                name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                fields: std::iter::IntoIterator::into_iter([
                                    (
                                        "light".to_string(),
                                        fastn_resolved::PropertyValue::Value {
                                            value: fastn_resolved::Value::String { text: "black".to_string() },
                                            is_mutable: false,
                                            line_number: 0,
                                        }
                                    ),
                                    (
                                        "dark".to_string(),
                                        fastn_resolved::PropertyValue::Value {
                                            value: fastn_resolved::Value::String { text: "white".to_string() },
                                            is_mutable: false,
                                            line_number: 0,
                                        }
                                    ),
                                ]).collect()
                            },
                            is_mutable: false,
                            line_number: 0,
                        }),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "inset".to_string(),
                        kind: fastn_resolved::Kind::boolean()
                            .into_kind_data(),
                        mutable: false,
                        access_modifier: Default::default(),
                        default: Some(fastn_resolved::PropertyValue::Value {
                            value: fastn_resolved::Value::Boolean { value: false },
                            is_mutable: false,
                            line_number: 0,
                        }),
                        line_number: 0,
                    },
                ]).collect(),
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_BACKDROP_FILTER.to_string(),
            fastn_resolved::Definition::OrType(fastn_resolved::OrType {
                name: fastn_builtins::constants::FTD_BACKDROP_FILTER.to_string(),
                variants: vec![
                    fastn_resolved::OrTypeVariant::Regular(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BACKDROP_FILTER_BLUR,
                        fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                            .into_kind_data(),
                        false,
                        None,
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Regular(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BACKDROP_FILTER_BRIGHTNESS,
                        fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                            .into_kind_data(),
                        false,
                        None,
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Regular(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BACKDROP_FILTER_CONTRAST,
                        fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                            .into_kind_data(),
                        false,
                        None,
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Regular(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BACKDROP_FILTER_GRAYSCALE,
                        fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                            .into_kind_data(),
                        false,
                        None,
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Regular(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BACKDROP_FILTER_INVERT,
                        fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                            .into_kind_data(),
                        false,
                        None,
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Regular(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BACKDROP_FILTER_OPACITY,
                        fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                            .into_kind_data(),
                        false,
                        None,
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Regular(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BACKDROP_FILTER_SEPIA,
                        fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                            .into_kind_data(),
                        false,
                        None,
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Regular(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BACKDROP_FILTER_SATURATE,
                        fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                            .into_kind_data(),
                        false,
                        None,
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Regular(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BACKDROP_FILTER_MULTI,
                        fastn_resolved::Kind::record(fastn_builtins::constants::FTD_BACKDROP_MULTI)
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
            fastn_builtins::constants::FTD_BACKDROP_MULTI.to_string(),
            fastn_resolved::Definition::Record(fastn_resolved::Record {
                name: fastn_builtins::constants::FTD_BACKDROP_MULTI.to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    fastn_resolved::Field {
                        name: "blur".to_string(),
                        kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                            .into_kind_data().into_optional(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "brightness".to_string(),
                        kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                            .into_kind_data().into_optional(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "contrast".to_string(),
                        kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                            .into_kind_data().into_optional(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "grayscale".to_string(),
                        kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                            .into_kind_data().into_optional(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "invert".to_string(),
                        kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                            .into_kind_data().into_optional(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "opacity".to_string(),
                        kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                            .into_kind_data().into_optional(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "sepia".to_string(),
                        kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                            .into_kind_data().into_optional(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "saturate".to_string(),
                        kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                            .into_kind_data().into_optional(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ]).collect(),
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_LENGTH_PAIR.to_string(),
            fastn_resolved::Definition::Record(fastn_resolved::Record {
                name: fastn_builtins::constants::FTD_LENGTH_PAIR.to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    fastn_resolved::Field {
                        name: "x".to_string(),
                        kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "y".to_string(),
                        kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ]).collect(),
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_BG_IMAGE.to_string(),
            fastn_resolved::Definition::Record(fastn_resolved::Record {
                name: fastn_builtins::constants::FTD_BG_IMAGE.to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    fastn_resolved::Field {
                        name: "src".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_IMAGE_SRC)
                            .into_kind_data().caption(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "repeat".to_string(),
                        kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_BACKGROUND_REPEAT)
                            .into_kind_data().into_optional(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "size".to_string(),
                        kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_BACKGROUND_SIZE)
                            .into_kind_data().into_optional(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "position".to_string(),
                        kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_BACKGROUND_POSITION)
                            .into_kind_data().into_optional(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ]).collect(),
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_LINEAR_GRADIENT_COLOR.to_string(),
            fastn_resolved::Definition::Record(fastn_resolved::Record {
                name: fastn_builtins::constants::FTD_LINEAR_GRADIENT_COLOR.to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    fastn_resolved::Field {
                        name: "color".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data().caption(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "start".to_string(),
                        kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                            .into_kind_data().into_optional(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "end".to_string(),
                        kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                            .into_kind_data().into_optional(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "stop-position".to_string(),
                        kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                            .into_kind_data().into_optional(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ]).collect(),
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_LINEAR_GRADIENT_DIRECTIONS.to_string(),
            fastn_resolved::Definition::OrType(fastn_resolved::OrType {
                name: fastn_builtins::constants::FTD_LINEAR_GRADIENT_DIRECTIONS.to_string(),
                variants: vec![
                    fastn_resolved::OrTypeVariant::Regular(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_LINEAR_GRADIENT_DIRECTIONS_ANGLE,
                        fastn_resolved::Kind::decimal()
                            .into_kind_data(),
                        false,
                        None,
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Regular(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_LINEAR_GRADIENT_DIRECTIONS_TURN,
                        fastn_resolved::Kind::decimal()
                            .into_kind_data(),
                        false,
                        None,
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_LINEAR_GRADIENT_DIRECTIONS_LEFT,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("to left")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_LINEAR_GRADIENT_DIRECTIONS_RIGHT,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("to right")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_LINEAR_GRADIENT_DIRECTIONS_TOP,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("to top")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_LINEAR_GRADIENT_DIRECTIONS_BOTTOM,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("to bottom")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_LINEAR_GRADIENT_DIRECTIONS_TOP_LEFT,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("to top left")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_LINEAR_GRADIENT_DIRECTIONS_BOTTOM_LEFT,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("to bottom left")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_LINEAR_GRADIENT_DIRECTIONS_TOP_RIGHT,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("to top right")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_LINEAR_GRADIENT_DIRECTIONS_BOTTOM_RIGHT,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("to bottom right")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                ],
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_LINEAR_GRADIENT.to_string(),
            fastn_resolved::Definition::Record(fastn_resolved::Record {
                name: fastn_builtins::constants::FTD_LINEAR_GRADIENT.to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    fastn_resolved::Field {
                        name: "direction".to_string(),
                        kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LINEAR_GRADIENT_DIRECTIONS)
                            .into_kind_data().into_optional(),
                        mutable: false,
                        default: Some(fastn_resolved::PropertyValue::Value {
                            value: fastn_resolved::Value::OrType {
                                name: fastn_builtins::constants::FTD_LINEAR_GRADIENT_DIRECTIONS.to_string(),
                                variant: fastn_builtins::constants::FTD_LINEAR_GRADIENT_DIRECTIONS_BOTTOM
                                    .to_string(),
                                full_variant: fastn_builtins::constants::FTD_LINEAR_GRADIENT_DIRECTIONS_BOTTOM.to_string(),
                                value: Box::new
                                    (fastn_resolved::PropertyValue::Value {
                                        value: fastn_resolved::Value::String {
                                            text: "bottom".to_string(),
                                        },
                                        is_mutable: false,
                                        line_number: 0
                                    }),
                            },
                            is_mutable: false,
                            line_number: 0,
                        }),
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "colors".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_LINEAR_GRADIENT_COLOR)
                            .into_list().into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ]).collect(),
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_BACKGROUND.to_string(),
            fastn_resolved::Definition::OrType(fastn_resolved::OrType {
                name: fastn_builtins::constants::FTD_BACKGROUND.to_string(),
                variants: vec![
                    fastn_resolved::OrTypeVariant::Regular(
                        fastn_resolved::Field::new(
                            fastn_builtins::constants::FTD_BACKGROUND_SOLID,
                            fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                                .into_kind_data(),
                            false,
                            None,
                            0,
                        )),
                    fastn_resolved::OrTypeVariant::Regular(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BACKGROUND_IMAGE,
                        fastn_resolved::Kind::record(fastn_builtins::constants::FTD_BG_IMAGE)
                            .into_kind_data(),
                        false,
                        None,
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Regular(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BACKGROUND_LINEAR_GRADIENT,
                        fastn_resolved::Kind::record(fastn_builtins::constants::FTD_LINEAR_GRADIENT)
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
            fastn_builtins::constants::FTD_BACKGROUND_REPEAT.to_string(),
            fastn_resolved::Definition::OrType(fastn_resolved::OrType {
                name: fastn_builtins::constants::FTD_BACKGROUND_REPEAT.to_string(),
                variants: vec![
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BACKGROUND_REPEAT_BOTH_REPEAT,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("repeat")
                                 .into_property_value(false, 0),),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BACKGROUND_REPEAT_X_REPEAT,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("repeat-x")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BACKGROUND_REPEAT_Y_REPEAT,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("repeat-y")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BACKGROUND_REPEAT_NO_REPEAT,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("no-repeat")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BACKGROUND_REPEAT_SPACE,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("space")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BACKGROUND_REPEAT_ROUND,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("round")
                            .into_property_value(false, 0)),
                        0,
                    )),
                ],
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_BACKGROUND_SIZE.to_string(),
            fastn_resolved::Definition::OrType(fastn_resolved::OrType {
                name: fastn_builtins::constants::FTD_BACKGROUND_SIZE.to_string(),
                variants: vec![
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BACKGROUND_SIZE_AUTO,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("auto")
                                 .into_property_value(false, 0),),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BACKGROUND_SIZE_COVER,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("cover")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BACKGROUND_SIZE_CONTAIN,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("contain")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::AnonymousRecord(fastn_resolved::Record {
                        name: fastn_builtins::constants::FTD_BACKGROUND_SIZE_LENGTH.to_string(),
                        fields: std::iter::IntoIterator::into_iter([
                            fastn_resolved::Field {
                                name: "x".to_string(),
                                kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                                    .into_kind_data(),
                                mutable: false,
                                default: None,
                                access_modifier: Default::default(),
                                line_number: 0,
                            },
                            fastn_resolved::Field {
                                name: "y".to_string(),
                                kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                                    .into_kind_data(),
                                mutable: false,
                                default: None,
                                access_modifier: Default::default(),
                                line_number: 0,
                            },
                        ]).collect(),
                        line_number: 0,
                    }),
                ],
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_BACKGROUND_POSITION.to_string(),
            fastn_resolved::Definition::OrType(fastn_resolved::OrType {
                name: fastn_builtins::constants::FTD_BACKGROUND_POSITION.to_string(),
                variants: vec![
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BACKGROUND_POSITION_LEFT,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("left")
                                 .into_property_value(false, 0),),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BACKGROUND_POSITION_CENTER,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("center")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BACKGROUND_POSITION_RIGHT,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("right")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BACKGROUND_POSITION_LEFT_TOP,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("left-top")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BACKGROUND_POSITION_LEFT_CENTER,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("left-center")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BACKGROUND_POSITION_LEFT_BOTTOM,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("left-bottom")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BACKGROUND_POSITION_CENTER_TOP,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("center-top")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BACKGROUND_POSITION_CENTER_CENTER,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("center-center")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BACKGROUND_POSITION_CENTER_BOTTOM,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("center-bottom")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BACKGROUND_POSITION_RIGHT_TOP,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("right-top")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BACKGROUND_POSITION_RIGHT_CENTER,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("right-center")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BACKGROUND_POSITION_RIGHT_BOTTOM,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("right-bottom")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::AnonymousRecord(fastn_resolved::Record {
                        name: fastn_builtins::constants::FTD_BACKGROUND_POSITION_LENGTH.to_string(),
                        fields: std::iter::IntoIterator::into_iter([
                            fastn_resolved::Field {
                                name: "x".to_string(),
                                kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                                    .into_kind_data(),
                                mutable: false,
                                default: None,
                                access_modifier: Default::default(),
                                line_number: 0,
                            },
                            fastn_resolved::Field {
                                name: "y".to_string(),
                                kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                                    .into_kind_data(),
                                mutable: false,
                                default: None,
                                access_modifier: Default::default(),
                                line_number: 0,
                            },
                        ]).collect(),
                        line_number: 0,
                    }),
                ],
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_ALIGN.to_string(),
            fastn_resolved::Definition::OrType(fastn_resolved::OrType {
                name: fastn_builtins::constants::FTD_ALIGN.to_string(),
                variants: vec![
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_ALIGN_TOP_LEFT,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string(
                                fastn_builtins::constants::FTD_ALIGN_TOP_LEFT,
                            )
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_ALIGN_TOP_CENTER,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string(
                                fastn_builtins::constants::FTD_ALIGN_TOP_CENTER,
                            )
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_ALIGN_TOP_RIGHT,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string(
                                fastn_builtins::constants::FTD_ALIGN_TOP_RIGHT,
                            )
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_ALIGN_LEFT,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string(fastn_builtins::constants::FTD_ALIGN_LEFT)
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_ALIGN_CENTER,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string(
                                fastn_builtins::constants::FTD_ALIGN_CENTER,
                            )
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_ALIGN_RIGHT,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string(
                                fastn_builtins::constants::FTD_ALIGN_RIGHT,
                            )
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_ALIGN_BOTTOM_LEFT,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string(
                                fastn_builtins::constants::FTD_ALIGN_BOTTOM_LEFT,
                            )
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_ALIGN_BOTTOM_CENTER,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string(
                                fastn_builtins::constants::FTD_ALIGN_BOTTOM_CENTER,
                            )
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_ALIGN_BOTTOM_RIGHT,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string(
                                fastn_builtins::constants::FTD_ALIGN_BOTTOM_RIGHT,
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
            fastn_builtins::constants::FTD_SPACING.to_string(),
            fastn_resolved::Definition::OrType(fastn_resolved::OrType {
                name: fastn_builtins::constants::FTD_SPACING.to_string(),
                variants: vec![
                    fastn_resolved::OrTypeVariant::Regular(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_SPACING_FIXED,
                        fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                            .into_kind_data(),
                        false,
                        None,
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_SPACING_SPACE_BETWEEN,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("space-between")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_SPACING_SPACE_EVENLY,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("space-evenly")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_SPACING_SPACE_AROUND,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("space-around")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                ],
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_IMAGE_FIT.to_string(),
            fastn_resolved::Definition::OrType(fastn_resolved::OrType {
                name: fastn_builtins::constants::FTD_IMAGE_FIT.to_string(),
                variants: vec![
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_IMAGE_FIT_NONE,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("none")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_IMAGE_FIT_COVER,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("cover")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_IMAGE_FIT_CONTAIN,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("contain")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_IMAGE_FIT_FILL,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("fill")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_IMAGE_FIT_SCALE_DOWN,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("scale-down")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),

                ],
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_IMAGE_FETCH_PRIORITY.to_string(),
            fastn_resolved::Definition::OrType(fastn_resolved::OrType {
                name: fastn_builtins::constants::FTD_IMAGE_FETCH_PRIORITY.to_string(),
                variants: vec![
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_IMAGE_FETCH_PRIORITY_AUTO,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("auto")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_IMAGE_FETCH_PRIORITY_LOW,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("low")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_IMAGE_FETCH_PRIORITY_HIGH,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("high")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                ],
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_ANCHOR.to_string(),
            fastn_resolved::Definition::OrType(fastn_resolved::OrType {
                name: fastn_builtins::constants::FTD_ANCHOR.to_string(),
                variants: vec![
                    fastn_resolved::OrTypeVariant::Regular(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_ANCHOR_ID,
                        fastn_resolved::Kind::string()
                            .into_kind_data(),
                        false,
                        None,
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_ANCHOR_PARENT,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("absolute")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_ANCHOR_WINDOW,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("fixed")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                ],
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_OVERFLOW.to_string(),
            fastn_resolved::Definition::OrType(fastn_resolved::OrType {
                name: fastn_builtins::constants::FTD_OVERFLOW.to_string(),
                variants: vec![
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_OVERFLOW_SCROLL,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("scroll")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_OVERFLOW_VISIBLE,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("visible")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_OVERFLOW_HIDDEN,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("hidden")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_OVERFLOW_AUTO,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("auto")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                ],
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_RESIZE.to_string(),
            fastn_resolved::Definition::OrType(fastn_resolved::OrType {
                name: fastn_builtins::constants::FTD_RESIZE.to_string(),
                variants: vec![
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_RESIZE_HORIZONTAL,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("horizontal")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_RESIZE_VERTICAL,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("vertical")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_RESIZE_BOTH,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("both")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                ],
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_CURSOR.to_string(),
            fastn_resolved::Definition::OrType(fastn_resolved::OrType {
                name: fastn_builtins::constants::FTD_CURSOR.to_string(),
                variants: vec![
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_CURSOR_DEFAULT,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("default")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_CURSOR_NONE,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("none")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_CURSOR_CONTEXT_MENU,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("context-menu")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_CURSOR_HELP,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("help")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_CURSOR_POINTER,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("pointer")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_CURSOR_PROGRESS,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("progress")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_CURSOR_WAIT,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("wait")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_CURSOR_CELL,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("cell")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_CURSOR_CROSSHAIR,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("crosshair")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_CURSOR_TEXT,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("text")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_CURSOR_VERTICAL_TEXT,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("vertical-text")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_CURSOR_ALIAS,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("alias")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_CURSOR_COPY,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("copy")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_CURSOR_MOVE,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("move")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_CURSOR_NO_DROP,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("no-drop")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_CURSOR_NOT_ALLOWED,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("not-allowed")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_CURSOR_GRAB,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("grab")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_CURSOR_GRABBING,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("grabbing")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_CURSOR_E_RESIZE,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("e-resize")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_CURSOR_N_RESIZE,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("n-resize")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_CURSOR_NE_RESIZE,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("ne-resize")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_CURSOR_NW_RESIZE,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("nw-resize")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_CURSOR_S_RESIZE,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("s-resize")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_CURSOR_SE_RESIZE,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("se-resize")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_CURSOR_SW_RESIZE,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("sw-resize")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_CURSOR_W_RESIZE,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("w-resize")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_CURSOR_EW_RESIZE,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("ew-resize")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_CURSOR_NS_RESIZE,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("ns-resize")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_CURSOR_NESW_RESIZE,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("nesw-resize")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_CURSOR_NWSE_RESIZE,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("nwse-resize")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_CURSOR_COL_RESIZE,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("col-resize")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_CURSOR_ROW_RESIZE,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("row-resize")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_CURSOR_ALL_SCROLL,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("all-scroll")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_CURSOR_ZOOM_IN,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("zoom-in")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_CURSOR_ZOOM_OUT,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("zoom-out")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                ],
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_ALIGN_SELF.to_string(),
            fastn_resolved::Definition::OrType(fastn_resolved::OrType {
                name: fastn_builtins::constants::FTD_ALIGN_SELF.to_string(),
                variants: vec![
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_ALIGN_SELF_START,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("start")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_ALIGN_SELF_CENTER,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("center")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_ALIGN_SELF_END,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("end")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                ],
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_TEXT_ALIGN.to_string(),
            fastn_resolved::Definition::OrType(fastn_resolved::OrType {
                name: fastn_builtins::constants::FTD_TEXT_ALIGN.to_string(),
                variants: vec![
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_TEXT_ALIGN_START,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("start")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_TEXT_ALIGN_CENTER,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("center")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_TEXT_ALIGN_END,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("end")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_TEXT_ALIGN_JUSTIFY,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("justify")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                ],
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_LINK_REL.to_string(),
            fastn_resolved::Definition::OrType(fastn_resolved::OrType {
                name: fastn_builtins::constants::FTD_LINK_REL.to_string(),
                variants: vec![
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_LINK_REL_NO_FOLLOW,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("no-follow")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_LINK_REL_SPONSORED,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("sponsored")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_LINK_REL_UGC,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("ugc")
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                ],
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_RESIZING.to_string(),
            fastn_resolved::Definition::OrType(fastn_resolved::OrType {
                name: fastn_builtins::constants::FTD_RESIZING.to_string(),
                variants: vec![
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_RESIZING_HUG_CONTENT,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string(
                                fastn_builtins::constants::FTD_RESIZING_HUG_CONTENT,
                            )
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_RESIZING_AUTO,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string(
                                fastn_builtins::constants::FTD_RESIZING_AUTO,
                            )
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_RESIZING_FILL_CONTAINER,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string(
                                fastn_builtins::constants::FTD_RESIZING_FILL_CONTAINER,
                            )
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Regular(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_RESIZING_FIXED,
                        fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
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
            fastn_builtins::constants::FTD_WHITESPACE.to_string(),
            fastn_resolved::Definition::OrType(fastn_resolved::OrType {
                name: fastn_builtins::constants::FTD_WHITESPACE.to_string(),
                variants: vec![
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_WHITESPACE_NORMAL,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("normal")
                                 .into_property_value(false, 0),),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_WHITESPACE_NOWRAP,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("nowrap")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_WHITESPACE_PRE,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("pre")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_WHITESPACE_PREWRAP,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("pre-wrap")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_WHITESPACE_PRELINE,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("pre-line")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_WHITESPACE_BREAKSPACES,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("break-spaces")
                            .into_property_value(false, 0)),
                        0,
                    )),
                ],
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_DISPLAY.to_string(),
            fastn_resolved::Definition::OrType(fastn_resolved::OrType {
                name: fastn_builtins::constants::FTD_DISPLAY.to_string(),
                variants: vec![
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_DISPLAY_BLOCK,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("block")
                                 .into_property_value(false, 0),),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_DISPLAY_INLINE,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("inline")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_DISPLAY_INLINE_BLOCK,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("inline-block")
                            .into_property_value(false, 0)),
                        0,
                    )),
                ],
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_LENGTH.to_string(),
            fastn_resolved::Definition::OrType(fastn_resolved::OrType {
                name: fastn_builtins::constants::FTD_LENGTH.to_string(),
                variants: vec![
                    fastn_resolved::OrTypeVariant::Regular(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_LENGTH_PX,
                        fastn_resolved::Kind::integer()
                            .into_kind_data()
                            .caption(),
                        false,
                        None,
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Regular(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_LENGTH_PERCENT,
                        fastn_resolved::Kind::decimal()
                            .into_kind_data()
                            .caption(),
                        false,
                        None,
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Regular(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_LENGTH_CALC,
                        fastn_resolved::Kind::string().into_kind_data().caption(),
                        false,
                        None,
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Regular(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_LENGTH_VH,
                        fastn_resolved::Kind::decimal()
                            .into_kind_data()
                            .caption(),
                        false,
                        None,
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Regular(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_LENGTH_VW,
                        fastn_resolved::Kind::decimal()
                            .into_kind_data()
                            .caption(),
                        false,
                        None,
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Regular(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_LENGTH_VMIN,
                        fastn_resolved::Kind::decimal()
                            .into_kind_data()
                            .caption(),
                        false,
                        None,
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Regular(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_LENGTH_VMAX,
                        fastn_resolved::Kind::decimal()
                            .into_kind_data()
                            .caption(),
                        false,
                        None,
                        0,
                    )),

                    fastn_resolved::OrTypeVariant::Regular(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_LENGTH_DVH,
                        fastn_resolved::Kind::decimal()
                            .into_kind_data()
                            .caption(),
                        false,
                        None,
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Regular(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_LENGTH_LVH,
                        fastn_resolved::Kind::decimal()
                            .into_kind_data()
                            .caption(),
                        false,
                        None,
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Regular(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_LENGTH_SVH,
                        fastn_resolved::Kind::decimal()
                            .into_kind_data()
                            .caption(),
                        false,
                        None,
                        0,
                    )),

                    fastn_resolved::OrTypeVariant::Regular(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_LENGTH_EM,
                        fastn_resolved::Kind::decimal()
                            .into_kind_data()
                            .caption(),
                        false,
                        None,
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Regular(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_LENGTH_REM,
                        fastn_resolved::Kind::decimal()
                            .into_kind_data()
                            .caption(),
                        false,
                        None,
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Regular(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_LENGTH_RESPONSIVE,
                        fastn_resolved::Kind::record(fastn_builtins::constants::FTD_RESPONSIVE_LENGTH)
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
            fastn_builtins::constants::FTD_RESPONSIVE_LENGTH.to_string(),
            fastn_resolved::Definition::Record(fastn_resolved::Record {
                name: fastn_builtins::constants::FTD_RESPONSIVE_LENGTH.to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    fastn_resolved::Field {
                        name: "desktop".to_string(),
                        kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                            .into_kind_data()
                            .caption(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "mobile".to_string(),
                        kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                            .into_kind_data(),
                        mutable: false,
                        access_modifier: Default::default(),
                        default: Some(fastn_resolved::PropertyValue::Reference {
                            name: fastn_builtins::constants::FTD_RESPONSIVE_LENGTH_DESKTOP.to_string(),
                            kind: fastn_resolved::Kind::string().into_kind_data(),
                            source: fastn_resolved::PropertyValueSource::Local(
                                fastn_builtins::constants::FTD_RESPONSIVE_LENGTH.to_string(),
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
            fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
            fastn_resolved::Definition::OrType(fastn_resolved::OrType {
                name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                variants: vec![
                    fastn_resolved::OrTypeVariant::Regular(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_FONT_SIZE_PX,
                        fastn_resolved::Kind::integer()
                            .into_kind_data()
                            .caption(),
                        false,
                        None,
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Regular(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_FONT_SIZE_EM,
                        fastn_resolved::Kind::decimal()
                            .into_kind_data()
                            .caption(),
                        false,
                        None,
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Regular(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_FONT_SIZE_REM,
                        fastn_resolved::Kind::decimal()
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
            fastn_builtins::constants::FTD_REGION.to_string(),
            fastn_resolved::Definition::OrType(fastn_resolved::OrType {
                name: fastn_builtins::constants::FTD_REGION.to_string(),
                variants: vec![
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_REGION_H1,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(   fastn_resolved::Value::new_string("h1")
                                    .into_property_value(false, 0),),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_REGION_H2,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("h2")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_REGION_H3,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("h3")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_REGION_H4,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("h4")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_REGION_H5,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("h5")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_REGION_H6,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("h6")
                            .into_property_value(false, 0)),
                        0,
                    )),
                ],
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_TEXT_INPUT_TYPE.to_string(),
            fastn_resolved::Definition::OrType(fastn_resolved::OrType {
                name: fastn_builtins::constants::FTD_TEXT_INPUT_TYPE.to_string(),
                variants: vec![
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_TEXT_INPUT_TYPE_TEXT,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(   fastn_resolved::Value::new_string("text")
                                    .into_property_value(false, 0),),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_TEXT_INPUT_TYPE_EMAIL,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("email")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_TEXT_INPUT_TYPE_PASSWORD,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("password")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_TEXT_INPUT_TYPE_URL,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("url")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_TEXT_INPUT_TYPE_DATETIME,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("datetime-local")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_TEXT_INPUT_TYPE_DATE,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("date")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_TEXT_INPUT_TYPE_TIME,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("time")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_TEXT_INPUT_TYPE_MONTH,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("month")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_TEXT_INPUT_TYPE_WEEK,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("week")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_TEXT_INPUT_TYPE_COLOR,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("color")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_TEXT_INPUT_TYPE_FILE,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("file")
                            .into_property_value(false, 0)),
                        0,
                    )),
                ],
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_LOADING.to_string(),
            fastn_resolved::Definition::OrType(fastn_resolved::OrType {
                name: fastn_builtins::constants::FTD_LOADING.to_string(),
                variants: vec![
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_LOADING_EAGER,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(   fastn_resolved::Value::new_string("eager")
                                    .into_property_value(false, 0),),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_LOADING_LAZY,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("lazy")
                            .into_property_value(false, 0)),
                        0,
                    )),
                ],
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_BORDER_STYLE.to_string(),
            fastn_resolved::Definition::OrType(fastn_resolved::OrType {
                name: fastn_builtins::constants::FTD_BORDER_STYLE.to_string(),
                variants: vec![
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BORDER_STYLE_DASHED,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(   fastn_resolved::Value::new_string("dashed")
                                    .into_property_value(false, 0),),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BORDER_STYLE_DOTTED,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(   fastn_resolved::Value::new_string("dotted")
                                    .into_property_value(false, 0),),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BORDER_STYLE_DOUBLE,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(   fastn_resolved::Value::new_string("double")
                                    .into_property_value(false, 0),),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BORDER_STYLE_GROOVE,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(   fastn_resolved::Value::new_string("groove")
                                    .into_property_value(false, 0),),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BORDER_STYLE_INSET,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(   fastn_resolved::Value::new_string("inset")
                                    .into_property_value(false, 0),),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BORDER_STYLE_OUTSET,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(   fastn_resolved::Value::new_string("outset")
                                    .into_property_value(false, 0),),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BORDER_STYLE_RIDGE,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(   fastn_resolved::Value::new_string("ridge")
                                    .into_property_value(false, 0),),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_BORDER_STYLE_SOLID,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(   fastn_resolved::Value::new_string("solid")
                                    .into_property_value(false, 0),),
                        0,
                    )),
                ],
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_TEXT_STYLE.to_string(),
            fastn_resolved::Definition::OrType(fastn_resolved::OrType {
                name: fastn_builtins::constants::FTD_TEXT_STYLE.to_string(),
                variants: vec![
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_TEXT_STYLE_UNDERLINE,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("underline").into_property_value(false, 0),),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_TEXT_STYLE_STRIKE,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("strike").into_property_value(false, 0),),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_TEXT_STYLE_ITALIC,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("italic").into_property_value(false, 0),),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_TEXT_STYLE_WEIGHT_HEAVY,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("heavy").into_property_value(false, 0),),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_TEXT_STYLE_WEIGHT_EXTRA_BOLD,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("extra-bold").into_property_value(false, 0),),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_TEXT_STYLE_WEIGHT_BOLD,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("bold").into_property_value(false, 0),),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_TEXT_STYLE_WEIGHT_SEMI_BOLD,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("semi-bold").into_property_value(false, 0),),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_TEXT_STYLE_WEIGHT_MEDIUM,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("medium").into_property_value(false, 0),),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_TEXT_STYLE_WEIGHT_REGULAR,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("regular").into_property_value(false, 0),),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_TEXT_STYLE_WEIGHT_LIGHT,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("light").into_property_value(false, 0),),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_TEXT_STYLE_WEIGHT_EXTRA_LIGHT,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("extra-light").into_property_value(false, 0),),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_TEXT_STYLE_WEIGHT_HAIRLINE,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("hairline").into_property_value(false, 0),),
                        0,
                    )),
                ],
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_TEXT_TRANSFORM.to_string(),
            fastn_resolved::Definition::OrType(fastn_resolved::OrType {
                name: fastn_builtins::constants::FTD_TEXT_TRANSFORM.to_string(),
                variants: vec![
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_TEXT_TRANSFORM_NONE,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(   fastn_resolved::Value::new_string("none")
                                    .into_property_value(false, 0),),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_TEXT_TRANSFORM_CAPITALIZE,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("capitalize")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_TEXT_TRANSFORM_UPPERCASE,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("uppercase")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_TEXT_TRANSFORM_LOWERCASE,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("lowercase")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_TEXT_TRANSFORM_INITIAL,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("initial")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_TEXT_TRANSFORM_INHERIT,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("inherit")
                            .into_property_value(false, 0)),
                        0,
                    )),
                ],
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_TYPE.to_string(),
            fastn_resolved::Definition::Record(fastn_resolved::Record {
                name: fastn_builtins::constants::FTD_TYPE.to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    fastn_resolved::Field {
                        name: "size".to_string(),
                        kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_FONT_SIZE)
                            .into_optional()
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "line-height".to_string(),
                        kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_FONT_SIZE)
                            .into_optional()
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "letter-spacing".to_string(),
                        kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_FONT_SIZE)
                            .into_optional()
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "weight".to_string(),
                        kind: fastn_resolved::Kind::integer()
                            .into_optional()
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "font-family".to_string(),
                        kind: fastn_resolved::Kind::string()
                            .into_list()
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ])
                    .collect(),
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_RESPONSIVE_TYPE.to_string(),
            fastn_resolved::Definition::Record(fastn_resolved::Record {
                name: fastn_builtins::constants::FTD_RESPONSIVE_TYPE.to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    fastn_resolved::Field {
                        name: "desktop".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_TYPE)
                            .into_kind_data().caption(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "mobile".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_TYPE)
                            .into_kind_data(),
                        mutable: false,
                        access_modifier: Default::default(),
                        default: Some(fastn_resolved::PropertyValue::Reference {
                            name: fastn_builtins::constants::FTD_RESPONSIVE_TYPE_DESKTOP.to_string(),
                            kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_TYPE)
                                .into_kind_data(),
                            source: fastn_resolved::PropertyValueSource::Local(
                                fastn_builtins::constants::FTD_RESPONSIVE_TYPE.to_string(),
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
            fastn_resolved::Definition::Variable(fastn_resolved::Variable {
                name: "ftd#dark-mode".to_string(),
                kind: fastn_resolved::Kind::boolean().into_kind_data(),
                mutable: true,
                value: fastn_resolved::PropertyValue::Value {
                    value: fastn_resolved::Value::Boolean { value: false },
                    is_mutable: true,
                    line_number: 0,
                },
                conditional_value: vec![],
                line_number: 0,
                is_static: false,
            }),
        ),
        (
            "ftd#empty".to_string(),
            fastn_resolved::Definition::Variable(fastn_resolved::Variable {
                name: "ftd#empty".to_string(),
                kind: fastn_resolved::Kind::string().into_kind_data(),
                mutable: false,
                value: fastn_resolved::PropertyValue::Value {
                    value: fastn_resolved::Value::String { text: "".to_string() },
                    is_mutable: false,
                    line_number: 0,
                },
                conditional_value: vec![],
                line_number: 0,
                is_static: false,
            }),
        ),
        (
            "ftd#space".to_string(),
            fastn_resolved::Definition::Variable(fastn_resolved::Variable {
                name: "ftd#space".to_string(),
                kind: fastn_resolved::Kind::string().into_kind_data(),
                mutable: false,
                value: fastn_resolved::PropertyValue::Value {
                    value: fastn_resolved::Value::String { text: " ".to_string() },
                    is_mutable: false,
                    line_number: 0,
                },
                conditional_value: vec![],
                line_number: 0,
                is_static: false,
            }),
        ),
        (
            "ftd#nbsp".to_string(),
            fastn_resolved::Definition::Variable(fastn_resolved::Variable {
                name: "ftd#nbsp".to_string(),
                kind: fastn_resolved::Kind::string().into_kind_data(),
                mutable: false,
                value: fastn_resolved::PropertyValue::Value {
                    value: fastn_resolved::Value::String { text: "&nbsp;".to_string() },
                    is_mutable: false,
                    line_number: 0,
                },
                conditional_value: vec![],
                line_number: 0,
                is_static: false,
            }),
        ),
        (
            "ftd#non-breaking-space".to_string(),
            fastn_resolved::Definition::Variable(fastn_resolved::Variable {
                name: "ftd#non-breaking-space".to_string(),
                kind: fastn_resolved::Kind::string().into_kind_data(),
                mutable: false,
                value: fastn_resolved::PropertyValue::Value {
                    value: fastn_resolved::Value::String { text: "&nbsp;".to_string() },
                    is_mutable: false,
                    line_number: 0,
                },
                conditional_value: vec![],
                line_number: 0,
                is_static: false,
            }),
        ),
        (
            "ftd#system-dark-mode".to_string(),
            fastn_resolved::Definition::Variable(fastn_resolved::Variable {
                name: "ftd#system-dark-mode".to_string(),
                kind: fastn_resolved::Kind::boolean().into_kind_data(),
                mutable: true,
                value: fastn_resolved::PropertyValue::Value {
                    value: fastn_resolved::Value::Boolean { value: false },
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
            fastn_resolved::Definition::Variable(fastn_resolved::Variable {
                name: "ftd#follow-system-dark-mode".to_string(),
                kind: fastn_resolved::Kind::boolean().into_kind_data(),
                mutable: true,
                value: fastn_resolved::PropertyValue::Value {
                    value: fastn_resolved::Value::Boolean { value: true },
                    is_mutable: true,
                    line_number: 0,
                },
                conditional_value: vec![],
                line_number: 0,
                is_static: false,
            }),
        ),
        (
            "ftd#permanent-redirect".to_string(),
            fastn_resolved::Definition::Component(fastn_resolved::ComponentDefinition {
                name: "ftd#permanent-redirect".to_string(),
                arguments: vec![
                    fastn_resolved::Argument::default(
                        "url",
                        fastn_resolved::Kind::string()
                            .into_kind_data().caption_or_body(),
                    ),
                ],
                definition: fastn_resolved::ComponentInvocation::from_name("ftd.kernel"),
                css: None,
                line_number: 0,
            }),
        ),
        (
            "ftd#temporary-redirect".to_string(),
            fastn_resolved::Definition::Component(fastn_resolved::ComponentDefinition {
                name: "ftd#temporary-redirect".to_string(),
                arguments: vec![
                    fastn_resolved::Argument::default(
                        "url",
                        fastn_resolved::Kind::string()
                            .into_kind_data().caption_or_body(),
                    ),
                ],
                definition: fastn_resolved::ComponentInvocation::from_name("ftd.kernel"),
                css: None,
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_BACKGROUND_COLOR.to_string(),
            fastn_resolved::Definition::Record(fastn_resolved::Record {
                name: fastn_builtins::constants::FTD_BACKGROUND_COLOR.to_string(),
                fields: vec![
                    fastn_resolved::Field {
                        name: "base".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "step-1".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "step-2".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "overlay".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "code".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ],
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_CTA_COLOR.to_string(),
            fastn_resolved::Definition::Record(fastn_resolved::Record {
                name: fastn_builtins::constants::FTD_CTA_COLOR.to_string(),
                fields: vec![
                    fastn_resolved::Field {
                        name: "base".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "hover".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "pressed".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "disabled".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "focused".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "border".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "border-disabled".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "text".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "text-disabled".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ],
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_PST.to_string(),
            fastn_resolved::Definition::Record(fastn_resolved::Record {
                name: fastn_builtins::constants::FTD_PST.to_string(),
                fields: vec![
                    fastn_resolved::Field {
                        name: "primary".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "secondary".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "tertiary".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ],
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_BTB.to_string(),
            fastn_resolved::Definition::Record(fastn_resolved::Record {
                name: fastn_builtins::constants::FTD_BTB.to_string(),
                fields: vec![
                    fastn_resolved::Field {
                        name: "base".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "text".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "border".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ],
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_CUSTOM_COLORS.to_string(),
            fastn_resolved::Definition::Record(fastn_resolved::Record {
                name: fastn_builtins::constants::FTD_CUSTOM_COLORS.to_string(),
                fields: vec![
                    fastn_resolved::Field {
                        name: "one".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "two".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "three".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "four".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "five".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "six".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "seven".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "eight".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "nine".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "ten".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ],
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_COLOR_SCHEME.to_string(),
            fastn_resolved::Definition::Record(fastn_resolved::Record {
                name: fastn_builtins::constants::FTD_COLOR_SCHEME.to_string(),
                fields: vec![
                    fastn_resolved::Field {
                        name: "background".to_string(),
                        kind: fastn_resolved::Kind::record("ftd#background-colors")
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "border".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "border-strong".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "text".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "text-strong".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        line_number: 0,
                        access_modifier: Default::default(),
                    },
                    fastn_resolved::Field {
                        name: "shadow".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        line_number: 0,
                        access_modifier: Default::default(),
                    },
                    fastn_resolved::Field {
                        name: "scrim".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        line_number: 0,
                        access_modifier: Default::default(),
                    },
                    fastn_resolved::Field {
                        name: "cta-primary".to_string(),
                        kind: fastn_resolved::Kind::record("ftd#cta-colors").into_kind_data(),
                        mutable: false,
                        default: None,
                        line_number: 0,
                        access_modifier: Default::default(),
                    },
                    fastn_resolved::Field {
                        name: "cta-secondary".to_string(),
                        kind: fastn_resolved::Kind::record("ftd#cta-colors").into_kind_data(),
                        mutable: false,
                        default: None,
                        line_number: 0,
                        access_modifier: Default::default(),
                    },
                    fastn_resolved::Field {
                        name: "cta-tertiary".to_string(),
                        kind: fastn_resolved::Kind::record("ftd#cta-colors").into_kind_data(),
                        mutable: false,
                        default: None,
                        line_number: 0,
                        access_modifier: Default::default(),
                    },
                    fastn_resolved::Field {
                        name: "cta-danger".to_string(),
                        kind: fastn_resolved::Kind::record("ftd#cta-colors").into_kind_data(),
                        mutable: false,
                        default: None,
                        line_number: 0,
                        access_modifier: Default::default(),
                    },
                    fastn_resolved::Field {
                        name: "accent".to_string(),
                        kind: fastn_resolved::Kind::record("ftd#pst").into_kind_data(),
                        mutable: false,
                        default: None,
                        line_number: 0,
                        access_modifier: Default::default(),
                    },
                    fastn_resolved::Field {
                        name: "error".to_string(),
                        kind: fastn_resolved::Kind::record("ftd#btb").into_kind_data(),
                        mutable: false,
                        default: None,
                        line_number: 0,
                        access_modifier: Default::default(),
                    },
                    fastn_resolved::Field {
                        name: "success".to_string(),
                        kind: fastn_resolved::Kind::record("ftd#btb").into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "info".to_string(),
                        kind: fastn_resolved::Kind::record("ftd#btb").into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "warning".to_string(),
                        kind: fastn_resolved::Kind::record("ftd#btb").into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "custom".to_string(),
                        kind: fastn_resolved::Kind::record("ftd#custom-colors").into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ],
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_TYPE_DATA.to_string(),
            fastn_resolved::Definition::Record(fastn_resolved::Record {
                name: fastn_builtins::constants::FTD_TYPE_DATA.to_string(),
                fields: vec![fastn_resolved::Field {
                    name: "heading-large".to_string(),
                    kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_RESPONSIVE_TYPE)
                        .into_kind_data(),
                    mutable: false,
                    default: None,
                    access_modifier: Default::default(),
                    line_number: 0
                }, fastn_resolved::Field {
                    name: "heading-medium".to_string(),
                    kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_RESPONSIVE_TYPE)
                        .into_kind_data(),
                    mutable: false,
                    default: None,
                    access_modifier: Default::default(),
                    line_number: 0
                }, fastn_resolved::Field {
                    name: "heading-small".to_string(),
                    kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_RESPONSIVE_TYPE)
                        .into_kind_data(),
                    mutable: false,
                    default: None,
                    access_modifier: Default::default(),
                    line_number: 0
                }, fastn_resolved::Field {
                    name: "heading-hero".to_string(),
                    kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_RESPONSIVE_TYPE)
                        .into_kind_data(),
                    mutable: false,
                    default: None,
                    access_modifier: Default::default(),
                    line_number: 0
                }, fastn_resolved::Field {
                    name: "heading-tiny".to_string(),
                    kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_RESPONSIVE_TYPE)
                        .into_kind_data(),
                    mutable: false,
                    default: None,
                    access_modifier: Default::default(),
                    line_number: 0
                }, fastn_resolved::Field {
                    name: "copy-small".to_string(),
                    kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_RESPONSIVE_TYPE)
                        .into_kind_data(),
                    mutable: false,
                    default: None,
                    access_modifier: Default::default(),
                    line_number: 0
                }, fastn_resolved::Field {
                    name: "copy-regular".to_string(),
                    kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_RESPONSIVE_TYPE)
                        .into_kind_data(),
                    mutable: false,
                    default: None,
                    access_modifier: Default::default(),
                    line_number: 0
                }, fastn_resolved::Field {
                    name: "copy-large".to_string(),
                    kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_RESPONSIVE_TYPE)
                        .into_kind_data(),
                    mutable: false,
                    default: None,
                    access_modifier: Default::default(),
                    line_number: 0
                }, fastn_resolved::Field {
                    name: "fine-print".to_string(),
                    kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_RESPONSIVE_TYPE)
                        .into_kind_data(),
                    mutable: false,
                    default: None,
                    access_modifier: Default::default(),
                    line_number: 0
                }, fastn_resolved::Field {
                    name: "blockquote".to_string(),
                    kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_RESPONSIVE_TYPE)
                        .into_kind_data(),
                    mutable: false,
                    default: None,
                    access_modifier: Default::default(),
                    line_number: 0
                }, fastn_resolved::Field {
                    name: "source-code".to_string(),
                    kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_RESPONSIVE_TYPE)
                        .into_kind_data(),
                    mutable: false,
                    default: None,
                    access_modifier: Default::default(),
                    line_number: 0
                }, fastn_resolved::Field {
                    name: "button-small".to_string(),
                    kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_RESPONSIVE_TYPE)
                        .into_kind_data(),
                    mutable: false,
                    default: None,
                    access_modifier: Default::default(),
                    line_number: 0
                }, fastn_resolved::Field {
                    name: "button-medium".to_string(),
                    kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_RESPONSIVE_TYPE)
                        .into_kind_data(),
                    mutable: false,
                    default: None,
                    access_modifier: Default::default(),
                    line_number: 0,
                }, fastn_resolved::Field {
                    name: "button-large".to_string(),
                    kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_RESPONSIVE_TYPE)
                        .into_kind_data(),
                    mutable: false,
                    default: None,
                    access_modifier: Default::default(),
                    line_number: 0,
                }, fastn_resolved::Field {
                    name: "link".to_string(),
                    kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_RESPONSIVE_TYPE)
                        .into_kind_data(),
                    mutable: false,
                    default: None,
                    access_modifier: Default::default(),
                    line_number: 0,
                }, fastn_resolved::Field {
                    name: "label-large".to_string(),
                    kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_RESPONSIVE_TYPE)
                        .into_kind_data(),
                    mutable: false,
                    default: None,
                    access_modifier: Default::default(),
                    line_number: 0
                }, fastn_resolved::Field {
                    name: "label-small".to_string(),
                    kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_RESPONSIVE_TYPE)
                        .into_kind_data(),
                    mutable: false,
                    default: None,
                    access_modifier: Default::default(),
                    line_number: 0
                },],
                line_number: 0
            })
        ),
        (
            "ftd#font-display".to_string(),
            fastn_resolved::Definition::Variable(fastn_resolved::Variable {
                name: "ftd#font-display".to_string(),
                kind: fastn_resolved::Kind::string().into_kind_data(),
                mutable: true,
                value: fastn_resolved::PropertyValue::Value {
                    value: fastn_resolved::Value::new_string("sans-serif"),
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
            fastn_resolved::Definition::Variable(fastn_resolved::Variable {
                name: "ftd#font-copy".to_string(),
                kind: fastn_resolved::Kind::string().into_kind_data(),
                mutable: true,
                value: fastn_resolved::PropertyValue::Value {
                    value: fastn_resolved::Value::new_string("sans-serif"),
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
            fastn_resolved::Definition::Variable(fastn_resolved::Variable {
                name: "ftd#font-code".to_string(),
                kind: fastn_resolved::Kind::string().into_kind_data(),
                mutable: true,
                value: fastn_resolved::PropertyValue::Value {
                    value: fastn_resolved::Value::new_string("sans-serif"),
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
            fastn_resolved::Definition::Variable(fastn_resolved::Variable {
                name: "ftd#default-types".to_string(),
                kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_TYPE_DATA).into_kind_data(),
                mutable: true,
                value: fastn_resolved::PropertyValue::Value {
                    value: fastn_resolved::Value::Record {
                        name: fastn_builtins::constants::FTD_TYPE_DATA.to_string(),
                        fields: std::iter::IntoIterator::into_iter([
                            // HEADING TYPES -------------------------------------------
                            (
                                "heading-hero".to_string(),
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Record {
                                        name: fastn_builtins::constants::FTD_RESPONSIVE_TYPE.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "desktop".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                fastn_resolved::PropertyValue::Reference {
                                                                    name: "ftd#font-display".to_string(),
                                                                    kind:
                                                                    fastn_resolved::Kind::string().into_kind_data(),
                                                                    source:
                                                                    fastn_resolved::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value:
                                                                    fastn_resolved::Value::Integer {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                fastn_resolved::PropertyValue::Reference {
                                                                    name: "ftd#font-display".to_string(),
                                                                    kind:
                                                                    fastn_resolved::Kind::string().into_kind_data(),
                                                                    source:
                                                                    fastn_resolved::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value:
                                                                    fastn_resolved::Value::Integer {
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
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Record {
                                        name: fastn_builtins::constants::FTD_RESPONSIVE_TYPE.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "desktop".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                fastn_resolved::PropertyValue::Reference {
                                                                    name: "ftd#font-display".to_string(),
                                                                    kind:
                                                                    fastn_resolved::Kind::string().into_kind_data(),
                                                                    source:
                                                                    fastn_resolved::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value:
                                                                    fastn_resolved::Value::Integer {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                fastn_resolved::PropertyValue::Reference {
                                                                    name: "ftd#font-display".to_string(),
                                                                    kind:
                                                                    fastn_resolved::Kind::string().into_kind_data(),
                                                                    source:
                                                                    fastn_resolved::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value:
                                                                    fastn_resolved::Value::Integer {
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
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Record {
                                        name: fastn_builtins::constants::FTD_RESPONSIVE_TYPE.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "desktop".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                fastn_resolved::PropertyValue::Reference {
                                                                    name: "ftd#font-display".to_string(),
                                                                    kind:
                                                                    fastn_resolved::Kind::string().into_kind_data(),
                                                                    source:
                                                                    fastn_resolved::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value:
                                                                    fastn_resolved::Value::Integer {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                fastn_resolved::PropertyValue::Reference {
                                                                    name: "ftd#font-display".to_string(),
                                                                    kind:
                                                                    fastn_resolved::Kind::string().into_kind_data(),
                                                                    source:
                                                                    fastn_resolved::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value:
                                                                    fastn_resolved::Value::Integer {
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
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Record {
                                        name: fastn_builtins::constants::FTD_RESPONSIVE_TYPE.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "desktop".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                fastn_resolved::PropertyValue::Reference {
                                                                    name: "ftd#font-display".to_string(),
                                                                    kind:
                                                                    fastn_resolved::Kind::string().into_kind_data(),
                                                                    source:
                                                                    fastn_resolved::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value:
                                                                    fastn_resolved::Value::Integer {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                fastn_resolved::PropertyValue::Reference {
                                                                    name: "ftd#font-display".to_string(),
                                                                    kind:
                                                                    fastn_resolved::Kind::string().into_kind_data(),
                                                                    source:
                                                                    fastn_resolved::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value:
                                                                    fastn_resolved::Value::Integer {
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
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Record {
                                        name: fastn_builtins::constants::FTD_RESPONSIVE_TYPE.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "desktop".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                fastn_resolved::PropertyValue::Reference {
                                                                    name: "ftd#font-display".to_string(),
                                                                    kind:
                                                                    fastn_resolved::Kind::string().into_kind_data(),
                                                                    source:
                                                                    fastn_resolved::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value:
                                                                    fastn_resolved::Value::Integer {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                fastn_resolved::PropertyValue::Reference {
                                                                    name: "ftd#font-display".to_string(),
                                                                    kind:
                                                                    fastn_resolved::Kind::string().into_kind_data(),
                                                                    source:
                                                                    fastn_resolved::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value:
                                                                    fastn_resolved::Value::Integer {
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
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Record {
                                        name: fastn_builtins::constants::FTD_RESPONSIVE_TYPE.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "desktop".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                fastn_resolved::PropertyValue::Reference {
                                                                    name: "ftd#font-copy".to_string(),
                                                                    kind:
                                                                    fastn_resolved::Kind::string().into_kind_data(),
                                                                    source:
                                                                    fastn_resolved::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value:
                                                                    fastn_resolved::Value::Integer {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                fastn_resolved::PropertyValue::Reference {
                                                                    name: "ftd#font-copy".to_string(),
                                                                    kind:
                                                                    fastn_resolved::Kind::string().into_kind_data(),
                                                                    source:
                                                                    fastn_resolved::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value:
                                                                    fastn_resolved::Value::Integer {
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
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Record {
                                        name: fastn_builtins::constants::FTD_RESPONSIVE_TYPE.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "desktop".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                fastn_resolved::PropertyValue::Reference {
                                                                    name: "ftd#font-copy".to_string(),
                                                                    kind:
                                                                    fastn_resolved::Kind::string().into_kind_data(),
                                                                    source:
                                                                    fastn_resolved::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value:
                                                                    fastn_resolved::Value::Integer {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                fastn_resolved::PropertyValue::Reference {
                                                                    name: "ftd#font-copy".to_string(),
                                                                    kind:
                                                                    fastn_resolved::Kind::string().into_kind_data(),
                                                                    source:
                                                                    fastn_resolved::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value:
                                                                    fastn_resolved::Value::Integer {
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
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Record {
                                        name: fastn_builtins::constants::FTD_RESPONSIVE_TYPE.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "desktop".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                fastn_resolved::PropertyValue::Reference {
                                                                    name: "ftd#font-copy".to_string(),
                                                                    kind:
                                                                    fastn_resolved::Kind::string().into_kind_data(),
                                                                    source:
                                                                    fastn_resolved::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value:
                                                                    fastn_resolved::Value::Integer {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                fastn_resolved::PropertyValue::Reference {
                                                                    name: "ftd#font-copy".to_string(),
                                                                    kind:
                                                                    fastn_resolved::Kind::string().into_kind_data(),
                                                                    source:
                                                                    fastn_resolved::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value:
                                                                    fastn_resolved::Value::Integer {
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
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Record {
                                        name: fastn_builtins::constants::FTD_RESPONSIVE_TYPE.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "desktop".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                fastn_resolved::PropertyValue::Reference {
                                                                    name: "ftd#font-code".to_string(),
                                                                    kind:
                                                                    fastn_resolved::Kind::string().into_kind_data(),
                                                                    source:
                                                                    fastn_resolved::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value:
                                                                    fastn_resolved::Value::Integer {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                fastn_resolved::PropertyValue::Reference {
                                                                    name: "ftd#font-code".to_string(),
                                                                    kind:
                                                                    fastn_resolved::Kind::string().into_kind_data(),
                                                                    source:
                                                                    fastn_resolved::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value:
                                                                    fastn_resolved::Value::Integer {
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
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Record {
                                        name: fastn_builtins::constants::FTD_RESPONSIVE_TYPE.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "desktop".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                fastn_resolved::PropertyValue::Reference {
                                                                    name: "ftd#font-code".to_string(),
                                                                    kind:
                                                                    fastn_resolved::Kind::string().into_kind_data(),
                                                                    source:
                                                                    fastn_resolved::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value:
                                                                    fastn_resolved::Value::Integer {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                fastn_resolved::PropertyValue::Reference {
                                                                    name: "ftd#font-code".to_string(),
                                                                    kind:
                                                                    fastn_resolved::Kind::string().into_kind_data(),
                                                                    source:
                                                                    fastn_resolved::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value:
                                                                    fastn_resolved::Value::Integer {
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
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Record {
                                        name: fastn_builtins::constants::FTD_RESPONSIVE_TYPE.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "desktop".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                fastn_resolved::PropertyValue::Reference {
                                                                    name: "ftd#font-code".to_string(),
                                                                    kind:
                                                                    fastn_resolved::Kind::string().into_kind_data(),
                                                                    source:
                                                                    fastn_resolved::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value:
                                                                    fastn_resolved::Value::Integer {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                fastn_resolved::PropertyValue::Reference {
                                                                    name: "ftd#font-code".to_string(),
                                                                    kind:
                                                                    fastn_resolved::Kind::string().into_kind_data(),
                                                                    source:
                                                                    fastn_resolved::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value:
                                                                    fastn_resolved::Value::Integer {
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
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Record {
                                        name: fastn_builtins::constants::FTD_RESPONSIVE_TYPE.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "desktop".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                fastn_resolved::PropertyValue::Reference {
                                                                    name: "ftd#font-display".to_string(),
                                                                    kind:
                                                                    fastn_resolved::Kind::string().into_kind_data(),
                                                                    source:
                                                                    fastn_resolved::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value:
                                                                    fastn_resolved::Value::Integer {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                fastn_resolved::PropertyValue::Reference {
                                                                    name: "ftd#font-display".to_string(),
                                                                    kind:
                                                                    fastn_resolved::Kind::string().into_kind_data(),
                                                                    source:
                                                                    fastn_resolved::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value:
                                                                    fastn_resolved::Value::Integer {
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
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Record {
                                        name: fastn_builtins::constants::FTD_RESPONSIVE_TYPE.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "desktop".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                fastn_resolved::PropertyValue::Reference {
                                                                    name: "ftd#font-display".to_string(),
                                                                    kind:
                                                                    fastn_resolved::Kind::string().into_kind_data(),
                                                                    source:
                                                                    fastn_resolved::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value:
                                                                    fastn_resolved::Value::Integer {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                fastn_resolved::PropertyValue::Reference {
                                                                    name: "ftd#font-display".to_string(),
                                                                    kind:
                                                                    fastn_resolved::Kind::string().into_kind_data(),
                                                                    source:
                                                                    fastn_resolved::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value:
                                                                    fastn_resolved::Value::Integer {
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
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Record {
                                        name: fastn_builtins::constants::FTD_RESPONSIVE_TYPE.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "desktop".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                fastn_resolved::PropertyValue::Reference {
                                                                    name: "ftd#font-display".to_string(),
                                                                    kind:
                                                                    fastn_resolved::Kind::string().into_kind_data(),
                                                                    source:
                                                                    fastn_resolved::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value:
                                                                    fastn_resolved::Value::Integer {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                fastn_resolved::PropertyValue::Reference {
                                                                    name: "ftd#font-display".to_string(),
                                                                    kind:
                                                                    fastn_resolved::Kind::string().into_kind_data(),
                                                                    source:
                                                                    fastn_resolved::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value:
                                                                    fastn_resolved::Value::Integer {
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
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Record {
                                        name: fastn_builtins::constants::FTD_RESPONSIVE_TYPE.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "desktop".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                fastn_resolved::PropertyValue::Reference {
                                                                    name: "ftd#font-display".to_string(),
                                                                    kind:
                                                                    fastn_resolved::Kind::string().into_kind_data(),
                                                                    source:
                                                                    fastn_resolved::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value:
                                                                    fastn_resolved::Value::Integer {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                fastn_resolved::PropertyValue::Reference {
                                                                    name: "ftd#font-display".to_string(),
                                                                    kind:
                                                                    fastn_resolved::Kind::string().into_kind_data(),
                                                                    source:
                                                                    fastn_resolved::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value:
                                                                    fastn_resolved::Value::Integer {
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
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Record {
                                        name: fastn_builtins::constants::FTD_RESPONSIVE_TYPE.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "desktop".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                fastn_resolved::PropertyValue::Reference {
                                                                    name: "ftd#font-display".to_string(),
                                                                    kind:
                                                                    fastn_resolved::Kind::string().into_kind_data(),
                                                                    source:
                                                                    fastn_resolved::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value:
                                                                    fastn_resolved::Value::Integer {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                fastn_resolved::PropertyValue::Reference {
                                                                    name: "ftd#font-display".to_string(),
                                                                    kind:
                                                                    fastn_resolved::Kind::string().into_kind_data(),
                                                                    source:
                                                                    fastn_resolved::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value:
                                                                    fastn_resolved::Value::Integer {
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
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Record {
                                        name: fastn_builtins::constants::FTD_RESPONSIVE_TYPE.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "desktop".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                fastn_resolved::PropertyValue::Reference {
                                                                    name: "ftd#font-display".to_string(),
                                                                    kind:
                                                                    fastn_resolved::Kind::string().into_kind_data(),
                                                                    source:
                                                                    fastn_resolved::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value:
                                                                    fastn_resolved::Value::Integer {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_TYPE.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "font-family".to_string(),
                                                                fastn_resolved::PropertyValue::Reference {
                                                                    name: "ftd#font-display".to_string(),
                                                                    kind:
                                                                    fastn_resolved::Kind::string().into_kind_data(),
                                                                    source:
                                                                    fastn_resolved::PropertyValueSource::Global,
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ),
                                                            (
                                                                "size".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::OrType {
                                                                        name: fastn_builtins::constants::FTD_FONT_SIZE.to_string(),
                                                                        variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        full_variant: fastn_builtins::constants::FTD_FONT_SIZE_PX.to_string(),
                                                                        value: Box::new
                                                                            (fastn_resolved::PropertyValue::Value {
                                                                                value: fastn_resolved::Value::Integer {
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
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value:
                                                                    fastn_resolved::Value::Integer {
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
            fastn_resolved::Definition::Variable(fastn_resolved::Variable {
                name: "ftd#default-colors".to_string(),
                kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR_SCHEME)
                    .into_kind_data(),
                mutable: true,
                value: fastn_resolved::PropertyValue::Value {
                    value: fastn_resolved::Value::Record {
                        name: fastn_builtins::constants::FTD_COLOR_SCHEME.to_string(),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                "background".to_string(),
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Record {
                                        name: fastn_builtins::constants::FTD_BACKGROUND_COLOR.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "base".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value:
                                                                fastn_resolved::Value::String {
                                                                    text: "#e7e7e4".to_string(),
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            }
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value:
                                                                fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value:
                                                                fastn_resolved::Value::String {
                                                                    text: "#f3f3f3".to_string(),
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            }
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value:
                                                                fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value:
                                                                fastn_resolved::Value::String {
                                                                    text: "#c9cece".to_string(),
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            }
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value:
                                                                fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value:
                                                                fastn_resolved::Value::String {
                                                                    text: "rgba(0, 0, 0, 0.8)"
                                                                        .to_string(),
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            }
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value:
                                                                fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value:
                                                                fastn_resolved::Value::String {
                                                                    text: "#F5F5F5".to_string(),
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            }
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value:
                                                                fastn_resolved::Value::String {
                                                                    text: "#21222C".to_string(),
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
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Record {
                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([(
                                            "light".to_string(),
                                            fastn_resolved::PropertyValue::Value {
                                                value:
                                                fastn_resolved::Value::String {
                                                    text: "#434547".to_string(),
                                                },
                                                is_mutable: false,
                                                line_number: 0,
                                            }
                                        ), (
                                            "dark".to_string(),
                                            fastn_resolved::PropertyValue::Value {
                                                value:
                                                fastn_resolved::Value::String {
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
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Record {
                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([(
                                            "light".to_string(),
                                            fastn_resolved::PropertyValue::Value {
                                                value:
                                                fastn_resolved::Value::String {
                                                    text: "#919192".to_string(),
                                                },
                                                is_mutable: false,
                                                line_number: 0,
                                            }
                                        ), (
                                            "dark".to_string(),
                                            fastn_resolved::PropertyValue::Value {
                                                value:
                                                fastn_resolved::Value::String {
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
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Record {
                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([(
                                            "light".to_string(),
                                            fastn_resolved::PropertyValue::Value {
                                                value:
                                                fastn_resolved::Value::String {
                                                    text: "#584b42".to_string(),
                                                },
                                                is_mutable: false,
                                                line_number: 0,
                                            }
                                        ), (
                                            "dark".to_string(),
                                            fastn_resolved::PropertyValue::Value {
                                                value:
                                                fastn_resolved::Value::String {
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
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Record {
                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([(
                                            "light".to_string(),
                                            fastn_resolved::PropertyValue::Value {
                                                value:
                                                fastn_resolved::Value::String {
                                                    text: "#141414".to_string(),
                                                },
                                                is_mutable: false,
                                                line_number: 0,
                                            }
                                        ), (
                                            "dark".to_string(),
                                            fastn_resolved::PropertyValue::Value {
                                                value:
                                                fastn_resolved::Value::String {
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
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Record {
                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([(
                                            "light".to_string().to_string(),
                                            fastn_resolved::PropertyValue::Value {
                                                value:
                                                fastn_resolved::Value::String {
                                                    text: "#007f9b".to_string(),
                                                },
                                                is_mutable: false,
                                                line_number: 0,
                                            },
                                        ), (
                                            "dark".to_string(),
                                            fastn_resolved::PropertyValue::Value {
                                                value:
                                                fastn_resolved::Value::String {
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
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Record {
                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([(
                                            "light".to_string(),
                                            fastn_resolved::PropertyValue::Value {
                                                value:
                                                fastn_resolved::Value::String {
                                                    text: "#007f9b".to_string(),
                                                },
                                                is_mutable: false,
                                                line_number: 0,
                                            },
                                        ), (
                                            "dark".to_string(),
                                            fastn_resolved::PropertyValue::Value {
                                                value:
                                                fastn_resolved::Value::String {
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
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Record {
                                        name: fastn_builtins::constants::FTD_CTA_COLOR.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "base".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#2dd4bf".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#2c9f90".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#2cc9b5".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "rgba(44, 201, 181, 0.1)".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#2cbfac".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#2b8074".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#feffff".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
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
                                            ),
                                            (
                                                "border-disabled".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#65b693".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string().to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#65b693".to_string()
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
                                                "text-disabled".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#65b693".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#65b693".to_string()
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
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Record {
                                        name: fastn_builtins::constants::FTD_CTA_COLOR.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "base".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#4fb2df".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#40afe1".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#4fb2df".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "rgba(79, 178, 223, 0.1)".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#4fb1df".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#209fdb".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#584b42".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
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
                                            ),
                                            (
                                                "border-disabled".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#65b693".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#65b693".to_string()
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
                                                "text-disabled".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#65b693".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#65b693".to_string()
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
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Record {
                                        name: fastn_builtins::constants::FTD_CTA_COLOR.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "base".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#556375".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#c7cbd1".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#3b4047".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "rgba(85, 99, 117, 0.1)".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#e0e2e6".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#e2e4e7".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#ffffff".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
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
                                            ),
                                            (
                                                "border-disabled".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#65b693".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#65b693".to_string()
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
                                                "text-disabled".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#65b693".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#65b693".to_string()
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
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Record {
                                        name: fastn_builtins::constants::FTD_CTA_COLOR.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "base".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#1C1B1F".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#1C1B1F".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#1C1B1F".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#1C1B1F".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#1C1B1F".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#1C1B1F".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
                                                                        text: "#1C1B1F".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0,
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
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
                                            ),
                                            (
                                                "border-disabled".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#feffff".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
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
                                            ),
                                            (
                                                "text-disabled".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([(
                                                            "light".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
                                                                    text: "#feffff".to_string()
                                                                },
                                                                is_mutable: false,
                                                                line_number: 0,
                                                            },
                                                        ), (
                                                            "dark".to_string(),
                                                            fastn_resolved::PropertyValue::Value {
                                                                value: fastn_resolved::Value::String {
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
                                "accent".to_string(),
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Record {
                                        name: fastn_builtins::constants::FTD_PST.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "primary".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
                                                                        text: "#2dd4bf".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
                                                                        text: "#4fb2df".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
                                                                        text: "#c5cbd7".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
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
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Record {
                                        name: fastn_builtins::constants::FTD_BTB.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "base".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
                                                                        text: "#f5bdbb".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
                                                                        text: "#311b1f".to_string()
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
                                                                        text: "#c62a21".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
                                                                        text: "#df2b2b".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
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
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Record {
                                        name: fastn_builtins::constants::FTD_BTB.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "base".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
                                                                        text: "#e3f0c4".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
                                                                        text: "#467b28".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
                                                                        text: "#3d741f".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
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
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Record {
                                        name: fastn_builtins::constants::FTD_BTB.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "base".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
                                                                        text: "#c4edfd".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
                                                                        text: "#15223a".to_string()
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
                                                                        text: "#205694".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
                                                                        text: "#1f6feb".to_string()
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
                                                                        text: "#205694".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
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
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Record {
                                        name: fastn_builtins::constants::FTD_BTB.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "base".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
                                                                        text: "#fbefba".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
                                                                        text: "#966220".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
                                                                        text: "#966220".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
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
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Record {
                                        name: fastn_builtins::constants::FTD_CUSTOM_COLORS.to_string(),
                                        fields: std::iter::IntoIterator::into_iter([
                                            (
                                                "one".to_string(),
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
                                                                        text: "#ed753a".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
                                                                        text: "#f3db5f".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
                                                                        text: "#8fdcf8".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
                                                                        text: "#7a65c7".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
                                                                        text: "#eb57be".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
                                                                        text: "#ef8dd6".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
                                                                        text: "#7564be".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
                                                                        text: "#d554b3".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
                                                                        text: "#ec8943".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
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
                                                fastn_resolved::PropertyValue::Value {
                                                    value: fastn_resolved::Value::Record {
                                                        name: fastn_builtins::constants::FTD_COLOR.to_string(),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                "light".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
                                                                        text: "#da7a4a".to_string()
                                                                    },
                                                                    is_mutable: false,
                                                                    line_number: 0
                                                                }
                                                            ), (
                                                                "dark".to_string(),
                                                                fastn_resolved::PropertyValue::Value {
                                                                    value: fastn_resolved::Value::String {
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
            fastn_builtins::constants::FTD_BREAKPOINT_WIDTH_DATA.to_string(),
            fastn_resolved::Definition::Record(fastn_resolved::Record {
                name: fastn_builtins::constants::FTD_BREAKPOINT_WIDTH_DATA.to_string(),
                fields: vec![fastn_resolved::Field {
                    name: "mobile".to_string(),
                    kind: fastn_resolved::Kind::integer().into_kind_data().caption(),
                    mutable: false,
                    default: None,
                    access_modifier: Default::default(),
                    line_number: 0
                }],
                line_number: 0
            })
        ),
        (
            fastn_builtins::constants::FTD_BREAKPOINT_WIDTH.to_string(),
            fastn_resolved::Definition::Variable(fastn_resolved::Variable {
                name: fastn_builtins::constants::FTD_BREAKPOINT_WIDTH.to_string(),
                kind: fastn_resolved::Kind::record
                    (fastn_builtins::constants::FTD_BREAKPOINT_WIDTH_DATA).into_kind_data(),
                mutable: true,
                value: fastn_resolved::PropertyValue::Value {
                    value: fastn_resolved::Value::Record {
                        name: fastn_builtins::constants::FTD_BREAKPOINT_WIDTH_DATA.to_string(),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                "mobile".to_string(),
                                fastn_resolved::PropertyValue::Value {
                                    value: fastn_resolved::Value::Integer {
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
            fastn_builtins::constants::FTD_DEVICE_DATA.to_string(),
            fastn_resolved::Definition::OrType(fastn_resolved::OrType {
                name: fastn_builtins::constants::FTD_DEVICE_DATA.to_string(),
                variants: vec![
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_DEVICE_DATA_MOBILE,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("mobile")
                                 .into_property_value(false, 0),),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_DEVICE_DATA_DESKTOP,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("desktop")
                                 .into_property_value(false, 0),),
                        0,
                    )),
                ],
                line_number: 0
            })
        ),
        (
            fastn_builtins::constants::FTD_DEVICE.to_string(),
            fastn_resolved::Definition::Variable(fastn_resolved::Variable {
                name: fastn_builtins::constants::FTD_DEVICE.to_string(),
                kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_DEVICE_DATA)
                    .into_kind_data(),
                mutable: true,
                value: fastn_resolved::PropertyValue::Value {
                    value: fastn_resolved::Value::OrType {
                        name: fastn_builtins::constants::FTD_DEVICE_DATA.to_string(),
                        variant: fastn_builtins::constants::FTD_DEVICE_DATA_MOBILE.to_string(),
                        full_variant: fastn_builtins::constants::FTD_DEVICE_DATA_MOBILE.to_string(),
                        value: Box::new(fastn_resolved::Value::new_string("mobile")
                            .into_property_value(false, 0))
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
            fastn_builtins::constants::FTD_MASK_IMAGE_DATA.to_string(),
            fastn_resolved::Definition::Record(fastn_resolved::Record {
                name: fastn_builtins::constants::FTD_MASK_IMAGE_DATA.to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    fastn_resolved::Field {
                        name: "src".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_IMAGE_SRC)
                            .into_kind_data().caption().into_optional(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "linear-gradient".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_LINEAR_GRADIENT)
                            .into_kind_data()
                            .into_optional(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "color".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                            .into_kind_data()
                            .into_optional(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ]).collect(),
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_MASK_SIZE.to_string(),
            fastn_resolved::Definition::OrType(fastn_resolved::OrType {
                name: fastn_builtins::constants::FTD_MASK_SIZE.to_string(),
                variants: vec![
                    fastn_resolved::OrTypeVariant::Regular(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_MASK_SIZE_FIXED,
                        fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                            .into_kind_data(),
                        false,
                        None,
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_MASK_SIZE_AUTO,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string(
                                fastn_builtins::constants::FTD_MASK_SIZE_AUTO,
                            )
                                .into_property_value(false, 0),
                        ),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_MASK_SIZE_COVER,
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string(
                                fastn_builtins::constants::FTD_MASK_SIZE_CONTAIN,
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
            fastn_builtins::constants::FTD_MASK_REPEAT.to_string(),
            fastn_resolved::Definition::OrType(fastn_resolved::OrType {
                name: fastn_builtins::constants::FTD_MASK_REPEAT.to_string(),
                variants: vec![
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_MASK_REPEAT_BOTH_REPEAT,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("repeat")
                                 .into_property_value(false, 0),),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_MASK_REPEAT_X_REPEAT,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("repeat-x")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_MASK_REPEAT_Y_REPEAT,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("repeat-y")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_MASK_REPEAT_NO_REPEAT,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("no-repeat")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_MASK_REPEAT_SPACE,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("space")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_MASK_REPEAT_ROUND,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("round")
                            .into_property_value(false, 0)),
                        0,
                    )),
                ],
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_MASK_POSITION.to_string(),
            fastn_resolved::Definition::OrType(fastn_resolved::OrType {
                name: fastn_builtins::constants::FTD_MASK_POSITION.to_string(),
                variants: vec![
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_MASK_POSITION_LEFT,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("left")
                                 .into_property_value(false, 0),),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_MASK_POSITION_CENTER,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("center")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_MASK_POSITION_RIGHT,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("right")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_MASK_POSITION_LEFT_TOP,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("left-top")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_MASK_POSITION_LEFT_CENTER,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("left-center")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_MASK_POSITION_LEFT_BOTTOM,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("left-bottom")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_MASK_POSITION_CENTER_TOP,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("center-top")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_MASK_POSITION_CENTER_CENTER,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("center-center")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_MASK_POSITION_CENTER_BOTTOM,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("center-bottom")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_MASK_POSITION_RIGHT_TOP,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("right-top")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_MASK_POSITION_RIGHT_CENTER,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("right-center")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Constant(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_MASK_POSITION_RIGHT_BOTTOM,
                        fastn_resolved::Kind::string()
                            .into_kind_data()
                            .caption(),
                        false,
                        Some(fastn_resolved::Value::new_string("right-bottom")
                            .into_property_value(false, 0)),
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::AnonymousRecord(fastn_resolved::Record {
                        name: fastn_builtins::constants::FTD_MASK_POSITION_LENGTH.to_string(),
                        fields: std::iter::IntoIterator::into_iter([
                            fastn_resolved::Field {
                                name: "x".to_string(),
                                kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                                    .into_kind_data(),
                                mutable: false,
                                default: None,
                                access_modifier: Default::default(),
                                line_number: 0,
                            },
                            fastn_resolved::Field {
                                name: "y".to_string(),
                                kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                                    .into_kind_data(),
                                mutable: false,
                                default: None,
                                access_modifier: Default::default(),
                                line_number: 0,
                            },
                        ]).collect(),
                        line_number: 0,
                    }),
                ],
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_MASK_MULTI_DATA.to_string(),
            fastn_resolved::Definition::Record(fastn_resolved::Record {
                name: fastn_builtins::constants::FTD_MASK_MULTI_DATA.to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    fastn_resolved::Field {
                        name: "image".to_string(),
                        kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_MASK_IMAGE_DATA)
                            .into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "size".to_string(),
                        kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_MASK_SIZE)
                            .into_kind_data()
                            .into_optional(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "size-x".to_string(),
                        kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_MASK_SIZE)
                            .into_kind_data()
                            .into_optional(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "size-y".to_string(),
                        kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_MASK_SIZE)
                            .into_kind_data()
                            .into_optional(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "repeat".to_string(),
                        kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_MASK_REPEAT)
                            .into_kind_data()
                            .into_optional(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "position".to_string(),
                        kind: fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_MASK_POSITION)
                            .into_kind_data()
                            .into_optional(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ]).collect(),
                line_number: 0,
            }),
        ),
        (
            fastn_builtins::constants::FTD_MASK.to_string(),
            fastn_resolved::Definition::OrType(fastn_resolved::OrType {
                name: fastn_builtins::constants::FTD_MASK.to_string(),
                variants: vec![
                    fastn_resolved::OrTypeVariant::Regular(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_MASK_IMAGE,
                        fastn_resolved::Kind::record(fastn_builtins::constants::FTD_MASK_IMAGE_DATA)
                            .into_kind_data(),
                        false,
                        None,
                        0,
                    )),
                    fastn_resolved::OrTypeVariant::Regular(fastn_resolved::Field::new(
                        fastn_builtins::constants::FTD_MASK_MULTI,
                        fastn_resolved::Kind::record(fastn_builtins::constants::FTD_MASK_MULTI_DATA)
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
            "ftd#integer-field".to_string(),
            fastn_resolved::Definition::Record(fastn_resolved::Record {
                name: "ftd#integer-field".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    fastn_resolved::Field {
                        name: "name".to_string(),
                        kind: fastn_resolved::Kind::string().into_kind_data().caption(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "value".to_string(),
                        kind: fastn_resolved::Kind::integer().into_kind_data(),
                        mutable: false,
                        default: Some(fastn_resolved::PropertyValue::Value {
                            value: fastn_resolved::Value::Integer {
                                value: 0
                            },
                            is_mutable: false,
                            line_number: 0,
                        }),
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "error".to_string(),
                        kind: fastn_resolved::Kind::string().into_optional().into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ]).collect(),
                line_number: 0,
            }),
        ),
        (
            "ftd#decimal-field".to_string(),
            fastn_resolved::Definition::Record(fastn_resolved::Record {
                name: "ftd#decimal-field".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    fastn_resolved::Field {
                        name: "name".to_string(),
                        kind: fastn_resolved::Kind::string().into_kind_data().caption(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "value".to_string(),
                        kind: fastn_resolved::Kind::decimal().into_kind_data(),
                        mutable: false,
                        default: Some(fastn_resolved::PropertyValue::Value {
                            value: fastn_resolved::Value::Decimal {
                                value: 0.0,
                            },
                            is_mutable: false,
                            line_number: 0,
                        }),
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "error".to_string(),
                        kind: fastn_resolved::Kind::string().into_optional().into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ]).collect(),
                line_number: 0,
            }),
        ),
        (
            "ftd#boolean-field".to_string(),
            fastn_resolved::Definition::Record(fastn_resolved::Record {
                name: "ftd#boolean-field".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    fastn_resolved::Field {
                        name: "name".to_string(),
                        kind: fastn_resolved::Kind::string().into_kind_data().caption(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "value".to_string(),
                        kind: fastn_resolved::Kind::boolean().into_kind_data(),
                        mutable: false,
                        default: Some(fastn_resolved::PropertyValue::Value {
                            value: fastn_resolved::Value::Boolean {
                                value: false,
                            },
                            is_mutable: false,
                            line_number: 0,
                        }),
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "error".to_string(),
                        kind: fastn_resolved::Kind::string().into_optional().into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ]).collect(),
                line_number: 0,
            }),
        ),
        (
            "ftd#string-field".to_string(),
            fastn_resolved::Definition::Record(fastn_resolved::Record {
                name: "ftd#string-field".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    fastn_resolved::Field {
                        name: "name".to_string(),
                        kind: fastn_resolved::Kind::string().into_kind_data().caption(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "value".to_string(),
                        kind: fastn_resolved::Kind::string().into_kind_data(),
                        mutable: false,
                        default: Some(fastn_resolved::PropertyValue::Value {
                            value: fastn_resolved::Value::String {
                                text: "".to_string(),
                            },
                            is_mutable: false,
                            line_number: 0,
                        }),
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "error".to_string(),
                        kind: fastn_resolved::Kind::string().into_optional().into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ]).collect(),
                line_number: 0,
            }),
        ),
        (
            "ftd#http-method".to_string(),
            fastn_resolved::Definition::OrType(fastn_resolved::OrType {
                name: "ftd#http-method".to_string(),
                variants: vec![
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        "ftd#http-method.GET",
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("GET")
                                .into_property_value(false, 0),
                        ),
                        0
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        "ftd#http-method.POST",
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("POST")
                                .into_property_value(false, 0),
                        ),
                        0
                    )),
                ],
                line_number: 0,
            }),
        ),
        (
            "ftd#http-redirect".to_string(),
            fastn_resolved::Definition::OrType(fastn_resolved::OrType {
                name: "ftd#http-redirect".to_string(),
                variants: vec![
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        "ftd#http-redirect.follow",
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("follow")
                                .into_property_value(false, 0),
                        ),
                        0
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        "ftd#http-redirect.manual",
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("manual")
                                .into_property_value(false, 0),
                        ),
                        0
                    )),
                    fastn_resolved::OrTypeVariant::new_constant(fastn_resolved::Field::new(
                        "ftd#http-redirect.error",
                        fastn_resolved::Kind::string().into_kind_data(),
                        false,
                        Some(
                            fastn_resolved::Value::new_string("error")
                                .into_property_value(false, 0),
                        ),
                        0
                    )),
                ],
                line_number: 0,
            }),
        ),
    ];

    things.into_iter().collect()
}

pub fn default_migration_bag() -> indexmap::IndexMap<String, fastn_resolved::Definition> {
    let test_things = vec![(
        "fastn#migration".to_string(),
        fastn_resolved::Definition::Component(fastn_migration_function()),
    )];
    test_things.into_iter().collect()
}

pub fn fastn_migration_function() -> fastn_resolved::ComponentDefinition {
    fastn_resolved::ComponentDefinition {
        name: "fastn#migration".to_string(),
        arguments: [vec![
            fastn_resolved::Argument::default(
                "title",
                fastn_resolved::Kind::string()
                    .into_kind_data()
                    .caption()
                    .into_optional(),
            ),
            fastn_resolved::Argument::default(
                "query",
                fastn_resolved::Kind::string().into_kind_data().body(),
            ),
        ]]
        .concat()
        .into_iter()
        .collect(),
        definition: fastn_resolved::ComponentInvocation::from_name("ftd.kernel"),
        css: None,
        line_number: 0,
    }
}

pub fn default_test_bag() -> indexmap::IndexMap<String, fastn_resolved::Definition> {
    let test_things = vec![
        (
            fastn_builtins::constants::FASTN_GET_QUERY_PARAMS.to_string(),
            fastn_resolved::Definition::Record(fastn_resolved::Record {
                name: fastn_builtins::constants::FASTN_GET_QUERY_PARAMS.to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    fastn_resolved::Field {
                        name: "key".to_string(),
                        kind: fastn_resolved::Kind::string().into_kind_data().caption(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                    fastn_resolved::Field {
                        name: "value".to_string(),
                        kind: fastn_resolved::Kind::string().into_kind_data(),
                        mutable: false,
                        default: None,
                        access_modifier: Default::default(),
                        line_number: 0,
                    },
                ])
                .collect(),
                line_number: 0,
            }),
        ),
        (
            "fastn#get".to_string(),
            fastn_resolved::Definition::Component(fastn_get_function()),
        ),
        (
            "fastn#post".to_string(),
            fastn_resolved::Definition::Component(fastn_post_function()),
        ),
        (
            "fastn#redirect".to_string(),
            fastn_resolved::Definition::Component(fastn_redirect_function()),
        ),
        (
            "fastn#test".to_string(),
            fastn_resolved::Definition::Component(fastn_test_function()),
        ),
    ];
    test_things.into_iter().collect()
}

pub fn fastn_get_function() -> fastn_resolved::ComponentDefinition {
    fastn_resolved::ComponentDefinition {
        name: "fastn#get".to_string(),
        arguments: [vec![
            fastn_resolved::Argument::default(
                "title",
                fastn_resolved::Kind::string().into_kind_data().caption(),
            ),
            fastn_resolved::Argument::default(
                "url",
                fastn_resolved::Kind::string().into_kind_data(),
            ),
            fastn_resolved::Argument::default(
                "test",
                fastn_resolved::Kind::string()
                    .into_kind_data()
                    .into_optional(),
            ),
            fastn_resolved::Argument::default(
                "http-status",
                fastn_resolved::Kind::string()
                    .into_kind_data()
                    .into_optional(),
            ),
            fastn_resolved::Argument::default(
                "http-location",
                fastn_resolved::Kind::string()
                    .into_kind_data()
                    .into_optional(),
            ),
            fastn_resolved::Argument::default(
                "http-redirect",
                fastn_resolved::Kind::string()
                    .into_kind_data()
                    .into_optional(),
            ),
            fastn_resolved::Argument::default(
                "id",
                fastn_resolved::Kind::string()
                    .into_kind_data()
                    .into_optional(),
            ),
            fastn_resolved::Argument::default(
                "query-params",
                fastn_resolved::Kind::record(fastn_builtins::constants::FASTN_GET_QUERY_PARAMS)
                    .into_list()
                    .into_kind_data(),
            ),
        ]]
        .concat()
        .into_iter()
        .collect(),
        definition: fastn_resolved::ComponentInvocation::from_name("ftd.kernel"),
        css: None,
        line_number: 0,
    }
}

pub fn fastn_post_function() -> fastn_resolved::ComponentDefinition {
    fastn_resolved::ComponentDefinition {
        name: "fastn#post".to_string(),
        arguments: [vec![
            fastn_resolved::Argument::default(
                "title",
                fastn_resolved::Kind::string().into_kind_data().caption(),
            ),
            fastn_resolved::Argument::default(
                "url",
                fastn_resolved::Kind::string().into_kind_data(),
            ),
            fastn_resolved::Argument::default(
                "body",
                fastn_resolved::Kind::string().into_kind_data().body(),
            ),
            fastn_resolved::Argument::default(
                "test",
                fastn_resolved::Kind::string()
                    .into_kind_data()
                    .into_optional(),
            ),
            fastn_resolved::Argument::default(
                "http-status",
                fastn_resolved::Kind::string()
                    .into_kind_data()
                    .into_optional(),
            ),
            fastn_resolved::Argument::default(
                "http-location",
                fastn_resolved::Kind::string()
                    .into_kind_data()
                    .into_optional(),
            ),
            fastn_resolved::Argument::default(
                "http-redirect",
                fastn_resolved::Kind::string()
                    .into_kind_data()
                    .into_optional(),
            ),
            fastn_resolved::Argument::default(
                "id",
                fastn_resolved::Kind::string()
                    .into_kind_data()
                    .into_optional(),
            ),
        ]]
        .concat()
        .into_iter()
        .collect(),
        definition: fastn_resolved::ComponentInvocation::from_name("ftd.kernel"),
        css: None,
        line_number: 0,
    }
}

pub fn fastn_redirect_function() -> fastn_resolved::ComponentDefinition {
    fastn_resolved::ComponentDefinition {
        name: "fastn#redirect".to_string(),
        arguments: vec![fastn_resolved::Argument::default(
            "http-redirect",
            fastn_resolved::Kind::string().into_kind_data().caption(),
        )],
        definition: fastn_resolved::ComponentInvocation::from_name("ftd.kernel"),
        css: None,
        line_number: 0,
    }
}

pub fn fastn_test_function() -> fastn_resolved::ComponentDefinition {
    fastn_resolved::ComponentDefinition {
        name: "fastn#test".to_string(),
        arguments: [vec![
            fastn_resolved::Argument::default(
                "title",
                fastn_resolved::Kind::string()
                    .into_kind_data()
                    .caption()
                    .into_optional(),
            ),
            fastn_resolved::Argument::default(
                "fixtures",
                fastn_resolved::Kind::string().into_list().into_kind_data(),
            ),
        ]]
        .concat()
        .into_iter()
        .collect(),
        definition: fastn_resolved::ComponentInvocation::from_name("ftd.kernel"),
        css: None,
        line_number: 0,
    }
}

static BUILTINS: std::sync::LazyLock<indexmap::IndexMap<String, fastn_resolved::Definition>> =
    std::sync::LazyLock::new(default_bag);

pub fn builtins() -> &'static indexmap::IndexMap<String, fastn_resolved::Definition> {
    &BUILTINS
}

pub fn image_function() -> fastn_resolved::ComponentDefinition {
    fastn_resolved::ComponentDefinition {
        name: "ftd#image".to_string(),
        arguments: [
            common_arguments(),
            vec![
                fastn_resolved::Argument::default(
                    "src",
                    fastn_resolved::Kind::record(fastn_builtins::constants::FTD_IMAGE_SRC)
                        .into_kind_data()
                        .caption(),
                ),
                fastn_resolved::Argument::default(
                    "fit",
                    fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_IMAGE_FIT)
                        .into_kind_data()
                        .into_optional(),
                ),
                fastn_resolved::Argument::default(
                    "alt",
                    fastn_resolved::Kind::string()
                        .into_kind_data()
                        .into_optional(),
                ),
                fastn_resolved::Argument::default(
                    "fetch-priority",
                    fastn_resolved::Kind::or_type(
                        fastn_builtins::constants::FTD_IMAGE_FETCH_PRIORITY,
                    )
                    .into_kind_data()
                    .into_optional(),
                ),
            ],
        ]
        .concat()
        .into_iter()
        .collect(),
        definition: fastn_resolved::ComponentInvocation::from_name("ftd.kernel"),
        css: None,
        line_number: 0,
    }
}

pub fn audio_function() -> fastn_resolved::ComponentDefinition {
    fastn_resolved::ComponentDefinition {
        name: "ftd#audio".to_string(),
        arguments: [
            common_arguments(),
            vec![
                fastn_resolved::Argument::default(
                    "src",
                    fastn_resolved::Kind::string().into_kind_data(),
                ),
                fastn_resolved::Argument::default(
                    "controls",
                    fastn_resolved::Kind::boolean()
                        .into_optional()
                        .into_kind_data(),
                ),
                fastn_resolved::Argument::default(
                    "loop",
                    fastn_resolved::Kind::boolean()
                        .into_optional()
                        .into_kind_data(),
                ),
                fastn_resolved::Argument::default(
                    "autoplay",
                    fastn_resolved::Kind::boolean()
                        .into_optional()
                        .into_kind_data(),
                ),
                fastn_resolved::Argument::default(
                    "muted",
                    fastn_resolved::Kind::boolean()
                        .into_optional()
                        .into_kind_data(),
                ),
            ],
        ]
        .concat()
        .into_iter()
        .collect(),
        definition: fastn_resolved::ComponentInvocation::from_name("ftd.kernel"),
        css: None,
        line_number: 0,
    }
}

pub fn video_function() -> fastn_resolved::ComponentDefinition {
    fastn_resolved::ComponentDefinition {
        name: "ftd#video".to_string(),
        arguments: [
            common_arguments(),
            vec![
                fastn_resolved::Argument::default(
                    "src",
                    fastn_resolved::Kind::record(fastn_builtins::constants::FTD_VIDEO_SRC)
                        .into_kind_data()
                        .caption(),
                ),
                fastn_resolved::Argument::default(
                    "fit",
                    fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_IMAGE_FIT)
                        .into_kind_data()
                        .into_optional(),
                ),
                fastn_resolved::Argument::default(
                    "controls",
                    fastn_resolved::Kind::boolean()
                        .into_optional()
                        .into_kind_data(),
                ),
                fastn_resolved::Argument::default(
                    "loop",
                    fastn_resolved::Kind::boolean()
                        .into_optional()
                        .into_kind_data(),
                ),
                fastn_resolved::Argument::default(
                    "autoplay",
                    fastn_resolved::Kind::boolean()
                        .into_optional()
                        .into_kind_data(),
                ),
                fastn_resolved::Argument::default(
                    "muted",
                    fastn_resolved::Kind::boolean()
                        .into_optional()
                        .into_kind_data(),
                ),
                fastn_resolved::Argument::default(
                    "poster",
                    fastn_resolved::Kind::record(fastn_builtins::constants::FTD_IMAGE_SRC)
                        .into_optional()
                        .into_kind_data(),
                ),
            ],
        ]
        .concat()
        .into_iter()
        .collect(),
        definition: fastn_resolved::ComponentInvocation::from_name("ftd.kernel"),
        css: None,
        line_number: 0,
    }
}

pub fn boolean_function() -> fastn_resolved::ComponentDefinition {
    fastn_resolved::ComponentDefinition {
        name: "ftd#boolean".to_string(),
        arguments: [
            text_arguments(),
            common_arguments(),
            vec![
                fastn_resolved::Argument::default(
                    "value",
                    fastn_resolved::Kind::boolean()
                        .into_kind_data()
                        .caption_or_body(),
                ),
                fastn_resolved::Argument::default(
                    "style",
                    fastn_resolved::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
                fastn_resolved::Argument::default(
                    "format",
                    fastn_resolved::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
                fastn_resolved::Argument::default(
                    "text-align",
                    fastn_resolved::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
            ],
        ]
        .concat()
        .into_iter()
        .collect(),
        definition: fastn_resolved::ComponentInvocation::from_name("ftd.kernel"),
        css: None,
        line_number: 0,
    }
}

pub fn checkbox_function() -> fastn_resolved::ComponentDefinition {
    fastn_resolved::ComponentDefinition {
        name: "ftd#checkbox".to_string(),
        arguments: [
            common_arguments(),
            vec![
                fastn_resolved::Argument::default(
                    "checked",
                    fastn_resolved::Kind::boolean()
                        .into_optional()
                        .into_kind_data(),
                ),
                fastn_resolved::Argument::default(
                    "enabled",
                    fastn_resolved::Kind::boolean()
                        .into_optional()
                        .into_kind_data(),
                ),
            ],
        ]
        .concat()
        .into_iter()
        .collect(),
        definition: fastn_resolved::ComponentInvocation::from_name("ftd.kernel"),
        css: None,
        line_number: 0,
    }
}

pub fn text_input_function() -> fastn_resolved::ComponentDefinition {
    fastn_resolved::ComponentDefinition {
        name: "ftd#text-input".to_string(),
        arguments: [
            text_arguments(),
            common_arguments(),
            vec![
                fastn_resolved::Argument::default(
                    "placeholder",
                    fastn_resolved::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
                fastn_resolved::Argument::default(
                    "value",
                    fastn_resolved::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
                fastn_resolved::Argument::default(
                    "default-value",
                    fastn_resolved::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
                fastn_resolved::Argument::default(
                    "multiline",
                    fastn_resolved::Kind::boolean()
                        .into_optional()
                        .into_kind_data(),
                ),
                fastn_resolved::Argument::default(
                    "enabled",
                    fastn_resolved::Kind::boolean()
                        .into_optional()
                        .into_kind_data(),
                ),
                fastn_resolved::Argument::default(
                    "max-length",
                    fastn_resolved::Kind::integer()
                        .into_optional()
                        .into_kind_data(),
                ),
                fastn_resolved::Argument::default(
                    "type",
                    fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_TEXT_INPUT_TYPE)
                        .into_optional()
                        .into_kind_data(),
                ),
            ],
        ]
        .concat()
        .into_iter()
        .collect(),
        definition: fastn_resolved::ComponentInvocation::from_name("ftd.kernel"),
        css: None,
        line_number: 0,
    }
}

pub fn integer_function() -> fastn_resolved::ComponentDefinition {
    fastn_resolved::ComponentDefinition {
        name: "ftd#integer".to_string(),
        arguments: [
            text_arguments(),
            common_arguments(),
            vec![
                fastn_resolved::Argument::default(
                    "value",
                    fastn_resolved::Kind::integer()
                        .into_kind_data()
                        .caption_or_body(),
                ),
                fastn_resolved::Argument::default(
                    "style",
                    fastn_resolved::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
                fastn_resolved::Argument::default(
                    "format",
                    fastn_resolved::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
                fastn_resolved::Argument::default(
                    "text-align",
                    fastn_resolved::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
            ],
        ]
        .concat()
        .into_iter()
        .collect(),
        definition: fastn_resolved::ComponentInvocation::from_name("ftd.kernel"),
        css: None,
        line_number: 0,
    }
}

pub fn decimal_function() -> fastn_resolved::ComponentDefinition {
    fastn_resolved::ComponentDefinition {
        name: "ftd#decimal".to_string(),
        arguments: [
            text_arguments(),
            common_arguments(),
            vec![
                fastn_resolved::Argument::default(
                    "value",
                    fastn_resolved::Kind::decimal()
                        .into_kind_data()
                        .caption_or_body(),
                ),
                fastn_resolved::Argument::default(
                    "style",
                    fastn_resolved::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
                fastn_resolved::Argument::default(
                    "format",
                    fastn_resolved::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
            ],
        ]
        .concat()
        .into_iter()
        .collect(),
        definition: fastn_resolved::ComponentInvocation::from_name("ftd.kernel"),
        css: None,
        line_number: 0,
    }
}

pub fn markup_function() -> fastn_resolved::ComponentDefinition {
    fastn_resolved::ComponentDefinition {
        name: "ftd#text".to_string(),
        arguments: [
            text_arguments(),
            common_arguments(),
            vec![fastn_resolved::Argument::default(
                "text",
                fastn_resolved::Kind::string()
                    .into_kind_data()
                    .caption_or_body(),
            )],
        ]
        .concat()
        .into_iter()
        .collect(),
        definition: fastn_resolved::ComponentInvocation::from_name("ftd.kernel"),
        css: None,
        line_number: 0,
    }
}

pub fn row_function() -> fastn_resolved::ComponentDefinition {
    fastn_resolved::ComponentDefinition {
        name: "ftd#row".to_string(),
        arguments: [
            container_root_arguments(),
            container_arguments(),
            common_arguments(),
        ]
        .concat()
        .into_iter()
        .collect(),
        definition: fastn_resolved::ComponentInvocation::from_name("ftd.kernel"),
        css: None,
        line_number: 0,
    }
}

pub fn rive_function() -> fastn_resolved::ComponentDefinition {
    use itertools::Itertools;

    fastn_resolved::ComponentDefinition {
        name: "ftd#rive".to_string(),
        arguments: [
            common_arguments()
                .into_iter()
                .filter(|v| v.name.ne("id"))
                .collect_vec(),
            vec![
                fastn_resolved::Argument::default(
                    "id",
                    fastn_resolved::Kind::string().into_kind_data().caption(),
                ),
                fastn_resolved::Argument::default(
                    "src",
                    fastn_resolved::Kind::string().into_kind_data(),
                ),
                fastn_resolved::Argument::default(
                    "canvas-width",
                    fastn_resolved::Kind::integer()
                        .into_optional()
                        .into_kind_data(),
                ),
                fastn_resolved::Argument::default(
                    "canvas-height",
                    fastn_resolved::Kind::integer()
                        .into_optional()
                        .into_kind_data(),
                ),
                fastn_resolved::Argument::default(
                    "state-machine",
                    fastn_resolved::Kind::string().into_list().into_kind_data(),
                ),
                fastn_resolved::Argument {
                    name: "autoplay".to_string(),
                    kind: fastn_resolved::Kind::boolean().into_kind_data(),
                    mutable: false,
                    default: Some(fastn_resolved::PropertyValue::Value {
                        value: fastn_resolved::Value::Boolean { value: true },
                        is_mutable: false,
                        line_number: 0,
                    }),
                    access_modifier: Default::default(),
                    line_number: 0,
                },
                fastn_resolved::Argument::default(
                    "artboard",
                    fastn_resolved::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
            ],
        ]
        .concat()
        .into_iter()
        .collect(),
        definition: fastn_resolved::ComponentInvocation::from_name("ftd.kernel"),
        css: None,
        line_number: 0,
    }
}

pub fn container_function() -> fastn_resolved::ComponentDefinition {
    fastn_resolved::ComponentDefinition {
        name: "ftd#container".to_string(),
        arguments: [
            container_root_arguments(),
            common_arguments(),
            vec![fastn_resolved::Argument::default(
                "display",
                fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_DISPLAY)
                    .into_optional()
                    .into_kind_data(),
            )],
        ]
        .concat()
        .into_iter()
        .collect(),
        definition: fastn_resolved::ComponentInvocation::from_name("ftd.kernel"),
        css: None,
        line_number: 0,
    }
}

pub fn desktop_function() -> fastn_resolved::ComponentDefinition {
    fastn_resolved::ComponentDefinition {
        name: "ftd#desktop".to_string(),
        arguments: [container_root_arguments()].concat().into_iter().collect(),
        definition: fastn_resolved::ComponentInvocation::from_name("ftd.kernel"),
        css: None,
        line_number: 0,
    }
}

pub fn mobile_function() -> fastn_resolved::ComponentDefinition {
    fastn_resolved::ComponentDefinition {
        name: "ftd#mobile".to_string(),
        arguments: [container_root_arguments()].concat().into_iter().collect(),
        definition: fastn_resolved::ComponentInvocation::from_name("ftd.kernel"),
        css: None,
        line_number: 0,
    }
}

pub fn code_function() -> fastn_resolved::ComponentDefinition {
    fastn_resolved::ComponentDefinition {
        name: "ftd#code".to_string(),
        arguments: [
            text_arguments(),
            common_arguments(),
            vec![
                fastn_resolved::Argument::default(
                    "text",
                    fastn_resolved::Kind::string()
                        .into_kind_data()
                        .caption_or_body(),
                ),
                // TODO: Added `txt` as default
                fastn_resolved::Argument::default(
                    "lang",
                    fastn_resolved::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
                // TODO: Added `CODE_DEFAULT_THEME` as default
                fastn_resolved::Argument::default(
                    "theme",
                    fastn_resolved::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
                fastn_resolved::Argument::default_with_value(
                    "show-line-number",
                    fastn_resolved::Kind::boolean().into_kind_data(),
                    fastn_resolved::Value::Boolean { value: false }.into_property_value(false, 0),
                ),
            ],
        ]
        .concat()
        .into_iter()
        .collect(),
        definition: fastn_resolved::ComponentInvocation::from_name("ftd.kernel"),
        css: None,
        line_number: 0,
    }
}

pub fn iframe_function() -> fastn_resolved::ComponentDefinition {
    fastn_resolved::ComponentDefinition {
        name: "ftd#iframe".to_string(),
        arguments: [
            common_arguments(),
            vec![
                fastn_resolved::Argument::default(
                    "src",
                    fastn_resolved::Kind::string()
                        .into_optional()
                        .into_kind_data()
                        .caption(),
                ),
                fastn_resolved::Argument::default(
                    "youtube",
                    fastn_resolved::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                ),
                fastn_resolved::Argument::default(
                    "srcdoc",
                    fastn_resolved::Kind::string()
                        .into_optional()
                        .into_kind_data()
                        .body(),
                ),
                fastn_resolved::Argument::default(
                    "loading",
                    fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LOADING)
                        .into_optional()
                        .into_kind_data(),
                ),
            ],
        ]
        .concat()
        .into_iter()
        .collect(),
        definition: fastn_resolved::ComponentInvocation::from_name("ftd.kernel"),
        css: None,
        line_number: 0,
    }
}

pub fn column_function() -> fastn_resolved::ComponentDefinition {
    fastn_resolved::ComponentDefinition {
        name: "ftd#column".to_string(),
        arguments: [
            container_root_arguments(),
            container_arguments(),
            common_arguments(),
        ]
        .concat()
        .into_iter()
        .collect(),
        definition: fastn_resolved::ComponentInvocation::from_name("ftd.kernel"),
        css: None,
        line_number: 0,
    }
}

pub fn document_function() -> fastn_resolved::ComponentDefinition {
    fastn_resolved::ComponentDefinition {
        name: "ftd#document".to_string(),
        arguments: [vec![
            fastn_resolved::Argument::default(
                "favicon",
                fastn_resolved::Kind::record(fastn_builtins::constants::FTD_RAW_IMAGE_SRC)
                    .into_optional()
                    .into_kind_data(),
            ),
            fastn_resolved::Argument::default(
                "breakpoint",
                fastn_resolved::Kind::record(fastn_builtins::constants::FTD_BREAKPOINT_WIDTH_DATA)
                    .into_optional()
                    .into_kind_data(),
            ),
            fastn_resolved::Argument::default(
                "facebook-domain-verification",
                fastn_resolved::Kind::string()
                    .into_optional()
                    .into_kind_data(),
            ),
            fastn_resolved::Argument::default(
                "title",
                fastn_resolved::Kind::string()
                    .into_optional()
                    .into_kind_data()
                    .caption_or_body(),
            ),
            fastn_resolved::Argument {
                name: "og-title".to_string(),
                kind: fastn_resolved::Kind::string()
                    .into_optional()
                    .into_kind_data(),
                mutable: false,
                default: Some(fastn_resolved::PropertyValue::Reference {
                    name: "ftd#document.title".to_string(),
                    kind: fastn_resolved::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                    source: fastn_resolved::PropertyValueSource::Local("document".to_string()),
                    is_mutable: false,
                    line_number: 0,
                }),
                access_modifier: Default::default(),
                line_number: 0,
            },
            fastn_resolved::Argument {
                name: "twitter-title".to_string(),
                kind: fastn_resolved::Kind::string()
                    .into_optional()
                    .into_kind_data(),
                mutable: false,
                default: Some(fastn_resolved::PropertyValue::Reference {
                    name: "ftd#document.title".to_string(),
                    kind: fastn_resolved::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                    source: fastn_resolved::PropertyValueSource::Local("document".to_string()),
                    is_mutable: false,
                    line_number: 0,
                }),
                access_modifier: Default::default(),
                line_number: 0,
            },
            fastn_resolved::Argument::default(
                "description",
                fastn_resolved::Kind::string()
                    .into_optional()
                    .into_kind_data(),
            ),
            fastn_resolved::Argument {
                name: "og-description".to_string(),
                kind: fastn_resolved::Kind::string()
                    .into_optional()
                    .into_kind_data(),
                mutable: false,
                default: Some(fastn_resolved::PropertyValue::Reference {
                    name: "ftd#document.description".to_string(),
                    kind: fastn_resolved::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                    source: fastn_resolved::PropertyValueSource::Local("document".to_string()),
                    is_mutable: false,
                    line_number: 0,
                }),
                access_modifier: Default::default(),
                line_number: 0,
            },
            fastn_resolved::Argument {
                name: "twitter-description".to_string(),
                kind: fastn_resolved::Kind::string()
                    .into_optional()
                    .into_kind_data(),
                mutable: false,
                default: Some(fastn_resolved::PropertyValue::Reference {
                    name: "ftd#document.description".to_string(),
                    kind: fastn_resolved::Kind::string()
                        .into_optional()
                        .into_kind_data(),
                    source: fastn_resolved::PropertyValueSource::Local("document".to_string()),
                    is_mutable: false,
                    line_number: 0,
                }),
                access_modifier: Default::default(),
                line_number: 0,
            },
            fastn_resolved::Argument::default(
                "og-image",
                fastn_resolved::Kind::record(fastn_builtins::constants::FTD_RAW_IMAGE_SRC)
                    .into_optional()
                    .into_kind_data(),
            ),
            fastn_resolved::Argument {
                name: "twitter-image".to_string(),
                kind: fastn_resolved::Kind::record(fastn_builtins::constants::FTD_RAW_IMAGE_SRC)
                    .into_optional()
                    .into_kind_data(),
                mutable: false,
                default: Some(fastn_resolved::PropertyValue::Reference {
                    name: "ftd#document.og-image".to_string(),
                    kind: fastn_resolved::Kind::string().into_kind_data(),
                    source: fastn_resolved::PropertyValueSource::Local("document".to_string()),
                    is_mutable: false,
                    line_number: 0,
                }),
                access_modifier: Default::default(),
                line_number: 0,
            },
            fastn_resolved::Argument::default(
                "theme-color",
                fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                    .into_optional()
                    .into_kind_data(),
            ),
            fastn_resolved::Argument::default(
                "children",
                fastn_resolved::Kind::subsection_ui()
                    .into_list()
                    .into_kind_data(),
            ),
            fastn_resolved::Argument::default(
                "colors",
                fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR_SCHEME)
                    .into_optional()
                    .into_kind_data(),
            ),
            fastn_resolved::Argument::default(
                "types",
                fastn_resolved::Kind::record(fastn_builtins::constants::FTD_TYPE_DATA)
                    .into_optional()
                    .into_kind_data(),
            ),
        ]]
        .concat()
        .into_iter()
        .collect(),
        definition: fastn_resolved::ComponentInvocation::from_name("ftd.kernel"),
        css: None,
        line_number: 0,
    }
}

fn container_root_arguments() -> Vec<fastn_resolved::Argument> {
    vec![
        fastn_resolved::Argument::default(
            "children",
            fastn_resolved::Kind::subsection_ui()
                .into_list()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "colors",
            fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR_SCHEME)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "types",
            fastn_resolved::Kind::record(fastn_builtins::constants::FTD_TYPE_DATA)
                .into_optional()
                .into_kind_data(),
        ),
    ]
}

fn container_arguments() -> Vec<fastn_resolved::Argument> {
    vec![
        fastn_resolved::Argument::default(
            "wrap",
            fastn_resolved::Kind::boolean()
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "align-content",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_ALIGN)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "spacing",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_SPACING)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "backdrop-filter",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_BACKDROP_FILTER)
                .into_optional()
                .into_kind_data(),
        ),
    ]
}

fn common_arguments() -> Vec<fastn_resolved::Argument> {
    vec![
        fastn_resolved::Argument::default(
            "opacity",
            fastn_resolved::Kind::decimal()
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "shadow",
            fastn_resolved::Kind::record(fastn_builtins::constants::FTD_SHADOW)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "sticky",
            fastn_resolved::Kind::boolean()
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "rel",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LINK_REL)
                .into_list()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "download",
            fastn_resolved::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "id",
            fastn_resolved::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "border-style",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_BORDER_STYLE)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "border-style-left",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_BORDER_STYLE)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "border-style-right",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_BORDER_STYLE)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "border-style-top",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_BORDER_STYLE)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "border-style-bottom",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_BORDER_STYLE)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "border-style-vertical",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_BORDER_STYLE)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "border-style-horizontal",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_BORDER_STYLE)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "z-index",
            fastn_resolved::Kind::integer()
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "white-space",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_WHITESPACE)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "text-transform",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_TEXT_TRANSFORM)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "region",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_REGION)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "left",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "right",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "top",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "bottom",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "anchor",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_ANCHOR)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "role",
            fastn_resolved::Kind::record(fastn_builtins::constants::FTD_RESPONSIVE_TYPE)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "cursor",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_CURSOR)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "classes",
            fastn_resolved::Kind::string().into_list().into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "js",
            fastn_resolved::Kind::string().into_list().into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "css",
            fastn_resolved::Kind::string().into_list().into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "open-in-new-tab",
            fastn_resolved::Kind::boolean()
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "resize",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_RESIZE)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "overflow",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_OVERFLOW)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "overflow-x",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_OVERFLOW)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "overflow-y",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_OVERFLOW)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "align-self",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_ALIGN_SELF)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "background",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_BACKGROUND)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "border-color",
            fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "color",
            fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "max-width",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_RESIZING)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "min-width",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_RESIZING)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "min-height",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_RESIZING)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "max-height",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_RESIZING)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "width",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_RESIZING)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "height",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_RESIZING)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "padding",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "padding-vertical",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "padding-horizontal",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "padding-left",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "padding-right",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "padding-top",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "padding-bottom",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "margin",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "margin-vertical",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "margin-horizontal",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "margin-left",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "margin-right",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "margin-top",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "margin-bottom",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "border-width",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "border-bottom-width",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "border-bottom-color",
            fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "border-top-width",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "border-top-color",
            fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "border-left-width",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "border-left-color",
            fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "border-right-width",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "border-right-color",
            fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "border-radius",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "border-top-left-radius",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "border-top-right-radius",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "border-bottom-left-radius",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "border-bottom-right-radius",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "link",
            fastn_resolved::Kind::string()
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "selectable",
            fastn_resolved::Kind::boolean()
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "mask",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_MASK)
                .into_optional()
                .into_kind_data(),
        ),
    ]
}

fn text_arguments() -> Vec<fastn_resolved::Argument> {
    vec![
        fastn_resolved::Argument::default(
            "display",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_DISPLAY)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "text-align",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_TEXT_ALIGN)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "line-clamp",
            fastn_resolved::Kind::integer()
                .into_kind_data()
                .into_optional(),
        ),
        fastn_resolved::Argument::default(
            "text-indent",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_LENGTH)
                .into_kind_data()
                .into_optional(),
        ),
        fastn_resolved::Argument::default(
            "style",
            fastn_resolved::Kind::or_type(fastn_builtins::constants::FTD_TEXT_STYLE)
                .into_list()
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "link-color",
            fastn_resolved::Kind::record(fastn_builtins::constants::FTD_COLOR)
                .into_optional()
                .into_kind_data(),
        ),
        fastn_resolved::Argument::default(
            "text-shadow",
            fastn_resolved::Kind::record(fastn_builtins::constants::FTD_SHADOW)
                .into_optional()
                .into_kind_data(),
        ),
    ]
}

/*fn kernel_component() -> fastn_resolved::ComponentDefinition {
    fastn_resolved::ComponentDefinition {
        name: "ftd.kernel".to_string(),
        arguments: vec![],
        definition: fastn_resolved::Component {
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
