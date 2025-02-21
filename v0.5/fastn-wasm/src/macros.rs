#[macro_export]
macro_rules! func0ret {
    ($linker:expr, $func_name:literal, $func:expr) => {{
        $linker
            .func_new_async(
                "env",
                $func_name,
                wasmtime::FuncType::new(
                    &fastn_wasm::WASM_ENGINE,
                    [].iter().cloned(),
                    [wasmtime::ValType::I32].iter().cloned(),
                ),
                |caller: wasmtime::Caller<'_, Self>, _params, results| {
                    Box::new(async move {
                        results[0] = wasmtime::Val::I32($func(caller).await?);
                        Ok(())
                    })
                },
            )
            .unwrap();
    }};
}

#[macro_export]
macro_rules! func2 {
    ($linker:expr, $func_name:literal, $func:expr) => {{
        $linker
            .func_new_async(
                "env",
                $func_name,
                wasmtime::FuncType::new(
                    &fastn_wasm::WASM_ENGINE,
                    [wasmtime::ValType::I32, wasmtime::ValType::I32]
                        .iter()
                        .cloned(),
                    [].iter().cloned(),
                ),
                |caller: wasmtime::Caller<'_, Self>, params, _results| {
                    Box::new(async move {
                        let v1 = params[0].i32().unwrap();
                        let v2 = params[1].i32().unwrap();
                        $func(caller, v1, v2).await?;
                        Ok(())
                    })
                },
            )
            .unwrap();
    }};
}

#[macro_export]
macro_rules! func2ret {
    ($linker:expr, $func_name:literal, $func:expr) => {{
        $linker
            .func_new_async(
                "env",
                $func_name,
                wasmtime::FuncType::new(
                    &fastn_wasm::WASM_ENGINE,
                    [wasmtime::ValType::I32, wasmtime::ValType::I32]
                        .iter()
                        .cloned(),
                    [wasmtime::ValType::I32].iter().cloned(),
                ),
                |caller: wasmtime::Caller<'_, Self>, params, results| {
                    Box::new(async move {
                        let v1 = params[0].i32().unwrap();
                        let v2 = params[1].i32().unwrap();
                        results[0] = wasmtime::Val::I32($func(caller, v1, v2).await?);
                        Ok(())
                    })
                },
            )
            .unwrap();
    }};
}

#[macro_export]
macro_rules! func3ret {
    ($linker:expr, $func_name:literal, $func:expr) => {{
        $linker
            .func_new_async(
                "env",
                $func_name,
                wasmtime::FuncType::new(
                    &fastn_wasm::WASM_ENGINE,
                    [
                        wasmtime::ValType::I32,
                        wasmtime::ValType::I32,
                        wasmtime::ValType::I32,
                    ]
                    .iter()
                    .cloned(),
                    [wasmtime::ValType::I32].iter().cloned(),
                ),
                |caller: wasmtime::Caller<'_, Self>, params, results| {
                    Box::new(async move {
                        let v1 = params[0].i32().unwrap();
                        let v2 = params[1].i32().unwrap();
                        let v3 = params[2].i32().unwrap();
                        results[0] = wasmtime::Val::I32($func(caller, v1, v2, v3).await?);
                        Ok(())
                    })
                },
            )
            .unwrap();
    }};
}
