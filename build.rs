fn main() {
    // https://docs.rs/diesel_migrations/latest/diesel_migrations/macro.embed_migrations.html#automatic-rebuilds
    println!("cargo:rerun-if-changed=migrations");
}
