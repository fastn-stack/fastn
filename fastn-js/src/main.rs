fn main() {
    let start = std::time::Instant::now();
    println!("{}", fastn_js::ssr_str(js()).unwrap());
    println!("elapsed: {:?}", start.elapsed());

    let start = std::time::Instant::now();
    println!("{}", fastn_js::ssr(&js_constructor()).unwrap());
    println!("elapsed: {:?}", start.elapsed());
}

fn js() -> &'static str {
    r#"
        function main (root) {
            let number = 10;
            let i = fastn_dom.createKernel(root, fastn_dom.ElementKind.Integer);
            i.setStaticProperty(fastn_dom.PropertyKind.IntegerValue, number);
            i.done();
        }

        fastnVirtual.ssr(main)
    "#
}

fn js_constructor() -> Vec<fastn_js::Ast> {
    vec![fastn_js::component0("main", vec![])]
}
