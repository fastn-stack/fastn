pub async fn close_cr(config: &fastn_core::Config, cr: &str) -> fastn_core::Result<()> {
    let cr = cr.parse::<usize>()?;
    let cr_about = fastn_core::cr::get_cr_meta(config, cr).await?.unset_open();
    fastn_core::cr::create_cr_meta(config, &cr_about).await?;
    Ok(())
}
