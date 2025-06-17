#[derive(thiserror::Error, Debug)]
pub enum SSRError {
    #[error("Error executing JavaScript: {0}")]
    EvalError(String),

    #[error("Error deserializing value: {0}")]
    DeserializeError(String),
}

type Result<T> = std::result::Result<T, SSRError>;

pub fn run_test(js: &str) -> Result<Vec<bool>> {
    #[cfg(target_os = "windows")]
    {
        Ok(rquickjs::Context::full(&rquickjs::Runtime::new().unwrap())
            .unwrap()
            .with(|ctx| ctx.eval::<Vec<bool>, _>(js).unwrap()))
    }
    #[cfg(not(target_os = "windows"))]
    {
        // Added logging support from console from within context
        let context = quick_js::Context::builder()
            .console(
                |level: quick_js::console::Level, args: Vec<quick_js::JsValue>| {
                    eprintln!("{}: {:?}", level, args);
                },
            )
            .build()
            .unwrap();
        Ok::<Vec<bool>, SSRError>(context.eval_as::<Vec<bool>>(js).unwrap())
    }
}

pub fn ssr_str(js: &str) -> Result<Vec<String>> {
    let all_js = fastn_js::all_js_with_test();

    let js = format!("{all_js}{js}");

    #[cfg(target_os = "windows")]
    {
        Ok(rquickjs::Context::full(&rquickjs::Runtime::new().unwrap())
            .unwrap()
            .with(|ctx| ctx.eval::<Vec<String>, _>(js).unwrap()))
    }
    #[cfg(not(target_os = "windows"))]
    {
        // Added logging support from console from within context
        let context = quick_js::Context::builder()
            .console(
                |level: quick_js::console::Level, args: Vec<quick_js::JsValue>| {
                    eprintln!("{}: {:?}", level, args);
                },
            )
            .build()
            .unwrap();
        Ok::<_, SSRError>(context.eval_as::<Vec<String>>(js.as_str()).unwrap())
    }
}

pub fn ssr(ast: &[fastn_js::Ast]) -> Result<Vec<String>> {
    let js = ssr_raw_string("foo", fastn_js::to_js(ast, "foo").as_str());
    ssr_str(&js)
}

/// Returns (ssr_body, meta_tags)
pub fn ssr_with_js_string(package_name: &str, js: &str) -> Result<(String, String)> {
    let js = ssr_raw_string(package_name, js);
    let ssr_res = ssr_str(&js)?;

    assert_eq!(
        ssr_res.len(),
        2,
        "ssr_with_js_string executes js `ssr` function somewhere down the line which always returns an array of 2 elems"
    );

    let mut ssr_res = ssr_res.into_iter();

    Ok((
        ssr_res.next().expect("vec has at least 2 items"),
        ssr_res.next().expect("vec has at least 2 items"),
    ))
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
