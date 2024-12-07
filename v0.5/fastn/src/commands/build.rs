impl fastn::commands::Build {
    /// `fastn build` goes through the entire site and builds it
    ///
    /// other commands like `fastn serve`, `fastn static`, `fastn render` do not directly read ftd
    /// files in a fastn package. they expect that `fastn build` has already been called. any change
    /// in a fastn package since the last `fastn build` is not visible till `fastn build` is called
    /// again.
    ///
    /// `fastn build` first calls `fastn update` to ensure all dependencies are up to date. this can
    /// be skipped by using `fastn build --offline`. this is done to speed up the build process, and
    /// also to ensure that the build is reproducible. `fastn update` stores the downloaded packages
    /// in `.fastn/packages` directory in the package root. this folder should be added to
    /// `.gitignore`.
    ///
    /// `fastn build` stores its compiled output in a `.fastn/build` directory in the package root.
    /// this folder should be added to `.gitignore` to avoid checking in the build files. the
    /// content of this folder can change between releases of fastn.
    ///
    /// `.fastn/build/hashes.json` stores the content-hash of each file in the package. this is used
    /// to determine if a file has changed since the last build. if a file has not changed, it is
    /// not re-compiled. this is done to speed up the build process.
    ///
    /// `.fastn/build/hash.json` also stores dependencies of each file. if a file is not changed,
    /// but one of its dependencies has changed, the file is re-compiled.
    ///
    ///
    /// `fastn build --watch` can run a file watcher and re-build the site on any change in the
    /// package.
    ///
    /// `fastn build --strict` runs the build in strict mode. in this mode, all warnings are treated
    /// as errors, including invalid formatting.
    pub async fn run(self, _package: fastn_package::Package) {
        // go through the entire package, and compile all the files
        for _document in changed_documents() {
            // check if we already have JS, if not compile it
            // fastn_compiler::compile(&mut config, &document).await;
            // compile the document
            // store the compiled output in .fastn/build
            // store the hash of the compiled output in .fastn/build/hashes.json
        }
        todo!()
    }
}

// this only returns ftd files, static files are ignored
fn changed_documents() -> Vec<String> {
    todo!()
}
