mod connect;
pub use connect::connect;

mod query;
pub use query::query;

mod batch_execute;
pub use batch_execute::batch_execute;

mod execute;
pub use execute::execute;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Query {
    sql: String,
    binds: Vec<ft_sys_shared::SqliteRawValue>,
}
