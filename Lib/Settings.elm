module Lib.Settings exposing (..)

import Element as E
import F
import System as S


page : F.Element msg
page =
    let
        para : E.Element msg
        para =
            E.paragraph [ E.centerX ] [ E.text "You can change your password, email etc here." ]
    in
    F.e E.column [ E.centerX, E.centerY, E.width S.s384 ] [ para ]
