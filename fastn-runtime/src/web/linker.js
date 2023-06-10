(function() {
    const RUNTIME_WASM = "/-/runtime.wasm";

    let fastn = {
        runtime_instance: null,
        doc_instance: null,
        importObject: {}
    };

    function init(doc) {
        if (!!window.WebAssembly) {
            console.log("browser does not support WebAssembly");
            return;
        }

        WebAssembly.instantiateStreaming(fetch(RUNTIME_WASM), fastn.import_object).then(
          function(obj) {
            fastn.runtime_instance = obj.instance;
            continue_after_instance();
          }
        );

        WebAssembly.instantiateStreaming(fetch("doc.wasm"), fastn.import_object).then(
          function(obj) {
            fastn.doc_instance = obj.instance;
            continue_after_instance();
          }
        );
    }

    function continue_after_instance() {
        if (fastn.runtime_instance == null || fastn.doc_instance == null) {
            if (!!fastn.runtime_instance) {
                console.log("waiting for doc.wasm to load");
                return;
            } else {
                console.log("waiting for runtime.wasm to load");
                return;
            }
        }

        console.log("both instances are ready");
        // we first initialise the runtime_instance (so Memory struct gets created).
        fastn.runtime_instance.exports.main();
        fastn.doc_instance.exports.main();
    }

    window.fastn = fastn;
})()