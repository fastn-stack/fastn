mod connect;
pub use connect::connect;

mod query;
pub use query::query;

mod batch_execute;
pub use batch_execute::batch_execute;

mod execute;
pub use execute::execute;
use fastn_ds::wasm::exports::sqlite::query::Value;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Query {
    sql: String,
    binds: Vec<Value>,
}
