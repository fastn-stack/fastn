#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Expression {
    pub expression: fastn_grammar::evalexpr::ExprNode,
    pub references: fastn_type::Map<fastn_type::PropertyValue>,
    pub line_number: usize,
}

impl Expression {
    pub fn new(
        expression: fastn_grammar::evalexpr::ExprNode,
        references: fastn_type::Map<fastn_type::PropertyValue>,
        line_number: usize,
    ) -> Expression {
        Expression {
            expression,
            references,
            line_number,
        }
    }
}
