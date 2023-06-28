pub fn is_kernel(s: &str) -> bool {
    ["ftd#text", "ftd#row", "ftd#column"].contains(&s)
}

pub fn name_to_js(s: &str) -> String {
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

pub struct ExpressionGenerator;

impl ExpressionGenerator {
    pub fn to_js(&self, node: &fastn_grammar::evalexpr::ExprNode) -> String {
        self.to_js_(node, true, &[])
    }

    pub fn to_js_(
        &self,
        node: &fastn_grammar::evalexpr::ExprNode,
        root: bool,
        arguments: &[(String, bool)],
    ) -> String {
        use itertools::Itertools;

        if self.is_root(node.operator()) {
            let result = node
                .children()
                .iter()
                .map(|children| self.to_js_(children, false, arguments))
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
                let val =
                    fastn_js::utils::trim_brackets(self.to_js_(children, true, arguments).trim());
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
                result.push(self.to_js_(children, false, arguments));
            }
            return format!("[{}]", result.join(","));
        }

        if let Some(function_name) = self.function_name(node.operator()) {
            let mut result = vec![];
            if let Some(child) = node.children().first() {
                for children in child.children() {
                    let mut value = self.to_js_(children, false, arguments);
                    if self.is_tuple(children.operator()) {
                        value = value[1..value.len() - 1].to_string();
                    }
                    result.push(value);
                }
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
            return vec![
                prefix.to_string(),
                self.to_js_(first, false, arguments),
                node.operator().to_string(),
                self.to_js_(second, false, arguments),
            ]
            .join("");
        }

        if let Some(operator) = self.has_operator(node.operator()) {
            // Todo: if node.children().len() != 2 {throw error}
            let first = node.children().first().unwrap(); //todo remove unwrap()
            if matches!(node.operator(), fastn_grammar::evalexpr::Operator::Not)
                || matches!(node.operator(), fastn_grammar::evalexpr::Operator::Neg)
            {
                return vec![operator, self.to_js_(first, false, arguments)].join("");
            }
            let second = node.children().get(1).unwrap(); //todo remove unwrap()
            return vec![
                self.to_js_(first, false, arguments),
                operator,
                self.to_js_(second, false, arguments),
            ]
            .join("");
        }

        if let Some(operator) = self.has_function(node.operator()) {
            let mut result = vec![];
            for children in node.children() {
                result.push(self.to_js_(children, false, arguments));
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

pub fn trim_brackets(s: &str) -> String {
    if s.starts_with('(') && s.ends_with(')') {
        return s[1..s.len() - 1].to_string();
    }
    s.to_string()
}
