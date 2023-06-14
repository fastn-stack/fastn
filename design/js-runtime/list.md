# How Do We Handle Lists?

```ftd
-- person $p: H

-- person list people:

-- person: A
-- person: B
-- person: $p

-- end: people

-- boolean $show-title: true

-- show-person: $p
for: ($p, $idx) in $people
idx: { idx + 2 }
show-title: $show-title

-- component show-person:
person $p:

-- ftd.text: $show-person.p.name

-- end: show-person
```

```js
let people = fastn.mutableList();

people.push({name: 'A'});
people.push({name: 'B'});

let show_title = fastn.mutable(true);

fastn.forLoop(root, people, [show_title], function(v) {
    // element_constructor
    return showPerson(v, show_title);
});


function showPerson(parent, person, show_title) {
    let n = fastn_dom.createKernel(parent, fastn_dom.ElementKind.Text);
    n.setDynamicProperty(fastn_dom.PropertyKind.StringValue, [person], function() {
        person.get()
    })
    n.done();
}
```

`fastn.listLoop` will create a fragment (`document.createDocumentFragment()`) and insert it in the root element. The
node returned by the constructor will be added in the fragment, not in the node.

Since a list should have common type, we should create a static class also, which has same signature as mutable, so code
works same on both static and mutable values.

We have to kind of modifications: a. modifying the elements of the list, b. modifying the list itself. 

To modify element of a list we do not have to do anything special. Eg if we modify `$p` things will just work, the
closure we passed in `showPerson()` would be triggered, it will return a new value, and DOM would get updated with that
value.

To modify the list itself, we have to call `push()`, `insertAt()` or `removeAt()` etc. If we add an element in the
end then also we have no issues, we create a new node. But if we insert an element in the middle, or we remove an
element from the middle. If the `element_constructor` is not dependent on the order, again we have no issue, we just
attach a new node in the fragment at the right place.

If the `element_constructor` is dependent on the order, then we have to do some work.
