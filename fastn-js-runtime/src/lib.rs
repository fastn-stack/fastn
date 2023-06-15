extern crate self as fastn_js_runtime;

#[rquickjs::bind(object)]
pub fn add2(a: f32, b: f32) -> f32 {
    a + b
}

pub fn ssr(_js: &str) -> String {
    rquickjs::Context::full(&rquickjs::Runtime::new().unwrap())
        .unwrap()
        .with(|ctx| {
            let glob = ctx.globals();
            glob.init_def::<Add2>().unwrap();

            ctx.eval::<i32, _>(r#"add2(10, 2)"#).unwrap()
        })
        .to_string()
}
