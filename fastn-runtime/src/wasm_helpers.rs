pub trait WasmType {
    fn extract(idx: usize, vals: &[wasmtime::Val]) -> Self;
    fn the_type() -> wasmtime::ValType;
    fn to_wasm(&self) -> wasmtime::Val;
}

impl WasmType for i32 {
    fn extract(idx: usize, vals: &[wasmtime::Val]) -> i32 {
        vals.i32(idx)
    }
    fn the_type() -> wasmtime::ValType {
        wasmtime::ValType::I32
    }
    fn to_wasm(&self) -> wasmtime::Val {
        wasmtime::Val::I32(*self)
    }
}

impl WasmType for bool {
    fn extract(idx: usize, vals: &[wasmtime::Val]) -> bool {
        vals.boolean(idx)
    }
    fn the_type() -> wasmtime::ValType {
        wasmtime::ValType::I32
    }
    fn to_wasm(&self) -> wasmtime::Val {
        wasmtime::Val::I32(*self as i32)
    }
}

impl WasmType for fastn_runtime::NodeKey {
    fn extract(idx: usize, vals: &[wasmtime::Val]) -> Self {
        vals.key(idx)
    }
    fn the_type() -> wasmtime::ValType {
        wasmtime::ValType::ExternRef
    }
    fn to_wasm(&self) -> wasmtime::Val {
        wasmtime::Val::ExternRef(Some(wasmtime::ExternRef::new(*self)))
    }
}

impl WasmType for fastn_runtime::PointerKey {
    fn extract(idx: usize, vals: &[wasmtime::Val]) -> Self {
        vals.ptr(idx)
    }
    fn the_type() -> wasmtime::ValType {
        wasmtime::ValType::ExternRef
    }
    fn to_wasm(&self) -> wasmtime::Val {
        wasmtime::Val::ExternRef(Some(wasmtime::ExternRef::new(*self)))
    }
}

pub trait Params {
    fn i32(&self, idx: usize) -> i32;
    fn f32(&self, idx: usize) -> f32;
    fn key(&self, idx: usize) -> fastn_runtime::NodeKey;
    fn ptr(&self, idx: usize) -> fastn_runtime::PointerKey;
    fn boolean(&self, idx: usize) -> bool;
}

impl Params for [wasmtime::Val] {
    fn i32(&self, idx: usize) -> i32 {
        self[idx].i32().unwrap()
    }

    fn f32(&self, idx: usize) -> f32 {
        self[idx].f32().unwrap()
    }

    fn key(&self, idx: usize) -> fastn_runtime::NodeKey {
        *self[idx]
            .externref()
            .unwrap()
            .expect("externref gone?")
            .data()
            .downcast_ref()
            .unwrap()
    }
    fn ptr(&self, idx: usize) -> fastn_runtime::PointerKey {
        *self[idx]
            .externref()
            .unwrap()
            .expect("externref gone?")
            .data()
            .downcast_ref()
            .unwrap()
    }

    fn boolean(&self, idx: usize) -> bool {
        self.i32(idx) != 0
    }
}

pub trait LinkerExt {
    fn func0(
        &mut self,
        name: &str,
        func: impl Fn(&mut fastn_runtime::Memory) + Send + Sync + 'static,
    );
    // fn func1<T: ParamExtractor<T>>(&mut self, name: &str, func: impl Fn(&mut fastn_runtime::Memory, T) + Send + Sync + 'static);
    fn func1(
        &mut self,
        name: &str,
        arg1: wasmtime::ValType,
        func: impl Fn(&mut fastn_runtime::Memory, &wasmtime::Val) + Send + Sync + 'static,
    );
    fn func2(
        &mut self,
        name: &str,
        arg1: wasmtime::ValType,
        arg2: wasmtime::ValType,
        func: impl Fn(&mut fastn_runtime::Memory, &wasmtime::Val, &wasmtime::Val)
            + Send
            + Sync
            + 'static,
    );
    fn func0ret<O: WasmType>(
        &mut self,
        name: &str,
        func: impl Fn(&mut fastn_runtime::Memory) -> O + Send + Sync + 'static,
    );
    fn func1ret<T: WasmType, O: WasmType>(
        &mut self,
        name: &str,
        func: impl Fn(&mut fastn_runtime::Memory, T) -> O + Send + Sync + 'static,
    );
    fn func2ret<T1: WasmType, T2: WasmType, O: WasmType>(
        &mut self,
        name: &str,
        func: impl Fn(&mut fastn_runtime::Memory, T1, T2) -> O + Send + Sync + 'static,
    );
}

