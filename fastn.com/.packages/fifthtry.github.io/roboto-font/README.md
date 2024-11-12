# Roboto : FPM Font Package

This repository contains a [fpm font package](https://fpm.dev/featured/fonts/) containing [Google Font: 
Roboto](https://fonts.google.com/specimen/Roboto/about).

Roboto has a dual nature. It has a mechanical skeleton and the forms are largely
geometric. At the same time, the font features friendly and open curves. While 
some grotesks distort their letterforms to force a rigid rhythm, Roboto doesn’t
compromise, allowing letters to be settled into their natural width.

Designers: Christian Robertson, Principal design

## How To Use This Font In Your FPM Package:

[Read the docs and demo](https://fifthtry.github.io/roboto).

TLRD:

Include fifthtry.github.io/roboto package into `FPM.ftd` file:

```ftd
;-- fpm.dependency: fifthtry.github.io/roboto
```

Inside your `FPM/config.ftd` use the font:

```ftd
;-- import: fifthtry.github.io/roboto/assets as roboto

;-- fpm.type.headline-small: $roboto.fonts.Roboto
```

Now if in any file you do:

```ftd
;-- ftd.text:
role: $fpm.type.headline-small
```

You will see the `roboto` font.

## 👀 Want to learn more?

Feel free to check [our documentation](https://fpm.dev/) or jump into our [FifthTry Discord 
server](https://discord.gg/bucrdvptYd).

## License

Since Roboto  Font is under [Apache License](https://fonts.google.com/specimen/Roboto/about), this FPM wrapper is also
under [Apache License](LICENSE).




