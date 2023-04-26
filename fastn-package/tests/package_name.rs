#[test]
fn fastn_package_test() {
    use fastn_package::initializer::test::*;

    futures::executor::block_on(async {
        match dbg!(fastn_package::initialize(TestInitializer::default()).await) {
            Err(InitialisePackageError::FastnFTDError { source }) => match source {
                FastnFTDError::ReadFTDFile { source } => match source {
                    FileAsStringError::FileDoesNotExist { name, source } => {
                        assert_eq!(name, "FASTN.ftd");
                        assert_eq!(source.kind(), std::io::ErrorKind::NotFound);
                    }
                    _ => panic!("unexpected error"),
                },
                _ => panic!("unexpected error"),
            },
            _ => panic!("unexpected error"),
        }
    });
}
