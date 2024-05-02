use magic_crypt::MagicCryptTrait;

pub async fn encrypt(
    mut caller: wasmtime::Caller<'_, fastn_ds::wasm::Store>,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<i32> {
    let input = fastn_ds::wasm::helpers::get_str(ptr, len, &mut caller)?;
    #[cfg(feature = "hostn")]
    let secret_key = ft_aws::fastn_secret_key();
    #[cfg(not(feature = "hostn"))]
    let secret_key = std::env::var("FASTN_SECRET_KEY").unwrap();
    let mc_obj = magic_crypt::new_magic_crypt!(secret_key, 256);
    let o = mc_obj.encrypt_to_base64(input.as_str()).as_str().to_owned();
    fastn_ds::wasm::helpers::send_bytes(&o.into_bytes(), &mut caller).await
}

pub async fn decrypt(
    mut caller: wasmtime::Caller<'_, fastn_ds::wasm::Store>,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<i32> {
    let input = fastn_ds::wasm::helpers::get_str(ptr, len, &mut caller)?;
    #[cfg(feature = "hostn")]
    let secret_key = ft_aws::fastn_secret_key();
    #[cfg(not(feature = "hostn"))]
    let secret_key = std::env::var("FASTN_SECRET_KEY").unwrap();
    let mc_obj = magic_crypt::new_magic_crypt!(secret_key, 256);
    let o = mc_obj
        .decrypt_base64_to_string(input)
        .map_err(|e| ft_sys_shared::DecryptionError::Generic(format!("{e:?}")));
    fastn_ds::wasm::helpers::send_json(o, &mut caller).await
}
