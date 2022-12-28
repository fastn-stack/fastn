# FTD Cheat Sheet

This cheatsheet describes 0.3 syntax.

## Variables And Basic Types

```ftd
-- boolean foo: true


-- integer x: 10
-- decimal y: 1.0
-- string message: Hello World

-- string multi-line-message:

This is can scan multiple paras.
```

By default all variable are immutable.

## Mutable Variable

To make a variable mutable, declare them with `$` prefix:

```ftd
-- boolean $foo: true

-- $foo: false
```

Conditional updates:

```ftd
-- boolean $foo: true
-- boolean bar: true

-- $foo: false
if: { bar }
```

## Optional Variables

```ftd
;; both are equivalent
-- optional boolean bar: NULL
-- optional boolean bar:

-- optional string $message: hello

-- $message: NULL

;; Not yet implemented
-- $message: \NULL  
```

## Records

Records are like `struct` in Rust or C. Or classes in other languages. They currently
only store data, methods are not yet allowed.

```ftd
-- record person:
caption name:
optional integer age:
optional body bio:

;; name is cattion so it goes in the "section line"
-- person me: Alice
age: 10

She sits on the floor and reads a book all day.

;; body ends when a new section starts or end of file is reached 

;; we are not specifying the age and bio as they are optional
-- person you: Bob

;; caption is an alias for string type
-- person jack:
name: jack

;; field lookup uses `.dot` syntax

-- string name: $me.name
```

Fields can be given default values:

```ftd
-- record person:
caption name:
;; age will be 18 by default
integer age: 18
;; nickname if not specified will be same as name
string nickname: $person.name
optional body bio:
```

Record can refer to themselves:

```ftd
-- record employee:
caption name:
string title: 
optional employee manager:

-- employee bob: Bob
title: CEO

-- employee jack: Jack
title: Programmer
manager: $bob
```


## Or Type

`or-type` are like `enum` or Rust.

```ftd
-- record hsl:
integer hue:
decimal saturation:
decimal lightness:

;; we are creating a new type called color
-- or-type color:

;; new type can be defined as well
-- record rgb:
integer red:
integer green:
integer blue:

;; it can refer to existing types
-- hsl hsl:

;; hex value of the color
-- string hex:

;; one of the css named colors
-- string css:

;; or-type must have an end clause
-- end: color

;; we are defining a variable called `red` using the `color.rgb` variant
;; the type `red` is `color`
-- color.rgb red:
red: 255
green: 0
blue: 0

-- color.hex green: #00FF00
```

`or-type` can also contain constant variants:

```ftd
-- record rgb:
integer red:
integer green:
integer blue:

-- or-type color:

-- rgb rgb:
-- constant rgb red:
red: 255
green: 0
blue: 0

-- end: color
```

Now `$color.red` is a named constant.

## Lists

```ftd
-- integer list foo:
-- integer: 10
-- integer: 20
-- integer: 30
-- end: foo

-- color list colors:

-- color.rgb:
red: 255
green: 0
blue: 0

-- color.hex: #00FF00

-- end: colors
```

Record containing a list:

```ftd
-- record person:
caption name:
person list friends:



-- person alice: Alice
-- alice.friends:

-- person: Bob

;; friends of bob:
-- person.friends:
-- person: Jill
-- person: Sam
-- end: person.friends

;; jack has no friends
-- person: Jack

-- end: alice.friends
```

list of list not yet supported.

## Function

```ftd
-- integer add(x,y):
integer x:
integer y:

sum = x + y;
x + y
```

NOTE: `-- integer add(x,y):` can not contain space yet, eg `-- integer add(x, y):` is
not allowed.

By default, arguments are immutable, to make them mutable use `$` prefix:

```ftd
-- void increment(what,by_how_much):
integer $what:
integer by_how_much:

what += by_how_much
```

# In Built Functions

## `isempty()`

It works with lists, optionals and strings.

```ftd
-- string list foo:
-- end: foo

-- boolean $empty: true

-- $empty: false
if: {!isempty(foo)}
```

# Kernel Components

FTD comes with following kernel components:

```ftd
-- ftd.text: hello world
```

## `ftd.text` Attributes

### `optional caption or body`: the text to show

```ftd
-- ftd.text:

This is a body text.
```

### `text-align`: `ftd.text-align`

```ftd
-- or-type text-align:
-- constant string start: start
-- constant string center: center
-- constant string end: end
-- constant string justify: justify
-- end: text-align
```

## Common Attributes

