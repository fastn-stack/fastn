use std::fmt;

use fastn_grammar::evalexpr::{error::EvalexprResult, value::Value};

pub(crate) mod builtin;

/// A helper trait to enable cloning through `Fn` trait objects.
trait ClonableFn
where
    Self: Fn(&Value) -> EvalexprResult<Value>,
    Self: Send + Sync + 'static,
{
    fn dyn_clone(&self) -> Box<dyn ClonableFn>;
}

impl<F> ClonableFn for F
where
    F: Fn(&Value) -> EvalexprResult<Value>,
    F: Send + Sync + 'static,
    F: Clone,
{
    fn dyn_clone(&self) -> Box<dyn ClonableFn> {
        Box::new(self.clone()) as _
    }
}

/// A user-defined function.
/// Functions can be used in expressions by storing them in a `Context`.
///
/// # Examples
///
/// ```rust
/// use fastn_grammar::evalexpr::*;
///
/// let mut context = HashMapContext::new();
/// context.set_function("id".into(), Function::new(|argument| {
///     Ok(argument.clone())
/// })).unwrap(); // Do proper error handling here
/// assert_eq!(eval_with_context("id(4)", &context), Ok(Value::from(4)));
/// ```
pub struct Function {
    function: Box<dyn ClonableFn>,
}

impl Clone for Function {
    fn clone(&self) -> Self {
        Self {
            function: self.function.dyn_clone(),
        }
    }
}

impl Function {
    /// Creates a user-defined function.
    ///
    /// The `function` is boxed for storage.
    pub fn new<F>(function: F) -> Self
    where
        F: Fn(&Value) -> EvalexprResult<Value>,
        F: Send + Sync + 'static,
        F: Clone,
    {
        Self {
            function: Box::new(function) as _,
        }
    }

    pub(crate) fn call(&self, argument: &Value) -> EvalexprResult<Value> {
        (self.function)(argument)
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Function {{ [...] }}")
    }
}
