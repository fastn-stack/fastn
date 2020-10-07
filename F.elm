module F exposing (..)

import Element as E


type alias Element msg =
     Extra msg -> E.Element msg


type alias Extra msg =
    { extra : List (E.Attribute msg)
    , device : E.Device
    }


e0 : E.Device -> Extra msg
e0 device =
    { extra = [], device = device }


e :
    (List (E.Attribute msg)
     -> List (E.Element msg)
     -> E.Element msg
    )
    -> List (E.Attribute msg)
    -> List (E.Element msg)
    -> Element msg
e ctr attrs children extra =
    ctr (attrs ++ extra.extra) children


e1 :
    (List (E.Attribute msg)
     -> a
     -> E.Element msg
    )
    -> List (E.Attribute msg)
    -> a
    -> Element msg
e1 ctr attrs child extra =
    ctr (attrs ++ extra.extra) child


type alias StringField msg =
    { value : String, message : String -> msg, error : E.Element msg }


type alias FocusableStringField msg =
    { value : String, message : String -> msg, error : E.Element msg, focus : Bool -> msg }
