# Compilation Of FTD to JS

## Module

We compile all ftd modules into a single JS file. It is possible to compile each ftd module into separate js files, and
use JavaScript's module system. We are not considering it for now, to keep things simple.

## Basic Global Variable

A global variable, eg in:

```ftd
-- integer x: 10
-- ftd.integer: $x 
```

```js
(function() {
    function main(root) {
        let x = 10;
        let i = fastn.create_kernel(root, fastn.ElementKind.Integer);
        i.set_property_static(fastn.Property.IntegerValue, x);
    }
})()
```

## Component Definition

Component definitions will be compiled into functions.


```ftd
-- integer $x: 10        ;; mutable
-- integer y: 20         ;; static
-- integer z: $x + $y   ;; formula 

-- foo: 
$x: $x

-- foo: 
$x: $x
 
-- ftd.integer: $z

-- component foo:
integer $x:

-- ftd.integer: { $foo.x + 20 }
$on-click$: { foo.x += 1 }

-- end: foo
```

```js
(function () {
    function main(root) {
        let x = fastn.mutable(10);
        let y = 20;

        let z = fastn.formula([x], function () {
            x.get() + y
        });

        let t = fastn_dom.createKernel(root, fastn_dom.ElementKind.Integer);
        t.set_property(fastn.Property.IntegerValue, [z], function () {
            z.get()
        });

        let f = foo(root, x);
        let f = foo(root, x);
    }

    function foo(root, x) {
        let i = fastn.create_kernel(root, fastn.ElementKind.Integer);

        i.add_event_handler(fastn.Event.Click, function () {
            x.set(x.get() + 1);
        });

        i.set_property(fastn.Property.IntegerValue, [x], function () {
            x.get() + 20
        });
    }
})();
```

## `fastn.Closure`

We are writing code in Rust, but its only for reference, code will actually be written in JS.

```rust
// formula and dynamic property
struct Closure {
    cached_value: Value,
    func: Fn,
    ui: Option<(Node, Property)>,
}

impl Closure {
    fn update_ui(&self) {
        if let Some(ui) = self.ui {
            ui.update(self.cached_value);
        }
    }
    fn update(&self) {
        self.cached_value = self.func();
        self.update_ui()
    }
}
```

## `fastn.Mutable`

```rust
struct Mutable {
    value: Value,
    closures: Vec<Closure>,
}

impl Multable {
    fn get(&self) -> Value {
        self.value
    }
    fn set(&self, new: Value) {
        self.value = new;
        for c in self.closures {
            c.call();
        }
    }
    fn add_closure(&mut self, closure: Closure) {
        self.closures.push(closure);
    }
}
```

## `Node.set_property()`

```rust
fn set_property(property: Property, deps: Vec<Mutable>, func: Fn) {
    let closure = Closure {
        cached_value: func(),
        func,
        ui: Some((this, property)),
    };
    closure.update_ui();

    for dep in deps {
        dep.add_closure(closure)
    }
}
```

## `fastn.formula()`

```rust
fn formula(deps: Vec<Mutable>, func: Fn) -> Closure {
    let v = func();

    let closure = Closure {
        cached_value: v,
        func,
        ui: None,
    };

    let m = Mutable {
        value: v,
        closures: vec![ closure ],
    };

    for dep in deps {
        dep.add_closure(closure)
    }

    return m
}
```
