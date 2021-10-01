module TextWithIntegerArgument exposing (..)

import Element as E
import Element.Font as EF
import F


hello : Int -> F.Element msg
hello arg =
    F.e E.el [ EF.size arg ] (E.text "Hello World!")
