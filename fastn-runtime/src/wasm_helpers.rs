pub trait StoreExtractor {
    fn extract<'a>(store: &'a mut wasmtime::Caller<fastn_runtime::Dom>) -> &'a mut Self;
}

impl StoreExtractor for fastn_runtime::Memory {
    fn extract<'a>(store: &'a mut wasmtime::Caller<fastn_runtime::Dom>) -> &'a mut Self {
        store.data_mut().memory_mut()
    }
}

impl StoreExtractor for fastn_runtime::Dom {
    fn extract<'a>(store: &'a mut wasmtime::Caller<fastn_runtime::Dom>) -> &'a mut Self {
        store.data_mut()
    }
}

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
        (*self).into()
    }
}

impl WasmType for fastn_runtime::dom::ElementKind {
    fn extract(idx: usize, vals: &[wasmtime::Val]) -> fastn_runtime::dom::ElementKind {
        fastn_runtime::dom::ElementKind::from(vals.i32(idx))
    }
    fn the_type() -> wasmtime::ValType {
        wasmtime::ValType::I32
    }
    fn to_wasm(&self) -> wasmtime::Val {
        let i: i32 = (*self).into();
        i.into()
    }
}

impl WasmType for f32 {
    fn extract(idx: usize, vals: &[wasmtime::Val]) -> f32 {
        vals.f32(idx)
    }
    fn the_type() -> wasmtime::ValType {
        wasmtime::ValType::F32
    }
    fn to_wasm(&self) -> wasmtime::Val {
        (*self).into()
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
    fn func0<SE: StoreExtractor>(
        &mut self,
        name: &str,
        func: impl Fn(&mut SE) + Send + Sync + 'static,
    );
    // fn func1<T: ParamExtractor<T>>(&mut self, name: &str, func: impl Fn(&mut fastn_runtime::Memory, T) + Send + Sync + 'static);
    fn func1<SE: StoreExtractor, T: WasmType>(
        &mut self,
        name: &str,
        func: impl Fn(&mut SE, T) + Send + Sync + 'static,
    );
    fn func2<SE: StoreExtractor, T1: WasmType, T2: WasmType>(
        &mut self,
        name: &str,
        func: impl Fn(&mut SE, T1, T2) + Send + Sync + 'static,
    );
    fn func0ret<SE: StoreExtractor, O: WasmType>(
        &mut self,
        name: &str,
        func: impl Fn(&mut SE) -> O + Send + Sync + 'static,
    );
    fn func1ret<SE: StoreExtractor, T: WasmType, O: WasmType>(
        &mut self,
        name: &str,
        func: impl Fn(&mut SE, T) -> O + Send + Sync + 'static,
    );
    fn func2ret<SE: StoreExtractor, T1: WasmType, T2: WasmType, O: WasmType>(
        &mut self,
        name: &str,
        func: impl Fn(&mut SE, T1, T2) -> O + Send + Sync + 'static,
    );
    fn func4ret<
        SE: StoreExtractor,
        T1: WasmType,
        T2: WasmType,
        T3: WasmType,
        T4: WasmType,
        O: WasmType,
    >(
        &mut self,
        name: &str,
        func: impl Fn(&mut SE, T1, T2, T3, T4) -> O + Send + Sync + 'static,
    );
}

impl LinkerExt for wasmtime::Linker<fastn_runtime::Dom> {
    fn func0<T: StoreExtractor>(
        &mut self,
        name: &str,
        func: impl Fn(&mut T) + Send + Sync + 'static,
    ) {
        self.func_new(
            "fastn",
            name,
            wasmtime::FuncType::new([].iter().cloned(), [].iter().cloned()),
            move |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, _params, _results| {
                func(T::extract(&mut caller));
                Ok(())
            },
        )
        .unwrap();
    }
    fn func1<SE: StoreExtractor, T: WasmType>(
        &mut self,
        name: &str,
        func: impl Fn(&mut SE, T) + Send + Sync + 'static,
    ) {
        self.func_new(
            "fastn",
            name,
            wasmtime::FuncType::new([T::the_type()].iter().cloned(), [].iter().cloned()),
            move |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, params, _results| {
                func(SE::extract(&mut caller), T::extract(0, params));
                Ok(())
            },
        )
        .unwrap();
    }
    // fn func2<SE: StoreExtractor, T1: WasmType, T2: WasmType>(
    //     &mut self,
    //     name: &str,
    //     func: impl Fn(&mut SE, &wasmtime::Val, &wasmtime::Val)
    //     + Send
    //     + Sync
    //     + 'static,
    // );

    fn func2<SE: StoreExtractor, T1: WasmType, T2: WasmType>(
        &mut self,
        name: &str,
        func: impl Fn(&mut SE, T1, T2) + Send + Sync + 'static,
    ) {
        self.func_new(
            "fastn",
            name,
            wasmtime::FuncType::new(
                [T1::the_type(), T2::the_type()].iter().cloned(),
                [].iter().cloned(),
            ),
            move |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, params, _results| {
                func(
                    SE::extract(&mut caller),
                    T1::extract(0, params),
                    T2::extract(1, params),
                );
                Ok(())
            },
        )
        .unwrap();
    }
    fn func0ret<SE: StoreExtractor, O: WasmType>(
        &mut self,
        name: &str,
        func: impl Fn(&mut SE) -> O + Send + Sync + 'static,
    ) {
        self.func_new(
            "fastn",
            name,
            wasmtime::FuncType::new([].iter().cloned(), [].iter().cloned()),
            move |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, _params, results| {
                results[0] = func(SE::extract(&mut caller)).to_wasm();
                Ok(())
            },
        )
        .unwrap();
    }
    fn func1ret<SE: StoreExtractor, T: WasmType, O: WasmType>(
        &mut self,
        name: &str,
        func: impl Fn(&mut SE, T) -> O + Send + Sync + 'static,
    ) {
        self.func_new(
            "fastn",
            name,
            wasmtime::FuncType::new([T::the_type()].iter().cloned(), [].iter().cloned()),
            move |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, params, results| {
                results[0] = func(SE::extract(&mut caller), T::extract(0, params)).to_wasm();
                Ok(())
            },
        )
        .unwrap();
    }
    fn func2ret<SE: StoreExtractor, T1: WasmType, T2: WasmType, O: WasmType>(
        &mut self,
        name: &str,
        func: impl Fn(&mut SE, T1, T2) -> O + Send + Sync + 'static,
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
                    SE::extract(&mut caller),
                    T1::extract(0, params),
                    T2::extract(1, params),
                )
                .to_wasm();
                Ok(())
            },
        )
        .unwrap();
    }
    fn func4ret<
        SE: StoreExtractor,
        T1: WasmType,
        T2: WasmType,
        T3: WasmType,
        T4: WasmType,
        O: WasmType,
    >(
        &mut self,
        name: &str,
        func: impl Fn(&mut SE, T1, T2, T3, T4) -> O + Send + Sync + 'static,
    ) {
        self.func_new(
            "fastn",
            name,
            wasmtime::FuncType::new(
                [
                    T1::the_type(),
                    T2::the_type(),
                    T3::the_type(),
                    T4::the_type(),
                ]
                .iter()
                .cloned(),
                [].iter().cloned(),
            ),
            move |mut caller: wasmtime::Caller<'_, fastn_runtime::Dom>, params, results| {
                results[0] = func(
                    SE::extract(&mut caller),
                    T1::extract(0, params),
                    T2::extract(1, params),
                    T3::extract(2, params),
                    T4::extract(3, params),
                )
                .to_wasm();
                Ok(())
            },
        )
        .unwrap();
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
