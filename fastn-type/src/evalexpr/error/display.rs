use std::fmt;

use fastn_type::evalexpr::EvalexprError;

impl fmt::Display for EvalexprError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use fastn_type::evalexpr::EvalexprError::*;
        match self {
            WrongOperatorArgumentAmount { expected, actual } => write!(
                f,
                "An operator expected {} arguments, but got {}.",
                expected, actual
            ),
            WrongFunctionArgumentAmount { expected, actual } => write!(
                f,
                "A function expected {} arguments, but got {}.",
                expected, actual
            ),
            ExpectedString { actual } => {
                write!(f, "Expected a Value::String, but got {:?}.", actual)
            }
            ExpectedInt { actual } => write!(f, "Expected a Value::Int, but got {:?}.", actual),
            ExpectedFloat { actual } => write!(f, "Expected a Value::Float, but got {:?}.", actual),
            ExpectedNumber { actual } => write!(
                f,
                "Expected a Value::Float or Value::Int, but got {:?}.",
                actual
            ),
            ExpectedNumberOrString { actual } => write!(
                f,
                "Expected a Value::Number or a Value::String, but got {:?}.",
                actual
            ),
            ExpectedBoolean { actual } => {
                write!(f, "Expected a Value::Boolean, but got {:?}.", actual)
            }
            ExpectedTuple { actual } => write!(f, "Expected a Value::Tuple, but got {:?}.", actual),
            ExpectedFixedLenTuple {
                expected_len,
                actual,
            } => write!(
                f,
                "Expected a Value::Tuple of len {}, but got {:?}.",
                expected_len, actual
            ),
            ExpectedEmpty { actual } => write!(f, "Expected a Value::Empty, but got {:?}.", actual),
            AppendedToLeafNode => write!(f, "Tried to append a node to a leaf node."),
            PrecedenceViolation => write!(
                f,
                "Tried to append a node to another node with higher precedence."
            ),
            VariableIdentifierNotFound(identifier) => write!(
                f,
                "Variable identifier is not bound to anything by context: {:?}.",
                identifier
            ),
            FunctionIdentifierNotFound(identifier) => write!(
                f,
                "Function identifier is not bound to anything by context: {:?}.",
                identifier
            ),
            TypeError { expected, actual } => {
                write!(f, "Expected one of {:?}, but got {:?}.", expected, actual)
            }
            WrongTypeCombination { operator, actual } => write!(
                f,
                "The operator {:?} was called with a wrong combination of types: {:?}",
                operator, actual
            ),
            UnmatchedLBrace => write!(f, "Found an unmatched opening parenthesis '('."),
            UnmatchedRBrace => write!(f, "Found an unmatched closing parenthesis ')'."),
            MissingOperatorOutsideOfBrace { .. } => write!(
                f,
                "Found an opening parenthesis that is preceded by something that does not take \
                 any arguments on the right, or found a closing parenthesis that is succeeded by \
                 something that does not take any arguments on the left."
            ),
            UnmatchedPartialToken { first, second } => {
                if let Some(second) = second {
                    write!(
                        f,
                        "Found a partial token '{}' that should not be followed by '{}'.",
                        first, second
                    )
                } else {
                    write!(
                        f,
                        "Found a partial token '{}' that should be followed by another partial \
                         token.",
                        first
                    )
                }
            }
            AdditionError { augend, addend } => write!(f, "Error adding {} + {}", augend, addend),
            SubtractionError {
                minuend,
                subtrahend,
            } => write!(f, "Error subtracting {} - {}", minuend, subtrahend),
            NegationError { argument } => write!(f, "Error negating -{}", argument),
            MultiplicationError {
                multiplicand,
                multiplier,
            } => write!(f, "Error multiplying {} * {}", multiplicand, multiplier),
            DivisionError { dividend, divisor } => {
                write!(f, "Error dividing {} / {}", dividend, divisor)
            }
            ModulationError { dividend, divisor } => {
                write!(f, "Error modulating {} % {}", dividend, divisor)
            }
            InvalidRegex { regex, message } => write!(
                f,
                "Regular expression {:?} is invalid: {:?}",
                regex, message
            ),
            ContextNotMutable => write!(f, "Cannot manipulate context"),
            IllegalEscapeSequence(string) => write!(f, "Illegal escape sequence: {}", string),
            CustomMessage(message) => write!(f, "Error: {}", message),
        }
    }
}
