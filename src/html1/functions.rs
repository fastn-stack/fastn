pub struct FunctionGenerator {
    id: String,
}

impl FunctionGenerator {
    pub fn new(id: &str) -> FunctionGenerator {
        FunctionGenerator { id: id.to_string() }
    }

    pub fn get_functions(&self, node_data: &ftd::node::NodeData) -> ftd::html1::Result<String> {
        let mut vector = vec![];
        vector.extend(from_default_functions());
        for function in node_data
            .bag
            .values()
            .filter_map(|v| v.to_owned().function(node_data.name.as_str(), 0).ok())
        {
            vector.push(self.get_function(function)?)
        }

        Ok(vector.join("\n\n"))
    }

    pub fn get_function(
        &self,
        function: ftd::interpreter2::Function,
    ) -> ftd::html1::Result<String> {
        use itertools::Itertools;

        /*let node = dbg!(ftd::evalexpr::build_operator_tree(
            "a = a+b+f(a, b)+(j, k) + (a+b + g(a+j, k)); a"
        )
        .unwrap()); //Todo: remove unwrap
        dbg!(to_string(&node, true, &[]).as_str(),);*/

        let mut result = vec![];
        let arguments = function
            .arguments
            .iter()
            .map(|v| (v.name.to_string(), v.mutable))
            .collect_vec();
        for expression in function.expression {
            let node = ftd::evalexpr::build_operator_tree(expression.expression.as_str())?;
            result.push(ftd::html1::utils::trim_brackets(
                ExpressionGenerator
                    .to_string(&node, true, arguments.as_slice())
                    .as_str(),
            ));
        }
        let expressions = result.join("\n");
        let function_name = ftd::html1::utils::function_name_to_js_function(
            ftd::html1::utils::name_with_id(function.name.as_str(), self.id.as_str()).as_str(),
        );

        let mut arguments = arguments.iter().map(|(k, _)| k).join(",");

        if !arguments.is_empty() {
            arguments = format!("{},args,data,id", arguments);
        } else {
            arguments = "args,data,id".to_string();
        }

        Ok(format!(
            indoc::indoc! {"
                function {function_name}({arguments}){{
                    {expressions}
                }}

            "},
            function_name = function_name,
            arguments = arguments,
            expressions = expressions
        ))
    }
}

pub struct ExpressionGenerator;

impl ExpressionGenerator {
    pub fn to_string(
        &self,
        node: &ftd::evalexpr::ExprNode,
        root: bool,
        arguments: &[(String, bool)],
    ) -> String {
        use itertools::Itertools;

        if self.is_root(node.operator()) {
            let result = node
                .children()
                .iter()
                .map(|children| self.to_string(children, false, arguments))
                .collect_vec();
            let (is_assignment_or_chain, only_one_child) =
                node.children().first().map_or((false, true), |first| {
                    /*has_operator(dbg!(&first.operator())).is_none()*/
                    let is_assignment_or_chain =
                        self.is_assignment(first.operator()) || self.is_chain(first.operator());
                    (
                        is_assignment_or_chain,
                        is_assignment_or_chain
                            || self.has_value(first.operator()).is_some()
                            || self.is_tuple(first.operator()),
                    )
                });
            let f = if !only_one_child {
                format!("({})", result.join(""))
            } else {
                result.join("")
            };

            return if root && !is_assignment_or_chain && !f.is_empty() {
                format!("return {};", f)
            } else {
                f
            };
        }

        if self.is_chain(node.operator()) {
            let mut result = vec![];
            for children in node.children() {
                let val = ftd::html1::utils::trim_brackets(
                    self.to_string(children, true, arguments).trim(),
                );
                if !val.trim().is_empty() {
                    result.push(format!(
                        "{}{}",
                        val,
                        if val.ends_with(';') { "" } else { ";" }
                    ));
                }
            }
            return result.join("\n");
        }

        if self.is_tuple(node.operator()) {
            let mut result = vec![];
            for children in node.children() {
                result.push(self.to_string(children, false, arguments));
            }
            return format!("[{}]", result.join(","));
        }

        if let Some(function_name) = self.function_name(node.operator()) {
            let mut result = vec![];
            if let Some(child) = node.children().first() {
                for children in child.children() {
                    let mut value = self.to_string(children, false, arguments);
                    if self.is_tuple(children.operator()) {
                        value = value[1..value.len() - 1].to_string();
                    }
                    result.push(value);
                }
            }
            result.extend(["args".to_string(), "data".to_string(), "id".to_string()]);
            return format!("{}({})", function_name, result.join(","));
        }

        if self.is_assignment(node.operator()) {
            // Todo: if node.children().len() != 2 {throw error}
            let first = node.children().first().unwrap(); //todo remove unwrap()
            let second = node.children().get(1).unwrap(); //todo remove unwrap()
            let prefix = if !arguments.iter().any(|(v, _)| first.to_string().eq(v)) {
                "let "
            } else {
                ""
            };
            return vec![
                prefix.to_string(),
                self.to_string(first, false, arguments),
                node.operator().to_string(),
                self.to_string(second, false, arguments),
            ]
            .join("");
        }

        if let Some(operator) = self.has_operator(node.operator()) {
            // Todo: if node.children().len() != 2 {throw error}
            let first = node.children().first().unwrap(); //todo remove unwrap()
            if matches!(node.operator(), ftd::evalexpr::Operator::Not)
                || matches!(node.operator(), ftd::evalexpr::Operator::Neg)
            {
                return vec![operator, self.to_string(first, false, arguments)].join("");
            }
            let second = node.children().get(1).unwrap(); //todo remove unwrap()
            return vec![
                self.to_string(first, false, arguments),
                operator,
                self.to_string(second, false, arguments),
            ]
            .join("");
        }

        if let Some(operator) = self.has_function(node.operator()) {
            let mut result = vec![];
            for children in node.children() {
                result.push(self.to_string(children, false, arguments));
            }
            return format!("{}{}", operator.trim(), result.join(" "));
        }

        let value = if self.is_null(node.operator()) {
            "null".to_string()
        } else {
            node.operator().to_string()
        };

        format!(
            "{}{}",
            value,
            if arguments.iter().any(|(v, mutable)| value.eq(v) && *mutable) {
                ".value"
            } else {
                ""
            }
        )
    }

