pub fn ssr_str(js: &str) -> String {
    let all_js = fastn_js::all_js();
    dbg!(&js);
    let js = format!("{all_js}{js}");

    #[cfg(target_os = "windows")]
    {
        rquickjs::Context::full(&rquickjs::Runtime::new().unwrap())
            .unwrap()
            .with(|ctx| ctx.eval::<String, _>(js).unwrap())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let context = quick_js::Context::new().unwrap();
        context.eval_as::<String>(js.as_str()).unwrap()
    }
}

pub fn ssr(ast: &[fastn_js::Ast]) -> String {
    let js = format!("{}\nfastn_virtual.ssr(main)", fastn_js::to_js(ast));
    ssr_str(&js)
}
