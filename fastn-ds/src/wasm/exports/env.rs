pub async fn print(
    mut caller: wasmtime::Caller<'_, fastn_ds::wasm::Store>,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<()> {
    println!(
        "wasm: {}",
        fastn_ds::wasm::helpers::get_str(ptr, len, &mut caller).await?
    );

    Ok(())
}

pub async fn var(
    mut caller: wasmtime::Caller<'_, fastn_ds::wasm::Store>,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<i32> {
    let key = fastn_ds::wasm::helpers::get_str(ptr, len, &mut caller).await?;
    let value = std::env::var(key).ok();

    fastn_ds::wasm::helpers::send_json(value, &mut caller).await
}

pub async fn now(mut caller: wasmtime::Caller<'_, fastn_ds::wasm::Store>) -> wasmtime::Result<i32> {
    fastn_ds::wasm::helpers::send_json(chrono::Utc::now(), &mut caller).await
}

pub async fn ud(mut caller: wasmtime::Caller<'_, fastn_ds::wasm::Store>) -> wasmtime::Result<i32> {
    fastn_ds::wasm::helpers::send_json(caller.data().get_ud().await, &mut caller).await
}

fn random_seed() -> f64 {
    rand::random::<f64>()
}

pub async fn random(
    mut caller: wasmtime::Caller<'_, fastn_ds::wasm::Store>,
) -> wasmtime::Result<i32> {
    fastn_ds::wasm::helpers::send_json(random_seed(), &mut caller).await
}

impl<'a> fastn_ds::wasm::Store {
    async fn get_ud(&self) -> Option<ft_sys_shared::UserData> {
        if let Some(v) = self.ud.clone() {
            return Some(v);
        }

        #[cfg(feature = "hostn")]
        return get_debug_user(self.pool.as_ref().unwrap()).await;

        #[cfg(not(feature = "hostn"))]
        None
    }
}

#[cfg(feature = "hostn")]
async fn get_debug_user(pool: &ft_db::PgPool) -> Option<ft_sys_shared::UserData> {
    use ft_db::prelude::*;
    use ft_db::schema::ft_user;

    let username = match std::env::var("DEBUG_LOGGED_IN") {
        Ok(v) => {
            println!("using DEBUG_LOGGED_IN: {v}, should happen during development");
            v
        }
        Err(_) => return None,
    };

    let mut conn = pool.get().await.unwrap();

    let (id, name, email) = ft_user::table
        .select((ft_user::id, ft_user::name, ft_user::email))
        .filter(ft_user::username.eq(username.clone()))
        .first::<(i64, String, String)>(&mut conn)
        .await
        .inspect_err(|e| println!("failed to get user info: {e:?}"))
        .ok()?;

    Some(ft_sys_shared::UserData {
        id,
        username,
        name,
        email,
        verified_email: true,
    })
}
