#[derive(thiserror::Error, Debug)]
pub enum CreateError {}

#[derive(thiserror::Error, Debug)]
pub enum UpdateError {}

pub async fn create() -> Result<(), CreateError> {
    // call fastn build
    // read the content of the .build folder
    // pass this content to tejar and create two files LIST and DATA
    // call /api/create/ by passing the content of the LIST and META
    // call /api/upload-new-package by passing the missing entries and DATA
    // save package key and at home folder
    // show the subdomain to user or open browser directly
    println!("publish-static create called");
    Ok(())
}

pub async fn update() -> Result<(), UpdateError> {
    println!("publish-static update called");
    Ok(())
}
