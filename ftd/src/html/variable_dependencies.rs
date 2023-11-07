pub struct VariableDependencyGenerator<'a> {
    pub id: &'a str,
    pub doc: &'a ftd::interpreter::TDoc<'a>,
}

impl VariableDependencyGenerator<'_> {
    pub(crate) fn new<'a>(
        id: &'a str,
        doc: &'a ftd::interpreter::TDoc,
    ) -> VariableDependencyGenerator<'a> {
        VariableDependencyGenerator { id, doc }
    }

    pub(crate) fn get_dependencies(&self) -> ftd::VecMap<String> {
        let mut result: ftd::VecMap<String> = Default::default();
        for variable in self
            .doc
            .bag()
            .values()
            .filter_map(|v| v.clone().variable(self.doc.name, v.line_number()).ok())
        {
            dependencies_from_property_value(
                &mut result,
                &variable.value,
                &variable.name,
                self.doc,
            );
            for conditional_value in variable.conditional_value.iter() {
                for condition in conditional_value.condition.references.values() {
                    dependencies_from_property_value(
                        &mut result,
                        condition,
                        &variable.name,
                        self.doc,
                    );
                }
                dependencies_from_property_value(
                    &mut result,
                    &conditional_value.value,
                    &variable.name,
                    self.doc,
                );
            }
            if result.value.get(&variable.name).is_none() {
                result.extend(variable.name.to_string(), vec![]);
            }
        }

        return result;

        fn dependencies_from_property_value(
            result: &mut ftd::VecMap<String>,
            value: &ftd::interpreter::PropertyValue,
            name: &str,
            doc: &ftd::interpreter::TDoc,
        ) {
            let value = ftd::html::utils::dependencies_from_property_value(value, doc);
            for v in value {
                result.insert(v, name.to_string());
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
            v.sort_by(|&(_, a), &(_, b)| a.cmp(b));
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
                        Some(value) if *value > *order => {}
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

    pub(crate) fn get_set_functions(
        &self,
        var_dependencies: &ftd::VecMap<String>,
        test: bool,
    ) -> ftd::html::Result<String> {
        let (set_function, mut dep) = self.js_set_functions(var_dependencies, test);
        let mut js_resolve_functions = vec![];
        if !set_function.trim().is_empty() {
            js_resolve_functions.push(format!("window.set_value_{} = {{}};", self.id));
            js_resolve_functions.push(set_function);
        }
        let mut js_resolve_functions_available = false;

        if cfg!(test) || test {
            dep.sort();
        }

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
        Ok(js_resolve_functions.join("\n"))
    }

    fn js_set_functions(
        &self,
        var_dependencies: &ftd::VecMap<String>,
        test: bool,
    ) -> (String, Vec<String>) {
        let order = self.get_variable_order_dependencies();
        let mut result_1 = vec![];
        let mut result_2 = std::collections::HashSet::new();
        for (key, values) in order.value {
            let mut node_changes = std::collections::HashSet::new();
            let mut v = vec![];
            node_changes.extend(var_dependencies.get_value(key.as_str()));
            for val in values.iter() {
                v.push(format!(
                    indoc::indoc! {"
                        if(!!window[\"resolve_value_{id}\"] && !!window[\"resolve_value_{id}\"][\"{variable_name}\"]){{\
                            window[\"resolve_value_{id}\"][\"{variable_name}\"](data);
                        }} else {{
                            let value = resolve_reference(\"{set_variable_name}\", data, null);
                            set_data_value(data, \"{variable_name}\", value);
                        }}"
                    },
                    id = self.id,
                    variable_name = val,
                    set_variable_name = key,
                ));
                node_changes.extend(var_dependencies.get_value(val));
            }

            let mut node_changes_calls = {
                let mut node_changes_calls = vec![];
                for key in node_changes.iter() {
                    node_changes_calls.push(format!(
                        indoc::indoc! {"
                            window.ftd.utils.node_change_call(\"{id}\",\"{key}\", data);\
                            "
                        },
                        id = self.id,
                        key = key,
                    ))
                }
                node_changes_calls
            };

            if cfg!(test) || test {
                node_changes_calls.sort();
            }

            result_2.extend(values);
            if !v.is_empty() || !node_changes_calls.is_empty() {
                result_1.push(format!(
                    indoc::indoc! {"
                     window.set_value_{id}[\"{key}\"] = function (data, new_value, remaining) {{
                            window.ftd.utils.set_value_helper(data, \"{key}\", remaining, new_value);
                            {dependencies}
                            window.ftd.call_mutable_value_changes(\"{key}\", \"{id}\");
                            window.ftd.call_immutable_value_changes(\"{key}\", \"{id}\");
                            {node_changes_calls}
                     }};
                "},
                    id = self.id,
                    key = ftd::html::utils::js_reference_name(key.as_str()),
                    dependencies = v.join("\n"),
                    node_changes_calls = node_changes_calls.join("\n"),
                ));
            }
        }

        (result_1.join("\n"), result_2.into_iter().collect())
    }

    fn js_resolve_function(&self, key: &str) -> ftd::html::Result<String> {
        let variable = match self.doc.get_variable(key, 0) {
            Ok(variable) if !variable.conditional_value.is_empty() => variable,
            _ => return Ok("".to_string()),
        };

        let mut expressions = vec![];
        for condition in variable.conditional_value {
            let condition_str =
                ftd::html::utils::get_condition_string_(&condition.condition, false);
            if let Some(value_string) =
                ftd::html::utils::get_formatted_dep_string_from_property_value(
                    self.id,
                    self.doc,
                    &condition.value,
                    &None,
                    None,
                    false,
                )?
            {
                let value = format!(
                    "set_data_value(data, \"{}\", {});",
                    variable.name, value_string
                );
                expressions.push((Some(condition_str), value));
            }
        }

        if let Some(value_string) = ftd::html::utils::get_formatted_dep_string_from_property_value(
            self.id,
            self.doc,
            &variable.value,
            &None,
            None,
            false,
        )? {
            let value = format!(
                "set_data_value(data, \"{}\", {});",
                variable.name, value_string
            );
            expressions.push((None, value));
        }
        let value = ftd::html::utils::js_expression_from_list(expressions, None, "");
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
