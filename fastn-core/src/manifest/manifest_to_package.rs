impl fastn_core::Manifest {
    pub async fn to_package(
        &self,
        package_root: &fastn_ds::Path,
        ds: &fastn_ds::DocumentStore,
    ) -> fastn_core::Result<fastn_core::Package> {
        let mut package = fastn_core::Package::new("");
        package.resolve(&package_root.join("FASTN.ftd"), ds).await?;

        Ok(package)
    }
}
