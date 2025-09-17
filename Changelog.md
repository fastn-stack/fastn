# `fastn` Change Log

## 17 September 2025

### fastn: 0.4.113

- fix: Do not override query params of http processor's target url. PR #2209.

## 20 August 2025

### fastn: 0.4.112

- fix: Escape more chars while constructing a string for js output. See PR #2180.

### fastn: 0.4.111

- dce62c437 - Add support for simple function calls in `url` header of the `http` processor. See PR #2179.

## 29 July 2025

### fastn: 0.4.110

- d93c5b1da: Fix `for` loops counter when working across modules. See PR #2172 for more.

## 9 July 2025

### fastn: 0.4.109

- c17958678: Avoid infinite loop in certain conditions. See PR #2170 for more.

## 3 July 2025

### fastn: 0.4.108

- fix: Filter out `null` from url query params in the http processor.
- windows release: windows-2019 -> 2021 because 2019 is retired by Github.

## 30 June 2025

### fastn: 0.4.107

- bb676ea45 - `$ftd.set-current-language(lang: string)` function.
- 47c8e20a8 - Resolve requested language before auto processing imports.

## 19 June 2025

### fastn: 0.4.106

- 83cf66346 - Server render meta tags if request is from a bot.
- doc: `ftd.type`'s font-family attribute can take fallback fonts.

## 16 June 2025

### fastn: 0.4.105

- fix(http processor): Send unquoted string in url query param to preserve old
  behaviour.
- fix: Handle import of system packages from `inherited-` caller module
  correctly.
- fix: Only consider main package when resolving imports for `provided-via`.
- See: https://github.com/fastn-stack/fastn/pull/2151 for more details.

## 11 June 2025

### fastn: 0.4.104

- 348030b8a: Fix correctly reflect overriden components for system packages. See
  issue #2139.
- a42da86f6: Handle package local relative urls in http processor. See PR #2144.
- 7551fc8f4: Handle wasm modules in mountpoint of app dependencies when called
  through http processor. See PR #2144.

## 10 June 2025

### fastn: 0.4.103

- Fix: Send POST request body with a wasm+proxy:// url used in an http
  processor.
- 2757a1e68: Support for tuple style POST body params is in ftd.submit_form,
  this works similar to the ftd.http function.
- bcdf41325: Support form form level errors in ftd.submit_form.

## 25 May 2025

### fastn: 0.4.102

- 6e35b7911 - Build static fastn for x86_64-linux-gnu

## 09 May 2025

### fastn: 0.4.101

- Switch to UPL license.
- cfc0780b9: Fix: Consider `sitemap` items when resolving imports

## 28 March 2025

### fastn: 0.4.100

- Add `autofocus` attribute to `ftd.text-input` component.

# FTD Change Log (Old)

## 23 February 2023