    pub fn has_value(&self, operator: &ftd::evalexpr::Operator) -> Option<String> {
        match operator {
            ftd::evalexpr::Operator::Const { .. }
            | ftd::evalexpr::Operator::VariableIdentifierRead { .. }
            | ftd::evalexpr::Operator::VariableIdentifierWrite { .. } => Some(operator.to_string()),
            _ => None,
        }
    }

    pub fn has_function(&self, operator: &ftd::evalexpr::Operator) -> Option<String> {
        match operator {
            ftd::evalexpr::Operator::FunctionIdentifier { .. } => Some(operator.to_string()),
            _ => None,
        }
    }

    pub fn is_assignment(&self, operator: &ftd::evalexpr::Operator) -> bool {
        matches!(operator, ftd::evalexpr::Operator::Assign)
    }

    pub fn is_chain(&self, operator: &ftd::evalexpr::Operator) -> bool {
        matches!(operator, ftd::evalexpr::Operator::Chain)
    }

    pub fn is_tuple(&self, operator: &ftd::evalexpr::Operator) -> bool {
        matches!(operator, ftd::evalexpr::Operator::Tuple)
    }

    pub fn is_null(&self, operator: &ftd::evalexpr::Operator) -> bool {
        matches!(
            operator,
            ftd::evalexpr::Operator::Const {
                value: ftd::evalexpr::Value::Empty,
            }
        )
    }

    pub fn function_name(&self, operator: &ftd::evalexpr::Operator) -> Option<String> {
        if let ftd::evalexpr::Operator::FunctionIdentifier { identifier } = operator {
            Some(identifier.to_string())
        } else {
            None
        }
    }

    pub fn has_operator(&self, operator: &ftd::evalexpr::Operator) -> Option<String> {
        if self.has_value(operator).is_none()
            && self.has_function(operator).is_none()
            && !self.is_chain(operator)
            && !self.is_root(operator)
            && !self.is_tuple(operator)
            && !self.is_assignment(operator)
        {
            Some(operator.to_string())
        } else {
            None
        }
    }

    pub fn is_root(&self, operator: &ftd::evalexpr::Operator) -> bool {
        matches!(operator, ftd::evalexpr::Operator::RootNode)
    }
}

fn from_default_functions() -> Vec<String> {
    // todo: check ftd::interpreter2::default::default_functions()
    vec!["".to_string()]
}
