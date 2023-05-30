pub trait StoreExtractor<T> {
    fn extract<'a>(store: &'a mut wasmtime::Caller<T>) -> &'a mut Self;
}

pub trait WasmType {
    fn extract(idx: usize, vals: &[wasmtime::Val]) -> Self;
    fn the_type() -> wasmtime::ValType;
    fn to_wasm(&self) -> wasmtime::Val;
}

impl fastn_wasm::WasmType for f32 {
    fn extract(idx: usize, vals: &[wasmtime::Val]) -> f32 {
        vals[idx].f32().unwrap()
    }
    fn the_type() -> wasmtime::ValType {
        wasmtime::ValType::F32
    }
    fn to_wasm(&self) -> wasmtime::Val {
        (*self).into()
    }
}

impl fastn_wasm::WasmType for bool {
    fn extract(idx: usize, vals: &[wasmtime::Val]) -> bool {
        vals[idx].i32().unwrap() != 0
    }
    fn the_type() -> wasmtime::ValType {
        wasmtime::ValType::I32
    }
    fn to_wasm(&self) -> wasmtime::Val {
        wasmtime::Val::I32(*self as i32)
    }
}

impl fastn_wasm::WasmType for i32 {
    fn extract(idx: usize, vals: &[wasmtime::Val]) -> i32 {
        vals[idx].i32().unwrap()
    }
    fn the_type() -> wasmtime::ValType {
        wasmtime::ValType::I32
    }
    fn to_wasm(&self) -> wasmtime::Val {
        wasmtime::Val::I32(*self)
    }
}

impl fastn_wasm::WasmType for wasmtime::ExternRef {
    fn extract(idx: usize, vals: &[wasmtime::Val]) -> wasmtime::ExternRef {
        vals[idx].externref().unwrap().unwrap()
    }
    fn the_type() -> wasmtime::ValType {
        wasmtime::ValType::ExternRef
    }
    fn to_wasm(&self) -> wasmtime::Val {
        wasmtime::Val::ExternRef(Some(self.to_owned()))
    }
}

