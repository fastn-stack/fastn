/// Context with all Rust's constants in `f64::consts` available by default.
/// Alternatively, specifiy constants with `math_consts_context!(E, PI, TAU, ...)`
/// Available constants can be found in the [`core::f64::consts module`](https://doc.rust-lang.org/nightly/core/f64/consts/index.html).
#[macro_export]
macro_rules! math_consts_context {
    () => {
        $fastn_type::evalexpr::math_consts_context!(
            PI,
            TAU,
            FRAC_PI_2,
            FRAC_PI_3,
            FRAC_PI_4,
            FRAC_PI_6,
            FRAC_PI_8,
            FRAC_1_PI,
            FRAC_2_PI,
            FRAC_2_SQRT_PI,
            SQRT_2,
            FRAC_1_SQRT_2,
            E,
            LOG2_10,
            LOG2_E,
            LOG10_2,
            LOG10_E,
            LN_2,
            LN_10
        )
    };
    ($($name:ident),*) => {{
        use $fastn_type::evalexpr::ContextWithMutableVariables;
        $fastn_type::evalexpr::context_map! {
            $(
                stringify!($name) => core::f64::consts::$name,
            )*
        }
    }};
}
