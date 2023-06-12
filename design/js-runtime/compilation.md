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
        fastn.set_property(i, fastn.Property.IntegerValue, x);
    }
})()
```

## Component Definition

Component definitions will be compiled into functions.


```ftd
-- integer $x: 10        ;; mutable
-- integer y: 20         ;; static
-- integer z = $x + $y   ;; formula 

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
(function() {
    function main(root) {
        let x = fastn.mutable(10);
        let y = 20;
        
        let z = fastn.closure([x], function() {
            get(x) + y
        });
                
        let f = foo(root, x);
        let f = foo(root, x);                
    }
    
    function foo(root, x) {
        let i = fastn.create_kernel(root, fastn.ElementKind.Integer);
        
        fastn.add_event_handler(i, "click", [x], function() {
            x.set(x.get() + 1);
        });

        fastn.set_property(i, fastn.Property.IntegerValue, [x], function() {
            x.get() + 20
        });
    }
})()
```

## `fastn.Mutable`

We are writing code in Rust, but its only for reference, code will actually be written in JS.

```rust
// formula, dynamic property, and even handler?
struct Closure {
    cached_value: Value,
    func_args: Vec<Mutable>,
    func: Fn,
    ui: Option<(Node, Property)>,
}

impl Closure {
    fn call(&self) {
        self.cached_value = self.func();
        if let Some(ui) = self.ui {
            ui.update(self.cached_value);
        }
    }
}

struct Mutable {
    value: Value,
    closures: Vec<Closure>,
}

impl<T> Multable<T> {
    fn get(&self) -> T {
        self.value
    }
    fn set(&self, new: T) {
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
