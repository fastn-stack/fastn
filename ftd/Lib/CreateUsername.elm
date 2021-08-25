module Lib.CreateUsername exposing (..)

import Element as E
import Element.Background as Bg
import Element.Border as EB
import Element.Events as EE
import Element.Font as EF
import Element.Input as EI
import F
import Realm.Utils as RU
import System as S


page : F.FocusableStringField msg -> msg -> E.Element msg -> E.Element msg -> F.Element msg
page u submit sub usernameAvailable =
    let
        heading : E.Element msg
        heading =
            E.el
                [ EB.color S.gray4
                , E.width E.fill
                , E.paddingXY 18 12
                , Bg.color S.green2
                , EF.color S.green6
                ]
            <|
                E.text "Create Username"

        usernameLabel : EI.Label msg
        usernameLabel =
            EI.labelAbove [ E.paddingXY 4 0 ] (E.text "Username")

        usernamePlaceholder : EI.Placeholder msg
        usernamePlaceholder =
            EI.placeholder [] (E.text "username")

        username : E.Element msg
        username =
            E.el [ E.paddingXY 14 0, E.width E.fill, RU.onEnter submit ] <|
                EI.username
                    [ E.width E.fill
                    , EI.focusedOnLoad
                    , EE.onFocus (u.focus True)
                    , EE.onLoseFocus (u.focus False)
                    ]
                    { onChange = u.message
                    , text = u.value
                    , placeholder = Just usernamePlaceholder
                    , label = usernameLabel
                    }
    in
    F.e E.column
        [ E.centerX
        , E.centerY
        , E.width S.s384
        , E.spacing 12
        , Bg.color S.gray7
        , EB.color S.green2
        , EB.width S.border4
        , EB.rounded S.borderRadius4
        ]
        [ heading
        , username
        , u.error
        , usernameAvailable
        , sub
        ]
