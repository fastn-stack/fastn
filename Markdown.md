# Thoughts on Source Code Syntax Highlighting

We can have code in three possible ways: 1. inline code in markdown, 2. code blocks in
markdown and 3. code section.

We can get rid of 2, though it will make it hard for people to copy paste markdown
content to `.5m` files. Why would we want to do it? Because we want "one way to do it",
and code sections are superior to markdown in nearly every way:

1. In markdown, triple ticks before and after, vs `-- code`, same number of characters,
   but much more clearer.
2. Code gets proper "title", useful for code listing in table of content etc, and useful
   in general.
3. Support for optional line numbering, highlights and pointers.

Other than features, main advantage is code in markdown gets rendered as HTML, where as
if code is sent in explicit way, lots of code related things can be done without
tinkering with markdowns renderer's html output.

How should we handle code highlighting then?

## Support For Custom Rendering

Markdown compiles everything to HTML, which is good for dumping things in browser, but
not so much for custom renderers, eg native code, or eg scripts extracting information
from books.

Our books should be exportable in a format that allows custom renderers, book readers,
to be written to extend features.

For this we have to allow both basic output, eg raw source, that want to do custom code
parsing/highlighting, and to also output "rendered html", so basic renderers still get
benefit of first class code rendering.

## Rendered HTML

Code can be seen as a list of spans, `Vec<(Vec<class>, text)>`. We have a vec of class
to handle "highlights", first class say text color, and second class for highlighting.

For pointers we can similarly have some classes represent each pointer.

Should one be able to put formula in code comment and expect it to be rendered using
katex etc? The explanation of each pointer can be arbitrary markdown with inline
formula.

Should links in code be clickable? If desired we can make it
`Vec<(Vec<class>, text, Option<link_target>)`.

## Full Code Examples

A lot of times we write a very small snippet of code, which is not standalone, in the
interest of conciseness, but such snippets are not independently runnable.

Often writing something that is independently runnable is going to be easy, but where
it is possible, maybe there should be a way to write full examples, focus on a specific
area, and let client copy the entire script, or see code in "context".

## Code Playgrounds

Many languages have some way to try the code out, eg javascript, rust, go, each have
their playground versions. Our renderer should be able to use them, and other renderers
could extend this.





# Thoughts on Markdown

Unlike code, which can represented as list of spans, markdown gets compiled into nested
spans. We can technically "de-nest"/"flatten" it, but will make it harder, especially
when we get latex support for math/formula.

The simplest thing for both code and markdown would be to render them in HTML and expect
clients to parse HTML instead of writing custom types/serializers etc.




# Editing Model

We are going to allow local editing, the preferred way of editing, to allow proper git
usage, local editors etc, which I think is superior editing experience. And once
something is "done", it can be published.

Other is remote editing, where authors use web based editor to edit things.

The two can conflict. Google docs etc allow remote only editing, which is a bit clunky,
automatic version creation, not sure how well merge etc works when more then one author
is simultaneously working. Its a lot of effort to get acceptable, and then too, its
probably inferior to local editing over all.

Other problem is collaboration, sending PRs is possible in local editing, but nearly
impossible in remote editing. Unless we replicate all of "fork-book", PR review tools,
maybe even CI to check for lints etc.

I think github works well and we should leverage it by letting people know about source
of each book, and letting people go do collaborate there.

Does it mean we should completely disallow remote editing? You can not edit at all
without creating a github account as well?

One has to use 5th-local CLI, should it be mandatory for editing?

## Thinking from Pro Writers POV

These people are definitely going to get 5ht-local as well as github account. They will
be merging edits on Github, publishing locally or maybe even using CI.

What is ideal from their point of view:

- good tool for their editor of choice, atom/jetbrains/sublime
- good local preview: auto reload maybe

## Thinking from Casual, let me fix this small problem POV

These people may love Wikipedia like, no barrier to entry, here is the text input, just
change.

Should we record each edit proposed here and somehow let the original maintainers know
about it? What if we simply apply the edit and let maintainers know full wikipedia
style?

Whats their ideal software choice:

- good web based editor with preview. maybe even wyswyg.
- history of edits to go back?
- for collaborators: submit edits, allow to be notified when request is accepted, or
  some other comments
- for maintainers: list of edits requested by others. merge tool?

## POV: bloggers / personal knowledge base

5th is also a good blogging platform. They would prefer quick on the go editing, and
would not want local tool.

## Pro and Con Analysis

Advantage Remote:

- MAJOR:   easier to get started flow - one of the biggest hurdles in adoption of
           new tools
- MAJOR:   not every author understands github etc
- MEDIUM:  no need to maintain a local cli tool

Advantage Local:

- MAJOR:   no need to make web based collaborative editor, its a hard problem, or its
           too limiting if doesnt have.
- MAJOR:   many projects are already maintained over github, people love collaborating
           there.
- MAJOR:   contribute travis conf for projects to publish books on fifthtry when merged
           to master
- DECIDER: there is lot less software to write for me. we can then focus on making the
           books look damn good when published, or making a kick-ass book readers,
           online and mobile native

## Thinking from Growth Hacking POV

Growth can come from:

- some successful open source etc books adopting the platform, eg rust book.
- some people moving their blogs here








# Include Handling

For local content, include can refer to local files, but once something is uploaded to
server, those files won't be available. How to handle this?

One option is we inline included stuff during upload, so server always ignores include
attribute and only looks at code, while local ignores body if include is present.

What should happen when we are exporting stuff from server to local? You may be
exporting it on a machine where you do not have the included bits present. Should we
return the code in the exported stuff?

If we include code, the push followed by pull wont be idempotent. If we do not, then
others may not be able to edit if they do not have source available too.

Simplest thing could be to not support include feature at all because of all this
complexity.  



# Local Content Organization

Authors can work in three modes:

1. They have more than one top level projects, eg `~/books/amitu/...` and
   `~/books/tinder/...`, content in one may refer to another, and they have write
   access to both.

2. They only work on their books, but they have more than one books they are working on
   eg `~/books/python` and `~/books/rust`, content in one may refer to another.

3. They have only one book they are working on, `~/book`, this contains both track
   `~/book/python.5t` and `~/book/python/foo.5m` etc.


Track IDs are aways "user-id/track-slug". Module IDs are always
"user-id/slash-separated-slug". If "user-id/foo" track mentions "./bar", its referring
to "user-id/bar", and locally it would be present in bar.5m next to foo.5m.

Eg amitu/python: python.5t refers to ./python/hello. Locally there must be a file
python.5t, and a folder python/hello.5m.

If module id in a track file doesnt start with ., its assumed to be global path, eg
track "amitu/python", say refers to "tinder/osx/iterm". If a a file
"tinder/osx/iterm.5m" or "../tinder/osx/iterm.5m" is found next to python.5m, it will
be used.
