module UsingLib exposing (..)

import Element as E
import F
import Lib
import Lib2


hello1 : Lib.Msg -> F.Element msg
hello1 m =
    F.e E.el [] (E.text (Lib.msg m))


hello2 : Maybe Lib2.Msg -> F.Element msg
hello2 m =
    let
        m_ =
            Maybe.withDefault Bye m
    in
    F.e E.el [] (E.text (Lib2.msg m_))


hello3 : F.Element msg
hello3 =
    F.e E.el [] (E.text (Lib.msg Lib.Hello))
