# How Are We Handling Markdown?

Markdown parser creates a tree, with items like h1, link, list etc. We currently render these elements in HTML and
ftd authors can not change the way they are rendered. Say if we want to add an on-hover property to every inline `code`
block in Markdown text, we can not do it.

The design allows you to provide your own component constructors for every element in markdown.

## `ftd.markdown` module

We are creating a new module, `ftd.markdown`:

```ftd
;; content of ftd.markdown
-- component h1:
caption title:

-- end: h1
```

This module defines a component for each element we can encounter in markdown, e.g. h1, link etc.

## `markdown` argument to `ftd.text`

We add a new argument to `ftd.text`:

```ftd
-- component ftd.text:
caption or body text:
module markdown: ftd.markdown 
```

## `ds.markdown` module

`ds.markdown` component will provide their own module, `ds`.

```ftd
-- component markdown:
markdown: current-module 
```

We are planning `current-package`, so `current-module` goes well with it.

## JavaScript

```js
let t = fastn.mutable("# hello world");
let m = fastn.markdown(parent, [t], {h1: h1, link: link, list: list});

// for each h1 h2 etc we have a function defined already
function h1() {
    
}
```

Markdown parser will create a tree, and call `h1` etc. on the tree to convert it to a DOM tree. If the text changes 
entire DOM tree will be re-created.
