# How to run tests

The `ftd` tests are present in `t` folder.

To run all tests use:
`cargo test`

## `p1` test:

To run p1 tests use:
`cargo test p1_test_all -- --nocapture`
The test files of p1 is present in `t/p1/` folder


## `ast` test:

To run ast tests use:
`cargo test ast_test_all -- --nocapture`
The test files of ast is present in `t/ast/`


## `interpreter` test:

To run interpreter tests use:
`cargo test interpreter_test_all -- --nocapture`
The test files of interpreter is present in `t/interpreter/` folder



## `js` test:

To run js tests use:
`cargo test fastn_js_test_all -- --nocapture`
The test files of js is present in `t/js/` folder

To run the manual test:
`cargo test fastn_js_test_all -- --nocapture manual=true`


## How to run individual test file for all the above tests:

Append `path=<substring of test file name>` in the test command.
e.g. To run `01-basic.ftd` test in `js`, use
`cargo test fastn_js_test_all -- --nocapture path=01` or
`cargo test fastn_js_test_all -- --nocapture path=basic` or
`cargo test fastn_js_test_all -- --nocapture path=01-basic`


# How to fix tests

Append `fix=true` in the test command.
e.g. 
1. To fix all `js` tests:
   `cargo test fastn_js_test_all -- --nocapture fix=true`
2. To fix `01-basic.ftd` test in `js`, use: 
   `cargo test fastn_js_test_all -- --nocapture path=01 fix=true` or
   `cargo test fastn_js_test_all -- --nocapture fix=true path=01` etc.


# How to create tests

1. Go to the corresponding folder for which the test needs to be created.
   Suppose, if you want to create a new test for `js`. Then go to `t/js` 
   folder.
2. Create a new file, preferably, the file name format should be 
   `<test-number>-<what-test-is-related-to>.ftd`. Suppose, if you want to 
   create a test for list type variable and the latest test number in the 
   folder is `11`. The file name should be `12-list-type-variable.ftd`.
3. Write the `ftd` code in the newly created test file.
4. Then run `cargo test fastn_js_test_all -- --nocapture path=11 fix=true`. 
   This will create a new output file. For above example, 
   `12-list-type-variable.html` will be created. 
5. You can check the generated file if it matches with your expectation.
