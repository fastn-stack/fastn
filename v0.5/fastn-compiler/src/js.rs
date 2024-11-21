impl fastn_compiler::Compiler {
    pub(crate) fn js(&self) -> String {
        use fastn_resolved::tdoc::TDoc;
        use fastn_resolved_to_js::extensions::*;

        // this function should look a bit like ftd::js::document_into_js_ast(), we do not need
        // to construct the Document object there, but will fetch all the fields as variables

        // self.content should be all UR::R now
        let resolved_content = self.resolved_content();
        // every symbol in self.symbol_used in the bag must be UR::R now
        let needed_symbols = self.needed_symbols();
        let doc = fastn_compiler::TDoc {
            name: "",
            definitions: &needed_symbols,
            builtins: fastn_builtins::builtins(),
        };

        // Check if the document tree uses Rive, if so add the Rive script.
        let mut has_rive_components = false;
        let mut export_asts = vec![];

        let mut document_asts = vec![fastn_resolved_to_js::from_tree(
            resolved_content.as_slice(),
            &doc,
            &mut has_rive_components,
        )];

        for thing in needed_symbols.values() {
            if let fastn_resolved::Definition::Component(c) = thing {
                document_asts.push(c.to_ast(&doc, &mut has_rive_components));
            } else if let fastn_resolved::Definition::Variable(v) = thing {
                document_asts.push(v.to_ast(
                    &doc,
                    Some(fastn_js::GLOBAL_VARIABLE_MAP.to_string()),
                    &mut has_rive_components,
                ));
            } else if let fastn_resolved::Definition::WebComponent(web_component) = thing {
                document_asts.push(web_component.to_ast(&doc));
            } else if let fastn_resolved::Definition::Function(f) = thing {
                document_asts.push(f.to_ast(&doc));
            } else if let fastn_resolved::Definition::Export { from, to, .. } = thing {
                if doc.get_opt_record(from).is_some() {
                    continue;
                }
                export_asts.push(fastn_js::Ast::Export {
                    from: from.to_string(),
                    to: to.to_string(),
                })
            } else if let fastn_resolved::Definition::OrType(ot) = thing {
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
                document_asts.push(fastn_js::Ast::OrType(fastn_js::OrType {
                    name: ot.name.clone(),
                    variant: fastn_js::SetPropertyValue::Value(fastn_js::Value::Record {
                        fields,
                        other_references: vec![],
                    }),
                    prefix: Some(fastn_js::GLOBAL_VARIABLE_MAP.to_string()),
                }));
            }
        }

        document_asts.extend(export_asts);

        let mut scripts = fastn_resolved_to_js::utils::get_external_scripts(has_rive_components);
        scripts.push(fastn_resolved_to_js::utils::get_js_html(
            self.external_js_files(&needed_symbols).as_slice(),
        ));
        scripts.push(fastn_resolved_to_js::utils::get_css_html(
            self.external_css_files(&needed_symbols).as_slice(),
        ));

        let js_document_script = fastn_js::to_js(document_asts.as_slice(), "");

        js_document_script
    }
}

pub struct Output {
    js: String,
    css_files: Vec<String>,
    js_files: Vec<String>,
}
