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


The `or-type` can have three types of variant:

- `regular`: This accepts value and it has defined type/kind
- `constant`: This doesn't accept. The value is provided during declaration and is non-changeable.
- `anonymous-record`: The new record type/kind is created during declaration.
  It also accepts value.

```ftd
-- or-type color:

;; regular
-- string hex:

;; constant
-- const string foo-red: red

;; anonymous-record
-- record dl:
caption string dark:
string light: $dl.dark


;; Using regular type variant
-- ftd.text: Hello
color.hex: #ffffff

;; Using anonymous-record type variant
-- ftd.text: Hello
color.dl: blue
```

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

# Kernel Components

FTD comes with following kernel components:

## `ftd.text` - To display text or strings 

```ftd
-- ftd.text: hello world
```

## `ftd.text` Attributes

### `optional caption or body`: the text to show

```ftd
-- ftd.text:

This is a body text.
```

### `style`: `optional ftd.text-style list`

```ftd
-- or-type text-style:
 
-- constant string underline: underline
-- constant string italic: italic
-- constant string strike: strike
-- constant string heavy: heavy
-- constant string extra-bold: extra-bold
-- constant string semi-bold: semi-bold
-- constant string bold: bold
-- constant string medium: medium
-- constant string regular: regular
-- constant string light: light
-- constant string extra-light: extra-light
-- constant string hairline: hairline

-- end: text-style
```

### `text-align`: `ftd.text-align`
### `line-clamp`: `optional integer`

## `ftd.code` - To render a code block

```ftd 
-- ftd.code: 
lang: rs

def func() {
  println!("Hello World");
}
```

## `ftd.code` attributes

### `lang`: `optional string` -> To specify code language.
`Default lang = txt` 
### `theme`: `optional string` -> To specify the theme
`Default theme = base16.ocean-dark`

Currently includes these themes

`base16-ocean.dark`, `base16-eighties.dark`, `base16-mocha.dark`, `base16-ocean.light`
`Solarized (dark)`, `Solarized (light)`

