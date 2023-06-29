# How are we adding CSS properties to a node?

Earlier, we used to provide inline styles in DOM node. Now, we are using 
class. 

So let's say we have ftd file something like this

```ftd
-- ftd.text: Hello Ritesh
padding.px: 40
```

So we'll create corresponding class for each property (`padding`). To do 
this, we have created a function in js `attachCss`. 

## `attachCss` function

This function creates a unique class for each property and value pair. 
For instance, for above property `padding`, this function would create class,
`p-0`, lets say, which looks like this

```css
.p-0 {
    padding: 40px;
}
```

### In `ssr` mode

When our program runs in `ssr` mode, then all these classes get collected by 
`fastn_dom.classes` object and later converted into String by `fastn_dom.
getClassesAsString` function.

### In `normal` mode

When our program runs in `normal` mode, i.e., `client-side rendering` mode, 
then this function, first, tries to find corresponding class, if found then 
it attach the class to the node else it dynamically creates a class and 
attach it.

The problem with this approach is what if the value of property is mutable 
or is a formula having mutable value passed as parameter.

Consider the following code:

```ftd
-- integer $i: 1

-- ftd.text: Hello
margin.px: $i
$on-click$: { i = i + 1; }
```

Now for every click on the node, it will create a new class. Since the 
property `margin` has mutable value `i` which has infinite cardinality which 
in turn results in creating lots of classes. This is also true for 
properties having value as formula having mutable variables passed as parameter.

To save us from this insanity, we'll check if we are passing such values to 
the property, then we'll refrain `attachCss` from creating class and just 
attach inline style.


## `fastn_dom.getClassesAsString` function

This function converts the classes and it's properties stored in `fastn_dom.
classes` object to a corresponding string.

```js
fastn_dom.classes = {"p-0": {property: "padding", value: "40px"}}
```
For the above entry, the function will generate the following string

```css
.p-0 {
    padding: 40px;
}
```
