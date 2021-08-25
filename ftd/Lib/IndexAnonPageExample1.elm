module Lib.IndexAnonPageExample1 exposing (..)

import Browser as B
import Element as E
import Lib.Index


main : Program () () ()
main =
    let
        child : E.Element ()
        child =
            E.el [] (E.text "hello")
    in
    B.application
        { init = \_ _ _ -> ( (), Cmd.none )
        , view = \_ -> { title = "", body = [ E.layout [] (Lib.Index.anonPage () child []) ] }
        , update = \_ _ -> ( (), Cmd.none )
        , subscriptions = always Sub.none
        , onUrlRequest = always ()
        , onUrlChange = always ()
        }