- `padding`: `optional ftd.length`
- `padding`: `optional ftd.length`
- `padding`: `optional ftd.length`
- `padding-left`: `optional ftd.length`
- `padding-right`: `optional ftd.length`
- `padding-top`: `optional ftd.length`
- `padding-bottom`: `optional ftd.length`
- `padding-horizontal`: `optional ftd.length`
- `padding-vertical`: `optional ftd.length`
- `margin`: `optional ftd.length`
- `margin-left`: `optional ftd.length`
- `margin-right`: `optional ftd.length`
- `margin-top`: `optional ftd.length`
- `margin-bottom`: `optional ftd.length`
- `margin-horizontal`: `optional ftd.length`
- `margin-vertical`: `optional ftd.length`
- `border-width`: `optional ftd.length`
- `border-radius`: `optional ftd.length`
- `border-bottom-width`: `optional ftd.length`
- `border-top-width`: `optional ftd.length`
- `border-left-width`: `optional ftd.length`
- `border-right-width`: `optional ftd.length`
- `border-top-left-radius`: `optional ftd.length`
- `border-top-right-radius`: `optional ftd.length`
- `border-bottom-left-radius`: `optional ftd.length`
- `border-bottom-right-radius`: `optional ftd.length`


**`ftd.length`**

```ftd
-- or-type length:

-- record px:
caption integer value:

-- record percent:
caption decimal value:

-- record calc:
caption value:

-- record vh:
caption integer value:

-- record vw:
caption integer value:

-- record em:
caption decimal value:

-- record rem:
caption decimal value:

-- end: length
```

- `border-color`: `optional ftd.color`
- `border-bottom-color`: `optional ftd.color`
- `border-top-color`: `optional ftd.color`
- `border-left-color`: `optional ftd.color`
- `border-right-color`: `optional ftd.color`
- `color`: `optional ftd.color`

**`ftd.color`**


```ftd
-- record color:
caption light:
string dark: $color.light
```

- `min-width`: `optional ftd.resizing`
- `max-width`: `optional ftd.resizing`
- `min-height`: `optional ftd.resizing`
- `max-height`: `optional ftd.resizing`
- `width`: `optional ftd.resizing`
- `height`: `optional ftd.resizing`


**`ftd.resizing`**

```ftd
-- or-type resizing:

-- constant string fill-container: fill-container
-- constant string hug-content: hug-content
-- ftd.length fixed:

-- end: resizing
```

- `link`: `string`
- `open-in-new-tab`: `optional boolean`

- `background`: `optional ftd.fill`


**`ftd.fill`**

```ftd
-- or-type fill:

-- ftd.color solid:

-- end: fill
```

- `align-self`: `optional ftd.align-self`

**`ftd.align-self`**

```ftd
-- or-type align-self:

-- constant string start: start
-- constant string center: center
-- constant string end: end

-- end: align-self
```

- `overflow`: `optional ftd.overflow`
- `overflow-x`: `optional ftd.overflow`
- `overflow-y`: `optional ftd.overflow`

**`ftd.overflow`**

```ftd
-- or-type overflow:

-- constant string scroll: scroll
-- constant string visible: visible
-- constant string hidden: hidden
-- constant string auto: auto

-- end: overflow
```


## Container Attributes

- `spacing`: `optional ftd.length`
- `wrap`: `optional boolean`

- `align-content`: `optional ftd.align`

**`ftd.align`**

```ftd
-- or-type align:

-- constant string top-left: top-left
-- constant string top-center: top-center
-- constant string top-right: top-right
-- constant string right: right
-- constant string left: left
-- constant string center: center
-- constant string bottom-left: bottom-left
-- constant string bottom-center: bottom-center
-- constant string bottom-right: bottom-right

-- end: align
```


- `spacing-mode`: `optional ftd.spacing-mode`

**`ftd.spacing-mode`**

```ftd
-- or-type spacing-mode:

-- constant string space-between: space-between
-- constant string space-around: space-around
-- constant string space-evenly: space-evenly

-- end: spacing-mode
```

# Default functions

## `is_empty(a: any)`

This is FScript function. It gives if the value passed to argument `a` is null or empty.


```ftd
-- optional string name:

-- ftd.text: $name
if: { !is_empty(name) }

-- string list names:

-- display-name:
if: { !is_empty(names) }
```

## `enable_dark_mode()`

This is FScript function. This function enables the dark mode.

```ftd
-- ftd.text: Dark Mode
$on-click$: $set-dark()

-- void set-dark():

enable_dark_mode()
```

## `enable_light_mode()`

This is FScript function. This function enables the light mode.

```ftd
-- ftd.text: Light Mode
$on-click$: $set-light()

-- void set-light():

enable_light_mode()
```

## `enable_system_mode()`

This is FScript function. This function enables the system mode.

```ftd
-- ftd.text: System Mode
$on-click$: $set-system()

-- void set-system():

enable_system_mode()
```





## How to run examples for FTD: 0.3

- Create empty files `<number>-<name>.ftd` and `<number>-<name>.html` in `t/html` 
  folder.
- Run `cargo test html_test_all -- --nocapture fix=true <optional: path=<prefix of the file name>>`

### Optional commands to check html in examples
- Run `cargo run`
- Run `cd docs`
- Run `python3 -m http.server 8000`
