//! A context defines methods to retrieve variable values and call functions for literals in an expression tree.
//! If mutable, it also allows to assign to variables.
//!
//! This crate implements two basic variants, the `EmptyContext`, that returns `None` for each identifier and cannot be manipulated, and the `HashMapContext`, that stores its mappings in hash maps.
//! The HashMapContext is type-safe and returns an error if the user tries to assign a value of a different type than before to an identifier.

use std::{collections::HashMap, iter};

use fastn_resolved::evalexpr::{
    function::Function,
    value::{value_type::ValueType, Value},
    EvalexprError, EvalexprResult,
};

mod predefined;

/// An immutable context.
pub trait Context {
    /// Returns the value that is linked to the given identifier.
    fn get_value(&self, identifier: &str) -> Option<&Value>;

    /// Calls the function that is linked to the given identifier with the given argument.
    /// If no function with the given identifier is found, this method returns `EvalexprError::FunctionIdentifierNotFound`.
    fn call_function(&self, identifier: &str, argument: &Value) -> EvalexprResult<Value>;
}

/// A context that allows to assign to variables.
pub trait ContextWithMutableVariables: Context {
    /// Sets the variable with the given identifier to the given value.
    fn set_value(&mut self, _identifier: String, _value: Value) -> EvalexprResult<()> {
        Err(EvalexprError::ContextNotMutable)
    }
}

/// A context that allows to assign to function identifiers.
pub trait ContextWithMutableFunctions: Context {
    /// Sets the function with the given identifier to the given function.
    fn set_function(&mut self, _identifier: String, _function: Function) -> EvalexprResult<()> {
        Err(EvalexprError::ContextNotMutable)
    }
}

