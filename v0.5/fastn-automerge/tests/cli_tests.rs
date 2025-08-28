struct CliTest {
    temp_dir: tempfile::TempDir,
    db_path: std::path::PathBuf,
}

impl CliTest {
    fn new(_test_name: &str) -> Self {
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp directory");
        let db_path = temp_dir.path().join("test.sqlite");

        Self { temp_dir, db_path }
    }

    fn run(&self, args: &[&str]) -> CliOutput<'_> {
        let output = std::process::Command::new("cargo")
            .arg("run")
            .arg("-p")
            .arg("fastn-automerge")
            .arg("--")
            .arg("--db")
            .arg(&self.db_path)
            .args(args)
            .output()
            .expect("Failed to run CLI command");

        CliOutput { output, test: self }
    }

    fn init(&self) -> CliOutput<'_> {
        self.run(&["init"])
    }

    fn create(&self, path: &str, json: &str) -> CliOutput<'_> {
        self.run(&["create", path, json])
    }

    fn create_from_file(&self, path: &str, file: &str) -> CliOutput<'_> {
        self.run(&["create", path, "--file", file])
    }

    fn get(&self, path: &str) -> CliOutput<'_> {
        self.run(&["get", path])
    }

    fn get_pretty(&self, path: &str) -> CliOutput<'_> {
        self.run(&["get", path, "--pretty"])
    }

    fn get_to_file(&self, path: &str, output: &str) -> CliOutput<'_> {
        self.run(&["get", path, "--output", output])
    }

    fn update(&self, path: &str, json: &str) -> CliOutput<'_> {
        self.run(&["update", path, json])
    }

    fn set(&self, path: &str, json: &str) -> CliOutput<'_> {
        self.run(&["set", path, json])
    }

    fn delete(&self, path: &str) -> CliOutput<'_> {
        self.run(&["delete", path, "--confirm"])
    }

    fn list(&self) -> CliOutput<'_> {
        self.run(&["list"])
    }

    fn list_prefix(&self, prefix: &str) -> CliOutput<'_> {
        self.run(&["list", "--prefix", prefix])
    }

    fn history(&self, path: &str) -> CliOutput<'_> {
        self.run(&["history", path])
    }

    fn history_short(&self, path: &str) -> CliOutput<'_> {
        self.run(&["history", path, "--short"])
    }

    fn info(&self, path: &str) -> CliOutput<'_> {
        self.run(&["info", path])
    }
}

// No need for Drop implementation - tempfile handles cleanup automatically

struct CliOutput<'a> {
    output: std::process::Output,
    test: &'a CliTest,
}

impl<'a> CliOutput<'a> {
    fn should_succeed(self) -> Self {
        if !self.output.status.success() {
            panic!(
                "Command failed with exit code {:?}\nStderr: {}",
                self.output.status.code(),
                String::from_utf8_lossy(&self.output.stderr)
            );
        }
        self
    }

    fn should_fail(self) -> Self {
        if self.output.status.success() {
            panic!(
                "Command unexpectedly succeeded\nStdout: {}",
                String::from_utf8_lossy(&self.output.stdout)
            );
        }
        self
    }

    fn stdout_contains(self, text: &str) -> Self {
        let stdout = String::from_utf8_lossy(&self.output.stdout);
        if !stdout.contains(text) {
            panic!("Stdout does not contain '{text}'\nActual stdout: {stdout}");
        }
        self
    }

    fn stdout_not_contains(self, text: &str) -> Self {
        let stdout = String::from_utf8_lossy(&self.output.stdout);
        if stdout.contains(text) {
            panic!("Stdout unexpectedly contains '{text}'\nActual stdout: {stdout}");
        }
        self
    }

    fn file_exists(self, path: &str) -> Self {
        if !std::path::Path::new(path).exists() {
            panic!("File {path} does not exist");
        }
        self
    }

    fn file_contains(self, path: &str, text: &str) -> Self {
        let content =
            std::fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to read file {path}"));
        if !content.contains(text) {
            panic!("File {path} does not contain '{text}'\nActual content: {content}");
        }
        self
    }

