# Syntax Highlighting

We are only implementing static parsing for now. In 0.4 we will continue to use syntect highlighter and innerHTML.

```rust
fn main() {
    println!("Hello, world!"); // <1>
}
```

```js
function main(parent) {
    let t = fastn_dom.createKernel(parent, fastn_dom.ElementKind.CodeContainer);
    ds_keyword(t, "fn");
    ds_space(t, " ");
    ds_identifier(t, "main", code_context);
    ds_space(t, " ");
    ds_paren(t, "(");
    ds_string(t, `"hello world"`);

    // `// <1>` will call `note`.
    ds_code_note(t, 1, code_context);
    // `// {foo}` will call foo
    foo(t, code_context); 
}
```

Refer https://crates.io/crates/tree-sitter-highlight for actual `highlight_names` we will be using. For each 
`highlight_name` we will have a `ds_` function.

## `code_context`

```ftd
-- record code-context:
integer line-number:
string current-line:
string full-code:
string lang:
```

## "Intellisence"

Any of the `highlight_name` can have associated help text, which will be another "ftd.ui", that will be shown on hover.

```js
function main(parent) {
    let t = fastn_dom.createKernel(parent, fastn_dom.ElementKind.CodeContainer);
    ds_keyword(t, "fn");
    ds_space(t, " ");
    // main has main_help associated with it. ds_identifier can change the look of the main, to indicate to reader that
    // main has help associated with it. And on hover, main_help will be shown.
    ds_identifier(t, "main", {help: main_help});
}

function main_help() {
    
}
```


## Command Click: jump to definition

Any of the symbols can have an associated link, which is where the user will be taken when they command click on the 
symbol. We will pass `link` as an argument to `ds_` functions.


```ftd
-- fastn.package:
python-root: python/src
```

```ftd
-- ds.code:
lang: py
code-processor: lsp
python-root: python/src
diff: <old-commit-hash>

import foo

def main():
    foo.bar
```

```js
function main(parent) {
    let t = fastn_dom.createKernel(parent, fastn_dom.ElementKind.CodeContainer);
    let line = fastn_dom.createKernel(t, fastn_dom.ElementKind.CodeLine);
    ds_line_number(line, code_context);
    ds_line_diff(line);
    ds_line_blame(line);
    ds_keyword(line, "fn");
    ds_space(line, " ");
    // main has main_help associated with it. ds_identifier can change the look of the main, to indicate to reader that
    // main has help associated with it. And on hover, main_help will be shown.
    ds_identifier(line, "main", {link: main_link});
}
```

## ftd.code-ui module

We will create such a module, with UI for all UI we support. We support all the `highlight_names`. We also support 
gutter elements like `line_number`. We support `note`. We also allow arbitrary components in comments (`// {foo}`).

Other gutter items are diff, and blame so far. 

## Code Processors

We have to create a bunch of code processors, like `lsp`, which when enabled adds help and jump to definition, view all
references etc to all symbols. `line-no` processor adds line number. `diff` processor adds diff UI. `blame` processor
adds blame UI.

Depending on which all code processors you have included in the code block, the generated JS will be different. We only
have a fixed number of possible UI, eg line_no, diff-ui etc, which will be part of our `ftd.code-ui` module, so any 
package can fully customise the look and feel of the code block.

## Expand Collapse Regions

```js
function main(parent) {
    let t = fastn_dom.createKernel(parent, fastn_dom.ElementKind.CodeContainer);
    let region_1 = ds_region(t, region_context);
    ds_region_gutter(region_1, code_context, region_context);
    let line = fastn_dom.createKernel(region_1, fastn_dom.ElementKind.CodeLine);
    ds_line_number(line, code_context);
    ds_line_diff(line);
    ds_line_blame(line);
    ds_keyword(line, "fn");
    ds_space(line, " ");
    // main has main_help associated with it. ds_identifier can change the look of the main, to indicate to reader that
    // main has help associated with it. And on hover, main_help will be shown.
    ds_identifier(line, "main", {link: main_link});
}
```

`region_context` is the text to show when the region is collapsed.
