#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[wasm_bindgen::prelude::wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen::prelude::wasm_bindgen(js_namespace = fastn)]
    fn doc_main();

    #[wasm_bindgen::prelude::wasm_bindgen(js_namespace = fastn)]
    fn call_by_index();

    #[wasm_bindgen::prelude::wasm_bindgen(js_namespace = fastn)]
    fn void_by_index();
}
