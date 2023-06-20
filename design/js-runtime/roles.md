# roles

Some properties are not just values, but roles. The exact value that gets attached to the DOM depends on the role, and
is determined by the runtime. For example the color role keeps track of two colors, and based on user's color 
preferences, the color changes.

## CSS Classes

We use css classes to manage roles. For each role type, eg color is a role type, we have a unique prefix, eg all color
roles will have a class name starting with `c_`. The class name is generated from the role id, which is unique among
all roles for that role type. The ID is auto incremented integer, so the third role created will have the id 3.

When the role is created, we immediately create the corresponding class. When the role is attached to a DOM node, we
attach the corresponding class to the node.

Example:

```css
body.dark .c_3_color {
    color: red;
}

body.light .c_3_color {
    color: green;
}
```

The job of the runtime is to attach the correct class to the body. The descendent selector then ensures all elements
get the right color.

### `_color` suffix

We attached the role `c_3` to the DOM node as the `color` property. We encode the property we attach in the name of
the class.

```css
body.dark .c_3_border_color {
    border-color: red;
}

body.light .c_3_color > {
    border-color: green;
}
```

### SSR

In SSR mode we keep track of all unique classes used by the app. We then generate a CSS file with all the classes.

### Non SSR Mode

In regular mode, after page is running in browser, whenever a class is needed (is being attached to the node) and is 
found missing, we add it to the DOM. W can optimise it by keeping an in-memory cache of all the classes that are 
attached so far, and only add the missing ones. 

## color

We have a color type, with .dark and .light to represent colors in light and dark modes.

When we construct such a color in ftd, we create a `fastn.color()`. This will create a global role, with a unique
`id`. When the role is attached to DOM node, the class corresponding to the `id` will be added to the node.

```ftd
-- ftd.color c:
light: green
dark: red

-- ftd.text: hello
color: $c
```

```js
// note that `fastn.color` accepts both static and mutable values for the two colors
let c = fastn.color("green", "red");

// c.id is globally unique (among all colors), c.class_name is `c_{c.id}`. 
let e = fastn_dom.createKernel(parent, fastn_dom.ElementKind.Text);

// this attaches `c_{c.id}_color` class to the element
e.setProperty(fastn_dom.PropertyKind.Color, c);
```

## Type

For typography we use https://fastn.com/built-in-types#ftd-responsive-type, eg:

```ftd
-- ftd.type desktop-type:
size.px: 40
weight: 900
font-family: cursive
line-height.px: 65
letter-spacing.px: 5

-- ftd.type mobile-type:
size.px: 20
weight: 100
font-family: fantasy
line-height.px: 35
letter-spacing.px: 3

-- ftd.responsive-type responsive-typography:
desktop: $desktop-type
mobile: $mobile-type

-- ftd.text: Hello World
role: $responsive-typography
```

```js
let desktop_type = fastn.type({
    size: fastn.mutable(40),
    weight: fastn.mutable(900),
    font_family: fastn.mutable("cursive"),
    line_height: fastn.mutable(65),
    letter_spacing: fastn.mutable(5)
});
let mobile_type = fastn.type({
    size: fastn.mutable(20),
    weight: fastn.mutable(100),
    font_family: fastn.mutable("fantasy"),
    line_height: fastn.mutable(35),
    letter_spacing: fastn.mutable(3)
});

let responsive_typography = fastn.responsiveType({
    desktop: desktop_type,
    mobile: mobile_type
});

let text = fastn.text("Hello World");
text.setProperty(fastn.PropertyKind.Type, responsive_typography);
```

## Length

```ftd
-- ftd.responsive-length p:
desktop.px: 20
mobile.percent: 10

-- ftd.text: Hello
padding.responsive: $p
border-width.responsive: $p
```

```js
let p = fastn.responsiveLength({
    desktop: fastn.mutable({"px": 20}),
    mobile: fastn.mutable({"percent": 10})
});

let text = fastn.text("Hello");
// attaches `p_{p.id}_padding` class to the element
text.setProperty(fastn.PropertyKind.Padding, p);
// attaches `p_{p.id}_border_width` class to the element
text.setProperty(fastn.PropertyKind.BorderWidth, p);
```
