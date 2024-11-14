use fastn_type::evalexpr::error::{EvalexprError, EvalexprResult};
use std::convert::TryFrom;

mod display;
pub mod value_type;

/// The type used to represent integers in `Value::Int`.
pub type IntType = i64;

/// The type used to represent floats in `Value::Float`.
pub type FloatType = f64;

/// The type used to represent tuples in `Value::Tuple`.
pub type TupleType = Vec<Value>;

/// The type used to represent empty values in `Value::Empty`.
pub type EmptyType = ();

/// The value of the empty type to be used in rust.
pub const EMPTY_VALUE: () = ();

/// The value type used by the parser.
/// Values can be of different subtypes that are the variants of this enum.
#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub enum Value {
    /// A string value.
    String(String),
    /// A float value.
    Float(FloatType),
    /// An integer value.
    Int(IntType),
    /// A boolean value.
    Boolean(bool),
    /// A tuple value.
    Tuple(TupleType),
    /// An empty value.
    Empty,
}

impl Value {
    /// Returns true if `self` is a `Value::String`.
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }
    /// Returns true if `self` is a `Value::Int`.
    pub fn is_int(&self) -> bool {
        matches!(self, Value::Int(_))
    }

    /// Returns true if `self` is a `Value::Float`.
    pub fn is_float(&self) -> bool {
        matches!(self, Value::Float(_))
    }

    /// Returns true if `self` is a `Value::Int` or `Value::Float`.
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Int(_) | Value::Float(_))
    }

    /// Returns true if `self` is a `Value::Boolean`.
    pub fn is_boolean(&self) -> bool {
        matches!(self, Value::Boolean(_))
    }

    /// Returns true if `self` is a `Value::Tuple`.
    pub fn is_tuple(&self) -> bool {
        matches!(self, Value::Tuple(_))
    }

    /// Returns true if `self` is a `Value::Empty`.
    pub fn is_empty(&self) -> bool {
        matches!(self, Value::Empty)
    }

    /// Clones the value stored in `self` as `String`, or returns `Err` if `self` is not a `Value::String`.
    pub fn as_string(&self) -> EvalexprResult<String> {
        match self {
            Value::String(string) => Ok(string.clone()),
            value => Err(EvalexprError::expected_string(value.clone())),
        }
    }

    /// Clones the value stored in `self` as `IntType`, or returns `Err` if `self` is not a `Value::Int`.
    pub fn as_int(&self) -> EvalexprResult<IntType> {
        match self {
            Value::Int(i) => Ok(*i),
            value => Err(EvalexprError::expected_int(value.clone())),
        }
    }

    /// Clones the value stored in  `self` as `FloatType`, or returns `Err` if `self` is not a `Value::Float`.
    pub fn as_float(&self) -> EvalexprResult<FloatType> {
        match self {
            Value::Float(f) => Ok(*f),
            value => Err(EvalexprError::expected_float(value.clone())),
        }
    }

    /// Clones the value stored in  `self` as `FloatType`, or returns `Err` if `self` is not a `Value::Float` or `Value::Int`.
    /// Note that this method silently converts `IntType` to `FloatType`, if `self` is a `Value::Int`.
    pub fn as_number(&self) -> EvalexprResult<FloatType> {
        match self {
            Value::Float(f) => Ok(*f),
            Value::Int(i) => Ok(*i as FloatType),
            value => Err(EvalexprError::expected_number(value.clone())),
        }
    }

    /// Clones the value stored in  `self` as `bool`, or returns `Err` if `self` is not a `Value::Boolean`.
    pub fn as_boolean(&self) -> EvalexprResult<bool> {
        match self {
            Value::Boolean(boolean) => Ok(*boolean),
            value => Err(EvalexprError::expected_boolean(value.clone())),
        }
    }

    /// Clones the value stored in `self` as `TupleType`, or returns `Err` if `self` is not a `Value::Tuple`.
    pub fn as_tuple(&self) -> EvalexprResult<TupleType> {
        match self {
            Value::Tuple(tuple) => Ok(tuple.clone()),
            value => Err(EvalexprError::expected_tuple(value.clone())),
        }
    }

    /// Clones the value stored in `self` as `TupleType` or returns `Err` if `self` is not a `Value::Tuple` of the required length.
    pub fn as_fixed_len_tuple(&self, len: usize) -> EvalexprResult<TupleType> {
        match self {
            Value::Tuple(tuple) => {
                if tuple.len() == len {
                    Ok(tuple.clone())
                } else {
                    Err(EvalexprError::expected_fixed_len_tuple(len, self.clone()))
                }
            }
            value => Err(EvalexprError::expected_tuple(value.clone())),
        }
    }

    /// Returns `()`, or returns`Err` if `self` is not a `Value::Tuple`.
    pub fn as_empty(&self) -> EvalexprResult<()> {
        match self {
            Value::Empty => Ok(()),
            value => Err(EvalexprError::expected_empty(value.clone())),
        }
    }
}

