# Manrope : FPM Font Package

This repository contains a [fpm font package](https://fpm.dev/featured/fonts/) containing [Google Font: 
Manrope](https://fonts.google.com/specimen/maprope/about).


Manrope is an open-source modern sans-serif font family, designed by Mikhail Sharanda in 2018. In 2019, Mirko Velimirovic worked with Mikhail Sharanda to convert Manrope into a variable font.

Designers: Mikhail Sharanda, Principal design

## How To Use This Font In Your FPM Package:

[Read the docs and demo](https://fifthtry.github.io/manrope-font).

TLRD:

Include fifthtry.github.io/manrope-font package into `fastn.ftd` file:

```ftd
;-- fastn.dependency: fifthtry.github.io/manrope-font
```

Inside your `FPM/config.ftd` use the font:

```ftd
;-- import: fifthtry.github.io/manrope-font/assets as manrope

;-- fastn.type.headline-small: $manrope.fonts.Manrope
```

Now if in any file you do:

```ftd
;-- ftd.text:
role: $inherited.types.headline-small
```

You will see the `manrope` font.

## 👀 Want to learn more?

Feel free to check [our documentation](https://fastn.dev/) or jump into our [FifthTry Discord 
server](https://discord.gg/bucrdvptYd).

## License

Since Lato Font is under [Open Font Licence](https://fonts.google.com/specimen/Manrope/about?query=manrope), this FPM wrapper is also
under [Open Font Licence](LICENSE).





