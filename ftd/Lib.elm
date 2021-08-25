module Lib exposing (..)

import Element as E
import Element.Background as Bg
import Element.Border as EB
import Element.Events as EE
import Element.Font as EF
import Element.Region as ER
import F
import Realm.Utils as RU
import Style as ST
import System as S


linkButton : String -> String -> F.Element msg
linkButton text url =
    F.e1 E.link [ E.pointer ] { url = url, label = RU.text [] text }


msgButton : String -> msg -> F.Element msg
msgButton t msg =
    F.e1 E.el [ E.pointer, EE.onClick msg ] (E.text t)


error : String -> F.Element msg
error t =
    F.e1 E.el [ E.paddingXY 16 0, EF.color S.red4, EF.size 14 ] (E.text t)


submit : String -> msg -> F.Element msg
submit label msg =
    F.e1 RU.button
        [ EB.width 1
        , EB.color ST.success
        , E.paddingXY 20 7
        , Bg.color ST.success
        , EF.color ST.primary
        , E.mouseOver [ Bg.color ST.success, EF.color ST.primary ]
        , EB.rounded 2
        , E.focused []
        ]
        { onPress = Just msg, label = ST.p_14 [ RU.maxContent ] <| E.text label }


submitWithError : String -> F.Element msg
submitWithError label =
    F.e1 RU.button
        [ EB.width 1
        , E.paddingXY 20 7
        , EB.color ST.disabled
        , EF.color ST.primary
        , Bg.color ST.disabled
        , EB.rounded 2
        , E.focused []
        ]
        { onPress = Nothing, label = ST.p_14 [ RU.maxContent ] <| E.text label }


pageTitle : String -> F.Element msg
pageTitle t =
    F.e1 RU.text [ EF.size 32, EF.bold, ER.heading 1 ] t
