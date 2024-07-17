# fastn-preact

This is an attempt to create a fastn renderer using preact.

- [x] basic "mutable" and binding (event handling, conditional attributes)
- [x] component with mutable argument sent by caller
- [x] js-interop: set-value/get-value
- [x] record global, component mutates single field
- [ ] list global, component mutates single item
- [ ] list of record test
- [ ] global formula
- [ ] component level formula
- [ ] server side rendering
- [ ] processor

## Note On `useState` and Globals

We are using [preact's `useState`](https://preactjs.com/guide/v10/hooks/#usestate) as
the central state management mechanism. From their docs:

> When you call the setter and the state is different, it will trigger a rerender starting
> from the component where that useState has been used.

Since all globals are stored at top level node, any change in global will trigger
re-rendering of the entire dom tree.

Does the virtual dom diffing algorithm in preact is smart enough to only update the
changed nodes? Is this efficient?

One option we have is to "promote" globals to the nodes where they are used. E.g.,
if a `global` is only used by one `component`, can we store it in that component's
state?