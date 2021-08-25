module Lib.IndexBasic exposing (..)

import Browser as B
import Element as E
import Element.Background as Bg
import Lib.Index


main : Program () () ()
main =
    B.application
        { init = \_ _ _ -> ( (), Cmd.none )
        , view = \_ -> { title = "", body = [ E.layout [] (Lib.Index.anonPage () (E.el [ Bg.color yello ] (E.text "child")) []) ] }
        , update = \_ _ -> ( (), Cmd.none )
        , subscriptions = always Sub.none
        , onUrlRequest = always ()
        , onUrlChange = always ()
        }