Also see InspiredGitHub from [here](https://github.com/sethlopezme/InspiredGitHub.tmtheme)

### `body`: `string` -> specify the code to display
### `line-clamp`: `optional integer` -> clamps the specified number of lines
### `text-align`: `ftd.text-align` -> specify text alignment
(Refer or-type `ftd.text-align` to know all possible values)

## `ftd.decimal` - To display decimal values

```ftd 
-- decimal pi: 3.142

-- ftd.decimal: 1.5

;; To display decimal variables
-- ftd.decimal: $pi
```

## `ftd.decimal` attributes

### `value`: `caption or body` -> decimal value to be rendered

### `text-align`: `ftd.text-align -> specify text alignment

### `line-clamp`: `optional integer`

## `ftd.iframe` - To render an iframe

```ftd 
-- ftd.iframe: 
src: https://www.fifthtry.com/

-- ftd.iframe: 
youtube: 10MHfy3b3c8

-- ftd.iframe:

<p>Hello world!</p>
```

## `ftd.iframe` attributes

### `src`: `optional caption string`
### `youtube`: `optional string` -> It accepts the youtube `vid` or `video id`.
### `srcdoc`: `optional body string`

Either src or youtube or srcdoc value is required. If any two or more than 
two of these are given or none is given would result to an error.

### 'loading': `optional ftd.loading`
Default: `lazy`

```ftd
-- or-type loading:

-- constant string lazy: lazy
-- constant string eager: eager

-- end: loading
```


## `ftd.text-input` - To take text input from user 

```ftd 
-- ftd.text-input: 
placeholder: Type Something Here...
type: password

-- ftd.text-input:
placeholder: Type Something Here...
multiline: true
```

## `ftd.text-input` attributes

### `placeholder`: `optional string` -> Adjusts a visible text inside the input field
### `value`: `optional string`
### `default-value`: `optional string`
### `enabled`: `optional boolean` -> Sets whether the text-input is enabled or disabled
### `multiline`: `optional boolean` -> To allow multiline input
### `type`: `optional ftd.text-input-type` -> Sets the type of text input

```ftd
-- or-type text-input-type:
-- constant string text: text
-- constant string email: email
-- constant string password: password
-- constant string url: url
-- end: text-input-type
```

By default, `type` is set to `ftd.text-input-type.text`

## `ftd.checkbox` - To render a checkbox

This code will create a simple checkbox.
```ftd
-- ftd.checkbox:
```

To know the current value of checkbox, you can use
a special variable `$CHECKED` to access it.

```ftd
-- boolean $is-checked: false

-- ftd.checkbox:
$on-click$: $ftd.set-bool($a = $is-checked, v = $CHECKED)
```

## `ftd.checkbox` Attributes

### `checked`: `optional boolean` -> Default checkbox value
### `enabled`: `optional boolean` -> Sets whether the checkbox is enabled or disabled

By default, `checkbox` is not selected

```ftd
;; In this case, the checkbox will be 
;; pre-selected by default

-- ftd.checkbox:
checked: true
```

## `ftd.image` - To render an image

```ftd 
-- ftd.image: 
src: $assets.files.static.fifthtry-logo.svg
```

## `ftd.image` attributes

### `src`: `ftd.image-src`

```ftd
-- record image-src:
string light:
string dark: $light
```




## Common Attributes

- `id`: `optional string`
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

-- integer px:
-- decimal percent:
-- string calc:
-- integer vh:
-- integer vw:
-- integer vmin:
-- integer vmax:
-- decimal dvh;
-- decimal lvh;
-- decimal svh;
-- decimal em:
-- decimal rem:
-- ftd.responsive-length responsive:

-- end: length

-- record responsive-length:
ftd.length desktop:
ftd.length mobile: $responsive-length.desktop
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
- `width`: `optional ftd.resizing` (default: `auto`)
- `height`: `optional ftd.resizing` (default: `auto`)


**`ftd.resizing`**

```ftd
-- or-type resizing:

-- constant string fill-container: fill-container
-- constant string hug-content: hug-content
-- constant string auto: auto
-- ftd.length fixed:

-- end: resizing
```

- `link`: `string`
- `open-in-new-tab`: `optional boolean`

- `background`: `optional ftd.background`


**`ftd.background`**

```ftd
-- or-type background:

-- ftd.color solid:
-- ftd.background-image image:

-- end: background
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

- `cursor`: `optional ftd.cursor`


**`ftd.cursor`**

```ftd
-- or-type cursor:

-- constant string default: default
-- constant string none: none
-- constant string context-menu: context-menu
-- constant string help: help
-- constant string pointer: pointer
-- constant string progress: progress
-- constant string wait: wait
-- constant string cell: cell
-- constant string crosshair: crosshair
-- constant string text: text
-- constant string vertical-text: vertical-text
-- constant string alias: alias
-- constant string copy: copy
-- constant string move: move
-- constant string no-drop: no-drop
-- constant string not-allowed: not-allowed
-- constant string grab: grab
-- constant string grabbing: grabbing
-- constant string e-resize: e-resize
-- constant string n-resize: n-resize
-- constant string ne-resize: ne-resize
-- constant string nw-resize: nw-resize
-- constant string s-resize: s-resize
-- constant string se-resize: se-resize
-- constant string sw-resize: sw-resize
-- constant string w-resize: w-resize
-- constant string ew-resize: ew-resize
-- constant string ns-resize: ns-resize
-- constant string nesw-resize: nesw-resize
-- constant string nwse-resize: nwse-resize
-- constant string col-resize: col-resize
-- constant string row-resize: row-resize
-- constant string all-scroll: all-scroll
-- constant string zoom-in: zoom-in
-- constant string zoom-out: zoom-out

-- end: cursor
```

- `region`: `optional ftd.region`

**`ftd.region`**

```ftd
;; NOTE
;; 1. Using conditionals with region is not supported yet.
;; 2. Only one region can be specified as region value.

-- or-type region:

-- constant string h1: h1
-- constant string h2: h2
-- constant string h3: h3
-- constant string h4: h4
-- constant string h5: h5
-- constant string h6: h6

-- end: region
```

- `white-space`: `optional ftd.white-space`

**`ftd.white-space`**

```ftd 

-- or-type white-space:

-- constant string normal: normal
-- constant string nowrap: nowrap
-- constant string pre: pre 
-- constant string pre-wrap: pre-wrap
-- constant string pre-line: pre-line
-- constant string break-spaces: break-spaces

-- end: white-space
```

- `text-transform`: `optional ftd.text-transform`

**`ftd.text-transform`**

```ftd 
-- or-type text-transform:

-- constant string none: none
-- constant string capitalize: capitalize
-- constant string uppercase: uppercase
-- constant string lowercase: lowercase
-- constant string initial: initial 
-- constant string inherit: inherit

-- end: text-transform
```

- `classes`: string list (classes are created in css)


- `border-style`: `optional ftd.border-style list`

**`ftd.border-style`**

```ftd
-- or-type border-style:

-- constant string dotted: dotted
-- constant string dashed: dashed
-- constant string solid: solid
-- constant string double: double
-- constant string groove: groove 
-- constant string ridge: ridge
-- constant string inset: inset
-- constant string outset: outset

-- end: border-style
```


## Container Attributes

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


- `spacing`: `optional ftd.spacing`

**`ftd.spacing`**

```ftd
-- or-type spacing:

-- ftd.length fixed:
-- constant string space-between: space-between
-- constant string space-around: space-around
-- constant string space-evenly: space-evenly

-- end: spacing
```

- `resize`: `optional ftd.resize`


**`ftd.resize`**

```ftd
-- or-type resize:

-- constant string both: both
-- constant string horizontal: horizontal
-- constant string vertical: vertical

-- end: resize
```

- `role`: `optional ftd.responsive-type`


```ftd
-- record responsive-type:
caption ftd.type desktop:
ftd.type mobile: $responsive-type.desktop

-- record type:
optional ftd.font-size size:
optional ftd.font-size line-height:
optional ftd.font-size letter-spacing:
optional integer weight:
optional string font-family:

-- or-type font-size:

-- integer px:
-- decimal em:
-- decimal rem:

-- end: font-size
```


- `anchor`: `optional ftd.anchor`

```ftd
-- or-type anchor:

-- constant string parent: absolute
-- constant string window: fixed
-- string id:

-- end: anchor
```

- `z-index`: `optional integer`

# Text Attributes

- `text-align`: `ftd.text-align`

### `ftd.text-align`

```ftd
-- or-type text-align:
-- constant string start: start
-- constant string center: center
-- constant string end: end
-- constant string justify: justify
-- end: text-align
```

- `line-clamp`: `optional integer`
- `sticky`: `optional boolean`

# Events

- `on-click`
- `on-change`
- `on-input`
- `on-blur`
- `on-focus`
- `on-mouse-enter`
- `on-mouse-leave`
- `on-click-outside`
- `on-global-key[<keys>]`
- `on-global-key-seq[<keys>]`

# Default functions

## `append($a: mutable list, v: string)`

This is a default ftd function that will append a string `v`
to the end of the given mutable string list `a`.

```ftd
-- void append(a,v):
string list $a:
string v:

ftd.append(a, v);
```

## `insert_at($a: mutable list, v: string, num: integer)`

This is a default ftd function that will insert a string `v`
at the index `num` in the given mutable string list `a`.

```ftd
-- void insert_at(a,v,num):
string list $a:
string v:
integer num:

ftd.insert_at(a, v, num);
```

## `delete_at($a: mutable list, v: integer)`

This is a default ftd function that will delete the string
from index `num` from the given mutable string list `a`.

```ftd
-- void delete_at(a,num):
string list $a:
integer num:

ftd.delete_at(a, num);
```

## `clear($a: mutable list)`

This is a default ftd function that will clear the given
mutable string list `a`.

```ftd
-- void clear(a):
string list $a:

ftd.clear(a);
```

## `set-list($a: mutable list, v: list)`

This is a default ftd function that will assign a new list `v` to the
existing mutable list `a`.

```ftd
-- void set_list(a,v):
string list $a:
string list v:

ftd.set_list(a, v);
```

## `toggle($a: bool)`

This is FScript function. It will toggle the boolean variable which is passed 
as argument `a` to this function.

```ftd
-- boolean $b: false

-- ftd.boolean: $b

-- ftd.text: Click to toggle
$on-click$: $ftd.toggle($a = $b)
```

## `increment($a: integer)`

This is FScript function. It will increment the integer variable by 1 which is passed
as argument `a` to this function.

```ftd
-- integer $x: 1

-- ftd.integer: $x

-- ftd.text: Click to increment by 1 
$on-click$: $ftd.increment($a = $x)
```

## `increment-by($a: integer, v: integer)`

This is FScript function. It will increment the integer variable by value `v` which is passed
as argument `a` to this function.

```ftd
-- integer $x: 1

-- ftd.integer: $x

-- ftd.text: Click to increment by 5 
$on-click$: $ftd.increment-by($a = $x, v = 5)
```

## `set-bool($a: bool, v: bool)`

This is FScript function. It will set the boolean variable by value `v` which is passed
as argument `a` to this function.

```ftd
-- boolean $b: false

-- ftd.boolean: $b

-- ftd.text: Click to set the boolean as true 
$on-click$: $ftd.set-bool($a = $b, v = true)
```

## `set-string($a: string, v: string)`

This is FScript function. It will set the string variable by value `v` which is passed
as argument `a` to this function.

```ftd
-- string $s: Hello

-- ftd.text: $s

-- ftd.text: Click to set the string as World 
$on-click$: $ftd.set-string($a = $s, v = World)
```

## `set-integer($a: integer, v: integer)`

This is FScript function. It will set the integer variable by value `v` which is passed
as argument `a` to this function.

```ftd
-- integer $x: 1

-- ftd.integer: $x

-- ftd.text: Click to set the integer as 100 
$on-click$: $ftd.set-integer($a = $x, v = 100)
```


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

This is FScript as well as a standard ftd function. This function enables the dark mode.

```ftd
-- ftd.text: Dark Mode
$on-click$: $set-dark()

-- void set-dark():

enable_dark_mode()
```

Alternatively you can do
```ftd
-- ftd.text: Click to set Dark Mode
$on-click$: $ftd.enable-dark-mode()
```

## `enable_light_mode()`

This is FScript as well as a standard ftd function. This function enables the light mode.

```ftd
-- ftd.text: Light Mode
$on-click$: $set-light()

-- void set-light():

enable_light_mode()
```

Alternatively you can do
```ftd
-- ftd.text: Click to set Light Mode
$on-click$: $ftd.enable-light-mode()
```

## `enable_system_mode()`

This is FScript as well as a standard ftd function. This function enables the system mode.

```ftd
-- ftd.text: System Mode
$on-click$: $set-system()

-- void set-system():

enable_system_mode()
```

Alternatively you can do
```ftd
-- ftd.text: Click to set System Mode
$on-click$: $ftd.enable-system-mode()
```


## `ftd.copy_to_clipboard()`

This is FScript as well as a standard ftd function. This function enables copy content in clipboard.

```ftd
-- ftd.text: Copy
$on-click$: $copy-me-call(text = Copy me ⭐️)

-- void copy-me-call(text):
string text:

ftd.copy_to_clipboard(text)
```

Alternatively you can do
```ftd
-- ftd.text: Click to set System Mode
$on-click$: $ftd.copy-to-clipboard(a = Copy me ⭐️)
```

## `ftd.http(url: string, method: string, ...request-data)`

This function is used to make http request

- For `GET` requests, the request-data will sent as `query parameters`

- For `POST` requests, the request-data will be sent as request `body`

- We can either pass `named` data or `unnamed` data as
  `request-data` values.

- For `named` data, the values need to be passed as `(key,value)` tuples.

  For example `ftd.http("www.fifthtry.com", "post", ("name": "John"),("age": 25))`

  request-data = `{ "name": "John", "age": 25 }`

```ftd
-- ftd.text: Click to send POST request
$on-click$: $http-call(url = https://www.fifthtry.com, method = post, name = John, age = 23)

-- void http-call(url,method,name,age):
string url:
string method:
string name:
integer age: 

;; Named request-data
ftd.http(url, method, ("name": name),("age": age))
```

- For `unnamed` data, i.e when keys are not passed with data values, then the keys will be indexed 
  based on the order in which these values are passed. 
 
  For example `http("www.fifthtry.com", "post", "John", 25)`

  request-data = `{ "0": "John", "1": 25 }`

```ftd
-- ftd.text: Click to send POST request
$on-click$: $http-call(url = https://www.fifthtry.com, method = post, name = John, age = 23)

-- void http-call(url,method,name,age):
string url:
string method:
string name:
integer age: 

;; Unnamed request-data
http(url, method, name, age)
```

- In case if a unnamed `record` variable is passed as request-data,
  in that case, the record's `field` names will be used as key values.

  For example: Let's say we have a `Person` record, and we have created a `alice` 
  record of `Person` type. And if we pass this `alice` variable as request-data
  then request-data will be `{ "name" : "Alice", "age": 22 }`

```ftd
-- record Person: 
caption name: 
integer age: 

-- Person alice: Alice
age: 22

-- ftd.text: Click to send POST request
$on-click$: $http-call(url = https://www.fifthtry.com, method = post, person = $alice)

-- void http-call(url,method,person):
string url:
string method:
Person person:

;; Unnamed record as request-data
http(url, method, person)
```

Response JSON:

- To redirect:
```json
{
  "redirect": "fifthtry.com"
}
```


- To reload:
```json
{
  "reload": true
}
```

- To update ftd data from backend:
```json
{
  "data": {
    "<module-name>#<variable-name>": <value>
  }
}
```

- [Discussions#511](https://github.com/ftd-lang/ftd/discussions/511)

- To update error data:
```json
{
  "errors": {
    "<module-name>#<variable-name>": <value>
  }
}
```

- [Discussions#511](https://github.com/ftd-lang/ftd/discussions/511)


# Some frequently used functions

## Clamp

- Regular Clamp

```ftd
-- integer $num: 0

-- ftd.integer: $num
$on-click$: $clamp($a = $num, by = 1, clamp = 6)

-- void clamp(a,by,clamp):
integer $a:
integer by:
integer clamp:


a = (a + by) % clamp
```

- Clamp with min and max

```ftd
-- integer $num: 1

-- ftd.integer: $num
$on-click$: $clamp($a = $num, by = 1, min = 1, max = 6)

-- void clamp(a,by,min,max):
integer $a:
integer by: 1
integer min: 0
integer max: 5


a = (((a - min) + by) % (max - min)) + min
```




# How to run examples for FTD: 0.3

- Create empty files `<number>-<name>.ftd` and `<number>-<name>.html` in `t/html` 
  folder.
- Run `cargo test html_test_all -- --nocapture fix=true <optional: path=<prefix of the file name>>`

## Optional commands to check html in examples
- Run `cargo run`
- Run `cd docs`
- Run `python3 -m http.server 8000`


# Default Variable

## `ftd.device`

The `ftd.device` variable is a mutable variable of type `ftd.
device-data` which is an or-type.

```ftd
-- or-type device-data:

-- constant string mobile: mobile
-- constant string desktop: desktop

-- end: device-data

-- ftd.device-data $device: desktop
```

## `ftd.breakpoint-width`

The `ftd.breakpoint-width` variable is a mutable variable of type `ftd.
breakpoint-width-data` which is a record.

```ftd
-- record breakpoint-width-data:
integer mobile:

-- ftd.breakpoint-width-data $breakpoint-width:
mobile: 768
```

## `ftd.font-display`
This variable is a mutable string variable which can be 
used to change the font family of `headings` and `labels` 
under inherited types which includes `heading-large`, 
`heading-medium`, `heading-small`,`heading-hero`, 
`label-big`, `label-small`

By default `ftd.font-display` is set to `sans-serif`

```ftd
-- $ftd.font-display: cursive

-- ftd.text: Hello world
role: $inherited.types.heading-large
```

## `ftd.font-copy`
This variable is a mutable string variable which can be
used to change the font family of `copy` type fonts
under inherited types which includes `copy-tight`,
`copy-relaxed` and `copy-large`

By default `ftd.font-copy` is set to `sans-serif`

```ftd
-- $ftd.font-copy: cursive

-- ftd.text: Hello world
role: $inherited.types.copy-large
```

## `ftd.font-code`
This variable is a mutable string variable which can be
used to change the font family of `fine-print` and `blockquote` fonts
under inherited types.

By default `ftd.font-code` is set to `sans-serif`

```ftd
-- $ftd.font-code: cursive

-- ftd.text: Hello world
role: $inherited.types.fine-print
```

## `inherited.types`

The `inherited.types` variable is of type `ftd.type-data` which is a record.

```ftd
-- record type-data:
ftd.responsive-type heading-hero:
ftd.responsive-type heading-large:
ftd.responsive-type heading-medium:
ftd.responsive-type heading-small:
ftd.responsive-type heading-tiny:
ftd.responsive-type copy-large:
ftd.responsive-type copy-regular:
ftd.responsive-type copy-small:
ftd.responsive-type fine-print:
ftd.responsive-type blockquote:
ftd.responsive-type source-code:
ftd.responsive-type label-large:
ftd.responsive-type label-small:
ftd.responsive-type button-large:
ftd.responsive-type button-medium:
ftd.responsive-type button-small:
ftd.responsive-type link:
```

The fields in record `type-data` are of type `ftd.responsive-type` which is 
another record with `desktop` and `mobile` fields of type `ftd.type`

In `inherited.types` variable, value of all the fields is same for both 
`desktop` and `mobile`. So just mentioning the value for one only.

For desktop: 

- `heading-hero`:
  size: 80
  line-height: 104
  weight: 400

- `heading-large`:
  size: 50
  line-height: 65
  weight: 400

- `heading-medium`:
  size: 38
  line-height: 57
  weight: 400

- `heading-small`:
  size: 24
  line-height: 31
  weight: 400

- `heading-tiny`:
  size: 20
  line-height: 26
  weight: 400

- `copy-large`:
  size: 22
  line-height: 34
  weight: 400

- `copy-regular`:
  size: 18
  line-height: 30
  weight: 400

- `copy-small`:
  size: 14
  line-height: 24
  weight: 400

- `fine-print`:
  size: 12
  line-height: 16
  weight: 400

- `blockquote`:
  size: 16
  line-height: 21
  weight: 400

- `source-code`:
  size: 18
  line-height: 30
  weight: 400

- `label-large`:
  size: 14
  line-height: 19
  weight: 400

- `label-small`:
  size: 12
  line-height: 16
  weight: 400

- `button-large`:
  size: 18
  line-height: 24
  weight: 400

- `button-medium`:
  size: 16
  line-height: 21
  weight: 400

- `button-small`:
  size: 14
  line-height: 19
  weight: 400

- `link`:
  size: 14
  line-height: 19
  weight: 400


## `inherited.colors`

The `inherited.colors` variable is of type `ftd.color-scheme` which is a record.

```ftd
-- record color-scheme:
ftd.background-colors background:
ftd.color border:
ftd.color border-strong:
ftd.color text:
ftd.color text-strong:
ftd.color shadow:
ftd.color scrim:
ftd.cta-colors cta-primary:
ftd.cta-colors cta-secondary:
ftd.cta-colors cta-tertiary:
ftd.cta-colors cta-danger:
ftd.pst accent:
ftd.btb error:
ftd.btb success:
ftd.btb info:
ftd.btb warning:
ftd.custom-colors custom:

-- record background-colors:
ftd.color base:
ftd.color step-1:
ftd.color step-2:
ftd.color overlay:
ftd.color code:

-- record cta-colors:
ftd.color base:
ftd.color hover:
ftd.color pressed:
ftd.color disabled:
ftd.color focused:
ftd.color border:
ftd.color text:

-- record pst:
ftd.color primary:
ftd.color secondary:
ftd.color tertiary:

-- record btb:
ftd.color base:
ftd.color text:
ftd.color border:

-- record custom-colors:
ftd.color one:
ftd.color two:
ftd.color three:
ftd.color four:
ftd.color five:
ftd.color six:
ftd.color seven:
ftd.color eight:
ftd.color nine:
ftd.color ten:
```

The `inherited.colors` has following value:

- `background`:
  1. `base`: `#18181b`
  2. `step-1`: `#141414`
  3. `step-2`: `#585656`
  4. `overlay`: `rgba(0, 0, 0, 0.8)`
  3. `code`: `#2B303B`
  
- `border`: `#434547`
- `border-strong`: `#919192`
- `text`: `#a8a29e`
- `text-strong`: `#ffffff`
- `shadow`: `#007f9b`
- `scrim`: `#007f9b`
- `cta-primary`:
  1. `base`: `#2dd4bf`
  2. `hover`: `#2c9f90`
  3. `pressed`: `#2cc9b5`
  4. `disabled`: `rgba(44, 201, 181, 0.1)`
  5. `focused`: `#2cbfac`
  6. `border`: `#2b8074`
  7. `text`: `#feffff`
- `cta-secondary`:
  1. `base`: `#4fb2df`
  2. `hover`: `#40afe1`
  3. `pressed`: `#4fb2df`
  4. `disabled`: `rgba(79, 178, 223, 0.1)`
  5. `focused`: `#4fb1df`
  6. `border`: `#209fdb`
  7. `text`: `#ffffff`
- `cta-tertiary`:
  1. `base`: `#556375`
  2. `hover`: `#c7cbd1`
  3. `pressed`: `#3b4047`
  4. `disabled`: `rgba(85, 99, 117, 0.1)`
  5. `focused`: `#e0e2e6`
  6. `border`: `#e2e4e7`
  7. `text`: `#ffffff`
- `cta-danger`:
  1. `base`: `#1C1B1F`
  2. `hover`: `#1C1B1F`
  3. `pressed`: `#1C1B1F`
  4. `disabled`: `#1C1B1F`
  5. `focused`: `#1C1B1F`
  6. `border`: `#1C1B1F`
  7. `text`: `#1C1B1F`
- `accent`:
  1. `primary`: `#2dd4bf`
  2. `secondary`: `#4fb2df`
  3. `tertiary`: `#c5cbd7`
- `error`:
  1. `base`: `#f5bdbb`
  2. `text`: `#c62a21`
  3. `border`: `#df2b2b`
- `success`:
  1. `base`: `#e3f0c4`
  2. `text`: `#467b28`
  3. `border`: `#3d741f`
- `info`:
  1. `base`: `#c4edfd`
  2. `text`: `#205694`
  3. `border`: `#205694`
- `warning`:
  1. `base`: `#fbefba`
  2. `text`: `#966220`
  3. `border`: `#966220`
- `custom`:
  1. `one`: `#ed753a`
  1. `two`: `#f3db5f`
  1. `three`: `#8fdcf8`
  1. `four`: `#7a65c7`
  1. `five`: `#eb57be`
  1. `six`: `#ef8dd6`
  1. `seven`: `#7564be`
  1. `eight`: `#d554b3`
  1. `nine`: `#ec8943`
  1. `ten`: `#da7a4a`


## Understanding Loop

`$loop$` loops over each item in an array, making the item available in a 
context argument in component


```ftd
-- string list names:

-- string: Ayushi
-- string: Arpita

-- end: names

-- ftd.text: $obj
$loop$: $names as $obj
```

The output would be:

```
Ayushi
Arpita
```

### `LOOP.COUNTER`

The current iteration of the loop (0-indexed)

```ftd
-- string list names:

-- string: Ayushi
-- string: Arpita

-- end: names

-- foo: $obj
idx: $LOOP.COUNTER
$loop$: $names as $obj


-- component foo:
caption name:
integer idx:

-- ftd.row:
spacing.px: 30

-- ftd.text: $foo.name
-- ftd.integer: $foo.idx

-- end: ftd.row

-- end: foo
```

The output would be:

```
Ayushi     0
Arpita     1
```
