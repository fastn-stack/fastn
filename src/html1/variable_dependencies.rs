pub struct VariableDependencyGenerator<'a> {
    pub id: &'a str,
    pub doc: &'a ftd::interpreter2::TDoc<'a>,
}

impl<'a> VariableDependencyGenerator<'a> {
    pub(crate) fn new(
        id: &'a str,
        doc: &'a ftd::interpreter2::TDoc,
    ) -> VariableDependencyGenerator<'a> {
        VariableDependencyGenerator { id, doc }
    }

    pub(crate) fn get_dependencies(&self) -> ftd::VecMap<String> {
        let mut result: ftd::VecMap<String> = Default::default();
        for variable in self
            .doc
            .bag
            .values()
            .filter_map(|v| v.clone().variable(self.doc.name, v.line_number()).ok())
        {
            dependencies_from_property_value(&mut result, &variable.value, &variable.name);
            for conditional_value in variable.conditional_value.iter() {
                for condition in conditional_value.condition.references.values() {
                    dependencies_from_property_value(&mut result, condition, &variable.name);
                }
                dependencies_from_property_value(
                    &mut result,
                    &conditional_value.value,
                    &variable.name,
                );
            }
        }

        return result;

        fn dependencies_from_property_value(
            result: &mut ftd::VecMap<String>,
            value: &ftd::interpreter2::PropertyValue,
            name: &str,
        ) {
            if let Some(ref_name) = value.reference_name() {
                result.insert(ref_name.to_string(), name.to_string());
            } else if let Some(function_call) = value.get_function() {
                for property_value in function_call.values.values() {
                    dependencies_from_property_value(result, property_value, name);
                }
            }
        }
    }

    pub(crate) fn get_variable_order_dependencies(&self) -> ftd::VecMap<String> {
        use itertools::Itertools;

        let mut visited: ftd::Map<ftd::Map<usize>> = Default::default();
        let graph = self.get_dependencies();
        for key in graph.value.keys() {
            if visited.contains_key(key) {
                continue;
            }
            get_variable_order_dependencies_(&mut visited, key, &graph);
        }

        let mut dependencies: ftd::VecMap<String> = Default::default();

        for (variable_name, dependency) in visited.iter() {
            let mut v = Vec::from_iter(dependency);
            v.sort_by(|&(_, a), &(_, b)| a.cmp(&b));
            let g = v.iter().map(|v| v.0.to_string()).collect_vec();
            dependencies.extend(variable_name.to_string(), g);
        }

        return dependencies;

        fn get_variable_order_dependencies_(
            visited: &mut ftd::Map<ftd::Map<usize>>,
            key: &str,
            graph: &ftd::VecMap<String>,
        ) -> ftd::Map<usize> {
            let mut result: ftd::Map<usize> = Default::default();
            let dependencies = if let Some(dependencies) = graph.value.get(key) {
                dependencies
            } else {
                return result;
            };
            for d in dependencies {
                result.insert(d.to_string(), 1);
                let order_dependencies = get_variable_order_dependencies_(visited, d, graph);
                for (key, order) in order_dependencies.iter() {
                    match result.get(key) {
                        Some(value) if *value >= *order + 1 => {}
                        _ => {
                            result.insert(key.to_string(), *order + 1);
                        }
                    }
                }
            }
            visited.insert(key.to_string(), result.clone());
            result
        }
    }

    pub(crate) fn get_set_functions(&self) -> ftd::html1::Result<String> {
        let (set_function, dep) = self.js_set_functions();
        let mut js_resolve_functions = vec![];
        if !set_function.trim().is_empty() {
            js_resolve_functions.push(format!("window.set_value_{} = {{}};", self.id));
            js_resolve_functions.push(set_function);
        }
        let mut js_resolve_functions_available = false;

        for d in dep {
            let g = self.js_resolve_function(d.as_str())?;
            if !g.trim().is_empty() {
                if !js_resolve_functions_available {
                    js_resolve_functions.push(format!("window.resolve_value_{} = {{}};", self.id));
                    js_resolve_functions_available = true;
                }
                js_resolve_functions.push(g.trim().to_string());
            }
        }
        return Ok(js_resolve_functions.join("\n"));
    }

    fn js_set_functions(&self) -> (String, Vec<String>) {
        let order = self.get_variable_order_dependencies();
        let mut result_1 = vec![];
        let mut result_2 = std::collections::HashSet::new();
        for (key, values) in order.value {
            let mut v = vec![];
            for val in values.iter() {
                v.push(format!(
                    indoc::indoc! {"
                        if(!!window[\"resolve_value_{id}\"] && !!window[\"resolve_value_{id}\"][\"{variable_name}\"]){{\
                            window[\"resolve_value_{id}\"][\"{variable_name}\"](data);
                        }} else {{
                            data[\"{variable_name}\"] = data[\"{set_variable_name}\"]
                        }}"
                    },
                    id = self.id,
                    variable_name = val,
                    set_variable_name = key,
                ));
            }

            result_2.extend(values);
            result_1.push(format!(
                indoc::indoc! {"
                     window.set_value_{id}[\"{key}\"] = function (data, new_value) {{
                            data[\"{key}\"] = new_value;
                            {dependencies}
                     }};
                "},
                id = self.id,
                key = key,
                dependencies = v.join("\n"),
            ));
        }

        (result_1.join("\n"), result_2.into_iter().collect())
    }

    fn js_resolve_function(&self, key: &str) -> ftd::html1::Result<String> {
        let variable = match self.doc.get_variable(key, 0) {
            Ok(variable) if !variable.conditional_value.is_empty() => variable,
            _ => return Ok("".to_string()),
        };

        let mut expressions = vec![];
        for condition in variable.conditional_value {
            let condition_str = ftd::html1::utils::get_condition_string(&condition.condition);
            if let Some(value_string) =
                ftd::html1::utils::get_formatted_dep_string_from_property_value(
                    self.id,
                    self.doc,
                    &condition.value,
                    &None,
                )?
            {
                let value = format!("data[\"{}\"] = {};", variable.name, value_string);
                expressions.push((Some(condition_str), value));
            }
        }

        if let Some(value_string) = ftd::html1::utils::get_formatted_dep_string_from_property_value(
            self.id,
            self.doc,
            &variable.value,
            &None,
        )? {
            let value = format!("data[\"{}\"] = {};", variable.name, value_string);
            expressions.push((None, value));
        }
        let value = ftd::html1::utils::js_expression_from_list(expressions);
        Ok(format!(
            indoc::indoc! {"
                 window.resolve_value_{id}[\"{key}\"] = function(data) {{
                        {value}
                 }}
            "},
            id = self.id,
            key = variable.name,
            value = value,
        ))
    }
}
