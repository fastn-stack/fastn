use fastn_resolved::evalexpr::function::builtin::builtin_function;

use fastn_resolved::evalexpr::{
    context::Context, error::*, value::Value, ContextWithMutableVariables,
};

mod display;

/// An enum that represents operators in the operator tree.
#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub enum Operator {
    /// A root node in the operator tree.
    /// The whole expression is stored under a root node, as well as each subexpression surrounded by parentheses.
    RootNode,

    /// A binary addition operator.
    Add,
    /// A binary subtraction operator.
    Sub,
    /// A unary negation operator.
    Neg,
    /// A binary multiplication operator.
    Mul,
    /// A binary division operator.
    Div,
    /// A binary modulo operator.
    Mod,
    /// A binary exponentiation operator.
    Exp,

    /// A binary equality comparator.
    Eq,
    /// A binary inequality comparator.
    Neq,
    /// A binary greater-than comparator.
    Gt,
    /// A binary lower-than comparator.
    Lt,
    /// A binary greater-than-or-equal comparator.
    Geq,
    /// A binary lower-than-or-equal comparator.
    Leq,
    /// A binary logical and operator.
    And,
    /// A binary logical or operator.
    Or,
    /// A binary logical not operator.
    Not,

    /// A binary assignment operator.
    Assign,
    /// A binary add-assign operator.
    AddAssign,
    /// A binary subtract-assign operator.
    SubAssign,
    /// A binary multiply-assign operator.
    MulAssign,
    /// A binary divide-assign operator.
    DivAssign,
    /// A binary modulo-assign operator.
    ModAssign,
    /// A binary exponentiate-assign operator.
    ExpAssign,
    /// A binary and-assign operator.
    AndAssign,
    /// A binary or-assign operator.
    OrAssign,

    /// An n-ary tuple constructor.
    Tuple,
    /// An n-ary subexpression chain.
    Chain,

    /// A constant value.
    Const {
        /** The value of the constant. */
        value: Value,
    },
    /// A write to a variable identifier.
    // VariableIdentifierDefinition {
    //     /// The identifier of the variable.
    //     identifier: String,
    // },
    /// A write to a variable identifier.
    VariableIdentifierWrite {
        /// The identifier of the variable.
        identifier: String,
    },
    /// A read from a variable identifier.
    VariableIdentifierRead {
        /// The identifier of the variable.
        identifier: String,
    },
    /// A function identifier.
    FunctionIdentifier {
        /// The identifier of the function.
        identifier: String,
    },
}

impl Operator {
    pub(crate) fn value(value: Value) -> Self {
        Operator::Const { value }
    }

    pub(crate) fn variable_identifier_write(identifier: String) -> Self {
        Operator::VariableIdentifierWrite { identifier }
    }

    pub(crate) fn variable_identifier_read(identifier: String) -> Self {
        Operator::VariableIdentifierRead { identifier }
    }

    pub(crate) fn function_identifier(identifier: String) -> Self {
        Operator::FunctionIdentifier { identifier }
    }

    /// Returns the precedence of the operator.
    /// A high precedence means that the operator has priority to be deeper in the tree.
    pub(crate) const fn precedence(&self) -> i32 {
        use fastn_resolved::evalexpr::operator::Operator::*;
        match self {
            RootNode => 200,

            Add | Sub => 95,
            Neg => 110,
            Mul | Div | Mod => 100,
            Exp => 120,

            Eq | Neq | Gt | Lt | Geq | Leq => 80,
            And => 75,
            Or => 70,
            Not => 110,

            Assign | AddAssign | SubAssign | MulAssign | DivAssign | ModAssign | ExpAssign
            | AndAssign | OrAssign => 50,

            Tuple => 40,
            Chain => 0,

            Const { .. } => 200,
            VariableIdentifierWrite { .. } | VariableIdentifierRead { .. } => 200,
            FunctionIdentifier { .. } => 190,
        }
    }

    /// Returns true if chains of operators with the same precedence as this one should be evaluated left-to-right,
    /// and false if they should be evaluated right-to-left.
    /// Left-to-right chaining has priority if operators with different order but same precedence are chained.
    pub(crate) const fn is_left_to_right(&self) -> bool {
        use fastn_resolved::evalexpr::operator::Operator::*;
        !matches!(self, Assign | FunctionIdentifier { .. })
    }

    /// Returns true if chains of this operator should be flattened into one operator with many arguments.
    pub(crate) const fn is_sequence(&self) -> bool {
        use fastn_resolved::evalexpr::operator::Operator::*;
        matches!(self, Tuple | Chain)
    }