impl From<String> for Value {
    fn from(string: String) -> Self {
        Value::String(string)
    }
}

impl From<&str> for Value {
    fn from(string: &str) -> Self {
        Value::String(string.to_string())
    }
}

impl From<FloatType> for Value {
    fn from(float: FloatType) -> Self {
        Value::Float(float)
    }
}

impl From<IntType> for Value {
    fn from(int: IntType) -> Self {
        Value::Int(int)
    }
}

impl From<bool> for Value {
    fn from(boolean: bool) -> Self {
        Value::Boolean(boolean)
    }
}

impl From<TupleType> for Value {
    fn from(tuple: TupleType) -> Self {
        Value::Tuple(tuple)
    }
}

impl From<Value> for EvalexprResult<Value> {
    fn from(value: Value) -> Self {
        Ok(value)
    }
}

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Value::Empty
    }
}

impl TryFrom<Value> for String {
    type Error = EvalexprError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::String(value) = value {
            Ok(value)
        } else {
            Err(EvalexprError::ExpectedString { actual: value })
        }
    }
}

impl TryFrom<Value> for FloatType {
    type Error = EvalexprError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Float(value) = value {
            Ok(value)
        } else {
            Err(EvalexprError::ExpectedFloat { actual: value })
        }
    }
}

impl TryFrom<Value> for IntType {
    type Error = EvalexprError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Int(value) = value {
            Ok(value)
        } else {
            Err(EvalexprError::ExpectedInt { actual: value })
        }
    }
}

impl TryFrom<Value> for bool {
    type Error = EvalexprError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Boolean(value) = value {
            Ok(value)
        } else {
            Err(EvalexprError::ExpectedBoolean { actual: value })
        }
    }
}

impl TryFrom<Value> for TupleType {
    type Error = EvalexprError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Tuple(value) = value {
            Ok(value)
        } else {
            Err(EvalexprError::ExpectedTuple { actual: value })
        }
    }
}

impl TryFrom<Value> for () {
    type Error = EvalexprError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Empty = value {
            Ok(())
        } else {
            Err(EvalexprError::ExpectedEmpty { actual: value })
        }
    }
}

#[cfg(test)]
mod tests {
    use fastn_type::evalexpr::value::{TupleType, Value};

    #[test]
    fn test_value_conversions() {
        assert_eq!(
            Value::from("string").as_string(),
            Ok(String::from("string"))
        );
        assert_eq!(Value::from(3).as_int(), Ok(3));
        assert_eq!(Value::from(3.3).as_float(), Ok(3.3));
        assert_eq!(Value::from(true).as_boolean(), Ok(true));
        assert_eq!(
            Value::from(TupleType::new()).as_tuple(),
            Ok(TupleType::new())
        );
    }

    #[test]
    fn test_value_checks() {
        assert!(Value::from("string").is_string());
        assert!(Value::from(3).is_int());
        assert!(Value::from(3.3).is_float());
        assert!(Value::from(true).is_boolean());
        assert!(Value::from(TupleType::new()).is_tuple());
    }
}
