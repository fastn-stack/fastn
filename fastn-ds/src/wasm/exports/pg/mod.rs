mod batch_execute;
mod connect;
mod db_error;
mod execute;
mod query;

pub use batch_execute::batch_execute;
pub use connect::connect;
pub use db_error::pg_to_shared;
pub use execute::execute;
pub use query::query;

#[derive(serde::Deserialize, Debug)]
pub struct Query {
    sql: String,
    binds: Vec<Bind>,
}

#[derive(serde::Deserialize, Debug)]
struct Bind(u32, Option<Vec<u8>>);

impl tokio_postgres::types::ToSql for Bind {
    fn to_sql(
        &self,
        _ty: &tokio_postgres::types::Type,
        out: &mut bytes::BytesMut,
    ) -> Result<tokio_postgres::types::IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        if let Some(ref bytes) = self.1 {
            out.extend_from_slice(bytes);
            Ok(tokio_postgres::types::IsNull::No)
        } else {
            Ok(tokio_postgres::types::IsNull::Yes)
        }
    }

    fn accepts(_ty: &tokio_postgres::types::Type) -> bool
    where
        Self: Sized,
    {
        true
    }

    fn to_sql_checked(
        &self,
        ty: &tokio_postgres::types::Type,
        out: &mut bytes::BytesMut,
    ) -> Result<tokio_postgres::types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        let from_oid = tokio_postgres::types::Type::from_oid(self.0);
        if let Some(ref from_oid) = from_oid {
            if ty == &tokio_postgres::types::Type::VARCHAR
                && from_oid == &tokio_postgres::types::Type::TEXT
            {
                println!("treating TEXT and VARCHAR as same");
                return self.to_sql(ty, out);
            }
        }

        if from_oid.map(|d| ty != &d).unwrap_or(false) {
            return Err(Box::new(tokio_postgres::types::WrongType::new::<Self>(
                ty.clone(),
            )));
        }
        self.to_sql(ty, out)
    }
}
