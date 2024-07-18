# fastn-preact

This is an attempt to create a fastn renderer using preact.

- [x] basic "mutable" and binding (event handling, conditional attributes)
- [x] component with mutable argument sent by caller
- [x] js-interop: set-value/get-value
- [x] record global, component mutates single field
- [x] list global, component mutates single item
- [x] record with two mutations (on the same click handle we want to modify two
  fields to see if they are updated together)
- [x] nested record
- [x] list with two mutations
- [x] list of record test
- [ ] global formula
- [ ] component level formula
- [ ] server side rendering
- [ ] processor

## Examples

There is a `examples/` folder. You can run `fastn serve` from that folder to run
it on a local server.

TODO: publish the examples on GitHub Pages.

### Rule: Each Example Builds On Previous

The code used in earlier example may differ from later example. We are building the
features up slowly, and each example is a snapshot of the code at that point.

## Note On `useState` and Globals

We are using [preact's `useState`](https://preactjs.com/guide/v10/hooks/#usestate) as
the central state management mechanism. From their docs:

> When you call the setter and the state is different, it will trigger a rerender starting
> from the component where that useState has been used.

Since all globals are stored at top level node, any change in global will trigger
re-rendering of the entire dom tree.

Is the virtual dom diffing algorithm in preact smart enough to only update the
changed nodes? Is this efficient?

One option we have is to "promote" globals to the nodes where they are used. E.g.,
if a `global` is only used by one `component`, can we store it in that component's
state?

## What We Are Not Doing

In current `fastn` implementation, this is possible:

```ftd
-- integer $x: 10
-- integer $y: 23


-- integer list $counters:

-- integer: 1
-- integer: $x
-- integer: $y
-- integer: 42

-- end: $counters
```

We have defined two globals, and used them in another global list. If we modify the
`$x`, the `$counter[1]` will be updated automatically. They are one and the same.

This is not achievable by the techniques we have seen till `08-nested-list`.

Do we want to keep this semantics? I came up with this example to show the semantics,
but I am not sure if this is a good idea. I have never needed this in my own projects.

One objection is what happens if the `$x` was defined on some UI node, and that goes
away. Do we want to keep that global around?

Other is: say instead of `$counters[1]` referring to `$x`, it referred to
`$other-list[2]`. If I overwrite the second list with a new list, do we expect
`$counters[1]` to be updated? We have cases equivalent to value/pointer, and pointer
to pointer semantics of C to consider here. Rust has single ownership semantics.

In light of these open questions, I am not sure if we want to keep this feature.