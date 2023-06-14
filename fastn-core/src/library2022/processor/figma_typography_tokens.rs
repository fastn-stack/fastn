use serde::{Deserialize, Serialize};

pub fn process_typography_tokens(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &mut ftd::interpreter::TDoc,
    _config: &fastn_core::Config,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    let line_number = value.line_number();
    let mut variable_name: Option<String> = None;

    dbg!(&value);

    let mut light_colors: ftd::Map<ftd::Map<VT>> = ftd::Map::new();
    let mut dark_colors: ftd::Map<ftd::Map<VT>> = ftd::Map::new();

    // extract_light_dark_colors(
    //     value,
    //     doc,
    //     &mut variable_name,
    //     &mut light_colors,
    //     &mut dark_colors,
    //     line_number,
    // )?;

    let json_formatted_light =
        serde_json::to_string_pretty(&light_colors).expect("Not a serializable type");
    let json_formatted_dark =
        serde_json::to_string_pretty(&dark_colors).expect("Not a serializable type");

    let full_cs = format!(
        "{{\n\"{}-light\": {},\n\"{}-dark\": {}\n}}",
        variable_name
            .clone()
            .unwrap_or_else(|| "Unnamed-cs".to_string())
            .as_str(),
        json_formatted_light,
        variable_name
            .unwrap_or_else(|| "Unnamed-cs".to_string())
            .as_str(),
        json_formatted_dark
    );

    let response_json: serde_json::Value = serde_json::Value::String(full_cs);
    doc.from_json(&response_json, &kind, line_number)
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct VT {
    value: String,
    #[serde(rename = "type")]
    type_: String,
}