/// A context that allows to iterate over its variable names with their values.
///
/// **Note:** this trait will change after GATs are stabilised, because then we can get rid of the lifetime in the trait definition.
pub trait IterateVariablesContext<'a> {
    /// The iterator type for iterating over variable name-value pairs.
    type VariableIterator: 'a + Iterator<Item = (String, Value)>;
    /// The iterator type for iterating over variable names.
    type VariableNameIterator: 'a + Iterator<Item = String>;

    /// Returns an iterator over pairs of variable names and values.
    fn iter_variables(&'a self) -> Self::VariableIterator;

    /// Returns an iterator over variable names.
    fn iter_variable_names(&'a self) -> Self::VariableNameIterator;
}

/*/// A context that allows to retrieve functions programmatically.
pub trait GetFunctionContext: Context {
    /// Returns the function that is linked to the given identifier.
    ///
    /// This might not be possible for all functions, as some might be hard-coded.
    /// In this case, a special error variant should be returned (Not yet implemented).
    fn get_function(&self, identifier: &str) -> Option<&Function>;
}*/

/// A context that returns `None` for each identifier.
#[derive(Debug, Default)]
pub struct EmptyContext;

impl Context for EmptyContext {
    fn get_value(&self, _identifier: &str) -> Option<&Value> {
        None
    }

    fn call_function(&self, identifier: &str, _argument: &Value) -> EvalexprResult<Value> {
        Err(EvalexprError::FunctionIdentifierNotFound(
            identifier.to_string(),
        ))
    }
}

impl<'a> IterateVariablesContext<'a> for EmptyContext {
    type VariableIterator = iter::Empty<(String, Value)>;
    type VariableNameIterator = iter::Empty<String>;

    fn iter_variables(&self) -> Self::VariableIterator {
        iter::empty()
    }

    fn iter_variable_names(&self) -> Self::VariableNameIterator {
        iter::empty()
    }
}

/// A context that stores its mappings in hash maps.
///
/// *Value and function mappings are stored independently, meaning that there can be a function and a value with the same identifier.*
///
/// This context is type-safe, meaning that an identifier that is assigned a value of some type once cannot be assigned a value of another type.
#[derive(Clone, Debug, Default)]
pub struct HashMapContext {
    variables: HashMap<String, Value>,
    functions: HashMap<String, Function>,
}

impl HashMapContext {
    /// Constructs a `HashMapContext` with no mappings.
    pub fn new() -> Self {
        Default::default()
    }
}

impl Context for HashMapContext {
    fn get_value(&self, identifier: &str) -> Option<&Value> {
        self.variables.get(identifier)
    }

    fn call_function(&self, identifier: &str, argument: &Value) -> EvalexprResult<Value> {
        if let Some(function) = self.functions.get(identifier) {
            function.call(argument)
        } else {
            Err(EvalexprError::FunctionIdentifierNotFound(
                identifier.to_string(),
            ))
        }
    }
}

impl ContextWithMutableVariables for HashMapContext {
    fn set_value(&mut self, identifier: String, value: Value) -> EvalexprResult<()> {
        if let Some(existing_value) = self.variables.get_mut(&identifier) {
            if ValueType::from(&existing_value) == ValueType::from(&value) {
                *existing_value = value;
                return Ok(());
            } else {
                return Err(EvalexprError::expected_type(existing_value, value));
            }
        }

        // Implicit else, because `self.variables` and `identifier` are not unborrowed in else
        self.variables.insert(identifier, value);
        Ok(())
    }
}

impl ContextWithMutableFunctions for HashMapContext {
    fn set_function(&mut self, identifier: String, function: Function) -> EvalexprResult<()> {
        self.functions.insert(identifier, function);
        Ok(())
    }
}

impl<'a> IterateVariablesContext<'a> for HashMapContext {
    type VariableIterator = std::iter::Map<
        std::collections::hash_map::Iter<'a, String, Value>,
        fn((&String, &Value)) -> (String, Value),
    >;
    type VariableNameIterator =
        std::iter::Cloned<std::collections::hash_map::Keys<'a, String, Value>>;

    fn iter_variables(&'a self) -> Self::VariableIterator {
        self.variables
            .iter()
            .map(|(string, value)| (string.clone(), value.clone()))
    }

    fn iter_variable_names(&'a self) -> Self::VariableNameIterator {
        self.variables.keys().cloned()
    }
}

/// This macro provides a convenient syntax for creating a static context.
///
/// # Examples
///
/// ```rust
/// use fastn_resolved::evalexpr::*;
///
/// let ctx = fastn_resolved::context_map! {
///     "x" => 8,
///     "f" => Function::new(|_| Ok(42.into()))
/// }.unwrap(); // Do proper error handling here
///
/// assert_eq!(eval_with_context("x + f()", &ctx), Ok(50.into()));
/// ```
#[macro_export]
macro_rules! context_map {
    // Termination (allow missing comma at the end of the argument list)
    ( ($ctx:expr) $k:expr => Function::new($($v:tt)*) ) =>
        { $crate::context_map!(($ctx) $k => Function::new($($v)*),) };
    ( ($ctx:expr) $k:expr => $v:expr ) =>
        { $crate::context_map!(($ctx) $k => $v,)  };
    // Termination
    ( ($ctx:expr) ) => { Ok(()) };

    // The user has to specify a literal 'Function::new' in order to create a function
    ( ($ctx:expr) $k:expr => Function::new($($v:tt)*) , $($tt:tt)*) => {{
        $crate::evalexpr::ContextWithMutableFunctions::set_function($ctx, $k.into(), $crate::evalexpr::Function::new($($v)*))
            .and($crate::context_map!(($ctx) $($tt)*))
    }};
    // add a value, and chain the eventual error with the ones in the next values
    ( ($ctx:expr) $k:expr => $v:expr , $($tt:tt)*) => {{
        $crate::evalexpr::ContextWithMutableVariables::set_value($ctx, $k.into(), $v.into())
            .and($crate::context_map!(($ctx) $($tt)*))
    }};

    // Create a context, then recurse to add the values in it
    ( $($tt:tt)* ) => {{
        let mut context = $crate::evalexpr::HashMapContext::new();
        $crate::context_map!((&mut context) $($tt)*)
            .map(|_| context)
    }};
}
