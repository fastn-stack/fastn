//!
//! ## Quickstart
//!
//! Add `evalexpr` as dependency to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! evalexpr = "<desired version>"
//! ```
//!
//! Then you can use `evalexpr` to **evaluate expressions** like this:
//!
//! ```rust
//! use fastn_type::evalexpr::*;
//!
//! assert_eq!(eval("1 + 2 + 3"), Ok(Value::from(6)));
//! // `eval` returns a variant of the `Value` enum,
//! // while `eval_[type]` returns the respective type directly.
//! // Both can be used interchangeably.
//! assert_eq!(eval_int("1 + 2 + 3"), Ok(6));
//! assert_eq!(eval("1 - 2 * 3"), Ok(Value::from(-5)));
//! assert_eq!(eval("1.0 + 2 * 3"), Ok(Value::from(7.0)));
//! assert_eq!(eval("true && 4 > 2"), Ok(Value::from(true)));
//! ```
//!
//! You can **chain** expressions and **assign** to variables like this:
//!
//! ```rust
//! use fastn_type::evalexpr::*;
//!
//! let mut context = HashMapContext::new();
//! // Assign 5 to a like this
//! assert_eq!(eval_empty_with_context_mut("a = 5", &mut context), Ok(EMPTY_VALUE));
//! // The HashMapContext is type safe, so this will fail now
//! assert_eq!(eval_empty_with_context_mut("a = 5.0", &mut context),
//!            Err(EvalexprError::expected_int(Value::from(5.0))));
//! // We can check which value the context stores for a like this
//! assert_eq!(context.get_value("a"), Some(&Value::from(5)));
//! // And use the value in another expression like this
//! assert_eq!(eval_int_with_context_mut("a = a + 2; a", &mut context), Ok(7));
//! // It is also possible to save a bit of typing by using an operator-assignment operator
//! assert_eq!(eval_int_with_context_mut("a += 2; a", &mut context), Ok(9));
//! ```
//!
//! And you can use **variables** and **functions** in expressions like this:
//!
//! ```rust
//! use fastn_type::evalexpr::*;
//!
//! let context = fastn_type::context_map! {
//!     "five" => 5,
//!     "twelve" => 12,
//!     "f" => Function::new(|argument| {
//!         if let Ok(int) = argument.as_int() {
//!             Ok(Value::Int(int / 2))
//!         } else if let Ok(float) = argument.as_float() {
//!             Ok(Value::Float(float / 2.0))
//!         } else {
//!             Err(EvalexprError::expected_number(argument.clone()))
//!         }
//!     }),
//!     "avg" => Function::new(|argument| {
//!         let arguments = argument.as_tuple()?;
//!
//!         if let (Value::Int(a), Value::Int(b)) = (&arguments[0], &arguments[1]) {
//!             Ok(Value::Int((a + b) / 2))
//!         } else {
//!             Ok(Value::Float((arguments[0].as_number()? + arguments[1].as_number()?) / 2.0))
//!         }
//!     })
//! }.unwrap(); // Do proper error handling here
//!
//! assert_eq!(eval_with_context("five + 8 > f(twelve)", &context), Ok(Value::from(true)));
//! // `eval_with_context` returns a variant of the `Value` enum,
//! // while `eval_[type]_with_context` returns the respective type directly.
//! // Both can be used interchangeably.
//! assert_eq!(eval_boolean_with_context("five + 8 > f(twelve)", &context), Ok(true));
//! assert_eq!(eval_with_context("avg(2, 4) == 3", &context), Ok(Value::from(true)));
//! ```
//!
//! You can also **precompile** expressions like this:
//!
//! ```rust
//! use fastn_type::evalexpr::*;
//!
//! let precompiled = build_operator_tree("a * b - c > 5").unwrap(); // Do proper error handling here
//!
//! let mut context = fastn_type::context_map! {
//!     "a" => 6,
//!     "b" => 2,
//!     "c" => 3
//! }.unwrap(); // Do proper error handling here
//! assert_eq!(precompiled.eval_with_context(&context), Ok(Value::from(true)));
//!
//! context.set_value("c".into(), 8.into()).unwrap(); // Do proper error handling here
//! assert_eq!(precompiled.eval_with_context(&context), Ok(Value::from(false)));
//! // `Node::eval_with_context` returns a variant of the `Value` enum,
//! // while `Node::eval_[type]_with_context` returns the respective type directly.
//! // Both can be used interchangeably.
//! assert_eq!(precompiled.eval_boolean_with_context(&context), Ok(false));
//! ```
//!
//! ## Features
//!
//! ### Operators
//!
//! This crate offers a set of binary and unary operators for building expressions.
//! Operators have a precedence to determine their order of evaluation, where operators of higher precedence are evaluated first.
//! The precedence should resemble that of most common programming languages, especially Rust.
//! Variables and values have a precedence of 200, and function literals have 190.
//!
//! Supported binary operators:
//!
//! | Operator | Precedence | Description |
//! |----------|------------|-------------|
//! | ^ | 120 | Exponentiation |
//! | * | 100 | Product |
//! | / | 100 | Division (integer if both arguments are integers, otherwise float) |
//! | % | 100 | Modulo (integer if both arguments are integers, otherwise float) |
//! | + | 95 | Sum or String Concatenation |
//! | - | 95 | Difference |
//! | < | 80 | Lower than |
//! | \> | 80 | Greater than |
//! | <= | 80 | Lower than or equal |
//! | \>= | 80 | Greater than or equal |
//! | == | 80 | Equal |
//! | != | 80 | Not equal |
//! | && | 75 | Logical and |
//! | &#124;&#124; | 70 | Logical or |
//! | = | 50 | Assignment |
//! | += | 50 | Sum-Assignment or String-Concatenation-Assignment |
//! | -= | 50 | Difference-Assignment |
//! | *= | 50 | Product-Assignment |
//! | /= | 50 | Division-Assignment |
//! | %= | 50 | Modulo-Assignment |
//! | ^= | 50 | Exponentiation-Assignment |
//! | &&= | 50 | Logical-And-Assignment |
//! | &#124;&#124;= | 50 | Logical-Or-Assignment |
//! | , | 40 | Aggregation |
//! | ; | 0 | Expression Chaining |
//!
//! Supported unary operators:
//!
//! | Operator | Precedence | Description |
//! |----------|------------|-------------|
//! | - | 110 | Negation |
//! | ! | 110 | Logical not |
//!
//! Operators that take numbers as arguments can either take integers or floating point numbers.
//! If one of the arguments is a floating point number, all others are converted to floating point numbers as well, and the resulting value is a floating point number as well.
//! Otherwise, the result is an integer.
//! An exception to this is the exponentiation operator that always returns a floating point number.
//! Example:
//!
//! ```rust
//! use fastn_type::evalexpr::*;
//!
//! assert_eq!(eval("1 / 2"), Ok(Value::from(0)));
//! assert_eq!(eval("1.0 / 2"), Ok(Value::from(0.5)));
//! assert_eq!(eval("2^2"), Ok(Value::from(4.0)));
//! ```
//!
//! #### The Aggregation Operator
//!
//! The aggregation operator aggregates a set of values into a tuple.
//! A tuple can contain arbitrary values, it is not restricted to a single type.
//! The operator is n-ary, so it supports creating tuples longer than length two.
//! Example:
//!
//! ```rust
//! use fastn_type::evalexpr::*;
//!
//! assert_eq!(eval("1, \"b\", 3"),
//!            Ok(Value::from(vec![Value::from(1), Value::from("b"), Value::from(3)])));
//! ```
//!
//! To create nested tuples, use parentheses:
//!
//! ```rust
//! use fastn_type::evalexpr::*;
//!
//! assert_eq!(eval("1, 2, (true, \"b\")"), Ok(Value::from(vec![
//!     Value::from(1),
//!     Value::from(2),
//!     Value::from(vec![
//!         Value::from(true),
//!         Value::from("b")
//!     ])
//! ])));
//! ```
//!
//! #### The Assignment Operator
//!
//! This crate features the assignment operator, that allows expressions to store their result in a variable in the expression context.
//! If an expression uses the assignment operator, it must be evaluated with a mutable context.
//!
//! Note that assignments are type safe when using the `HashMapContext`.
//! That means that if an identifier is assigned a value of a type once, it cannot be assigned a value of another type.
//!
//! ```rust
//! use fastn_type::evalexpr::*;
//!
//! let mut context = HashMapContext::new();
//! assert_eq!(eval_with_context("a = 5", &context), Err(EvalexprError::ContextNotMutable));
//! assert_eq!(eval_empty_with_context_mut("a = 5", &mut context), Ok(EMPTY_VALUE));
//! assert_eq!(eval_empty_with_context_mut("a = 5.0", &mut context),
//!            Err(EvalexprError::expected_int(5.0.into())));
//! assert_eq!(eval_int_with_context("a", &context), Ok(5));
//! assert_eq!(context.get_value("a"), Some(5.into()).as_ref());
//! ```
//!
//! For each binary operator, there exists an equivalent operator-assignment operator.
//! Here are some examples:
//!
//! ```rust
//! use fastn_type::evalexpr::*;
//!
//! assert_eq!(eval_int("a = 2; a *= 2; a += 2; a"), Ok(6));
//! assert_eq!(eval_float("a = 2.2; a /= 2.0 / 4 + 1; a"), Ok(2.2 / (2.0 / 4.0 + 1.0)));
//! assert_eq!(eval_string("a = \"abc\"; a += \"def\"; a"), Ok("abcdef".to_string()));
//! assert_eq!(eval_boolean("a = true; a &&= false; a"), Ok(false));
//! ```
//!
//! #### The Expression Chaining Operator
//!
//! The expression chaining operator works as one would expect from programming languages that use the semicolon to end statements, like `Rust`, `C` or `Java`.
//! It has the special feature that it returns the value of the last expression in the expression chain.
//! If the last expression is terminated by a semicolon as well, then `Value::Empty` is returned.
//! Expression chaining is useful together with assignment to create small scripts.
//!
//! ```rust
//! use fastn_type::evalexpr::*;
//!
//! let mut context = HashMapContext::new();
//! assert_eq!(eval("1;2;3;4;"), Ok(Value::Empty));
//! assert_eq!(eval("1;2;3;4"), Ok(4.into()));
//!
//! // Initialization of variables via script
//! assert_eq!(eval_empty_with_context_mut("hp = 1; max_hp = 5; heal_amount = 3;", &mut context),
//!            Ok(EMPTY_VALUE));
//! // Precompile healing script
//! let healing_script = build_operator_tree("hp = min(hp + heal_amount, max_hp); hp").unwrap(); // Do proper error handling here
//! // Execute precompiled healing script
//! assert_eq!(healing_script.eval_int_with_context_mut(&mut context), Ok(4));
//! assert_eq!(healing_script.eval_int_with_context_mut(&mut context), Ok(5));
//! ```
//!
//! ### Contexts
//!
//! An expression evaluator that just evaluates expressions would be useful already, but this crate can do more.
//! It allows using [*variables*](#variables), [*assignments*](#the-assignment-operator), [*statement chaining*](#the-expression-chaining-operator) and [*user-defined functions*](#user-defined-functions) within an expression.
//! When assigning to variables, the assignment is stored in a context.
//! When the variable is read later on, it is read from the context.
//! Contexts can be preserved between multiple calls to eval by creating them yourself.
//! Here is a simple example to show the difference between preserving and not preserving context between evaluations:
//!
//! ```rust
//! use fastn_type::evalexpr::*;
//!
//! assert_eq!(eval("a = 5;"), Ok(Value::from(())));
//! // The context is not preserved between eval calls
//! assert_eq!(eval("a"), Err(EvalexprError::VariableIdentifierNotFound("a".to_string())));
//!
//! let mut context = HashMapContext::new();
//! assert_eq!(eval_with_context_mut("a = 5;", &mut context), Ok(Value::from(())));
//! // Assignments require mutable contexts
//! assert_eq!(eval_with_context("a = 6", &context), Err(EvalexprError::ContextNotMutable));
//! // The HashMapContext is type safe
//! assert_eq!(eval_with_context_mut("a = 5.5", &mut context),
//!            Err(EvalexprError::ExpectedInt { actual: Value::from(5.5) }));
//! // Reading a variable does not require a mutable context
//! assert_eq!(eval_with_context("a", &context), Ok(Value::from(5)));
//!
//! ```
//!
//! Note that the assignment is forgotten between the two calls to eval in the first example.
//! In the second part, the assignment is correctly preserved.
//! Note as well that to assign to a variable, the context needs to be passed as a mutable reference.
//! When passed as an immutable reference, an error is returned.
//!
//! Also, the `HashMapContext` is type safe.
//! This means that assigning to `a` again with a different type yields an error.
//! Type unsafe contexts may be implemented if requested.
//! For reading `a`, it is enough to pass an immutable reference.
//!
//! Contexts can also be manipulated in code.
//! Take a look at the following example:
//!
//! ```rust
//! use fastn_type::evalexpr::*;
//!
//! let mut context = HashMapContext::new();
//! // We can set variables in code like this...
//! context.set_value("a".into(), 5.into());
//! // ...and read from them in expressions
//! assert_eq!(eval_int_with_context("a", &context), Ok(5));
//! // We can write or overwrite variables in expressions...
//! assert_eq!(eval_with_context_mut("a = 10; b = 1.0;", &mut context), Ok(().into()));
//! // ...and read the value in code like this
//! assert_eq!(context.get_value("a"), Some(&Value::from(10)));
//! assert_eq!(context.get_value("b"), Some(&Value::from(1.0)));
//! ```
//!
//! Contexts are also required for user-defined functions.
//! Those can be passed one by one with the `set_function` method, but it might be more convenient to use the `context_map!` macro instead:
//!
//! ```rust
//! use fastn_type::evalexpr::*;
//!
//! let context = fastn_type::context_map!{
//!     "f" => Function::new(|args| Ok(Value::from(args.as_int()? + 5))),
//! }.unwrap_or_else(|error| panic!("Error creating context: {}", error));
//! assert_eq!(eval_int_with_context("f 5", &context), Ok(10));
//! ```
//!
//! For more information about user-defined functions, refer to the respective [section](#user-defined-functions).
//!
//! ### Builtin Functions
//!
//! This crate offers a set of builtin functions.
//!
//! | Identifier           | Argument Amount | Argument Types         | Description |
//! |----------------------|-----------------|------------------------|-------------|
//! | `min`                | >= 1            | Numeric                | Returns the minimum of the arguments |
//! | `max`                | >= 1            | Numeric                | Returns the maximum of the arguments |
//! | `len`                | 1               | String/Tuple           | Returns the character length of a string, or the amount of elements in a tuple (not recursively) |
//! | `floor`              | 1               | Numeric                | Returns the largest integer less than or equal to a number |
//! | `round`              | 1               | Numeric                | Returns the nearest integer to a number. Rounds half-way cases away from 0.0 |
//! | `ceil`               | 1               | Numeric                | Returns the smallest integer greater than or equal to a number |
//! | `if`                 | 3               | Boolean, Any, Any      | If the first argument is true, returns the second argument, otherwise, returns the third  |
//! | `typeof`             | 1               | Any                    | returns "string", "float", "int", "boolean", "tuple", or "empty" depending on the type of the argument  |
//! | `math::is_nan`       | 1               | Numeric                | Returns true if the argument is the floating-point value NaN, false if it is another floating-point value, and throws an error if it is not a number  |
//! | `math::is_finite`    | 1               | Numeric                | Returns true if the argument is a finite floating-point number, false otherwise  |
//! | `math::is_infinite`  | 1               | Numeric                | Returns true if the argument is an infinite floating-point number, false otherwise  |
//! | `math::is_normal`    | 1               | Numeric                | Returns true if the argument is a floating-point number that is neither zero, infinite, [subnormal](https://en.wikipedia.org/wiki/Subnormal_number), or NaN, false otherwise  |
//! | `math::ln`           | 1               | Numeric                | Returns the natural logarithm of the number |
//! | `math::log`          | 2               | Numeric, Numeric       | Returns the logarithm of the number with respect to an arbitrary base |
//! | `math::log2`         | 1               | Numeric                | Returns the base 2 logarithm of the number |
//! | `math::log10`        | 1               | Numeric                | Returns the base 10 logarithm of the number |
//! | `math::exp`          | 1               | Numeric                | Returns `e^(number)`, (the exponential function) |
//! | `math::exp2`         | 1               | Numeric                | Returns `2^(number)` |
//! | `math::pow`          | 2               | Numeric, Numeric       | Raises a number to the power of the other number |
//! | `math::cos`          | 1               | Numeric                | Computes the cosine of a number (in radians) |
//! | `math::acos`         | 1               | Numeric                | Computes the arccosine of a number. The return value is in radians in the range [0, pi] or NaN if the number is outside the range [-1, 1] |
//! | `math::cosh`         | 1               | Numeric                | Hyperbolic cosine function |
//! | `math::acosh`        | 1               | Numeric                | Inverse hyperbolic cosine function |
//! | `math::sin`          | 1               | Numeric                | Computes the sine of a number (in radians) |
//! | `math::asin`         | 1               | Numeric                | Computes the arcsine of a number. The return value is in radians in the range [-pi/2, pi/2] or NaN if the number is outside the range [-1, 1] |
//! | `math::sinh`         | 1               | Numeric                | Hyperbolic sine function |
//! | `math::asinh`        | 1               | Numeric                | Inverse hyperbolic sine function |
//! | `math::tan`          | 1               | Numeric                | Computes the tangent of a number (in radians) |
//! | `math::atan`         | 1               | Numeric                | Computes the arctangent of a number. The return value is in radians in the range [-pi/2, pi/2] |
//! | `math::atan2`        | 2               | Numeric, Numeric       | Computes the four quadrant arctangent in radians |
//! | `math::tanh`         | 1               | Numeric                | Hyperbolic tangent function |
//! | `math::atanh`        | 1               | Numeric                | Inverse hyperbolic tangent function. |
//! | `math::sqrt`         | 1               | Numeric                | Returns the square root of a number. Returns NaN for a negative number |
//! | `math::cbrt`         | 1               | Numeric                | Returns the cube root of a number |
//! | `math::hypot`        | 2               | Numeric                | Calculates the length of the hypotenuse of a right-angle triangle given legs of length given by the two arguments |
//! | `str::regex_matches` | 2               | String, String         | Returns true if the first argument matches the regex in the second argument (Requires `regex_support` feature flag) |
//! | `str::regex_replace` | 3               | String, String, String | Returns the first argument with all matches of the regex in the second argument replaced by the third argument (Requires `regex_support` feature flag) |
//! | `str::to_lowercase`  | 1               | String                 | Returns the lower-case version of the string |
//! | `str::to_uppercase`  | 1               | String                 | Returns the upper-case version of the string |
//! | `str::trim`          | 1               | String                 | Strips whitespace from the start and the end of the string |
//! | `str::from`          | >= 0            | Any                    | Returns passed value as string |
//! | `bitand`             | 2               | Int                    | Computes the bitwise and of the given integers |
//! | `bitor`              | 2               | Int                    | Computes the bitwise or of the given integers |
//! | `bitxor`             | 2               | Int                    | Computes the bitwise xor of the given integers |
//! | `bitnot`             | 1               | Int                    | Computes the bitwise not of the given integer |
//! | `shl`                | 2               | Int                    | Computes the given integer bitwise shifted left by the other given integer |
//! | `shr`                | 2               | Int                    | Computes the given integer bitwise shifted right by the other given integer |
//! | `random`             | 0               | Empty                  | Return a random float between 0 and 1. Requires the `rand` feature flag. |
//!
//! The `min` and `max` functions can deal with a mixture of integer and floating point arguments.
//! If the maximum or minimum is an integer, then an integer is returned.
//! Otherwise, a float is returned.
//!
//! The regex functions require the feature flag `regex_support`.
//!
//! ### Values
//!
//! Operators take values as arguments and produce values as results.
//! Values can be booleans, integer or floating point numbers, strings, tuples or the empty type.
//! Values are denoted as displayed in the following table.
//!
//! | Value type | Example |
//! |------------|---------|
//! | `Value::String` | `"abc"`, `""`, `"a\"b\\c"` |
//! | `Value::Boolean` | `true`, `false` |
//! | `Value::Int` | `3`, `-9`, `0`, `135412` |
//! | `Value::Float` | `3.`, `.35`, `1.00`, `0.5`, `123.554`, `23e4`, `-2e-3`, `3.54e+2` |
//! | `Value::Tuple` | `(3, 55.0, false, ())`, `(1, 2)` |
//! | `Value::Empty` | `()` |
//!
//! Integers are internally represented as `i64`, and floating point numbers are represented as `f64`.
//! Tuples are represented as `Vec<Value>` and empty values are not stored, but represented by Rust's unit type `()` where necessary.
//!
//! There exist type aliases for some of the types.
//! They include `IntType`, `FloatType`, `TupleType` and `EmptyType`.
//!
//! Values can be constructed either directly or using the `From` trait.
//! They can be decomposed using the `Value::as_[type]` methods.
//! The type of a value can be checked using the `Value::is_[type]` methods.
//!
//! **Examples for constructing a value:**
//!
//! | Code | Result |
//! |------|--------|
//! | `Value::from(4)` | `Value::Int(4)` |
//! | `Value::from(4.4)` | `Value::Float(4.4)` |
//! | `Value::from(true)` | `Value::Boolean(true)` |
//! | `Value::from(vec![Value::from(3)])` | `Value::Tuple(vec![Value::Int(3)])` |
//!
//! **Examples for deconstructing a value:**
//!
//! | Code | Result |
//! |------|--------|
//! | `Value::from(4).as_int()` | `Ok(4)` |
//! | `Value::from(4.4).as_float()` | `Ok(4.4)` |
//! | `Value::from(true).as_int()` | `Err(Error::ExpectedInt {actual: Value::Boolean(true)})` |
//!
//! Values have a precedence of 200.
//!
//! ### Variables
//!
//! This crate allows to compile parameterizable formulas by using variables.
//! A variable is a literal in the formula, that does not contain whitespace or can be parsed as value.
//! For working with variables, a [context](#contexts) is required.
//! It stores the mappings from variables to their values.
//!
//! Variables do not have fixed types in the expression itself, but are typed by the context.
//! Once a variable is assigned a value of a specific type, it cannot be assigned a value of another type.
//! This might change in the future and can be changed by using a type-unsafe context (not provided by this crate as of now).
//!
//! Here are some examples and counter-examples on expressions that are interpreted as variables:
//!
//! | Expression | Variable? | Explanation |
//! |------------|--------|-------------|
//! | `a` | yes | |
//! | `abc` | yes | |
//! | `a<b` | no | Expression is interpreted as variable `a`, operator `<` and variable `b` |
//! | `a b` | no | Expression is interpreted as function `a` applied to argument `b` |
//! | `123` | no | Expression is interpreted as `Value::Int` |
//! | `true` | no | Expression is interpreted as `Value::Bool` |
//! | `.34` | no | Expression is interpreted as `Value::Float` |
//!
//! Variables have a precedence of 200.
//!
//! ### User-Defined Functions
//!
//! This crate allows to define arbitrary functions to be used in parsed expressions.
//! A function is defined as a `Function` instance, wrapping an `fn(&Value) -> EvalexprResult<Value>`.
//! The definition needs to be included in the [`Context`](#contexts) that is used for evaluation.
//! As of now, functions cannot be defined within the expression, but that might change in the future.
//!
//! The function gets passed what ever value is directly behind it, be it a tuple or a single values.
//! If there is no value behind a function, it is interpreted as a variable instead.
//! More specifically, a function needs to be followed by either an opening brace `(`, another literal, or a value.
//! While not including special support for multi-valued functions, they can be realized by requiring a single tuple argument.
//!
//! Be aware that functions need to verify the types of values that are passed to them.
//! The `error` module contains some shortcuts for verification, and error types for passing a wrong value type.
//! Also, most numeric functions need to distinguish between being called with integers or floating point numbers, and act accordingly.
//!
//! Here are some examples and counter-examples on expressions that are interpreted as function calls:
//!
//! | Expression | Function? | Explanation |
//! |------------|--------|-------------|
//! | `a v` | yes | |
//! | `x 5.5` | yes | |
//! | `a (3, true)` | yes | |
//! | `a b 4` | yes | Call `a` with the result of calling `b` with `4` |
//! | `5 b` | no | Error, value cannot be followed by a literal |
//! | `12 3` | no | Error, value cannot be followed by a value |
//! | `a 5 6` | no | Error, function call cannot be followed by a value |
//!
//! Functions have a precedence of 190.
//!
//! ### [Serde](https://serde.rs)
//!
//! To use this crate with serde, the `serde_support` feature flag has to be set.
//! This can be done like this in the `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! evalexpr = {version = "7", features = ["serde_support"]}
//! ```
//!
//! This crate implements `serde::de::Deserialize` for its type `Node` that represents a parsed expression tree.
//! The implementation expects a [serde `string`](https://serde.rs/data-model.html) as input.
//! Example parsing with [ron format](docs.rs/ron):
//!
//! ```rust
//! # #[cfg(feature = "serde_support")] {
//! extern crate ron;
//! use fastn_type::evalexpr::*;
//!
//! let mut context = fastn_type::context_map!{
//!     "five" => 5
//! }.unwrap(); // Do proper error handling here
//!
//! // In ron format, strings are surrounded by "
//! let serialized_free = "\"five * five\"";
//! match ron::de::from_str::<ExprNode>(serialized_free) {
//!     Ok(free) => assert_eq!(free.eval_with_context(&context), Ok(Value::from(25))),
//!     Err(error) => {
//!         () // Handle error
//!     }
//! }
//! # }
//! ```
//!
//! With `serde`, expressions can be integrated into arbitrarily complex data.
//!
//! The crate also implements `Serialize` and `Deserialize` for the `HashMapContext`,
//! but note that only the variables get (de)serialized, not the functions.
//!
//! ## License
//!
//! This crate is primarily distributed under the terms of the MIT license.
//! See [LICENSE](LICENSE) for details.
//!

#![deny(missing_docs)]
#![forbid(unsafe_code)]

pub use fastn_type::evalexpr::{
    context::{
        Context, ContextWithMutableFunctions, ContextWithMutableVariables, EmptyContext,
        HashMapContext, IterateVariablesContext,
    },
    error::{EvalexprError, EvalexprResult},
    function::Function,
    interface::*,
    operator::Operator,
    token::PartialToken,
    tree::ExprNode,
    value::{value_type::ValueType, EmptyType, FloatType, IntType, TupleType, Value, EMPTY_VALUE},
};

mod context;
pub mod error;
mod function;
mod interface;
mod operator;
mod token;
mod tree;
mod value;

// Exports