impl LinkerExt for wasmtime::Linker<fastn_runtime::Dom> {
    fn func0(
        &mut self,
        name: &str,
        func: impl Fn(&mut fastn_runtime::Memory) + Send + Sync + 'static,
    ) {
        self.func_new(
            "fastn",
            name,
            wasmtime::FuncType::new([].iter().cloned(), [].iter().cloned()),
            move |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, _params, _results| {
                func(caller.memory_mut());
                Ok(())
            },
        )
        .unwrap();
    }
    fn func1(
        &mut self,
        name: &str,
        arg1: wasmtime::ValType,
        func: impl Fn(&mut fastn_runtime::Memory, &wasmtime::Val) + Send + Sync + 'static,
    ) {
        self.func_new(
            "fastn",
            name,
            wasmtime::FuncType::new([arg1].iter().cloned(), [].iter().cloned()),
            move |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, params, _results| {
                func(caller.memory_mut(), &params[0]);
                Ok(())
            },
        )
        .unwrap();
    }
    fn func2(
        &mut self,
        name: &str,
        arg1: wasmtime::ValType,
        arg2: wasmtime::ValType,
        func: impl Fn(&mut fastn_runtime::Memory, &wasmtime::Val, &wasmtime::Val)
            + Send
            + Sync
            + 'static,
    ) {
        self.func_new(
            "fastn",
            name,
            wasmtime::FuncType::new([arg1, arg2].iter().cloned(), [].iter().cloned()),
            move |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, params, _results| {
                func(caller.memory_mut(), &params[0], &params[1]);
                Ok(())
            },
        )
        .unwrap();
    }
    fn func0ret<O: WasmType>(
        &mut self,
        name: &str,
        func: impl Fn(&mut fastn_runtime::Memory) -> O + Send + Sync + 'static,
    ) {
        self.func_new(
            "fastn",
            name,
            wasmtime::FuncType::new([].iter().cloned(), [].iter().cloned()),
            move |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, _params, results| {
                results[0] = func(caller.memory_mut()).to_wasm();
                Ok(())
            },
        )
        .unwrap();
    }
    fn func1ret<T: WasmType, O: WasmType>(
        &mut self,
        name: &str,
        func: impl Fn(&mut fastn_runtime::Memory, T) -> O + Send + Sync + 'static,
    ) {
        self.func_new(
            "fastn",
            name,
            wasmtime::FuncType::new([T::the_type()].iter().cloned(), [].iter().cloned()),
            move |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, params, results| {
                results[0] = func(caller.memory_mut(), T::extract(0, params)).to_wasm();
                Ok(())
            },
        )
        .unwrap();
    }
    fn func2ret<T1: WasmType, T2: WasmType, O: WasmType>(
        &mut self,
        name: &str,
        func: impl Fn(&mut fastn_runtime::Memory, T1, T2) -> O + Send + Sync + 'static,
    ) {
        self.func_new(
            "fastn",
            name,
            wasmtime::FuncType::new(
                [T1::the_type(), T2::the_type()].iter().cloned(),
                [].iter().cloned(),
            ),
            move |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, params, results| {
                results[0] = func(
                    caller.memory_mut(),
                    T1::extract(0, params),
                    T2::extract(1, params),
                )
                .to_wasm();
                Ok(())
            },
        )
        .unwrap();
    }
}

impl From<fastn_runtime::PointerKey> for wasmtime::Val {
    fn from(value: fastn_runtime::PointerKey) -> Self {
        wasmtime::Val::ExternRef(Some(wasmtime::ExternRef::new(value)))
    }
}

pub trait CallerExt {
    fn memory(&self) -> &fastn_runtime::Memory;
    fn memory_mut(&mut self) -> &mut fastn_runtime::Memory;
}

impl CallerExt for wasmtime::Caller<'_, fastn_runtime::Dom> {
    fn memory(&self) -> &fastn_runtime::Memory {
        self.data().memory()
    }
    fn memory_mut(&mut self) -> &mut fastn_runtime::Memory {
        self.data_mut().memory_mut()
    }
}