pub trait LinkerExt<S> {
    fn func0<SE: StoreExtractor<S>>(
        &mut self,
        name: &str,
        func: impl Fn(&mut SE) + Send + Sync + 'static,
    );
    // fn func1<T: ParamExtractor>(&mut self, name: &str, func: impl Fn(&mut fastn_runtime::Memory, T) + Send + Sync + 'static);
    fn func1<SE: StoreExtractor<S>, T: WasmType>(
        &mut self,
        name: &str,
        func: impl Fn(&mut SE, T) + Send + Sync + 'static,
    );
    fn func2<SE: StoreExtractor<S>, T1: WasmType, T2: WasmType>(
        &mut self,
        name: &str,
        func: impl Fn(&mut SE, T1, T2) + Send + Sync + 'static,
    );
    fn func3<SE: StoreExtractor<S>, T1: WasmType, T2: WasmType, T3: WasmType>(
        &mut self,
        name: &str,
        func: impl Fn(&mut SE, T1, T2, T3) + Send + Sync + 'static,
    );
    fn func4caller<T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType>(
        &mut self,
        name: &str,
        func: impl Fn(wasmtime::Caller<'_, S>, T1, T2, T3, T4) + Send + Sync + 'static,
    );
    fn func0ret<SE: StoreExtractor<S>, O: WasmType>(
        &mut self,
        name: &str,
        func: impl Fn(&mut SE) -> O + Send + Sync + 'static,
    );
    fn func1ret<SE: StoreExtractor<S>, T: WasmType, O: WasmType>(
        &mut self,
        name: &str,
        func: impl Fn(&mut SE, T) -> O + Send + Sync + 'static,
    );
    fn func2ret<SE: StoreExtractor<S>, T1: WasmType, T2: WasmType, O: WasmType>(
        &mut self,
        name: &str,
        func: impl Fn(&mut SE, T1, T2) -> O + Send + Sync + 'static,
    );
    fn func4ret<
        SE: StoreExtractor<S>,
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

impl<S> LinkerExt<S> for wasmtime::Linker<S> {
    fn func0<SE: StoreExtractor<S>>(
        &mut self,
        name: &str,
        func: impl Fn(&mut SE) + Send + Sync + 'static,
    ) {
        self.func_new(
            "fastn",
            name,
            wasmtime::FuncType::new([].iter().cloned(), [].iter().cloned()),
            move |mut caller: wasmtime::Caller<'_, S>, _params, _results| {
                func(SE::extract(&mut caller));
                Ok(())
            },
        )
        .unwrap();
    }
    fn func1<SE: StoreExtractor<S>, T: WasmType>(
        &mut self,
        name: &str,
        func: impl Fn(&mut SE, T) + Send + Sync + 'static,
    ) {
        self.func_new(
            "fastn",
            name,
            wasmtime::FuncType::new([T::the_type()].iter().cloned(), [].iter().cloned()),
            move |mut caller: wasmtime::Caller<'_, S>, params, _results| {
                func(SE::extract(&mut caller), T::extract(0, params));
                Ok(())
            },
        )
        .unwrap();
    }
    fn func2<SE: StoreExtractor<S>, T1: WasmType, T2: WasmType>(
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
            move |mut caller: wasmtime::Caller<'_, S>, params, _results| {
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
    fn func3<SE: StoreExtractor<S>, T1: WasmType, T2: WasmType, T3: WasmType>(
        &mut self,
        name: &str,
        func: impl Fn(&mut SE, T1, T2, T3) + Send + Sync + 'static,
    ) {
        self.func_new(
            "fastn",
            name,
            wasmtime::FuncType::new(
                [T1::the_type(), T2::the_type()].iter().cloned(),
                [].iter().cloned(),
            ),
            move |mut caller: wasmtime::Caller<'_, S>, params, _results| {
                func(
                    SE::extract(&mut caller),
                    T1::extract(0, params),
                    T2::extract(1, params),
                    T3::extract(2, params),
                );
                Ok(())
            },
        )
        .unwrap();
    }
    fn func4caller<T1: WasmType, T2: WasmType, T3: WasmType, T4: WasmType>(
        &mut self,
        name: &str,
        func: impl Fn(wasmtime::Caller<'_, S>, T1, T2, T3, T4) + Send + Sync + 'static,
    ) {
        self.func_new(
            "fastn",
            name,
            wasmtime::FuncType::new(
                [T1::the_type(), T2::the_type()].iter().cloned(),
                [].iter().cloned(),
            ),
            move |caller: wasmtime::Caller<'_, S>, params, _results| {
                func(
                    caller,
                    T1::extract(0, params),
                    T2::extract(1, params),
                    T3::extract(2, params),
                    T4::extract(3, params),
                );
                Ok(())
            },
        )
        .unwrap();
    }
    fn func0ret<SE: StoreExtractor<S>, O: WasmType>(
        &mut self,
        name: &str,
        func: impl Fn(&mut SE) -> O + Send + Sync + 'static,
    ) {
        self.func_new(
            "fastn",
            name,
            wasmtime::FuncType::new([O::the_type()].iter().cloned(), [].iter().cloned()),
            move |mut caller: wasmtime::Caller<'_, S>, _params, results| {
                results[0] = func(SE::extract(&mut caller)).to_wasm();
                Ok(())
            },
        )
        .unwrap();
    }
    fn func1ret<SE: StoreExtractor<S>, T: WasmType, O: WasmType>(
        &mut self,
        name: &str,
        func: impl Fn(&mut SE, T) -> O + Send + Sync + 'static,
    ) {
        self.func_new(
            "fastn",
            name,
            wasmtime::FuncType::new(
                [T::the_type()].iter().cloned(),
                [O::the_type()].iter().cloned(),
            ),
            move |mut caller: wasmtime::Caller<'_, S>, params, results| {
                results[0] = func(SE::extract(&mut caller), T::extract(0, params)).to_wasm();
                Ok(())
            },
        )
        .unwrap();
    }
    fn func2ret<SE: StoreExtractor<S>, T1: WasmType, T2: WasmType, O: WasmType>(
        &mut self,
        name: &str,
        func: impl Fn(&mut SE, T1, T2) -> O + Send + Sync + 'static,
    ) {
        self.func_new(
            "fastn",
            name,
            wasmtime::FuncType::new(
                [T1::the_type(), T2::the_type()].iter().cloned(),
                [O::the_type()].iter().cloned(),
            ),
            move |mut caller: wasmtime::Caller<'_, S>, params, results| {
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
        SE: StoreExtractor<S>,
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
                [O::the_type()].iter().cloned(),
            ),
            move |mut caller: wasmtime::Caller<'_, S>, params, results| {
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
