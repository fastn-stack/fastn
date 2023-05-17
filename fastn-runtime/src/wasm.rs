pub struct Wasm {
    store: wasmtime::Store<Vec<fastn_runtime::operation::Operation>>,
    instance: wasmtime::Instance,
}

impl Wasm {
    pub fn new(wat: impl AsRef<[u8]>) -> Wasm {
        let engine = wasmtime::Engine::new(wasmtime::Config::new().async_support(false))
            .expect("cant create engine");
        let module = wasmtime::Module::new(&engine, wat).expect("cant parse module");
        let mut store = wasmtime::Store::new(&engine, Vec::new());

        let create_column = wasmtime::Func::wrap(
            &mut store,
            |mut caller: wasmtime::Caller<'_, Vec<fastn_runtime::operation::Operation>>,
             top: i32,
             left: i32,
             width: i32,
             height: i32,
             red: i32,
             blue: i32,
             green: i32|
             -> i32 {
                caller
                    .data_mut()
                    .push(fastn_runtime::operation::Operation::DrawRectangle(
                        fastn_runtime::Rectangle {
                            top: top as u32,
                            left: left as u32,
                            width: width as u32,
                            height: height as u32,
                            color: fastn_runtime::ColorValue {
                                red: red as u8,
                                blue: blue as u8,
                                green: green as u8,
                                alpha: 1.0,
                            },
                        },
                    ));

                1
            },
        );

        let instance = wasmtime::Instance::new(&mut store, &module, &[create_column.into()])
            .expect("cant create instance");
        Wasm { store, instance }
    }

    pub fn run(&mut self) -> Vec<fastn_runtime::operation::Operation> {
        let wasm_main = self
            .instance
            .get_typed_func::<(), ()>(&mut self.store, "main")
            .unwrap();

        wasm_main.call(&mut self.store, ()).unwrap();
        self.store.data().to_owned()
    }
}

pub fn t() {
    let engine = wasmtime::Engine::new(wasmtime::Config::new().async_support(false)).unwrap();
    // Next we can hook that up to a wasm module which uses it.
    let module = wasmtime::Module::new(
        &engine,
        r#"
        (module
            (import "ftd" "create_container" (func create_container (result externref)))
            (table 2 funcref)
            (elem (i32.const 0) $f1)

            (import "ftd" "add_padding_to_container" (func create_container (param externref i32) (result i32)))
            (func (export "container_padding") $f1 (result i32)
              i32.const 10
            )
            (func padding_from_x (param i32) (result i32)
              local.get 0
            )
            (func (export "document_main")
              id = ($create_container);
              (add_padding_to_container id 1)
              id2 = ($create_text);
              mut_x = (allocate_mut_integer); ;; local integer created (type: i32, location in memory)
              (add_constant_padding_to_text id2 (i32.const 0))
              (add_padding_fn_to_text_i32 id2 mut_x padding_from_x)
            )
            (func (export "callIntFunc_i32") (param $i i32) (param $x i32) (result i32)
                call_indirect (type $takes_i32_returns_i32))  (global.get $x) (local.get $i)
            (func (export "callBoolFunc") (param $i i32) (result i32)
                local.get $i
                call_indirect (type $return_i32))

        )
    "#,
    )
        .unwrap();

    let mut store = wasmtime::Store::new(&engine, ());

    // Create a custom `Func` which can execute arbitrary code inside of the
    // closure.
    // map

    let get_value1 = wasmtime::Func::wrap(&mut store, |a: i32| -> i32 {
        // let mut store = wasmtime::Store::<()>::default();
        // dbg!(&store);

        dbg!("get_value1", a);
        a
    });

    let get_value2 = wasmtime::Func::wrap(&mut store, |a: i32| -> i32 {
        dbg!("get_value2", a);
        a
    });

    let instance =
        wasmtime::Instance::new(&mut store, &module, &[get_value1.into(), get_value2.into()])
            .unwrap();

    let call_add_twice = instance
        .get_typed_func::<(), i32>(&mut store, "call_add_twice")
        .unwrap();

    dbg!(call_add_twice.call(&mut store, ()).unwrap(), 10);
}
