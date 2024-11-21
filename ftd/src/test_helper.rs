pub fn ftd_v2_interpret_helper(
    name: &str,
    source: &str,
) -> ftd::interpreter::Result<ftd::interpreter::Document> {
    let mut s = ftd::interpreter::interpret(name, source)?;
    let document;
    loop {
        match s {
            ftd::interpreter::Interpreter::Done { document: doc } => {
                document = doc;
                break;
            }
            ftd::interpreter::Interpreter::StuckOnImport {
                module, state: st, ..
            } => {
                let mut source = "".to_string();
                let mut foreign_variable = vec![];
                let mut foreign_function = vec![];
                if module.eq("test") {
                    foreign_variable.push("var".to_string());
                    foreign_function.push("fn".to_string());
                }
                if let Ok(value) = std::fs::read_to_string(format!("./ftd/t/html/{}.ftd", module)) {
                    source = value;
                }
                let document = ftd::interpreter::ParsedDocument::parse_with_line_number(
                    module.as_str(),
                    source.as_str(),
                    0,
                )?;

                s = st.continue_after_import(
                    module.as_str(),
                    document,
                    foreign_variable,
                    foreign_function,
                    0,
                )?;
            }
            ftd::interpreter::Interpreter::StuckOnProcessor {
                state, ast, module, ..
            } => {
                let variable_definition = ast.clone().get_variable_definition(module.as_str())?;
                let processor = variable_definition.processor.unwrap();
                let value = fastn_resolved::Value::String {
                    text: variable_definition
                        .value
                        .caption()
                        .unwrap_or(processor)
                        .to_uppercase()
                        .to_string(),
                };
                s = state.continue_after_processor(value, ast)?;
            }
            ftd::interpreter::Interpreter::StuckOnForeignVariable {
                state,
                module,
                variable,
                ..
            } => {
                if module.eq("test") {
                    let value = fastn_resolved::Value::String {
                        text: variable.to_uppercase().to_string(),
                    };
                    s = state.continue_after_variable(module.as_str(), variable.as_str(), value)?;
                } else {
                    return ftd::interpreter::utils::e2(
                        format!("Unknown module {}", module),
                        module.as_str(),
                        0,
                    );
                }
            }
        }
    }
    Ok(document)
}
