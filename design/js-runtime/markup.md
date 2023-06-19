# Markup

We have ftd specific extension to markdown, we call it `markup`. `markup` allows you to refer to specific component in
your text.

```ftd
-- ftd.text: hello {foo: markup}

-- component foo:
caption text:
-- end: foo
```

We have called the component `foo` using the `markup syntax`, `{<component-name>: <component-argument>}`. The component
could be defined in current module, or can be imported. If imported the full name has to be used, eg `foo.bar`. The
`<component-argument>` is passed as `caption` to the component, and if the component has marked the caption optional,
or provided a default value for caption, the `<component-argument>` can be omitted.

Currently the `markup` syntax does not allow you to pass any other argument, other than `caption`.

## Parsing Markup In Frontend

We are going to support markup syntax on dynamically constructed string, so frontend can generate strings, which may
refer to components which may not be present in the page at all. To ensure this does not happen we have to either place
some restrictions on the components you can use in markup, or we have to download component definitions on demand.

We are currently not considering download on demand. We are going to place restrictions on the components you can use.

## `always-include`

In normal mode we use tree shaking, any component that is not called when the page is getting rendered on the server
is not included in the bundle. We are going to allow a marker, `-- always-include: foo`, which will ensure that `foo`
is always included in the bundle.

## Missing Component

We will add `misssing-component` to `ftd.markdown` module, which will render the text with a red background. `doc-site`
etc can change the style to fit their theme.
