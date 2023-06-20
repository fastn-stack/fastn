# Component and Variable Registry

We are generating JS like this:

```js
function main (root) {
    let x = fastn.mutable(10);
    let y = 20;
    let z = fastn.formula([x], function () { return x.get() * y; })
    foo(root, x);
    foo(root, x);
    let i = fastn_dom.createKernel(root, fastn_dom.ElementKind.Integer);
    i.setProperty(fastn_dom.PropertyKind.IntegerValue, z);
    i.done();
}

function foo(root, x) {
    let i = fastn_dom.createKernel(root, fastn_dom.ElementKind.Integer);
    i.setDynamicProperty(fastn_dom.PropertyKind.IntegerValue, [x], function () { return x.get() + 20; });
    i.setStaticProperty(fastn_dom.PropertyKind.Color_RGB, "red");
    i.addEventHandler(fastn_dom.Event.Click, function () {
        x.set(x.get() + 1);
    });
    i.done();
}
```

As you see, when we need to refer to `x`, we directly refer to the variable `x`. When we need to refer to `foo`, we call
the function named `foo` in current scope.

All the globals across all modules are initialised in the `main()` function. All the components are converted to 
functions. This is all done at compile time, when the JS is generated.

For resolving components used in [markup](markup.md), and for doing [variable interpolation](variable-interpolation.md),
we will need to resolve variables and components at runtime. We will do this by maintaining a registry of all the 
variables and components.

```js
let foo = fastn.component("index.foo", function(idx, root, x) {
    let localRegistry = fastn.local_registry();
    
    let x2 = fastn.mutable(localRegistry, "x2", 10);
    
    let i = fastn_dom.createKernel(root, fastn_dom.ElementKind.Integer);
    i.setDynamicProperty(fastn_dom.PropertyKind.IntegerValue, [x], function () { return x.get() + 20; });
    i.setStaticProperty(fastn_dom.PropertyKind.Color_RGB, "red");
    i.addEventHandler(fastn_dom.Event.Click, function () {
        x.set(x.get() + 1);
    });
    i.done();
})

function main (root) {
    let x = fastn.mutable("index.x", 10);
    let y = fastn.static("index.y", 20);
    let z = fastn.formula("index.z", [x], function () { return x.get() * y; })
    
    foo(root, x);
    foo(root, x);
    let i = fastn_dom.createKernel(root, fastn_dom.ElementKind.Integer);
    i.setProperty(fastn_dom.PropertyKind.IntegerValue, z);
    i.done();
}
```
