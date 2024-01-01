pub fn trim_all_lines(s: &str) -> String {
    use itertools::Itertools;

    s.split('\n').map(|v| v.trim()).join("\n")
}

pub fn trim_start_once(s: &str, matches: &str) -> String {
    if let Some((_, p2)) = s.split_once(matches) {
        return p2.to_string();
    }
    s.to_string()
}

pub fn trim_end_once(s: &str, matches: &str) -> String {
    if let Some((p1, _)) = s.rsplit_once(matches) {
        return p1.to_string();
    }
    s.to_string()
}

pub fn trim_brackets(s: &str) -> String {
    if s.starts_with('(') && s.ends_with(')') {
        return s[1..s.len() - 1].to_string();
    }
    s.to_string()
}

pub(crate) fn name_with_id(s: &str, id: &str) -> String {
    format!("{}:{}", s, id)
}

pub(crate) fn function_name_to_js_function(s: &str) -> String {
    let mut s = s.to_string();
    if s.as_bytes()[0].is_ascii_digit() {
        s = format!("_{}", s);
    }
    s.replace('#', "__")
        .replace('-', "_")
        .replace(':', "___")
        .replace(',', "$")
        .replace("\\\\", "/")
        .replace('\\', "/")
        .replace(['/', '.'], "_")
}

pub(crate) fn js_reference_name(s: &str) -> String {
    ftd::interpreter::utils::js_reference_name(s)
}

pub(crate) fn full_data_id(id: &str, data_id: &str) -> String {
    if data_id.trim().is_empty() {
        id.to_string()
    } else {
        format!("{}:{}", data_id, id)
    }
}

pub(crate) fn node_change_id(id: &str, attr: &str) -> String {
    format!("{}__{}", id, attr)
}

pub(crate) fn get_formatted_dep_string_from_property_value(
    id: &str,
    doc: &ftd::interpreter::TDoc,
    property_value: &ftd::interpreter::PropertyValue,
    pattern_with_eval: &Option<(String, bool)>,
    field: Option<String>,
    string_needs_no_quotes: bool,
) -> ftd::html::Result<Option<String>> {
    /*let field = match field {
        None if property_value.kind().is_ftd_length()
            || property_value.kind().is_ftd_resizing_fixed() =>
        {
            Some("value".to_string())
        }
        Some(a) => Some(a),
        None => None,
    };*/

    let value_string = if let Some(value_string) =
        property_value.to_html_string(doc, field, id, string_needs_no_quotes)?
    {
        value_string
    } else {
        return Ok(None);
    };

    Ok(Some(match pattern_with_eval {
        Some((p, eval)) => {
            let mut pattern = format!("`{}`.format(JSON.stringify({}))", p, value_string);
            if *eval {
                pattern = format!("eval({})", pattern)
            }
            pattern
        }
        None => value_string,
    }))
}

pub(crate) fn get_condition_string(condition: &ftd::interpreter::Expression) -> String {
    get_condition_string_(condition, true)
}

pub(crate) fn get_condition_string_(
    condition: &ftd::interpreter::Expression,
    extra_args: bool,
) -> String {
    let node = condition.update_node_with_variable_reference();
    let expression = ftd::html::ExpressionGenerator.to_string_(&node, true, &[], extra_args);
    format!(
        indoc::indoc! {"
                function(){{
                    {expression}
                }}()"
        },
        expression = expression.trim(),
    )
}

