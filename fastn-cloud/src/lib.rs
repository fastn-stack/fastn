#[derive(thiserror::Error, Debug)]
pub enum CreateError {}

#[derive(thiserror::Error, Debug)]
pub enum UpdateError {}

pub async fn create() -> Result<(), CreateError> {
    println!("publish-static create called");
    Ok(())
}

pub async fn update() -> Result<(), UpdateError> {
    println!("publish-static update called");
    Ok(())
}
