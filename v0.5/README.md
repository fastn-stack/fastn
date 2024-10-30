# fastn 0.5

You will find the implementation of 0.5 fastn in this folder. This is a complete rewrite of the language,
trying to preserve as much compatibility with previous version as possible.

## Known Compatibility Changes

### Block Section Headers

We are getting rid of block section headers. They were a not so great solution to the problem of how do we pass
complex data as headers. They were also solution to long lines or multiline headers.

Earlier we used:

```ftd
-- foo:
-- foo.bar: ;; bar is a subheader of foo

this can be multiline 

string, lots of lines
```

The new syntax allows:

```ftd
-- foo:
bar: {
    this can be multiline
    
    string, lots of lines
}
```

The indentation is optional and will be stripped (based on the line with the least indentation) from all lines.

We will keep the old syntax for a while, but it will be deprecated. This was not used a lot, so it should
not be a big problem.

### Function Arguments

There is no need to repeat of arguments when defining function. This was always pain, and never really needed.

```ftd
-- void foo(a):  ;; the `a` is not really needed.
integer a:

.. body skipped ..
```

New syntax:

```ftd
-- void foo(): 
integer a:

.. body skipped ..
```

We still need the `()` after `foo`, because we need to know that `foo` is a function.