pub(crate) fn js_expression_from_list(
    expressions: Vec<(Option<String>, String)>,
    key: Option<&str>,
    default_for_null: &str,
) -> String {
    let mut conditions = vec![];
    let mut default = None;
    for (condition, expression) in expressions {
        if let Some(condition) = condition {
            conditions.push(format!(
                indoc::indoc! {"
                        {if_exp}({condition}){{
                            {expression}
                        }}
                    "},
                if_exp = if conditions.is_empty() {
                    "if"
                } else {
                    "else if"
                },
                condition = condition,
                expression = expression.trim(),
            ));
        } else {
            default = Some(expression)
        }
    }

    let default = match default {
        Some(d) if conditions.is_empty() => d,
        Some(d) => format!("else {{{}}}", d),
        None if !conditions.is_empty() && key.is_some() && !default_for_null.is_empty() => {
            format!("else {{ {} }}", default_for_null)
        }
        None => "".to_string(),
    };

    format!(
        indoc::indoc! {"
            {expressions}{default}
        "},
        expressions = conditions.join(" "),
        default = default,
    )
}

pub(crate) fn is_dark_mode_dependent(
    value: &ftd::interpreter::PropertyValue,
    doc: &ftd::interpreter::TDoc,
) -> ftd::html::Result<bool> {
    let value = value.clone().resolve(doc, value.line_number())?;
    Ok(value.is_record(ftd::interpreter::FTD_IMAGE_SRC)
        || value.is_record(ftd::interpreter::FTD_COLOR)
        || value.is_or_type_variant(ftd::interpreter::FTD_BACKGROUND_SOLID))
}

pub(crate) fn is_device_dependent(
    value: &ftd::interpreter::PropertyValue,
    doc: &ftd::interpreter::TDoc,
) -> ftd::html::Result<bool> {
    let value = value.clone().resolve(doc, value.line_number())?;
    if value.is_record(ftd::interpreter::FTD_RESPONSIVE_TYPE)
        || value.is_or_type_variant(ftd::interpreter::FTD_LENGTH_RESPONSIVE)
    {
        return Ok(true);
    }

    if value.is_or_type_variant(ftd::interpreter::FTD_RESIZING_FIXED) {
        let property_value = value.get_or_type(doc.name, 0)?.2;
        let value = property_value
            .clone()
            .resolve(doc, property_value.line_number())?;
        return Ok(value.is_record(ftd::interpreter::FTD_RESPONSIVE_TYPE)
            || value.is_or_type_variant(ftd::interpreter::FTD_LENGTH_RESPONSIVE));
    }

    Ok(false)
}

pub(crate) fn dependencies_from_property_value(
    property_value: &ftd::interpreter::PropertyValue,
    doc: &ftd::interpreter::TDoc,
) -> Vec<String> {
    if let Some(ref_name) = property_value.reference_name() {
        vec![ref_name.to_string()]
    } else if let Some(function_call) = property_value.get_function() {
        let mut result = vec![];
        for property_value in function_call.values.values() {
            result.extend(dependencies_from_property_value(property_value, doc));
        }
        result
    } else if property_value.is_value() && property_value.kind().is_ftd_length() {
        dependencies_from_length_property_value(property_value, doc)
    } else if property_value.is_value() && property_value.kind().is_ftd_background_color() {
        let mut values = vec![];
        let value = property_value.value("", 0).unwrap();
        let property_value = value
            .get_or_type(doc.name, property_value.line_number())
            .unwrap()
            .2;
        values.extend(dependencies_from_property_value(property_value, doc));
        values
    } else if property_value.is_value() && property_value.kind().is_ftd_resizing_fixed() {
        let value = property_value.value("", 0).unwrap();
        let property_value = value
            .get_or_type(doc.name, property_value.line_number())
            .unwrap()
            .2;
        if property_value.is_value() && property_value.kind().is_ftd_length() {
            dependencies_from_length_property_value(property_value, doc)
        } else {
            vec![]
        }
    } else if property_value.is_value()
        && (property_value.kind().is_ftd_image_src() || property_value.kind().is_ftd_color())
    {
        let value = property_value.value("", 0).unwrap();
        let property_value = value
            .record_fields(doc.name, property_value.line_number())
            .unwrap();
        let mut v = vec![];
        if let Some(pv) = property_value.get("light") {
            v.extend(dependencies_from_property_value(pv, doc));
        }
        if let Some(pv) = property_value.get("dark") {
            v.extend(dependencies_from_property_value(pv, doc));
        }
        v
    } else if property_value.is_value() && property_value.kind().is_ftd_responsive_type() {
        let value = property_value
            .value("", 0)
            .unwrap()
            .record_fields(doc.name, property_value.line_number())
            .unwrap();
        let mut values = vec![];
        for property_value in value.values() {
            if property_value.is_value() && property_value.kind().is_ftd_type() {
                let value = property_value
                    .value("", 0)
                    .unwrap()
                    .record_fields(doc.name, 0)
                    .unwrap();
                for property_value in value.values() {
                    if property_value.is_value() && property_value.kind().is_ftd_font_size() {
                        let value = property_value.value("", 0).unwrap();
                        let property_value = value.get_or_type(doc.name, 0).unwrap().2;
                        values.extend(dependencies_from_property_value(property_value, doc))
                    }
                }
            }
        }
        values
    } else {
        vec![]
    }
}

fn dependencies_from_length_property_value(
    property_value: &ftd::interpreter::PropertyValue,
    doc: &ftd::interpreter::TDoc,
) -> Vec<String> {
    if property_value.is_value() && property_value.kind().is_ftd_length() {
        let value = property_value
            .value(doc.name, property_value.line_number())
            .unwrap();
        if let Ok(property_value) = value.get_or_type(doc.name, property_value.line_number()) {
            dependencies_from_property_value(property_value.2, doc)
        } else if let Ok(property_value) =
            value.record_fields(doc.name, property_value.line_number())
        {
            let mut values = vec![];
            for field in property_value.values() {
                values.extend(dependencies_from_property_value(field, doc));
            }
            values
        } else {
            vec![]
        }
    } else {
        vec![]
    }
}

impl ftd::interpreter::PropertyValue {
    pub(crate) fn to_html_string(
        &self,
        doc: &ftd::interpreter::TDoc,
        field: Option<String>,
        id: &str,
        string_needs_no_quotes: bool,
    ) -> ftd::html::Result<Option<String>> {
        Ok(match self {
            ftd::interpreter::PropertyValue::Reference { name, .. } => Some(format!(
                "resolve_reference(\"{}\", data){}",
                js_reference_name(name),
                field.map(|v| format!(".{}", v)).unwrap_or_default()
            )),
            ftd::interpreter::PropertyValue::FunctionCall(function_call) => {
                let action = serde_json::to_string(&ftd::html::Action::from_function_call(
                    function_call,
                    id,
                    doc,
                )?)
                .unwrap();
                Some(format!(
                    "window.ftd.handle_function(event, '{}', '{}', this)",
                    id, action
                ))
            }
            ftd::interpreter::PropertyValue::Value {
                value, line_number, ..
            } => value.to_html_string(doc, *line_number, field, id, string_needs_no_quotes)?,
            _ => None,
        })
    }
}

impl ftd::interpreter::Value {
    // string_needs_no_quotes: for class attribute the value should be red-block not "red-block"
    pub(crate) fn to_html_string(
        &self,
        doc: &ftd::interpreter::TDoc,
        line_number: usize,
        field: Option<String>,
        id: &str,
        string_needs_no_quotes: bool,
    ) -> ftd::html::Result<Option<String>> {
        Ok(match self {
            ftd::interpreter::Value::String { text } if !string_needs_no_quotes => {
                Some(format!("\"{}\"", text))
            }
            ftd::interpreter::Value::String { text } if string_needs_no_quotes => {
                Some(text.to_string())
            }
            ftd::interpreter::Value::Integer { value } => Some(value.to_string()),
            ftd::interpreter::Value::Decimal { value } => Some(value.to_string()),
            ftd::interpreter::Value::Boolean { value } => Some(value.to_string()),
            ftd::interpreter::Value::List { data, .. } => {
                let mut values = vec![];
                for value in data {
                    let v = if let Some(v) = value
                        .clone()
                        .resolve(doc, line_number)?
                        .to_html_string(doc, value.line_number(), None, id, true)?
                    {
                        v
                    } else {
                        continue;
                    };
                    values.push(v);
                }
                Some(format!("{:?}", values.join(" ")))
            }
            ftd::interpreter::Value::Record { fields, .. }
                if field
                    .as_ref()
                    .map(|v| fields.contains_key(v))
                    .unwrap_or(false) =>
            {
                fields.get(&field.unwrap()).unwrap().to_html_string(
                    doc,
                    None,
                    id,
                    string_needs_no_quotes,
                )?
            }
            ftd::interpreter::Value::OrType {
                value,
                variant,
                full_variant,
                name,
                ..
            } => {
                let value = value.to_html_string(doc, field, id, string_needs_no_quotes)?;
                match value {
                    Some(value) if name.eq(ftd::interpreter::FTD_LENGTH) => {
                        if let Ok(pattern) = ftd::executor::Length::set_pattern_from_variant_str(
                            variant,
                            doc.name,
                            line_number,
                        ) {
                            Some(format!("`{}`.format(JSON.stringify({}))", pattern, value))
                        } else {
                            Some(value)
                        }
                    }
                    Some(value)
                        if name.eq(ftd::interpreter::FTD_RESIZING)
                            && variant.ne(ftd::interpreter::FTD_RESIZING_FIXED) =>
                    {
                        if let Ok(pattern) = ftd::executor::Resizing::set_pattern_from_variant_str(
                            variant,
                            full_variant,
                            doc.name,
                            line_number,
                        ) {
                            Some(format!("`{}`.format(JSON.stringify({}))", pattern, value))
                        } else {
                            Some(value)
                        }
                    }
                    Some(value) => Some(value),
                    None => None,
                }
            }
            ftd::interpreter::Value::Record { fields, .. } => {
                let mut values = vec![];
                for (k, v) in fields {
                    let value = if let Some(v) =
                        v.to_html_string(doc, field.clone(), id, string_needs_no_quotes)?
                    {
                        v
                    } else {
                        "null".to_string()
                    };
                    values.push(format!("\"{}\": {}", k, value));
                }

                Some(format!("{{{}}}", values.join(", ")))
            }
            ftd::interpreter::Value::Optional { data, .. } if data.is_none() => None,
            t => unimplemented!("{:?}", t),
        })
    }
}

pub(crate) fn events_to_string(events: Vec<(String, String, String)>) -> String {
    use itertools::Itertools;

    if events.is_empty() {
        return "".to_string();
    }

    let global_variables =
        "let global_keys = {};\nlet buffer = [];\nlet lastKeyTime = Date.now();".to_string();
    let mut keydown_seq_event = "".to_string();
    let mut keydown_events = indoc::indoc! {"
        document.addEventListener(\"keydown\", function(event) {
            let event_key =  window.ftd.utils.get_event_key(event);
            global_keys[event_key] = true;
            const currentTime = Date.now();
            if (currentTime - lastKeyTime > 1000) {{
                buffer = [];
            }}
            lastKeyTime = currentTime;
            if (event.target.nodeName === \"INPUT\" || event.target.nodeName === \"TEXTAREA\") {
                return;
            }          
            buffer.push(event_key);
    "}
    .to_string();

    for (keys, actions) in events.iter().filter_map(|e| {
        if let Some(keys) = e.1.strip_prefix("onglobalkeyseq[") {
            let keys = keys
                .trim_end_matches(']')
                .split('-')
                .map(to_key)
                .collect_vec();
            Some((keys, e.2.clone()))
        } else {
            None
        }
    }) {
        keydown_seq_event = format!(
            indoc::indoc! {"
                {string}
                if (buffer.join(',').includes(\"{sequence}\")) {{
                   {actions}
                    buffer = [];
                    let event_key =  window.ftd.utils.get_event_key(event);
                    global_keys[event_key] = false;
                    return;
                }}
            "},
            string = keydown_seq_event,
            sequence = keys.join(","),
            actions = actions,
        );
    }

    let keyup_events = r#"document.addEventListener("keyup", function(event) {
        let event_key = window.ftd.utils.get_event_key(event);
        global_keys[event_key] = false; })"#
        .to_string();

    for (keys, actions) in events.iter().filter_map(|e| {
        if let Some(keys) = e.1.strip_prefix("onglobalkey[") {
            let keys = keys
                .trim_end_matches(']')
                .split('-')
                .map(to_key)
                .collect_vec();
            Some((keys, e.2.clone()))
        } else {
            None
        }
    }) {
        let all_keys = keys
            .iter()
            .map(|v| format!("global_keys[\"{}\"]", v))
            .join(" && ");
        keydown_seq_event = format!(
            indoc::indoc! {"
                        {string}
                        if ({all_keys} && buffer.join(',').includes(\"{sequence}\")) {{
                            {actions}
                            buffer = [];
                            let event_key =  window.ftd.utils.get_event_key(event);
                            global_keys[event_key] = false;
                            return;
                        }}
                    "},
            string = keydown_seq_event,
            all_keys = all_keys,
            sequence = keys.join(","),
            actions = actions,
        );
    }

    if !keydown_seq_event.is_empty() {
        keydown_events = format!("{}\n\n{}}});", keydown_events, keydown_seq_event);
    }

    let mut string = "document.addEventListener(\"click\", function(event) {".to_string();
    for event in events.iter().filter(|e| e.1.eq("onclickoutside")) {
        string = format!(
            indoc::indoc! {"
                {string}
                if (document.querySelector(`[data-id=\"{data_id}\"]`).style.display !== \"none\" && !document.querySelector(`[data-id=\"{data_id}\"]`).contains(event.target)) {{
                    {event}
                }}
            "},
            string = string,
            data_id = event.0,
            event = event.2,
        );
    }
    string = format!("{}}});", string);

    if !keydown_seq_event.is_empty() {
        format!(
            "{}\n\n\n{}\n\n\n{}\n\n\n{}",
            string, global_variables, keydown_events, keyup_events
        )
    } else {
        string
    }
}

fn to_key(key: &str) -> String {
    match key {
        "ctrl" => "Control",
        "alt" => "Alt",
        "shift" => "Shift",
        "up" => "ArrowUp",
        "down" => "ArrowDown",
        "right" => "ArrowRight",
        "left" => "ArrowLeft",
        "esc" => "Escape",
        "dash" => "-",
        "space" => " ",
        t => t,
    }
    .to_string()
}

pub(crate) fn get_new_number(keys: &Vec<String>, name: &str) -> usize {
    let mut number = 0;
    for key in keys {
        if let Some(str_number) = key.strip_prefix(format!("{}_", name).as_str()) {
            let found_number = str_number.parse::<usize>().unwrap();
            if found_number >= number {
                number = found_number;
            }
        }
    }
    number
}

pub(crate) fn to_properties_string(
    id: &str,
    properties: &[(String, ftd::interpreter::Property)],
    doc: &ftd::interpreter::TDoc,
    node: &str,
) -> Option<String> {
    let mut properties_string = "".to_string();
    for (key, properties) in group_vec_to_map(properties).value {
        let mut expressions = vec![];
        for property in properties {
            let condition = property
                .condition
                .as_ref()
                .map(ftd::html::utils::get_condition_string);
            if let Ok(Some(value_string)) =
                ftd::html::utils::get_formatted_dep_string_from_property_value(
                    id,
                    doc,
                    &property.value,
                    &None,
                    None,
                    false,
                )
            {
                let value = format!("args[\"{}\"][\"{}\"] = {};", node, key, value_string);
                expressions.push((condition, value));
            }
        }
        let value =
            ftd::html::utils::js_expression_from_list(expressions, Some(key.as_str()), "null");
        properties_string = format!("{}\n\n{}", properties_string, value);
    }
    if properties_string.is_empty() {
        None
    } else {
        Some(format!(
            "args[\"{}\"]={{}};\n{}",
            node,
            properties_string.trim(),
        ))
    }
}

pub(crate) fn to_argument_string(
    id: &str,
    arguments: &[ftd::interpreter::Argument],
    doc: &ftd::interpreter::TDoc,
    node: &str,
) -> Option<String> {
    let mut properties_string = "".to_string();
    for argument in arguments {
        let mut result_value = "null".to_string();
        if let Some(ref value) = argument.value {
            if let Ok(Some(value_string)) =
                ftd::html::utils::get_formatted_dep_string_from_property_value(
                    id, doc, value, &None, None, false,
                )
            {
                result_value = value_string;
            }
        }
        properties_string = format!(
            "{}\nargs[\"{}\"][\"{}\"] = {};",
            properties_string, node, argument.name, result_value
        );
    }
    if properties_string.is_empty() {
        None
    } else {
        Some(format!(
            "args[\"{}\"]={{}};\n{}",
            node,
            properties_string.trim()
        ))
    }
}

fn group_vec_to_map<T>(vec: &[(String, T)]) -> ftd::VecMap<T>
where
    T: PartialEq + Clone,
{
    let mut map: ftd::VecMap<T> = ftd::VecMap::new();
    for (key, value) in vec {
        map.insert(key.to_string(), value.clone());
    }
    map
}

pub(crate) fn mutable_value(mutable_variables: &[String], id: &str) -> String {
    let mut values = vec![];
    if mutable_variables.is_empty() {
        return "".to_string();
    }
    for var in mutable_variables {
        values.push(format!(
            indoc::indoc! {"
                     window.ftd.mutable_value_{id}[\"{key}\"] = {{
                            \"get\": function() {{ return window.ftd.get_value(\"{id}\", \"{key}\");}},
                            \"set\": function(value) {{ window.ftd.set_value_by_id(\"{id}\", \"{key}\", value) }},
                            \"changes\": [],
                            \"on_change\": function(fun) {{ this.changes.push(fun); }}
                     }};
                "},
            id = id,
            key = ftd::html::utils::js_reference_name(var.as_str())
        ));
    }
    format!(
        "window.ftd.mutable_value_{} = {{}}; \n{}",
        id,
        values.join("\n")
    )
}

pub(crate) fn immutable_value(immutable_variables: &[String], id: &str) -> String {
    let mut values = vec![];
    if immutable_variables.is_empty() {
        return "".to_string();
    }
    for var in immutable_variables {
        values.push(format!(
            indoc::indoc! {"
                     window.ftd.immutable_value_{id}[\"{key}\"] = {{
                            \"get\": function() {{ return window.ftd.get_value(\"{id}\", \"{key}\");}},
                            \"changes\": [],
                            \"on_change\": function(fun) {{ this.changes.push(fun); }}
                     }};
                "},
            id = id,
            key = ftd::html::utils::js_reference_name(var.as_str())
        ));
    }
    format!(
        "window.ftd.immutable_value_{} = {{}}; \n{}",
        id,
        values.join("\n")
    )
}

pub fn get_js_html(external_js: &[String]) -> String {
    let mut result = "".to_string();
    for js in external_js {
        if let Some((js, tags)) = js.split_once(':') {
            result = format!("{}<script src=\"{}\" {}></script>", result, js, tags);
        } else {
            result = format!("{}<script src=\"{}\"></script>", result, js);
        }
    }
    result
}

pub fn get_rive_data_html(
    rive_data: &[ftd::executor::RiveData],
    id: &str,
    doc: &ftd::interpreter::TDoc,
) -> ftd::html::Result<String> {
    if rive_data.is_empty() {
        return Ok("".to_string());
    }

    let mut rive_elements: ftd::Map<ftd::executor::RiveData> = Default::default();
    for rive in rive_data {
        if let Some(rive_data) = rive_elements.get_mut(&rive.id) {
            rive_data.events.extend(rive.events.to_owned());
        } else {
            rive_elements.insert(rive.id.to_string(), rive.to_owned());
        }
    }

    let mut result = vec![];
    for rive in rive_elements.values() {
        result.push(get_rive_html(rive, id, doc)?);
    }

    Ok(format!(
        "<script src=\"https://unpkg.com/@rive-app/canvas@1.0.98\"></script><script>{}</script>",
        result.join("\n")
    ))
}

fn get_rive_html(
    rive: &ftd::executor::RiveData,
    id: &str,
    doc: &ftd::interpreter::TDoc,
) -> ftd::html::Result<String> {
    use itertools::Itertools;

    let rive_name = ftd::html::utils::function_name_to_js_function(
        ftd::html::utils::name_with_id(rive.id.as_str(), id).as_str(),
    );

    let state_machines = if rive.state_machine.len().eq(&1) {
        format!("'{}'", rive.state_machine[0])
    } else {
        format!(
            "[{}]",
            rive.state_machine
                .iter()
                .map(|v| format!("'{}'", v))
                .join(",")
        )
    };

    let artboard = rive
        .artboard
        .as_ref()
        .map_or("null".to_string(), |v| format!("'{}'", v));

    let events = get_rive_event(rive, id, doc)?;

    Ok(format!(
        indoc::indoc! {"
                window.{rive_name} = new rive.Rive({{
                    src: '{src}',
                    canvas: document.getElementById('{id}'),
                    autoplay: {autoplay},
                    stateMachines: {state_machines},
                    artboard: {artboard},
                    onLoad: (_) => {{
                        window.{rive_name}.resizeDrawingSurfaceToCanvas();
                    }},
                    {events}
                }});
            "},
        rive_name = rive_name,
        src = rive.src,
        id = rive.id,
        autoplay = rive.autoplay,
        state_machines = state_machines,
        artboard = artboard,
        events = events
    ))
}

fn get_rive_event(
    rive: &ftd::executor::RiveData,
    id: &str,
    doc: &ftd::interpreter::TDoc,
) -> ftd::html::Result<String> {
    let mut events_map: ftd::VecMap<(&String, &ftd::interpreter::FunctionCall)> =
        ftd::VecMap::new();
    for event in rive.events.iter() {
        let (event_name, input, action) = match &event.name {
            ftd::interpreter::EventName::RivePlay(timeline) => ("onPlay", timeline, &event.action),
            ftd::interpreter::EventName::RivePause(timeline) => {
                ("onPause", timeline, &event.action)
            }
            ftd::interpreter::EventName::RiveStateChange(state) => {
                ("onStateChange", state, &event.action)
            }
            _ => continue,
        };
        events_map.insert(event_name.to_string(), (input, action));
    }

    let mut events_vec = vec![];
    for (on, actions) in events_map.value {
        let mut actions_vec = vec![];
        for (input, action) in actions {
            let action = {
                let action = ftd::html::Action::from_function_call(action, id, doc)?.into_list();
                let serde_action = serde_json::to_string(&action).expect("");
                format!(
                    "window.ftd.handle_event(event, '{}', '{}', this)",
                    id, serde_action
                )
            };
            actions_vec.push(format!(
                indoc::indoc! {"
                      if (input === \"{input}\") {{
                        {action}
                      }}
                "},
                input = input,
                action = action
            ));
        }

        events_vec.push(format!(
            indoc::indoc! {"
                    {on}: (event) => {{
                        const inputs = event.data;
                        inputs.forEach((input) => {{
                          {actions_vec}
                        }});
                    }},
                "},
            on = on,
            actions_vec = actions_vec.join("\n")
        ));
    }
    Ok(events_vec.join("\n"))
}

pub fn get_css_html(external_css: &[String]) -> String {
    let mut result = "".to_string();
    for css in external_css {
        result = format!("{}<link rel=\"stylesheet\" href=\"{}\">", result, css);
    }
    result
}

pub fn get_meta_data(html_data: &ftd::html::HTMLData) -> String {
    let mut result = vec![];
    if let Some(ref title) = html_data.og_title {
        result.push(format!(
            "<meta property=\"og:title\" content=\"{}\">",
            title
        ));
    }
    if let Some(ref title) = html_data.twitter_title {
        result.push(format!(
            "<meta name=\"twitter:title\" content=\"{}\">",
            title
        ));
    }
    if let Some(ref description) = html_data.og_description {
        result.push(format!(
            "<meta property=\"og:description\" content=\"{}\">",
            description
        ));
    }
    if let Some(ref description) = html_data.description {
        result.push(format!(
            "<meta name=\"description\" content=\"{}\">",
            description
        ));
    }
    if let Some(ref title) = html_data.twitter_description {
        result.push(format!(
            "<meta name=\"twitter:description\" content=\"{}\">",
            title
        ));
    }
    if let Some(ref image) = html_data.og_image {
        result.push(format!(
            "<meta property=\"og:image\" content=\"{}\">",
            image
        ));
    }
    if let Some(ref image) = html_data.twitter_image {
        result.push(format!(
            "<meta property=\"twitter:image\" content=\"{}\">",
            image
        ));
    }
    if let Some(ref color) = html_data.theme_color {
        result.push(format!("<meta name=\"theme-color\" content=\"{}\">", color));
    }
    result.join("")
}
