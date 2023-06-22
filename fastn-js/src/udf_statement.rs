pub enum UDFStatement {
    VariableDeclaration {
        name: String,
        value: Box<UDFStatement>,
    },
    VariableAssignment {
        name: String,
        value: Box<UDFStatement>,
    },
    Addition {
        left: Box<UDFStatement>,
        right: Box<UDFStatement>,
    },
    Subtraction {
        left: Box<UDFStatement>,
        right: Box<UDFStatement>,
    },
    Multiplication {
        left: Box<UDFStatement>,
        right: Box<UDFStatement>,
    },
    Division {
        left: Box<UDFStatement>,
        right: Box<UDFStatement>,
    },
    Exponentiation {
        left: Box<UDFStatement>,
        right: Box<UDFStatement>,
    },
    Not {
        value: Box<UDFStatement>,
    },
    And {
        left: Box<UDFStatement>,
        right: Box<UDFStatement>,
    },
    Or {
        left: Box<UDFStatement>,
        right: Box<UDFStatement>,
    },
    Parens {
        value: Box<UDFStatement>,
    },
    Variable {
        name: String,
    },
    Integer {
        value: i64,
    },
    Decimal {
        value: f64,
    },
    Boolean {
        value: bool,
    },
    String {
        value: String,
    },
    Return {
        value: Box<UDFStatement>,
    },
    If {
        condition: Box<UDFStatement>,
        then: Box<UDFStatement>,
        otherwise: Box<UDFStatement>,
    },
    Block {
        statements: Vec<UDFStatement>,
    },
    Call {
        name: String,
        args: Vec<UDFStatement>,
    },
}
