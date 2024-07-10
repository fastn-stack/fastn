impl fastn_core::Manifest {
    pub async fn to_package(
        &self,
        package_root: &fastn_ds::Path,
        package_name: &str,
        ds: &fastn_ds::DocumentStore,
        main_package: &fastn_core::Package,
        session_id: &Option<String>,
    ) -> fastn_core::Result<fastn_core::Package> {
        let mut package = fastn_core::Package::new(package_name);
        package
            .resolve(
                &package_root.join(package_name).join("FASTN.ftd"),
                ds,
                session_id,
            )
            .await?;
        package.files = self.files.keys().map(|f| f.to_string()).collect();
        package.auto_import_language(
            main_package.requested_language.clone(),
            main_package.selected_language.clone(),
        )?;

        Ok(package)
    }
}
