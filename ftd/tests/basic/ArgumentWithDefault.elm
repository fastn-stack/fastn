module ArgumentWithDefault exposing (..)

import Element as E
import F


hello : Maybe String -> F.Element msg
hello arg =
    let
        arg_ =
            Maybe.withDefault "Hello World!" arg
    in
    F.e E.el [] (E.text arg_)
