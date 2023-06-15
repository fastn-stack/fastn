fn main() {
    let start = std::time::Instant::now();
    println!("{}", fastn_js_runtime::ssr(js()));
    println!("elapsed: {:?}", start.elapsed());
}

fn js() -> &'static str {
    return r#"
        function main (root) {
            let number = 10;
            let i = fastn_dom.createKernel(root, fastn_dom.ElementKind.Integer);
            i.setStaticProperty(fastn_dom.PropertyKind.IntegerValue, number);
            i.done();
        }

        fastn_virtual.ssr(main)
    "#;
}
