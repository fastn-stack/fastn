module TextWithBody exposing (..)

import Element as E
import F


hello : F.Element msg
hello =
    let
        t1 : E.Element msg
        t1 =
            E.el [] (E.text "Hello World!")

        t2 : E.Element msg
        t2 =
            E.el [] (E.text "Hello World 2!")
    in
    F.e E.row [] [ t1, t2 ]
