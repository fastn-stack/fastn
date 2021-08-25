module Enum exposing (..)

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


hello1 : Msg -> F.Element msg
hello1 m =
    F.e E.el [] (E.text (msg m))


hello2 : Maybe Msg -> F.Element msg
hello2 m =
    let
        m_ =
            Maybe.withDefault Bye m
    in
    F.e E.el [] (E.text (msg m_))


hello3 : F.Element msg
hello3 =
    F.e E.el [] (E.text (msg Hello))
