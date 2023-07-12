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
