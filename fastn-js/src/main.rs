fn main() {
    let start = std::time::Instant::now();
    println!("{}", fastn_js::ssr_str(js()));
    println!("elapsed: {:?}", start.elapsed());

    let start = std::time::Instant::now();
    println!("{}", fastn_js::ssr(&js_constructor()));
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

        fastn_virtual.ssr(main)
    "#
}

fn js_constructor() -> Vec<fastn_js::Func> {
    vec![fastn_js::func0("main", vec![])]
}
