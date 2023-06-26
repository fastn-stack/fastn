# Building

## Debugging

```sh
python3 -m http.server
```

## To Run Manual Tests

Run this from `ftd` folder.

This will build `ftd/t/js/*.manual.html` files. You can open them in browser.

```sh
cargo test fastn_js_test_all -- --nocapture manual=true
```

If you want to build a single file: 

```sh
cargo test fastn_js_test_all -- --nocapture manual=true path=02
```


## To Run All Tests

```sh
cargo test fastn_js_test_all
```

## To "Fix" Tests

If the tests are failing because you have made changes to JS/HTML etc, and snapshotted HTMLs are wrong, run
the following to update the snapshot:

```shell
cargo test fastn_js_test_all -- --nocapture fix=true
```

You can also pass `path` argument to update a single file.