<div align="center">

[![Contributors](https://img.shields.io/github/contributors/fastn-stack/fastn?color=dark-green)](https://github.com/fastn-stack/fastn/graphs/contributors)
[![Issues](https://img.shields.io/github/issues/fastn-stack/fastn)](https://github.com/fastn-stack/fastn/issues)
[![License](https://img.shields.io/github/license/fastn-stack/fastn)](https://github.com/fastn-stack/fastn/blob/main/LICENSE)
[![Discord](https://img.shields.io/discord/793929082483769345?logo=discord)](https://fastn.com/discord/)

</div>

<div align="center">
    <img src="assets/fastn.svg" width="150" alt="fastn"/>
</div>

# `fastn` - Full-stack Web Development Made Easy

`fastn` is a programming language and a web-framework for building user
interfaces and content-centric websites. `fastn` is easy to learn, especially
for non-programmers, but does not compromise on what you can build with it.

Install from https://fastn.com/install/ or download directly from [GitHub
Releases](https://github.com/fastn-stack/fastn/releases).

## Features

## Minimal Syntax

A Hello World program in `fastn`:

```ftd
;; comments begin with `;;`
;; save this file as index.ftd
-- ftd.text: Hello World! 😀
```

You'll also need a `FASTN.ftd` file that stores information about your fastn
package:

```ftd
-- import: fastn

;; your package name
-- fastn.package: my-first-fastn-package
```

Save these two files and run `fastn serve` from the project dir. Visit the
printed URL and you'll see "Hello World! 😀" printed in your browser.

In addition to `ftd.text`, other kernel components exist that helps you create
UIs. You can learn abou them at https://fastn.com/kernel/.

You can create custom components on top of these kernel components:

```ftd
;; Component Definition
-- component card:
;; these are the arguments along with their types. `caption` is just string
;; with a special position
caption title:
;; `ftd.image-src` is a record type that allows you to specify two image urls,
;; for dark and light mode.
ftd.image-src background:
;; `body` is a `string` type but gets a special position to help you write
;; multi-line texts.
body description:

;; component body begins after a newline
-- ftd.column:

-- ftd.image:
src: $card.background

-- ftd.text: $card.title
role: h2

-- ftd.text: $card.description

-- end: ftd.column

-- end: card

;; This is how you call the `card` component
-- card: Hello world! **markdown is supported!**
;; `$fastn-assets` is a special import. See: https://fastn.com/assets/
background: $fastn-assets.files.images.fastn.svg

A `body` is just a `string` type but gets a special position to help you
write multi-line texts. And markdown is supported so you can 
[ask for donation!](https://fastn.com/donate/) ;)
```

If you had used `string` instead of `caption` and `body` then you'd have to do:

```ftd
-- card: 
title: Hello world! **markdown is supported!**
background: $fastn-assets.files.images.fastn.svg

-- card.body:

A `body` is just a `string` type but gets a special position to help you
write multi-line texts. And markdown is supported so you can 
[ask for donation!](https://fastn.com/donate/) ;)

-- end: card
```

You can learn more about built in data types at https://fastn.com/built-in-types/.

A short **language tour** is available at https://fastn.com/geeks/.

## Routing

`fastn` support file-system based routing. For the following fs hierarchy:

```
├── ednet
│   ├── intro.ftd
│   └── xray.ftd
├── fastn
│   ├── index.ftd
├── FASTN.ftd
├── index.ftd
├── new.png
```

`/ednet/{intro, xray}`, `/fastn/`, `/` and `/new.png` URLs will be served by
`fastn serve` webserver automatically.

`fastn` also supports [dynamic-urls](https://fastn.com/dynamic-urls/),
[sitemap](https://fastn.com/understanding-sitemap/-/build/) and,
[url-mappings](https://fastn.com/redirects/-/backend/).

## Processors

processors are executed on the server side, and can be used to fetch data from
APIs, databases, or any other source. They are used to collect data before
rendering it on the client side.

```ftd
-- import: fastn/processors

-- record user:
string email:
string name:

-- user my-user:
$processor$: processors.http
url: https://jsonplaceholder.typicode.com/users/1

-- ftd.text: $my-user.email
```

See https://fastn.com/http/ to learn more about the `http` and other processors.

## `fastn` for Static Sites

`fastn` websites can be compiled into static html, css and, js and can be
deployed on any static hosting providers like [Github
Pages](https://fastn.com/github-pages/) and
[Vercel](https://fastn.com/vercel/).

## More Features

- Support for custom backends using WASM and [`ft-sdk`](https://github.com/fastn-stack/ft-sdk/).
- Support for custom css and js. See https://fastn.com/use-js-css/.
- First class support for for web-components. See https://fastn.com/web-component/.
- Easy to migrate from a static site generator like 11ty, Hugo, etc.
- Built-in package management system, opinionated [design
  system](https://design-system.fifthtry.site/), dark mode support, designed for
  [responsive UIs](https://fastn.com/making-responsive-pages/). Oh My!


## `fastn` 0.5

We're currently working on `fastn` 0.5, which will be a major release with
breaking changes. You can learn more about the development by going through
[existing github
discussions](https://github.com/orgs/fastn-stack/discussions?discussions_q=is%3Aopen+label%3A0.5-brainstorm).

## FifthTry Hosting

We, [FifthTry](https://www.fifthtry.com) also offer our own hosting solution for
your static and dynamic sites. Using FifthTry hosting frees you from devops
needs, and you get a fully integrated, managed hosting solution, that a
non-programmers can use with ease.

## Contributors

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tbody>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/Arpita-Jaiswal"><img src="https://avatars.githubusercontent.com/u/26044181?v=4?s=100" width="100px;" alt="Arpita Jaiswal"/><br /><sub><b>Arpita Jaiswal</b></sub></a><br /><a href="https://github.com/fastn-stack/fastn/commits?author=Arpita-Jaiswal" title="Code">💻</a> <a href="https://github.com/fastn-stack/fastn/commits?author=Arpita-Jaiswal" title="Documentation">📖</a> <a href="#example-Arpita-Jaiswal" title="Examples">💡</a> <a href="#eventOrganizing-Arpita-Jaiswal" title="Event Organizing">📋</a> <a href="#ideas-Arpita-Jaiswal" title="Ideas, Planning, & Feedback">🤔</a> <a href="#maintenance-Arpita-Jaiswal" title="Maintenance">🚧</a> <a href="#mentoring-Arpita-Jaiswal" title="Mentoring">🧑‍🏫</a> <a href="https://github.com/fastn-stack/fastn/pulls?q=is%3Apr+reviewed-by%3AArpita-Jaiswal" title="Reviewed Pull Requests">👀</a> <a href="#tool-Arpita-Jaiswal" title="Tools">🔧</a> <a href="https://github.com/fastn-stack/fastn/commits?author=Arpita-Jaiswal" title="Tests">⚠️</a> <a href="#tutorial-Arpita-Jaiswal" title="Tutorials">✅</a> <a href="#video-Arpita-Jaiswal" title="Videos">📹</a> <a href="#blog-Arpita-Jaiswal" title="Blogposts">📝</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://www.fifthtry.com"><img src="https://avatars.githubusercontent.com/u/58662?v=4?s=100" width="100px;" alt="Amit Upadhyay"/><br /><sub><b>Amit Upadhyay</b></sub></a><br /><a href="https://github.com/fastn-stack/fastn/commits?author=amitu" title="Code">💻</a> <a href="https://github.com/fastn-stack/fastn/commits?author=amitu" title="Documentation">📖</a> <a href="#example-amitu" title="Examples">💡</a> <a href="#eventOrganizing-amitu" title="Event Organizing">📋</a> <a href="#ideas-amitu" title="Ideas, Planning, & Feedback">🤔</a> <a href="#maintenance-amitu" title="Maintenance">🚧</a> <a href="#mentoring-amitu" title="Mentoring">🧑‍🏫</a> <a href="https://github.com/fastn-stack/fastn/pulls?q=is%3Apr+reviewed-by%3Aamitu" title="Reviewed Pull Requests">👀</a> <a href="#tool-amitu" title="Tools">🔧</a> <a href="https://github.com/fastn-stack/fastn/commits?author=amitu" title="Tests">⚠️</a> <a href="#tutorial-amitu" title="Tutorials">✅</a> <a href="#video-amitu" title="Videos">📹</a> <a href="#blog-amitu" title="Blogposts">📝</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/Heulitig"><img src="https://avatars.githubusercontent.com/u/106665190?v=4?s=100" width="100px;" alt="Rithik Seth"/><br /><sub><b>Rithik Seth</b></sub></a><br /><a href="https://github.com/fastn-stack/fastn/commits?author=Heulitig" title="Code">💻</a> <a href="https://github.com/fastn-stack/fastn/commits?author=Heulitig" title="Documentation">📖</a> <a href="https://github.com/fastn-stack/fastn/commits?author=Heulitig" title="Tests">⚠️</a> <a href="#ideas-Heulitig" title="Ideas, Planning, & Feedback">🤔</a> <a href="https://github.com/fastn-stack/fastn/pulls?q=is%3Apr+reviewed-by%3AHeulitig" title="Reviewed Pull Requests">👀</a> <a href="#maintenance-Heulitig" title="Maintenance">🚧</a> <a href="#blog-Heulitig" title="Blogposts">📝</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/gsalunke"><img src="https://avatars.githubusercontent.com/u/68585007?v=4?s=100" width="100px;" alt="Ganesh Salunke"/><br /><sub><b>Ganesh Salunke</b></sub></a><br /><a href="https://github.com/fastn-stack/fastn/commits?author=gsalunke" title="Code">💻</a> <a href="https://github.com/fastn-stack/fastn/commits?author=gsalunke" title="Documentation">📖</a> <a href="https://github.com/fastn-stack/fastn/commits?author=gsalunke" title="Tests">⚠️</a> <a href="#ideas-gsalunke" title="Ideas, Planning, & Feedback">🤔</a> <a href="#mentoring-gsalunke" title="Mentoring">🧑‍🏫</a> <a href="https://github.com/fastn-stack/fastn/pulls?q=is%3Apr+reviewed-by%3Agsalunke" title="Reviewed Pull Requests">👀</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/priyanka9634"><img src="https://avatars.githubusercontent.com/u/102957031?v=4?s=100" width="100px;" alt="Priyanka"/><br /><sub><b>Priyanka</b></sub></a><br /><a href="https://github.com/fastn-stack/fastn/commits?author=priyanka9634" title="Code">💻</a> <a href="https://github.com/fastn-stack/fastn/commits?author=priyanka9634" title="Documentation">📖</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/gargajit"><img src="https://avatars.githubusercontent.com/u/118595104?v=4?s=100" width="100px;" alt="Ajit Garg"/><br /><sub><b>Ajit Garg</b></sub></a><br /><a href="https://github.com/fastn-stack/fastn/commits?author=gargajit" title="Code">💻</a> <a href="https://github.com/fastn-stack/fastn/commits?author=gargajit" title="Documentation">📖</a> <a href="#blog-gargajit" title="Blogposts">📝</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/AbrarNitk"><img src="https://avatars.githubusercontent.com/u/17473503?v=4?s=100" width="100px;" alt="Abrar Khan"/><br /><sub><b>Abrar Khan</b></sub></a><br /><a href="https://github.com/fastn-stack/fastn/commits?author=AbrarNitk" title="Code">💻</a> <a href="https://github.com/fastn-stack/fastn/commits?author=AbrarNitk" title="Documentation">📖</a> <a href="https://github.com/fastn-stack/fastn/pulls?q=is%3Apr+reviewed-by%3AAbrarNitk" title="Reviewed Pull Requests">👀</a> <a href="https://github.com/fastn-stack/fastn/commits?author=AbrarNitk" title="Tests">⚠️</a></td>
    </tr>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/sharmashobhit"><img src="https://avatars.githubusercontent.com/u/1982566?v=4?s=100" width="100px;" alt="Shobhit Sharma"/><br /><sub><b>Shobhit Sharma</b></sub></a><br /><a href="https://github.com/fastn-stack/fastn/commits?author=sharmashobhit" title="Code">💻</a> <a href="https://github.com/fastn-stack/fastn/commits?author=sharmashobhit" title="Documentation">📖</a> <a href="https://github.com/fastn-stack/fastn/commits?author=sharmashobhit" title="Tests">⚠️</a></td>
      <td align="center" valign="top" width="14.28%"><a href="http://fifthtry.com"><img src="https://avatars.githubusercontent.com/u/106665143?v=4?s=100" width="100px;" alt="Aviral Verma"/><br /><sub><b>Aviral Verma</b></sub></a><br /><a href="https://github.com/fastn-stack/fastn/commits?author=AviralVerma13" title="Code">💻</a> <a href="https://github.com/fastn-stack/fastn/commits?author=AviralVerma13" title="Documentation">📖</a> <a href="https://github.com/fastn-stack/fastn/commits?author=AviralVerma13" title="Tests">⚠️</a> <a href="#ideas-AviralVerma13" title="Ideas, Planning, & Feedback">🤔</a></td>
    </tr>
  </tbody>
</table>

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

<!-- ALL-CONTRIBUTORS-LIST:END -->

## License

This project is licensed under the terms of the **UPL-1.0**.
