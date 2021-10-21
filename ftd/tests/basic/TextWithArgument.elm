module TextWithArgument exposing (..)

import Element as E
import F


hello : String -> F.Element msg
hello arg =
    F.e E.el [] (E.text arg)
