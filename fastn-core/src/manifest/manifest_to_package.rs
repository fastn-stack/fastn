impl fastn_core::Manifest {
    pub fn to_package(
        &self,
        package_root: &fastn_ds::Path,
        ds: &fastn_ds::DocumentStore,
    ) -> fastn_core::Result<fastn_core::Package> {
        let fastn_ftd = ds.read_to_string(&package_root.join("FASTN.ftd"))?;
    }
}
