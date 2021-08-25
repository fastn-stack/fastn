module Responsive exposing (..)

import Element as E
import F


hello : F.Extra msg -> E.Element msg
hello e =
    case e.device of
        E.Phone ->
            F.e E.el [] (E.text "Hello World Mobile!")

        _ ->
            F.e E.el [] (E.text "Hello World!")
