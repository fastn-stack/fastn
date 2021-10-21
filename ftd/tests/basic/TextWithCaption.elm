module TextWithCaption exposing (..)

import Element as E
import F


hello : F.Element msg
hello =
    F.e E.el [] (E.text "Hello World!")
