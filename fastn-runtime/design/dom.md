# Dom

[Back To Design Home](./).

We have two operating modes. In `internal-dom` mode we maintain full DOM tree, and in `browser-dom` mode we rely on an 
external system to maintain the dom.

The compiled wasm code expects a bunch of dom related functions that we provide. Eg `create_kernel()`, 
`set_property_i32()` and so on. Such functions mutate the dom.

Note fastn language does not have any concept of querying the UI, so you can never get a handle to a dom node, or
ask questions like if it is visible or what's it's content. The language is strictly one way. Documents contains data,
UI is derived from data, and UI has event handlers that can change data, and based on those data changes the UI will
get updated.

When the document is getting rendered on the server side, we operate in internal-dom mode. At the end of original page 
creation, the dom is converted to HTML, which transferred to the browser. In the browser the DOM is managed by the
browser, so we do not have to maintain our own DOM, this is called browser-dom mode. Refer [`browser.md`](browser.md)
for details.

When we are running in the native mode, say fastn uses WebGPU to render the DOM, or when we use curses based rendering,
the dom tree is maintained by us, and the renderer is used to transform the dom to things that can be rendered by WebGPU
or terminal. Refer [`gpu.md`](gpu.md) and [`terminal.md`](terminal.md) for details.

# Roles

For some attributes like `ftd.color`, `ftd.length` and `ftd.typography`, we create classes. In our  
[data-layer](data-layer.md), we treat these types as simple records, and store their data in `fastn_runtime::Memory.vec`.
We also store the pointers corresponding to each role in `fastn_runtime::Memory.text_roles: 
Vec<fastn_runtime::Pointerkey>` and so on. 

The only way to change the color of a text is to first construct a `ftd.color` instance, and then pass it to 
`dom.set_property_vec(node, TextColor.i32(), role)`. For each role we create a CSS class, eg if the ftd.color has 
pointer id of `1v1`, we will create a class `c_1v1` and attach it to the `node`. 

In case of `internal-dom`, we store the `fastn_runtime::Pointerkey`, eg

```rust
struct ColorPointer(fastn_runtime::Pointerkey);

struct TextStyle {
    color: Option<fastn_runtime::ColorPointer>,
}
```

In case of `browser-dom` (when running in browser), we directly modify the class list for the text node, eg

```js
document.getElementById("1v1").t.classList.push("c_1v1");
```

# Non Role Properties

Other properties like `align`, `href` etc, we store the computed property in the DOM in case of `internal-dom`, eg

```rust
enum Align {
    Left,
    Right,
    Justify
}

struct CommonStyle {
    align: Option<Align>
}
```

When we generate HTML such properties would be added inline to the dom node either as attribute or as inline style.