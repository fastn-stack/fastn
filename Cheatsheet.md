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
-- constant string end: end
-- end: text-align
```