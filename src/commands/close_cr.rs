pub async fn close_cr(config: &fastn::Config, cr: &str) -> fastn::Result<()> {
    let cr = cr.parse::<usize>()?;
    let cr_about = fastn::cr::get_cr_meta(config, cr).await?.unset_open();
    fastn::cr::create_cr_meta(config, &cr_about).await?;
    Ok(())
}