    #[allow(dead_code)]
    fn and(self) -> &'a CliTest {
        self.test
    }
}

#[test]
fn test_cli_init() {
    let test = CliTest::new("init");
    test.init()
        .should_succeed()
        .stdout_contains("Initialized database");
}

#[test]
fn test_cli_create_get_cycle() {
    let test = CliTest::new("create_get");

    test.init().should_succeed();

    test.create("/-/test", r#"{"name": "hello", "value": 42}"#)
        .should_succeed()
        .stdout_contains("Created document");

    test.get("/-/test")
        .should_succeed()
        .stdout_contains("hello")
        .stdout_contains("42");

    test.get_pretty("/-/test")
        .should_succeed()
        .stdout_contains("hello");
}

#[test]
fn test_cli_update_set() {
    let test = CliTest::new("update_set");

    test.init().should_succeed();
    test.create("/-/doc", r#"{"version": 1}"#).should_succeed();

    test.update("/-/doc", r#"{"version": 2}"#).should_succeed();

    test.get("/-/doc").should_succeed().stdout_contains("2");

    test.set("/-/new", r#"{"created": "by_set"}"#)
        .should_succeed();

    test.get("/-/new")
        .should_succeed()
        .stdout_contains("by_set");
}

#[test]
fn test_cli_list_delete() {
    let test = CliTest::new("list_delete");

    test.init().should_succeed();
    test.create("/-/doc1", r#"{"id": 1}"#).should_succeed();
    test.create("/-/doc2", r#"{"id": 2}"#).should_succeed();
    test.create("/-/users/alice", r#"{"name": "Alice"}"#)
        .should_succeed();

    test.list()
        .should_succeed()
        .stdout_contains("/-/doc1")
        .stdout_contains("/-/doc2")
        .stdout_contains("/-/users/alice");

    test.list_prefix("/-/users/")
        .should_succeed()
        .stdout_contains("/-/users/alice")
        .stdout_not_contains("/-/doc1");

    test.delete("/-/doc1").should_succeed();

    test.list()
        .should_succeed()
        .stdout_not_contains("/-/doc1")
        .stdout_contains("/-/doc2");
}

#[test]
fn test_cli_history_info() {
    let test = CliTest::new("history_info");

    test.init().should_succeed();
    test.create("/-/doc", r#"{"version": 1}"#).should_succeed();
    test.update("/-/doc", r#"{"version": 2}"#).should_succeed();

    test.info("/-/doc")
        .should_succeed()
        .stdout_contains("Total edits: 2")
        .stdout_contains("Document: /-/doc");

    test.history_short("/-/doc")
        .should_succeed()
        .stdout_contains("2 edits total");

    test.history("/-/doc")
        .should_succeed()
        .stdout_contains("Edit #1")
        .stdout_contains("Edit #2")
        .stdout_contains("Actor:");
}

#[test]
fn test_cli_file_io() {
    let test = CliTest::new("file_io");

    // Create temporary files in the same temp directory
    let input_file = test.temp_dir.path().join("input.json");
    let output_file = test.temp_dir.path().join("output.json");

    std::fs::write(&input_file, r#"{"from_file": true}"#).unwrap();

    test.init().should_succeed();

    test.create_from_file("/-/test", input_file.to_str().unwrap())
        .should_succeed();

    test.get_to_file("/-/test", output_file.to_str().unwrap())
        .should_succeed()
        .file_exists(output_file.to_str().unwrap())
        .file_contains(output_file.to_str().unwrap(), "from_file");
}

#[test]
fn test_cli_errors() {
    let test = CliTest::new("errors");

    // Test without init (should fail)
    test.get("/-/test").should_fail();

    // Now initialize for other tests
    test.init().should_succeed();

    // Test non-existent document
    test.get("/-/missing").should_fail();

    // Test duplicate create
    test.create("/-/doc", r#"{"id": 1}"#).should_succeed();
    test.create("/-/doc", r#"{"id": 2}"#).should_fail();

    // Test invalid JSON
    test.create("/-/bad", r#"{"invalid": json}"#).should_fail();

    // Test update non-existent
    test.update("/-/missing", r#"{"id": 1}"#).should_fail();
}
