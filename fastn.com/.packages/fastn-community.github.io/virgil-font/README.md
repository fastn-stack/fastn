# Raleway : FASTN Font Package

This repository contains a [fastn font package](https://fastn.com/featured/fonts/) containing [Virgil Font](https://github.com/excalidraw/virgil)).

The official font of [Excalidraw](https://github.com/excalidraw/virgil) by Ellinor Rapp.



## How To Use This Font In Your FASTN Package:

[Read the docs and demo](https://fastn-community.github.io/virgil-font).

TLRD:

Include fastn-community.github.io/virgil-font package into `FASTN.ftd` file:

```ftd
;-- fastn.dependency: fastn-community.github.io/virgil-font
```

Inside your `FASTN/ds.ftd` use the font:

```ftd
;-- import: fastn-community.github.io/virgil-font/assets as virgil

;-- fastn.type.headline-small: $virgil.fonts.virgil
```

Now if in any file you do:

```ftd
;-- ftd.text:
role: $inherited.types.headline-small
```

You will see the `Virgil` font.

## 👀 Want to learn more?

Feel free to check [our documentation](https://fastn.com/) or jump into our 
[Discord server](https://discord.gg/bucrdvptYd).

## License

Since Virgil  Font is under 
[OFL](https://github.com/excalidraw/virgil/blob/main/LICENSE.md), this FASTN wrapper is also under [OFL](LICENSE).




