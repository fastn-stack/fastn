pub async fn close_cr(config: &fpm::Config, cr: &str) -> fpm::Result<()> {
    let cr = cr.parse::<usize>()?;
    let cr_about = fpm::cr::get_cr_meta(config, cr).await?.unset_open();
    fpm::cr::create_cr_meta(config, &cr_about).await?;
    Ok(())
}