    /// True if this operator is a leaf, meaning it accepts no arguments.
    // Make this a const fn as soon as whatever is missing gets stable (issue #57563)
    pub(crate) fn is_leaf(&self) -> bool {
        self.max_argument_amount() == Some(0)
    }

    /// Returns the maximum amount of arguments required by this operator.
    pub(crate) const fn max_argument_amount(&self) -> Option<usize> {
        use fastn_resolved::evalexpr::operator::Operator::*;
        match self {
            Add | Sub | Mul | Div | Mod | Exp | Eq | Neq | Gt | Lt | Geq | Leq | And | Or
            | Assign | AddAssign | SubAssign | MulAssign | DivAssign | ModAssign | ExpAssign
            | AndAssign | OrAssign => Some(2),
            Tuple | Chain => None,
            Not | Neg | RootNode => Some(1),
            Const { .. } => Some(0),
            VariableIdentifierWrite { .. } | VariableIdentifierRead { .. } => Some(0),
            FunctionIdentifier { .. } => Some(1),
        }
    }

    /// Evaluates the operator with the given arguments and context.
    pub(crate) fn eval<C: Context>(
        &self,
        arguments: &[Value],
        context: &C,
    ) -> EvalexprResult<Value> {
        use fastn_resolved::evalexpr::operator::Operator::*;
        match self {
            RootNode => {
                if let Some(first) = arguments.first() {
                    Ok(first.clone())
                } else {
                    Ok(Value::Empty)
                }
            }
            Add => {
                expect_operator_argument_amount(arguments.len(), 2)?;
                expect_number_or_string(&arguments[0])?;
                expect_number_or_string(&arguments[1])?;

                if let (Ok(a), Ok(b)) = (arguments[0].as_string(), arguments[1].as_string()) {
                    let mut result = String::with_capacity(a.len() + b.len());
                    result.push_str(&a);
                    result.push_str(&b);
                    Ok(Value::String(result))
                } else if let (Ok(a), Ok(b)) = (arguments[0].as_int(), arguments[1].as_int()) {
                    let result = a.checked_add(b);
                    if let Some(result) = result {
                        Ok(Value::Int(result))
                    } else {
                        Err(EvalexprError::addition_error(
                            arguments[0].clone(),
                            arguments[1].clone(),
                        ))
                    }
                } else if let (Ok(a), Ok(b)) = (arguments[0].as_number(), arguments[1].as_number())
                {
                    Ok(Value::Float(a + b))
                } else if let (Ok(a), Ok(b)) = (arguments[0].as_string(), arguments[1].as_number())
                {
                    let b = format!("{}", b);
                    let mut result = String::with_capacity(a.len() + b.len());
                    result.push_str(&a);
                    result.push_str(&b);
                    Ok(Value::String(result))
                } else if let (Ok(a), Ok(b)) = (arguments[0].as_number(), arguments[1].as_string())
                {
                    let a = format!("{}", a);
                    let mut result = String::with_capacity(a.len() + b.len());
                    result.push_str(&a);
                    result.push_str(&b);
                    Ok(Value::String(result))
                } else {
                    Err(EvalexprError::wrong_type_combination(
                        self.clone(),
                        vec![(&arguments[0]).into(), (&arguments[1]).into()],
                    ))
                }
            }
            Sub => {
                expect_operator_argument_amount(arguments.len(), 2)?;
                arguments[0].as_number()?;
                arguments[1].as_number()?;

                if let (Ok(a), Ok(b)) = (arguments[0].as_int(), arguments[1].as_int()) {
                    let result = a.checked_sub(b);
                    if let Some(result) = result {
                        Ok(Value::Int(result))
                    } else {
                        Err(EvalexprError::subtraction_error(
                            arguments[0].clone(),
                            arguments[1].clone(),
                        ))
                    }
                } else {
                    Ok(Value::Float(
                        arguments[0].as_number()? - arguments[1].as_number()?,
                    ))
                }
            }
            Neg => {
                expect_operator_argument_amount(arguments.len(), 1)?;
                arguments[0].as_number()?;

                if let Ok(a) = arguments[0].as_int() {
                    let result = a.checked_neg();
                    if let Some(result) = result {
                        Ok(Value::Int(result))
                    } else {
                        Err(EvalexprError::negation_error(arguments[0].clone()))
                    }
                } else {
                    Ok(Value::Float(-arguments[0].as_number()?))
                }
            }
            Mul => {
                expect_operator_argument_amount(arguments.len(), 2)?;
                arguments[0].as_number()?;
                arguments[1].as_number()?;

                if let (Ok(a), Ok(b)) = (arguments[0].as_int(), arguments[1].as_int()) {
                    let result = a.checked_mul(b);
                    if let Some(result) = result {
                        Ok(Value::Int(result))
                    } else {
                        Err(EvalexprError::multiplication_error(
                            arguments[0].clone(),
                            arguments[1].clone(),
                        ))
                    }
                } else {
                    Ok(Value::Float(
                        arguments[0].as_number()? * arguments[1].as_number()?,
                    ))
                }
            }
            Div => {
                expect_operator_argument_amount(arguments.len(), 2)?;
                arguments[0].as_number()?;
                arguments[1].as_number()?;

                if let (Ok(a), Ok(b)) = (arguments[0].as_int(), arguments[1].as_int()) {
                    let result = a.checked_div(b);
                    if let Some(result) = result {
                        Ok(Value::Int(result))
                    } else {
                        Err(EvalexprError::division_error(
                            arguments[0].clone(),
                            arguments[1].clone(),
                        ))
                    }
                } else {
                    Ok(Value::Float(
                        arguments[0].as_number()? / arguments[1].as_number()?,
                    ))
                }
            }
            Mod => {
                expect_operator_argument_amount(arguments.len(), 2)?;
                arguments[0].as_number()?;
                arguments[1].as_number()?;

                if let (Ok(a), Ok(b)) = (arguments[0].as_int(), arguments[1].as_int()) {
                    let result = a.checked_rem(b);
                    if let Some(result) = result {
                        Ok(Value::Int(result))
                    } else {
                        Err(EvalexprError::modulation_error(
                            arguments[0].clone(),
                            arguments[1].clone(),
                        ))
                    }
                } else {
                    Ok(Value::Float(
                        arguments[0].as_number()? % arguments[1].as_number()?,
                    ))
                }
            }
            Exp => {
                expect_operator_argument_amount(arguments.len(), 2)?;
                arguments[0].as_number()?;
                arguments[1].as_number()?;

                Ok(Value::Float(
                    arguments[0].as_number()?.powf(arguments[1].as_number()?),
                ))
            }
            Eq => {
                expect_operator_argument_amount(arguments.len(), 2)?;

                Ok(Value::Boolean(arguments[0] == arguments[1]))
            }
            Neq => {
                expect_operator_argument_amount(arguments.len(), 2)?;

                Ok(Value::Boolean(arguments[0] != arguments[1]))
            }
            Gt => {
                expect_operator_argument_amount(arguments.len(), 2)?;
                expect_number_or_string(&arguments[0])?;
                expect_number_or_string(&arguments[1])?;

                if let (Ok(a), Ok(b)) = (arguments[0].as_string(), arguments[1].as_string()) {
                    Ok(Value::Boolean(a > b))
                } else if let (Ok(a), Ok(b)) = (arguments[0].as_int(), arguments[1].as_int()) {
                    Ok(Value::Boolean(a > b))
                } else {
                    Ok(Value::Boolean(
                        arguments[0].as_number()? > arguments[1].as_number()?,
                    ))
                }
            }
            Lt => {
                expect_operator_argument_amount(arguments.len(), 2)?;
                expect_number_or_string(&arguments[0])?;
                expect_number_or_string(&arguments[1])?;

                if let (Ok(a), Ok(b)) = (arguments[0].as_string(), arguments[1].as_string()) {
                    Ok(Value::Boolean(a < b))
                } else if let (Ok(a), Ok(b)) = (arguments[0].as_int(), arguments[1].as_int()) {
                    Ok(Value::Boolean(a < b))
                } else {
                    Ok(Value::Boolean(
                        arguments[0].as_number()? < arguments[1].as_number()?,
                    ))
                }
            }
            Geq => {
                expect_operator_argument_amount(arguments.len(), 2)?;
                expect_number_or_string(&arguments[0])?;
                expect_number_or_string(&arguments[1])?;

                if let (Ok(a), Ok(b)) = (arguments[0].as_string(), arguments[1].as_string()) {
                    Ok(Value::Boolean(a >= b))
                } else if let (Ok(a), Ok(b)) = (arguments[0].as_int(), arguments[1].as_int()) {
                    Ok(Value::Boolean(a >= b))
                } else {
                    Ok(Value::Boolean(
                        arguments[0].as_number()? >= arguments[1].as_number()?,
                    ))
                }
            }
            Leq => {
                expect_operator_argument_amount(arguments.len(), 2)?;
                expect_number_or_string(&arguments[0])?;
                expect_number_or_string(&arguments[1])?;

                if let (Ok(a), Ok(b)) = (arguments[0].as_string(), arguments[1].as_string()) {
                    Ok(Value::Boolean(a <= b))
                } else if let (Ok(a), Ok(b)) = (arguments[0].as_int(), arguments[1].as_int()) {
                    Ok(Value::Boolean(a <= b))
                } else {
                    Ok(Value::Boolean(
                        arguments[0].as_number()? <= arguments[1].as_number()?,
                    ))
                }
            }
            And => {
                expect_operator_argument_amount(arguments.len(), 2)?;
                let a = arguments[0].as_boolean()?;
                let b = arguments[1].as_boolean()?;

                Ok(Value::Boolean(a && b))
            }
            Or => {
                expect_operator_argument_amount(arguments.len(), 2)?;
                let a = arguments[0].as_boolean()?;
                let b = arguments[1].as_boolean()?;

                Ok(Value::Boolean(a || b))
            }
            Not => {
                expect_operator_argument_amount(arguments.len(), 1)?;
                let a = arguments[0].as_boolean()?;

                Ok(Value::Boolean(!a))
            }
            Assign | AddAssign | SubAssign | MulAssign | DivAssign | ModAssign | ExpAssign
            | AndAssign | OrAssign => Err(EvalexprError::ContextNotMutable),
            Tuple => Ok(Value::Tuple(arguments.into())),
            Chain => {
                if arguments.is_empty() {
                    return Err(EvalexprError::wrong_operator_argument_amount(0, 1));
                }

                Ok(arguments.last().cloned().unwrap_or(Value::Empty))
            }
            Const { value } => {
                expect_operator_argument_amount(arguments.len(), 0)?;

                Ok(value.clone())
            }
            VariableIdentifierWrite { identifier } => {
                expect_operator_argument_amount(arguments.len(), 0)?;

                Ok(identifier.clone().into())
            }
            VariableIdentifierRead { identifier } => {
                expect_operator_argument_amount(arguments.len(), 0)?;

                if let Some(value) = context.get_value(identifier).cloned() {
                    Ok(value)
                } else {
                    Err(EvalexprError::VariableIdentifierNotFound(
                        identifier.clone(),
                    ))
                }
            }
            FunctionIdentifier { identifier } => {
                expect_operator_argument_amount(arguments.len(), 1)?;
                let arguments = &arguments[0];

                match context.call_function(identifier, arguments) {
                    Err(EvalexprError::FunctionIdentifierNotFound(_)) => {
                        if let Some(builtin_function) = builtin_function(identifier) {
                            builtin_function.call(arguments)
                        } else {
                            Err(EvalexprError::FunctionIdentifierNotFound(
                                identifier.clone(),
                            ))
                        }
                    }
                    result => result,
                }
            }
        }
    }

