impl fastn_ds::wasm::Store {
    pub fn register_functions(&self, linker: &mut wasmtime::Linker<fastn_ds::wasm::Store>) {
        // general utility functions
        fastn_ds::func2!(linker, "env_print", fastn_ds::wasm::exports::env::print);
        fastn_ds::func0ret!(linker, "env_now", fastn_ds::wasm::exports::env::now);
        fastn_ds::func0ret!(linker, "env_ud", fastn_ds::wasm::exports::env::ud);
        fastn_ds::func2ret!(linker, "env_var", fastn_ds::wasm::exports::env::var);
        fastn_ds::func0ret!(linker, "env_random", fastn_ds::wasm::exports::env::random);

        // cryptography related stuff
        fastn_ds::func2ret!(
            linker,
            "crypto_encrypt",
            fastn_ds::wasm::exports::crypto::encrypt
        );
        fastn_ds::func2ret!(
            linker,
            "crypto_decrypt",
            fastn_ds::wasm::exports::crypto::decrypt
        );

        // sqlite
        fastn_ds::func2ret!(
            linker,
            "sqlite_connect",
            fastn_ds::wasm::exports::sqlite::connect
        );
        fastn_ds::func3ret!(
            linker,
            "sqlite_query",
            fastn_ds::wasm::exports::sqlite::query
        );

        // pg related stuff
        fastn_ds::func2ret!(linker, "pg_connect", fastn_ds::wasm::exports::pg::connect);
        fastn_ds::func3ret!(linker, "pg_query", fastn_ds::wasm::exports::pg::query);
        fastn_ds::func3ret!(linker, "pg_execute", fastn_ds::wasm::exports::pg::execute);
        fastn_ds::func3ret!(
            linker,
            "pg_batch_execute",
            fastn_ds::wasm::exports::pg::batch_execute
        );

        // request related stuff
        fastn_ds::func0ret!(
            linker,
            "http_get_request",
            fastn_ds::wasm::exports::http::get_request
        );

        fastn_ds::func2ret!(
            linker,
            "http_send_request",
            fastn_ds::wasm::exports::http::send_request
        );

        fastn_ds::func2!(
            linker,
            "http_send_response",
            fastn_ds::wasm::exports::http::send_response
        );
        fastn_ds::func2!(
            linker,
            "http_send_ftd",
            fastn_ds::wasm::exports::http::send_ftd
        );

        // document store related
        fastn_ds::func2ret!(
            linker,
            "hostn_tejar_write",
            fastn_ds::wasm::exports::ds::tejar_write
        );

        fastn_ds::func2ret!(
            linker,
            "hostn_tejar_read",
            fastn_ds::wasm::exports::ds::tejar_read
        );

        // aws
        fastn_ds::func2ret!(
            linker,
            "hostn_aws_pre_signed_request",
            fastn_ds::wasm::exports::aws::pre_signed_request
        );
    }
}
