# F-script appearances

- body of a function definition

```ftd
-- string foo():
string name:

if name == "" {
  "<default>"
} else {
  name
}
```

The body contains a list of expressions. The last expression is evaluated and
returned. There's no return keyword, if the function type is `void` then
nothing will be returned.

In the above example, `if` is an expression (just like in rust). Whatever it
evaluates to is returned from the function as it is the last (only) expression.
It has to evaluate to a `string` because the return type of `foo` is `string`
(type-checker's job).


- arg list of a function call

```ftd
-- ftd.text: Click Me!
$on-click$: ftd.set-string($a = $some-mut-ref, v = { 2 + 3 * 10 })
```

`{ 2 + 3 * 10 }` is an f-script block that will be evaluated and it's value
will be assigned to arg `v`.

- f-script blocks

These blocks appear in many places `fastn`, the example above is one such case.

Here's another example:

```ftd
-- greeting: {
    let names = ["Bob", "Alice"]
    let result = ""
    for name in names {
        result = result + " " + name
    }
    result
}

-- component greeting:
string msg:

-- ftd.text: $greeting.msg

-- end: component
```

The block contains a list of expressions. `let` is used to bind the result of
some expression to a variable.

The value of `result` is returned since it comes last in the list of
expressions.

# Features

## From `fastn 0.4`

- operators (see `fastn-grammar/src/evalexpr/operator/display.rs` for a list)
- Multiple expressions in body. The parser is able to parse multiple expression
  in a function body but, only the first expression is evaluated. For:

  ```ftd
  -- void handle(name, email):
  ftd.string-field name:
  ftd.string-field email:

  console.log(email.value);
  console.log("hello"); ;; this never evaluates!
  ```

  The generated js is:

  ```js
  let test__handle = function (args)
  {
    let __fastn_super_package_name__ = __fastn_package_name__;
    __fastn_package_name__ = "test";
    try {
      let __args__ = fastn_utils.getArgs({
      }, args);
      return (console.log(__args__.email.value));
      return (console.log("hello")); // THIS WILL NEVER BE EVALUATED!
    } finally {
      __fastn_package_name__ = __fastn_super_package_name__;
    }
  }
  ```

And that's it. Anything that the parser is not able to parse/identify is simply
converted to js if possible. Like the `console.log` call above. So it's mostly
safe to assume that whatever js you can write in one line is valid `f-script`
in 0.4.

Exceptions to above statement include resolving variables that are defined in
`fastn` and global variables. Global variables can be used like this:

```ftd
-- string some: someday ;; a global variable

-- void handle(name, email):
ftd.string-field name:
ftd.string-field email:
string x: $some ;; can only be accessed through `x` in `handle()`

console.log(x);
```

Declaring `x` like this will not be necessary in 0.5. Users will simply be able
to refer `some` that is defined outside of `handle`.

## Motivation behind proposed new features

The motivation to change f-script originates from the requirement that we want
to support multiple targets (desktop/mobile/TUI etc). To do this, f-script has
to become a base language that `fastn` will translate to:

- js for the web
- swift for ios/macos
- C# for Windows
- etcetera

Most of the interesting stuff happens in p-script, like registering events
(`$on-click$`). f-script is a simple procedural language that is mostly
insipired from `rust`.

## New in `fastn 0.5`

- `let` bindings

```ftd
{
  let name = "Siddhant";
  let adj = "";

  name + (adj || " (Programmer)")
}
```

- Control statements (`if..else`, `for`, `case`)

```ftd
{
  let res = if name == "" { ;; nested f-script block
    <expression>
  } else if name == "admin" {
    <expression>
  } else {
    <expression>
  };

  for { ;; infinite loop
  }

  ;; for <init>*; <cond> {...}
  ;; an init can be any expression that is executed once. A `let` binding for example
  ;; <cond> is and expression evaluated before the start of each iteration. Based on its result, the block is evaluated.
  ;; <init> can be ignored:
  for x <= 10 {
    ...
    x = x + 1;
  }

  ;; a for loop with <init>
  for let $x = 10; x < 100 {
    ...
    
    x = x + 1;
  }

  ;; `case` is same as `match` in rust for the most part
  ;; `match` name comes from the fact that it does pattern matching. We want to
  ;; say simpler things and saying "use `case` to test for multiple cases" is
  ;; simpler in my opinion.
  ;; see https://doc.rust-lang.org/reference/expressions/match-expr.html for grammar inspirations
  let ret = case res {
    "" => "<empty>",
    "admin" => "is admin",
  };
}
```

- Record instances

  It's possible to create instance of records that are defined in p-script:

  ```ftd
  -- record person:
  string name:
  integer sid:

  -- show-person: {
    ;; notice that it's mutable
    let $siddhant = person {
      name: "",
      sid: 4,
    };

    if siddhant.name == "" {
      siddhant.name = "Siddhant";
    }

    siddhant
  }

  -- component show-person:
  caption person p:

  ...
  ```