    /// Evaluates the operator with the given arguments and mutable context.
    pub(crate) fn eval_mut<C: ContextWithMutableVariables>(
        &self,
        arguments: &[Value],
        context: &mut C,
    ) -> EvalexprResult<Value> {
        use fastn_resolved::evalexpr::operator::Operator::*;
        match self {
            Assign => {
                expect_operator_argument_amount(arguments.len(), 2)?;
                let target = arguments[0].as_string()?;
                context.set_value(target, arguments[1].clone())?;

                Ok(Value::Empty)
            }
            AddAssign | SubAssign | MulAssign | DivAssign | ModAssign | ExpAssign | AndAssign
            | OrAssign => {
                expect_operator_argument_amount(arguments.len(), 2)?;

                let target = arguments[0].as_string()?;
                let left_value = Operator::VariableIdentifierRead {
                    identifier: target.clone(),
                }
                .eval(&Vec::new(), context)?;
                let arguments = vec![left_value, arguments[1].clone()];

                let result = match self {
                    AddAssign => Operator::Add.eval(&arguments, context),
                    SubAssign => Operator::Sub.eval(&arguments, context),
                    MulAssign => Operator::Mul.eval(&arguments, context),
                    DivAssign => Operator::Div.eval(&arguments, context),
                    ModAssign => Operator::Mod.eval(&arguments, context),
                    ExpAssign => Operator::Exp.eval(&arguments, context),
                    AndAssign => Operator::And.eval(&arguments, context),
                    OrAssign => Operator::Or.eval(&arguments, context),
                    _ => unreachable!(
                        "Forgot to add a match arm for an assign operation: {}",
                        self
                    ),
                }?;
                context.set_value(target, result)?;

                Ok(Value::Empty)
            }
            _ => self.eval(arguments, context),
        }
    }

    /// Returns the variable identifier read
    pub fn get_variable_identifier_read(&self) -> Option<String> {
        if let Operator::VariableIdentifierRead { identifier } = self {
            Some(identifier.to_string())
        } else {
            None
        }
    }

    /// Returns the variable identifier write
    pub fn get_variable_identifier_write(&self) -> Option<String> {
        if let Operator::VariableIdentifierWrite { identifier } = self {
            Some(identifier.to_string())
        } else {
            None
        }
    }
}
