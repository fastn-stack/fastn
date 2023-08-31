pub struct FunctionGenerator {
    id: String,
}

impl FunctionGenerator {
    pub fn new(id: &str) -> FunctionGenerator {
        FunctionGenerator { id: id.to_string() }
    }

    pub fn get_functions(&self, node_data: &ftd::node::NodeData) -> ftd::html::Result<String> {
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

    pub fn get_function(&self, function: ftd::interpreter::Function) -> ftd::html::Result<String> {
        use itertools::Itertools;

        /*let node = dbg!(fastn_grammar::evalexpr::build_operator_tree(
            "a = a+b+f(a, b)+(j, k) + (a+b + g(a+j, k)); a"
        )
        .unwrap()); //TODO: remove unwrap
        dbg!(to_string(&node, true, &[]).as_str(),);*/

        let mut result = vec![];
        let arguments = function
            .arguments
            .iter()
            .map(|v| (v.name.to_string(), v.mutable))
            .collect_vec();
        for expression in function.expression {
            let node =
                fastn_grammar::evalexpr::build_operator_tree(expression.expression.as_str())?;
            result.push(ftd::html::utils::trim_brackets(
                ExpressionGenerator
                    .to_string(&node, true, arguments.as_slice())
                    .as_str(),
            ));
        }
        let expressions = result.join("\n");
        let function_name = ftd::html::utils::function_name_to_js_function(
            ftd::html::utils::name_with_id(function.name.as_str(), self.id.as_str()).as_str(),
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
        node: &fastn_grammar::evalexpr::ExprNode,
        root: bool,
        arguments: &[(String, bool)],
    ) -> String {
        self.to_string_(node, root, arguments, true)
    }

    pub fn to_string_(
        &self,
        node: &fastn_grammar::evalexpr::ExprNode,
        root: bool,
        arguments: &[(String, bool)],
        extra_args: bool,
    ) -> String {
        use itertools::Itertools;

        if self.is_root(node.operator()) {
            let result = node
                .children()
                .iter()
                .map(|children| self.to_string_(children, false, arguments, extra_args))
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
                let val = ftd::html::utils::trim_brackets(
                    self.to_string_(children, true, arguments, extra_args)
                        .trim(),
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
                result.push(self.to_string_(children, false, arguments, extra_args));
            }
            return format!("[{}]", result.join(","));
        }

        if let Some(function_name) = self.function_name(node.operator()) {
            let mut result = vec![];
            if let Some(child) = node.children().first() {
                for children in child.children() {
                    let mut value = self.to_string_(children, false, arguments, extra_args);
                    if self.is_tuple(children.operator()) {
                        value = value[1..value.len() - 1].to_string();
                    }
                    result.push(value);
                }
            }
            if extra_args {
                result.extend(["args".to_string(), "data".to_string(), "id".to_string()]);
            }
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
            return [
                prefix.to_string(),
                self.to_string_(first, false, arguments, extra_args),
                node.operator().to_string(),
                self.to_string_(second, false, arguments, extra_args),
            ]
            .join("");
        }

        if let Some(operator) = self.has_operator(node.operator()) {
            // Todo: if node.children().len() != 2 {throw error}
            let first = node.children().first().unwrap(); //todo remove unwrap()
            if matches!(node.operator(), fastn_grammar::evalexpr::Operator::Not)
                || matches!(node.operator(), fastn_grammar::evalexpr::Operator::Neg)
            {
                return [
                    operator,
                    self.to_string_(first, false, arguments, extra_args),
                ]
                .join("");
            }
            let second = node.children().get(1).unwrap(); //todo remove unwrap()
            return [
                self.to_string_(first, false, arguments, extra_args),
                operator,
                self.to_string_(second, false, arguments, extra_args),
            ]
            .join("");
        }

        if let Some(operator) = self.has_function(node.operator()) {
            let mut result = vec![];
            for children in node.children() {
                result.push(self.to_string_(children, false, arguments, extra_args));
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

    pub fn has_value(&self, operator: &fastn_grammar::evalexpr::Operator) -> Option<String> {
        match operator {
            fastn_grammar::evalexpr::Operator::Const { .. }
            | fastn_grammar::evalexpr::Operator::VariableIdentifierRead { .. }
            | fastn_grammar::evalexpr::Operator::VariableIdentifierWrite { .. } => {
                Some(operator.to_string())
            }
            _ => None,
        }
    }

    pub fn has_function(&self, operator: &fastn_grammar::evalexpr::Operator) -> Option<String> {
        match operator {
            fastn_grammar::evalexpr::Operator::FunctionIdentifier { .. } => {
                Some(operator.to_string())
            }
            _ => None,
        }
    }

    pub fn is_assignment(&self, operator: &fastn_grammar::evalexpr::Operator) -> bool {
        matches!(operator, fastn_grammar::evalexpr::Operator::Assign)
    }

    pub fn is_chain(&self, operator: &fastn_grammar::evalexpr::Operator) -> bool {
        matches!(operator, fastn_grammar::evalexpr::Operator::Chain)
    }

    pub fn is_tuple(&self, operator: &fastn_grammar::evalexpr::Operator) -> bool {
        matches!(operator, fastn_grammar::evalexpr::Operator::Tuple)
    }

    pub fn is_null(&self, operator: &fastn_grammar::evalexpr::Operator) -> bool {
        matches!(
            operator,
            fastn_grammar::evalexpr::Operator::Const {
                value: fastn_grammar::evalexpr::Value::Empty,
            }
        )
    }

    pub fn function_name(&self, operator: &fastn_grammar::evalexpr::Operator) -> Option<String> {
        if let fastn_grammar::evalexpr::Operator::FunctionIdentifier { identifier } = operator {
            Some(identifier.to_string())
        } else {
            None
        }
    }

    pub fn has_operator(&self, operator: &fastn_grammar::evalexpr::Operator) -> Option<String> {
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

    pub fn is_root(&self, operator: &fastn_grammar::evalexpr::Operator) -> bool {
        matches!(operator, fastn_grammar::evalexpr::Operator::RootNode)
    }
}

fn from_default_functions() -> Vec<String> {
    // todo: check ftd::interpreter::default::default_functions()
    vec!["".to_string()]
}
