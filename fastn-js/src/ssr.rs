pub fn ssr_str(js: &str, doc_name: &str) -> String {
    let all_js = fastn_js::all_js_with_test();
    let js = format!("{all_js}{js}");

    #[cfg(target_os = "windows")]
    {
        rquickjs::Context::full(&rquickjs::Runtime::new().unwrap())
            .unwrap()
            .with(|ctx| ctx.eval::<String, _>(js).unwrap())
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
        context
            .eval_as::<String>(js.as_str())
            .map_err(|e| panic!("SSR Error: {}, doc_id: {}", e, doc_name))
            .unwrap()
    }
}

pub fn ssr(ast: &[fastn_js::Ast]) -> String {
    let js = format!("{}\nlet main_wrapper = function (parent) {{
            let parenti0 = fastn_dom.createKernel(parent, fastn_dom.ElementKind.Column);
            parenti0.setProperty(fastn_dom.PropertyKind.Width, fastn_dom.Resizing.FillContainer, inherited);
            parenti0.setProperty(fastn_dom.PropertyKind.Height, fastn_dom.Resizing.FillContainer, inherited);
            main(parenti0);
        }};
        fastnVirtual.ssr(main_wrapper);", fastn_js::to_js(ast,
                                                                                    "foo"));
    ssr_str(&js, "foo")
}

pub fn ssr_with_js_string(package_name: &str, js: &str, doc_name: &str) -> String {
    let js = format!("
        let __fastn_package_name__ = \"{}\";\n{}
        let main_wrapper = function(parent) {{
            let parenti0 = fastn_dom.createKernel(parent, fastn_dom.ElementKind.Column);
            parenti0.setProperty(fastn_dom.PropertyKind.Width, fastn_dom.Resizing.FillContainer, inherited);
            parenti0.setProperty(fastn_dom.PropertyKind.Height, fastn_dom.Resizing.FillContainer, inherited);
            main(parenti0);
        }};
        fastnVirtual.ssr(main_wrapper);", package_name, js);

    ssr_str(&js, doc_name)
}