- [Added web-component](https://github.com/ftd-lang/ftd/commit/f7c47c197f347bd2b48f0995b82aeaaf760ce44a)
- copy_to_clipboard -> ftd.copy_to_clipboard
- http -> ftd.http

## 2 February 2023

- [Added enabled property in ftd.checkbox and ftd.text-input](https://github.com/ftd-lang/ftd/commit/12425b68b56c2f475f3630ddb0484de70479aad0)

## 1 February 2023

<details>
<summary>Breaking Change: Renamed `fpm` To `fastn`</summary>
`fpm` cli is renamed to `fastn`. We renamed `FPM.ftd` to `FASTN.ftd` and 
`-- import: fpm` becomes `-- import: fastn`. We have also renamed github 
repository `fpm` to `fastn`.

- Fastn PR: https://github.com/ftd-lang/fastn/pull/755

</details>

<details>
<summary>Inbuilt <b>Clamp</b>: no longer supported
<a href="https://github.com/ftd-lang/ftd/blob/main/Cheatsheet.
md#clamp">Clamp example</a>
</summary>

- Regular Clamp

```ftd
-- integer $num: 0

-- ftd.integer: $num
$on-click$: $clamp($a = $num, by = 1, clamp = 6)

-- void clamp(a,by,clamp):
integer $a:
integer by:
integer clamp:


a = (a + by) % clamp
```

- Clamp with min and max

```ftd
-- integer $num: 1

-- ftd.integer: $num
$on-click$: $clamp($a = $num, by = 1, min = 1, max = 6)

-- void clamp(a,by,min,max):
integer $a:
integer by: 1
integer min: 0
integer max: 5


a = (((a - min) + by) % (max - min)) + min
```

</details>

## 31 January 2023

<details>

<summary><b>Breaking change</b> <a href="https://github.com/ftd-lang/ftd/pull/566/commits/e1722eacf386d3c515000c79a86e7ffb91f2df6c">Inherited types changed</a></summary>

Breaking changes

- `$inherited.types.copy-relaxed` -> `$inherited.types.copy-regular`
- `$inherited.types.copy-tight` -> `$inherited.types.copy-small`
- `$inherited.types.label-big` -> `$inherited.types.label-large`

Headings:

- `$inherited.types.heading-tiny` is added
- rest have weight, line-height, weight updates

Copy:

- added `$inherited.types.copy-regular` and `$inherited.types.copy-small`
- rest have size and `$inherited.types.line-height` changes

Specialized Text:

- `$inherited.types.source-code` is added
- rest have size and line-height changes

Labels:

- `$inherited.types.label-big` is changed to label-large
- `$inherited.types.label-small` is updated with new size and line-height values

Button:

- All button types which are added are new
- added `$inherited.types.button-large`, `$inherited.types.button-medium`,
  `$inherited.types.button-small`, link types

</details>

## 30 January 2023

- [Added ftd.checkbox](https://github.com/ftd-lang/ftd/pull/564/commits/483060b31dcce626599fc0bca8d7e6261d0c37a8)

## 27 January 2023

<details>

<summary><b>Breaking change</b>: <a href="https://github.com/ftd-lang/ftd/pull/557/commits/37569f9df70ce60f161a1260e6fa09827f8f0875">Merged spacing with spacing-mode</a></summary>

- use `spacing.fixed.px: 20` instead of `spacing.px: 20`
- use `spacing: space-around` instead of `spacing-mode: space-around` (same for
  `space-between` and `space-evenly`)

</details>

## 25 January 2023

- [Added sticky css](https://github.com/ftd-lang/ftd/pull/553/commits/a3b43d09b7b968d8242559e96dbff7c356104880)
- [Added
  `id` attr](https://github.com/ftd-lang/ftd/pull/554/commits/7321ba5253d565683e35e078606567f302633eaf)
- [Added slugify `id` for
  `region`s](https://github.com/ftd-lang/ftd/pull/556/commits/a419d0155bd4299c4efab91ad55557f92bc21f0f)
- [Added
  `LOOP.COUNTER`](https://github.com/ftd-lang/ftd/commit/9d31c722814d5cd9ded21be9de2b310b1d4cb0b8)

## 24 January 2023

- [Added border-style](https://github.com/ftd-lang/ftd/pull/549/commits/6f08e0ce2b9eeb5aa8da5bb418b60fcc0b221d05)
- [Added ftd.enable-dark-mode, ftd.enable-light-mode, ftd.enable-system-mode](https://github.com/ftd-lang/ftd/commit/723b1f50e3e1564c112c926ec024198fa843e42f)

## 23 January 2023

- [Added line-clamp](https://github.com/ftd-lang/ftd/pull/544/commits/b50d8ef371ead95679838e862d0ea956e7655b39)

## 19 January 2023

- [Added ftd.text-input](https://github.com/ftd-lang/ftd/pull/543/commits/b86f74b45322e53f8a9acf43155b4bb0aa1a19b3)

## 18 January 2023

- [Added on-blur, on-focus events](https://github.com/ftd-lang/ftd/pull/540/commits/d0416a7eb2d5b4fa6172b4f32cf442161427e4db)
- [Added on-change, on-input events](https://github.com/ftd-lang/ftd/commit/06d6d91fb10c63e01dbfbe02d4913b8b8e8f1594)
- [Added ftd.decimal](https://github.com/ftd-lang/ftd/pull/536/commits/114c1af8a9e159b11f9f2eb62dfd3839b1dd9e4b)
- [Added ftd fonts](https://github.com/ftd-lang/ftd/pull/535/commits/aeeb33f97645f97fc7360b46fe8ec9afc6d52416)

## 17 January 2023

- [Added
  `ftd.input`](https://github.com/ftd-lang/ftd/pull/535/commits/99702d33ce6b3485ed9a7481709cb85f3ee7fddf)

## 13 January 2023

- Major
  Change: [Converted executor from recursion to loop](https://github.com/ftd-lang/ftd/pull/529/commits/f305bc187f006bb49e2cbdaf1f35bbd62e151d67)

## 12 January 2023

- [Added
  `ftd.iframe`](https://github.com/ftd-lang/ftd/pull/523/commits/dbddbff69ff203e338b594f31c165a4fcf10afbe)
- [Added
  `z-index`](https://github.com/ftd-lang/ftd/pull/523/commits/6acf81e42290901ef127cf23687f39ea48e88d9a)

## 11th January 2023

- [Added text-transform css](https://github.com/ftd-lang/ftd/pull/529/commits/0cae01d1a5b9b7a3775bd60d1c36a8230e5d74cc)
- [Added `auto` variant in
  `ftd.resizing`](https://github.com/ftd-lang/ftd/pull/523/commits/939fce3398b6f5612eceffab8931c71d7341af55)

## 10th January 2023

- [Added white-space css](https://github.com/ftd-lang/ftd/pull/523/commits/af5b339f1b6ff04a0738dbbfda4186d57d27fd27)
- [Added basic ftd functions](https://github.com/ftd-lang/ftd/pull/524/commits/f268014568ef75e86e989ef80de0089ad614e07f)
- [Added
  `ftd.breakpoint-width`](https://github.com/ftd-lang/ftd/pull/524/commits/537b8cfd356f91e0059edbd04987c0a3f0dbf8a6)
- [
  `ftd.device` type string to or-type](https://github.com/ftd-lang/ftd/pull/524/commits/85da36d3eecddcefad8b3acc9800458d4c740f34)
- [Added
  `ftd.code`](https://github.com/ftd-lang/ftd/commit/5c5a8214d69276fe587949a364199ab8a2407e71)

## 9th January 2023

- [Added inherited type](https://github.com/ftd-lang/ftd/commit/b1fe8af46cd35c51c3b37312d9c1a6466a54d1e5)
- [Added inherited color](https://github.com/ftd-lang/ftd/commit/8c22529da64f449620f937ed18d466c6256dfb74)
- [Added ftd regions (v0.3)](https://github.com/ftd-lang/ftd/commit/cf460d1cc41734effc3cd998c943dc102eb4232d)

## 6th January 2023

- [Added `ftd.responsive-length` and `responsive` variant of type `ftd.
  responsive-length` in
  `ftd.length`](https://github.com/ftd-lang/ftd/commit/2376c2746670fc8fef67b909b5798bf16e3d8986)

## 5th January 2023

- [Added anchor, top, bottom, left and right property](https://github.com/ftd-lang/ftd/commit/d86de625f8786738862bc6aaf33cc8665c7f73f5)

## 4th January 2023

- [Added mouse-enter, mouse-leave, on-click-outside, on-global-key and on-global-key-seq](https://github.com/ftd-lang/ftd/commit/003f3262075abb009ace6cb76dbd9083d8a333a2)

## 3rd January 2023

- [Added role property](https://github.com/ftd-lang/ftd/commit/69bc02ad65358580f2247726aef78e1958b3716f)

## 2nd January 2023

- [Added cursor property and cursor as pointer in event](https://github.com/ftd-lang/ftd/commit/64aa657a13ab24d932d56a2ddf9bcb77982a7752)
- [Added http and copy_to_clipboard in build.js](https://github.com/ftd-lang/ftd/commit/7eb9e879ff94ced3ed53d7d1584d63975b1a6b2f)

## 30th December 2022

- Major Change: [`ftd.length variant from `anonymous record
  ` to `regular`](https://github.com/ftd-lang/ftd/commit/c4e7e591e515c5dfef1647e3f447e77a2f94c538)
- [Added set_value_by_id in js](https://github.com/ftd-lang/ftd/commit/e6f65267cbe57888e0fd510dd15bb56032bf8e7a)

## 29th December 2022

- Added CSS and JS
- Added classes property
- Added ftd.device

## 28th December 2022

- Added resize
- [Change min-height, min-width, max-width, max-height type from ftd.length to ftd.resizing](https://github.com/ftd-lang/ftd/commit/edad6b2899d940c11bd30c47fb15b08c6c04ad78)
- [or-type constant construction shorthand (only short-hand allowed)](https://github.com/ftd-lang/ftd/commit/a1ae3726eef848554ccf81a7f4270aeb6daa37ce)
  [The Video link](https://www.loom.com/share/ee239d4840a74eb087f53ad6445a49a8)

## 27th December 2022

- [Fix the stack overflow issue](https://github.com/ftd-lang/ftd/commit/d7438e7b0476be7cddf7ca5b67409f3515afb910)
- [Added benchmark](https://github.com/ftd-lang/ftd/commit/f7ed86c87f648547b1107c066383511645039290)
- [Added default function(is_empty, enable_dark_mode, enable_light_mode,
  enable_system_mode)](https://github.com/ftd-lang/ftd/commit/46d7a1596259e8a916d76228cb6997caaf3fb226)

## 26th December 2022

- [Added more variants in
  `ftd.length` (calc, vh, vw, em, rem)](https://github.com/ftd-lang/ftd/commit/60bd50c5a9306be1b305601c037e39810ef6206a)
- [Added
  `open-in-new-tab`](https://github.com/ftd-lang/ftd/commit/048024c468f8cc5a47f72dabdd2454499aaca314)

## 24th December 2022

- [created Cheatsheet files](https://github.com/ftd-lang/ftd/commit/8df76b5b66dd31b9c647a848c6dd4277b434c7fe)
