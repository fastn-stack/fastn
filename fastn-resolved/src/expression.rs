#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Expression {
    pub expression: fastn_resolved::evalexpr::ExprNode,
    pub references: fastn_resolved::Map<fastn_resolved::PropertyValue>,
    pub line_number: usize,
}

impl Expression {
    pub fn new(
        expression: fastn_resolved::evalexpr::ExprNode,
        references: fastn_resolved::Map<fastn_resolved::PropertyValue>,
        line_number: usize,
    ) -> Expression {
        Expression {
            expression,
            references,
            line_number,
        }
    }
}
