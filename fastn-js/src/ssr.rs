const DELETE_DENO: &str = "delete Deno;";

#[derive(thiserror::Error, Debug)]
pub enum SSRError {
    #[error("Error executing JavaScript: {0}")]
    EvalError(String),

    #[error("Error deserializing value: {0}")]
    DeserializeError(String),
}

pub fn run_test(js: &str) -> Result<Vec<bool>, SSRError> {
    let mut runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions::default());

    eval::<Vec<bool>>(
        &mut runtime,
        deno_core::FastString::from(format!("{DELETE_DENO}{js}")),
    )
}

pub fn ssr_str(js: &str) -> Result<String, SSRError> {
    let mut runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions::default());

    let all_js = fastn_js::all_js_with_test();
    let js = format!("{DELETE_DENO}{all_js}{js}");

    eval::<String>(&mut runtime, deno_core::FastString::from(js))
}

fn eval<T: deno_core::serde::Deserialize<'static>>(
    context: &mut deno_core::JsRuntime,
    code: deno_core::FastString,
) -> Result<T, SSRError> {
    let res = context.execute_script("<anon>", code);
    match res {
        Ok(global) => {
            let scope = &mut context.handle_scope();
            let local = deno_core::v8::Local::new(scope, global);
            // Deserialize a `v8` object into a Rust type using `serde_v8`,
            // in this case deserialize to a JSON `Value`.
            let deserialized_value = deno_core::serde_v8::from_v8::<T>(scope, local);

            match deserialized_value {
                Ok(value) => Ok(value),
                Err(err) => Err(SSRError::DeserializeError(format!(
                    "Cannot deserialize value: {:?}",
                    err
                ))),
            }
        }
        Err(err) => Err(SSRError::EvalError(format!("Evaling error: {:?}", err))),
    }
}

pub fn ssr(ast: &[fastn_js::Ast]) -> Result<String, SSRError> {
    let js = ssr_raw_string("foo", fastn_js::to_js(ast, "foo").as_str());
    ssr_str(&js)
}

pub fn ssr_with_js_string(package_name: &str, js: &str) -> Result<String, SSRError> {
    let js = ssr_raw_string(package_name, js);
    ssr_str(&js)
}

pub fn ssr_raw_string(package_name: &str, js: &str) -> String {
    format!("
        let __fastn_package_name__ = \"{}\";\n{}
        let main_wrapper = function(parent) {{
            let parenti0 = fastn_dom.createKernel(parent, fastn_dom.ElementKind.Column);
            parenti0.setProperty(fastn_dom.PropertyKind.Width, fastn_dom.Resizing.FillContainer, inherited);
            parenti0.setProperty(fastn_dom.PropertyKind.Height, fastn_dom.Resizing.FillContainer, inherited);
            main(parenti0);
        }};
        fastnVirtual.ssr(main_wrapper);", package_name, js)
}

pub fn ssr_raw_string_without_test(package_name: &str, js: &str) -> String {
    let all_js = fastn_js::all_js_without_test_and_ftd_langugage_js();
    let raw_string = ssr_raw_string(package_name, js);
    format!("{all_js}{raw_string}")
}
