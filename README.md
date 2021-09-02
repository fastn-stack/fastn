# FTD

## How to run ftd

### 1. Build ftd-rt wasm

```bash
cd ftd-rt
wasm-pack build --target web -- --features=wasm
```
This commmand will create `pkg` folder inside `ftd-rt`

### 2. Build FTD Files

```bash
cd ftd
cargo run
```
This will create HTML (`.html` and `-rt.html`) files inside `ftd/build` folder for all `.ftd` files inside `ftd/examples` folder

1. `-rt.html`: It is runtime file rendered using wasm.
2. `.html`: It is static html file.

### 3. View Files

Run a local server:

```bash
cd ftd/build
python -m http.server 8000
```
And visit: `http://localhost:8000/`.

## Documentation

Docs: [fifthtry.com/fifthtry/ftd/](https://www.fifthtry.com/fifthtry/ftd/)

amitu_heroku commit-id: 4d86bfe32349cf8ae89f3c21b96132380d530ced
