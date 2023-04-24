#[derive(thiserror::Error, Debug)]
pub enum InitializeDBError {}

pub(crate) fn initialize_db() -> Result<(), InitializeDBError> {
    todo!()
}
