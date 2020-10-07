module Lib exposing (..)

import Element as E
import F


type Msg
    = Hello
    | Bye


msg : Msg -> String
msg m =
    case m of
        Hello ->
            "Hello World!"

        Bye ->
            "Good Bye!"
