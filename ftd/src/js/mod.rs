#![allow(dead_code)]

use fastn_runtime::extensions::*;

#[cfg(test)]
#[macro_use]
mod ftd_test_helpers;

pub const CODE_DEFAULT_THEME: &str = "fastn-theme.dark";

pub fn all_js_without_test(package_name: &str) -> String {
    let all_js = fastn_js::all_js_without_test();
    let default_bag_js = fastn_js::to_js(default_bag_into_js_ast().as_slice(), package_name);
    format!("{all_js}\n{default_bag_js}")
}

/// This returns asts of things present in `ftd` module or `default_bag`
pub fn default_bag_into_js_ast() -> Vec<fastn_js::Ast> {
    let mut ftd_asts = vec![];
    let bag = ftd::interpreter::default::builtins();
    let doc = ftd::interpreter::TDoc {
        name: "",
        aliases: &ftd::interpreter::default::default_aliases(),
        bag: ftd::interpreter::BagOrState::Bag(bag),
    };
    let mut export_asts = vec![];
    for thing in ftd::interpreter::default::builtins().values() {
        if let ftd::interpreter::Thing::Variable(v) = thing {
            ftd_asts.push(v.to_ast(&doc, None, &mut false));
        } else if let ftd::interpreter::Thing::Function(f) = thing {
            if f.external_implementation {
                continue;
            }
            ftd_asts.push(f.to_ast(&doc));
        } else if let ftd::interpreter::Thing::Export { from, to, .. } = thing {
            export_asts.push(fastn_js::Ast::Export {
                from: from.to_string(),
                to: to.to_string(),
            })
        }
    }

    // Global default inherited variable
    ftd_asts.push(fastn_js::Ast::StaticVariable(fastn_js::StaticVariable {
        name: "inherited".to_string(),
        value: fastn_js::SetPropertyValue::Value(fastn_js::Value::Record {
            fields: vec![
                (
                    "colors".to_string(),
                    fastn_js::SetPropertyValue::Reference(
                        "ftd#default-colors__DOT__getClone()__DOT__setAndReturn\
                        (\"is_root\"__COMMA__\
                         true)"
                            .to_string(),
                    ),
                ),
                (
                    "types".to_string(),
                    fastn_js::SetPropertyValue::Reference(
                        "ftd#default-types__DOT__getClone()__DOT__setAndReturn\
                        (\"is_root\"__COMMA__\
                         true)"
                            .to_string(),
                    ),
                ),
            ],
            other_references: vec![],
        }),
        prefix: None,
    }));

    ftd_asts.extend(export_asts);
    ftd_asts
}

const IGNORE_GLOBAL: [&str; 2] = ["ftd#main-package", "ftd#app-urls"];

#[derive(Debug)]
pub struct JSAstData {
    /// This contains asts of things (other than `ftd`) and instructions/tree
    pub asts: Vec<fastn_js::Ast>,
    /// This contains external scripts provided by user and also `ftd`
    /// internally supports (like rive).
    pub scripts: Vec<String>,
}

pub fn document_into_js_ast(document: ftd::interpreter::Document) -> JSAstData {
    use fastn_runtime::extensions::*;
    use itertools::Itertools;

    let doc = ftd::interpreter::TDoc::new(&document.name, &document.aliases, &document.data);

    // Check if document tree has rive. This is used to add rive script.
    let mut has_rive_components = false;
    let mut document_asts = vec![fastn_runtime::from_tree(
        document.tree.as_slice(),
        &doc,
        &mut has_rive_components,
    )];
    let default_thing_name = ftd::interpreter::default::builtins()
        .into_iter()
        .map(|v| v.0)
        .collect_vec();

    // Fix the export order while generating ast
    // export item should be inserted as soon as `from` is available
    let mut export_asts: indexmap::IndexMap<String, Vec<fastn_js::Ast>> = Default::default();
    for (key, thing) in document.data.iter() {
        if default_thing_name.contains(&key) {
            continue;
        }
        if let ftd::interpreter::Thing::Export { from, to, .. } = thing {
            if doc.get_record(from, 0).is_ok() {
                continue;
            }
            if let Some(asts) = export_asts.get_mut(from) {
                asts.push(fastn_js::Ast::Export {
                    from: from.to_string(),
                    to: to.to_string(),
                });
            } else {
                export_asts.insert(
                    from.to_string(),
                    vec![fastn_js::Ast::Export {
                        from: from.to_string(),
                        to: to.to_string(),
                    }],
                );
            }
        }
    }

    for (key, thing) in document.data.iter() {
        if default_thing_name.contains(&key) {
            continue;
        }
        if let ftd::interpreter::Thing::Component(c) = thing {
            document_asts.push(c.to_ast(&doc, &mut has_rive_components));
        } else if let ftd::interpreter::Thing::Variable(v) = thing {
            let prefix = if IGNORE_GLOBAL.contains(&v.name.as_str()) {
                None
            } else {
                Some(fastn_js::GLOBAL_VARIABLE_MAP.to_string())
            };

            document_asts.push(v.to_ast(&doc, prefix, &mut has_rive_components));
        } else if let ftd::interpreter::Thing::WebComponent(web_component) = thing {
            document_asts.push(web_component.to_ast(&doc));
        } else if let ftd::interpreter::Thing::Function(f) = thing {
            document_asts.push(f.to_ast(&doc));
        } else if let ftd::interpreter::Thing::Export { .. } = thing {
            continue;
        } else if let ftd::interpreter::Thing::OrType(ot) = thing {
            let mut fields = vec![];
            for variant in &ot.variants {
                if let Some(value) = &variant.clone().fields().get(0).unwrap().value {
                    fields.push((
                        variant
                            .name()
                            .trim_start_matches(
                                format!(
                                    "{}.",
                                    fastn_resolved::OrType::or_type_name(ot.name.as_str())
                                )
                                .as_str(),
                            )
                            .to_string(),
                        value.to_fastn_js_value_with_none(&doc, &mut false),
                    ));
                }
            }
            let prefix = if IGNORE_GLOBAL.contains(&ot.name.as_str()) {
                None
            } else {
                Some(fastn_js::GLOBAL_VARIABLE_MAP.to_string())
            };
            document_asts.push(fastn_js::Ast::OrType(fastn_js::OrType {
                name: ot.name.clone(),
                variant: fastn_js::SetPropertyValue::Value(fastn_js::Value::Record {
                    fields,
                    other_references: vec![],
                }),
                prefix,
            }));
        }

        if let Some(ast) = export_asts.shift_remove(key) {
            document_asts.extend(ast);
        }
    }

    document_asts.extend(export_asts.into_iter().map(|(_k, v)| v).flatten());
    let mut scripts = fastn_runtime::utils::get_external_scripts(has_rive_components);
    scripts.push(fastn_runtime::utils::get_js_html(
        document.js.into_iter().collect_vec().as_slice(),
    ));
    scripts.push(fastn_runtime::utils::get_css_html(
        document.css.into_iter().collect_vec().as_slice(),
    ));

    JSAstData {
        asts: document_asts,
        scripts,
    }
}
