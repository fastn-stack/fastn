#[test]
fn fastn_package_test() {
    use fastn_package::initializer::test::*;

    futures::executor::block_on(async {
        match dbg!(fastn_package::initialize(TestInitializer::default()).await) {
            Err(InitializePackageError::FastnFTDError {
                source:
                    FastnFTDError::ReadFTDFile {
                        source: FileAsStringError::FileDoesNotExist { name, source },
                    },
            }) => {
                assert_eq!(name, "FASTN.ftd");
                assert_eq!(source.kind(), std::io::ErrorKind::NotFound);
            }
            _ => panic!("unexpected error"),
        }
    });
}
