<br/>

<div align="center">

![Contributors](https://img.shields.io/github/contributors/fifthtry/fpm?color=dark-green) ![Issues](https://img.shields.io/github/issues/fifthtry/fpm) ![License](https://img.shields.io/github/license/fifthtry/fpm) [![Discord](https://img.shields.io/discord/793929082483769345)](https://discord.com/channels/793929082483769345/)

</div>

# `fpm`: FTD Package Manager

[`ftd`](https://ftd.dev) is a language to create web pages or documents for 
publishing on the web. It starts with Markdown, but adds features to create full
page layouts, lets you create reusable "ftd components", and has first class 
support for data modelling, so the `ftd` document can be used as a data exchange
format as well (as a replacement of JSON/CSV etc.).

`fpm` is "`ftd` package manager", defines a package format for packaging `ftd` 
files. `fpm` packages can depend on other `fpm` packages, and `fpm` can install
all the dependencies of a package.

`fpm` can also convert `ftd` files to static HTML files, so you can publish FTD 
files on GitHub pages, S3 etc. static site hosting sites.


## `fpm-repo`

`fpm` packages can be shared with others by using `.tar.gz` format defined by `fpm`
or via Git repository (any version control system for that matter). `fpm` is 
capable of downloading package dependencies for packages that are hosted on version
control systems or tarballs served over HTTP.

`fpm` has a sister project, `fpm-repo` (under development), which can be used for
first class package hosting. You can choose if your `fpm` package would be natively
hosted on say GitHub or via a `fpm-repo` you have created or available as SAAS.

`fpm-repo` has advantages over GitHub etc. hosting as a `fpm` package is inherently 
a website, and `fpm-repo` shows the site directly, without a static build step.

`ftd` has some dynamic features, which an be used when using `fpm-repo`.

`fpm-repo` also offers web based editing experience, with built-in version control,
change request based workflow (equivalent to GitHub Pull Request workflow).
