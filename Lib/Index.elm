module Lib.Index exposing (..)

import Common.Content as Content
import Element as E
import Element.Background as Bg
import Element.Border as EB
import Element.Events as EE
import Element.Font as EF
import Element.Input as EI
import F
import Icons
import Lib
import Realm.Utils as RU exposing (edges)
import Routes
import Style as ST
import System as S


type alias Testimony =
    { text : RU.Rendered, name : String, designation : String, id : Int }


testimonialData : List Testimony
testimonialData =
    [ { text = RU.Rendered """AmitU has really struggled with the problem
            the is tackling at fifthtry while working with acko, he wrote a 3000
            page documentation, acko.pdf. He is a fanatic of writing things down,
            and this tool he has created will mean he doesn’t have to shout at everyone :-).

            <strong>Fifthtry will def help acko and every tech startup I have worked with. </strong>
            Especially now that everyone is working remote, <strong>good documentation
            is secret sauce for companies who will make remote work vs who will suffer.</strong>"""
      , name = "Deepak Angrula"
      , designation = "Ex SVP, Acko Insurance"
      , id = 0
      }
    , { text = RU.Rendered """Amitu is an acute technologist who understands a problem from all
            perspectives, business and product and only settles with a solution (again from a business
            and architecture perspective) that would serve the long term instead of quick hacks that solve
            one symptom after another"""
      , name = "Kushal Kothari"
      , designation = "Udaan"
      , id = 1
      }
    , { text = RU.Rendered """This is a very helpful tool,  most of the time we face the issue of
            documentation not keeping up with feature changes. With this tool development and
            documentation will be in sync."""
      , name = "Manish Agrawal"
      , designation = "Founder, CEO Techment Technology"
      , id = 2
      }
    , { text = RU.Rendered """Our latest enterprise SaaS integeration offering has been getting significant
            traction in the last 8-10 months - bringing its own set of (good to have :) challenges.


            I’ve been exploring multiple options to ensure that everyone invovled -
            from dev to support - is on the same page about what’s in-front of the customer all the time.
            Turned out this is easier said than done, but based on our initial investigations
            <strong>fifthtry looks quite promising to avoid these cross-team confusions.</strong>"""
      , name = "Kumar G Varun"
      , designation = "Oracle, Ex-Splunk"
      , id = 3
      }
    , { text = RU.Rendered """What an amazing product. It solves almost al the key documentation
            problems engineering teams face today and in such a delight easy way!"""
      , name = "Devendra Rane"
      , designation = "SVP, Paytm"
      , id = 4
      }
    , { text = RU.Rendered """Engineering managers leaving without documenting everything is biggest risk to companies,
            and fifthtry can solve this problem for good!"""
      , name = "Kiran Patil"
      , designation = "Kiran Patil, Founder &  CEO Growisto"
      , id = 5
      }
    ]


maxWidth : E.Device -> E.Attribute msg
maxWidth d =
    case d.class of
        E.Phone ->
            E.width E.fill

        E.Tablet ->
            E.width E.fill

        _ ->
            E.width (E.maximum 1280 E.fill)


minHeight : E.Device -> E.Attribute msg
minHeight d =
    case d.class of
        E.Phone ->
            RU.style "min-height" "auto"

        E.Tablet ->
            RU.style "min-height" "auto"

        _ ->
            RU.style "min-height" "100vh"


blockPadding : E.Device -> Int -> Int -> E.Attribute msg
blockPadding d t b =
    case d.class of
        E.Phone ->
            E.paddingEach { left = 15, right = 15, top = t, bottom = b }

        _ ->
            E.paddingEach { left = 20, right = 20, top = t, bottom = b }


title : E.Device -> List (E.Attribute msg) -> String -> E.Element msg
title d att s =
    let
        el =
            E.text s
    in
    case d.class of
        E.Phone ->
            ST.h4 ([ E.width E.fill, EF.color ST.primary, EF.extraBold, RU.style "line-height" "40px" ] ++ att) <|
                el

        E.Tablet ->
            ST.h4 ([ E.width (E.px 670), EF.color ST.primary, EF.extraBold, RU.style "line-height" "40px" ] ++ att) <|
                el

        _ ->
            ST.h2 ([ E.width (E.px 670), EF.color ST.primary, EF.extraBold, RU.style "line-height" "55px" ] ++ att) <|
                el


subtitle : E.Device -> List (E.Attribute msg) -> String -> E.Element msg
subtitle d att s =
    case d.class of
        E.Phone ->
            E.paragraph ([ E.width E.fill, EF.size 16, E.paddingEach { edges | bottom = 30 } ] ++ att)
                [ RU.text [ EF.color ST.secondaryLightXX, RU.style "line-height" "22px", EF.extraLight ]
                    s
                ]

        E.Tablet ->
            E.paragraph ([ E.width (E.px 500), EF.size 18, E.paddingEach { edges | bottom = 50 } ] ++ att)
                [ RU.text [ EF.color ST.secondaryLightXX, RU.style "line-height" "26px", EF.extraLight ]
                    s
                ]

        _ ->
            E.paragraph ([ E.width (E.px 500), EF.size 20, E.paddingEach { edges | bottom = 100 } ] ++ att)
                [ RU.text [ EF.color ST.secondaryLightXX, RU.style "line-height" "32px", EF.extraLight ]
                    s
                ]


subtitleMid : E.Device -> List (E.Attribute msg) -> String -> E.Element msg
subtitleMid d att s =
    case d.class of
        E.Phone ->
            E.paragraph ([ E.width E.fill, EF.size 16, EF.color ST.secondary, RU.style "line-height" "22px" ] ++ att)
                [ RU.text []
                    s
                ]

        E.Tablet ->
            E.paragraph ([ E.width E.fill, EF.size 18, EF.color ST.secondary, RU.style "line-height" "26px" ] ++ att)
                [ RU.text []
                    s
                ]

        _ ->
            E.paragraph ([ E.width E.fill, EF.size 20, EF.color ST.secondary, RU.style "line-height" "32px" ] ++ att)
                [ RU.text []
                    s
                ]


testimonialsList : E.Device -> Int -> msg -> msg -> E.Element msg
testimonialsList device i prev next =
    let
        ( lst, p ) =
            case device.class of
                E.Phone ->
                    ( List.filter (\d -> d.id == i) testimonialData, 0 )

                _ ->
                    ( List.filter (\d -> d.id == i || d.id == i + 1) testimonialData, 50 )

        ( buttonsBottom, height ) =
            case device.class of
                E.Phone ->
                    ( RU.style "bottom" "50px", E.minimum 520 E.fill )

                E.Tablet ->
                    ( RU.style "bottom" "50px", E.minimum 620 E.fill )

                _ ->
                    ( RU.style "bottom" "100px", E.fill )
    in
    RU.column [ E.height height ]
        [ RU.row [ E.spacing 60, E.centerX, E.width E.fill, E.paddingXY p 0, RU.style "align-items" "baseline" ] <|
            List.map (testimonyView device) lst
        , RU.row [ E.centerX, E.spacing 60, RU.style "position" "absolute", buttonsBottom ]
            [ E.el [ E.padding 10, Bg.color ST.primary, ST.shadowLight, EB.rounded 60 ] <|
                Icons.arrowGreen [ E.pointer, EE.onClick prev, E.width (E.px 40), RU.style "transform" "rotate(-180deg)" ]
            , E.el [ E.padding 10, Bg.color ST.primary, ST.shadowLight, EB.rounded 60 ] <|
                Icons.arrowGreen [ E.pointer, EE.onClick next, E.width (E.px 40) ]
            ]
        ]


testimonyView : E.Device -> Testimony -> E.Element msg
testimonyView device { text, name, designation } =
    E.column
        [ E.paddingEach { left = 40, top = 20, right = 30, bottom = 20 }
        , EB.rounded 10
        , ST.shadowSpreaded
        , Bg.color ST.success
        , E.width (E.fillPortion 1)
        ]
        [ case device.class of
            E.Phone ->
                ST.p_14 [ E.paddingEach { edges | bottom = 30 }, EF.color ST.primary ] <| RU.html [] text

            _ ->
                ST.p [ E.paddingEach { edges | bottom = 30 }, EF.color ST.primary ] <| RU.html [] text
        , ST.p_14_bold [ EF.color ST.primary ] <| E.text name
        , ST.p_14 [ EF.color ST.secondaryLightXX ] <| E.text designation
        ]


testimonialsText : E.Device -> E.Element msg
testimonialsText e =
    let
        width =
            case e.class of
                E.Phone ->
                    E.width E.fill

                E.Tablet ->
                    E.width E.fill

                _ ->
                    E.width (E.px 700)
    in
    E.row [ width, E.centerX ]
        [ title e [ EF.center, EF.color ST.secondary, E.centerX ] "What people are saying about us"
        ]


page : Content.Colors -> Int -> E.Element msg -> String -> F.Element msg
page colors width switcher userMessage =
    let
        switcher_ : E.Element msg
        switcher_ =
            E.row [ EB.rounded S.borderRadius4, E.width (E.px width) ] [ switcher ]

        userMessage_ : E.Element msg
        userMessage_ =
            E.textColumn
                [ E.width E.fill ]
                [ E.text userMessage ]
    in
    F.e E.textColumn
        [ E.width E.fill, Bg.color colors.background, EB.widthEach { edges | top = 1, bottom = 1 }, EB.color colors.separator ]
        [ switcher_, userMessage_ ]


betaRequestForm : F.StringField msg -> F.StringField msg -> E.Element msg -> F.Element msg
betaRequestForm name email sub =
    let
        nameField : E.Element msg
        nameField =
            EI.text [ E.width (E.px 350), EB.rounded 2, EB.color S.white ]
                { onChange = name.message
                , text = name.value
                , placeholder = Just (EI.placeholder [] (ST.p_14 [ EF.color ST.secondaryLight ] <| E.text "Name"))
                , label = EI.labelHidden "Name"
                }
    in
    F.e E.column
        [ E.spacing 20 ]
        [ nameField
        , name.error
        , EI.text [ E.width (E.px 350), EB.rounded 2, EB.color S.white ]
            { onChange = email.message
            , text = email.value
            , placeholder = Just (EI.placeholder [] (ST.p_14 [ EF.color ST.secondaryLight ] <| E.text "Email"))
            , label = EI.labelHidden "Email"
            }
        , email.error
        , sub
        ]


anonPage : msg -> msg -> E.Element msg -> E.Element msg -> F.Extra msg -> E.Element msg
anonPage signin_ scroll child testimonyChild e =
    let
        heroBg : E.Element msg
        heroBg =
            E.row
                [ E.width E.fill
                , E.height E.fill
                , Bg.gradient { angle = 90, steps = [ S.orangeLow, S.orangeHigh ] }
                ]
                []

        actionButton =
            Lib.msgButton "Sign in" signin_ { device = e.device, extra = [ E.alignRight ] }

        blog : E.Element msg
        blog =
            Lib.linkButton "Blog" "/blog/" { device = e.device, extra = [ E.alignRight ] }

        signin : E.Element msg
        signin =
            Lib.msgButton "Sign in" signin_ { device = e.device, extra = [ E.alignRight, EF.size 16, EF.color ST.primary ] }

        header : E.Element msg
        header =
            E.row [ maxWidth e.device, blockPadding e.device 20 0, E.centerX ]
                [ E.link [ E.alignLeft, E.pointer ]
                    { url = Routes.index
                    , label = ST.h4 [ EF.color ST.primary ] <| E.text "FifthTry"
                    }
                , signin
                ]

        fifth : E.Element msg
        fifth =
            E.row [ EF.size 36, EF.bold, EF.color S.gray2 ]
                [ RU.text [] "Fifth"

                -- , E.html (H.sup [] [ H.text "th" ])
                , RU.text [] " Try"
                ]

        line1 : E.Element msg
        line1 =
            let
                ( l, att ) =
                    case e.device.class of
                        E.Phone ->
                            ( 70, [ EF.center ] )

                        E.Tablet ->
                            ( 70, [] )

                        _ ->
                            ( 100, [] )
            in
            E.column [ maxWidth e.device, blockPadding e.device l 0, E.centerX ]
                [ title e.device att "Tired of documentation not keeping up with constantly evolving code?"
                ]

        line2 : E.Element msg
        line2 =
            let
                att =
                    case e.device.class of
                        E.Phone ->
                            [ EF.center ]

                        E.Tablet ->
                            []

                        _ ->
                            []
            in
            subtitle e.device
                att
                """
                        FifthTry is a documentation tool that integrates with
                        Github Pull Request, to ensure that no code goes live without updated documentation.
                    """

        mainButton : E.Element msg
        mainButton =
            let
                att =
                    case e.device.class of
                        E.Phone ->
                            [ E.paddingXY 15 15
                            , EF.size 14
                            , E.width E.fill
                            , EF.center
                            ]

                        E.Tablet ->
                            [ E.paddingXY 20 15
                            , EF.size 16
                            ]

                        _ ->
                            [ E.paddingXY 30 18
                            , EF.size 18
                            ]
            in
            Lib.msgButton "Say Goodbye To Documentation Rot"
                scroll
                { device = e.device
                , extra =
                    [ Bg.color ST.accent2
                    , EB.rounded 1
                    , EF.color S.white
                    , EF.bold
                    ]
                        ++ att
                }

        -- [ Bg.color S.orange, E.paddingXY 20 15, EB.rounded 5, EF.color S.white, EF.extraBold ]
        heroText : E.Element msg
        heroText =
            case e.device.class of
                E.Phone ->
                    E.column [ E.width E.fill ]
                        [ line2
                        , mainButton
                        ]

                E.Tablet ->
                    E.column [ E.width E.fill ]
                        [ line2
                        , mainButton
                        ]

                _ ->
                    E.column []
                        [ line2
                        , mainButton
                        ]

        screenshot : E.Element msg
        screenshot =
            case e.device.class of
                E.Phone ->
                    E.image [ E.width E.fill, E.centerX, E.paddingEach { edges | top = 30 } ]
                        { src = "/static/home/hero-min.png", description = "Laptop with FifthTry Demo" }

                E.Tablet ->
                    E.image [ E.width (E.px 400), E.alignRight ]
                        { src = "/static/home/hero-min.png", description = "Laptop with FifthTry Demo" }

                _ ->
                    E.image [ E.width (E.px 650), E.alignRight ]
                        { src = "/static/home/hero-min.png", description = "Laptop with FifthTry Demo" }

        heroSection : E.Element msg
        heroSection =
            case e.device.class of
                E.Phone ->
                    E.column [ maxWidth e.device, blockPadding e.device 30 30, E.centerX, RU.style "align-items" "flex-start" ]
                        [ heroText, screenshot ]

                E.Tablet ->
                    E.row [ maxWidth e.device, blockPadding e.device 50 0, E.centerX, RU.style "align-items" "flex-start" ]
                        [ heroText, screenshot ]

                _ ->
                    E.row [ maxWidth e.device, blockPadding e.device 80 0, E.centerX, RU.style "align-items" "flex-start" ]
                        [ heroText, screenshot ]

        changeRequestImage : E.Element msg
        changeRequestImage =
            let
                ( w1, w2 ) =
                    case e.device.class of
                        E.Phone ->
                            ( E.fill, 20 )

                        E.Tablet ->
                            ( E.fill, 40 )

                        _ ->
                            ( E.px 950, 80 )
            in
            E.image [ E.width w1, E.paddingEach { edges | top = w2 }, E.centerX ]
                { src = "/static/home/changeRequest-min.png", description = "FifthTry change requests" }

        changeRequestText : E.Element msg
        changeRequestText =
            let
                ( w1, w2 ) =
                    case e.device.class of
                        E.Phone ->
                            ( E.fill, E.fill )

                        E.Tablet ->
                            ( E.fill, E.maximum 600 E.fill )

                        _ ->
                            ( E.px 750, E.px 700 )
            in
            E.column [ E.centerX ]
                [ title e.device
                    [ EF.color ST.primary, E.paddingEach { edges | bottom = 40 }, E.width w1, EF.center ]
                    "FifthTry lets you raise Change Requests"
                , E.column [ E.width w2, E.centerX ]
                    [ subtitleMid e.device
                        [ EF.color ST.primary, E.centerX ]
                        """
                        FifthTry has a review based workflow for documentation, just like Github Pull Request.
                        Instead of directly editing documents, FifthTry users can create change requests to implement changes to documentation.
                        """
                    ]
                ]

        changeRequestView : E.Element msg
        changeRequestView =
            E.column [ E.width E.fill, minHeight e.device, blockPadding e.device 40 0, E.centerX, Bg.color ST.informational ]
                [ changeRequestText, changeRequestImage ]

        documentationSection : E.Element msg
        documentationSection =
            let
                ( w1, w2 ) =
                    case e.device.class of
                        E.Phone ->
                            ( E.fill, E.fill )

                        E.Tablet ->
                            ( E.maximum 600 E.fill, E.fill )

                        _ ->
                            ( E.px 700, E.px 900 )
            in
            E.column [ maxWidth e.device, minHeight e.device, blockPadding e.device 40 0, E.centerX ]
                [ E.column [ E.centerX, E.paddingEach { edges | bottom = 30 } ]
                    [ title e.device [ EF.color ST.secondary, EF.center ] "What kind of documentation?"
                    ]
                , E.column [ E.width w1, E.centerX ]
                    [ subtitleMid e.device
                        []
                        """
                        UI screens, behaviour and journeys, database tables, APIs, how to
                        contribute to, deploy, or monitor microservice, everything that makes
                        web or mobile product teams productive.
                        """
                    ]
                , E.image [ E.width w2, E.paddingEach { edges | top = 50 }, E.centerX ]
                    { src = "/static/home/documentype-min.png", description = "FifthTry documentation" }
                ]

        githubChecksImage : E.Element msg
        githubChecksImage =
            let
                ( w1, w2 ) =
                    case e.device.class of
                        E.Phone ->
                            ( E.fill, 20 )

                        E.Tablet ->
                            ( E.fill, 40 )

                        _ ->
                            ( E.px 1000, 80 )
            in
            E.image [ E.width w1, E.paddingEach { edges | top = w2 }, E.centerX ]
                { src = "/static/home/checks-min.png", description = "FifthTry change requests" }

        githubChecksText : E.Element msg
        githubChecksText =
            let
                ( w1, w2 ) =
                    case e.device.class of
                        E.Phone ->
                            ( E.fill, E.fill )

                        E.Tablet ->
                            ( E.maximum 600 E.fill, E.fill )

                        _ ->
                            ( E.px 750, E.px 800 )
            in
            E.column [ E.width E.fill, E.centerX ]
                [ title e.device
                    [ E.centerX, EF.center, EF.color ST.secondary, E.paddingEach { edges | bottom = 40 }, E.width w2 ]
                    "Every time code changes documentation must change"
                , E.column [ E.width w1, E.centerX ]
                    [ subtitleMid e.device
                        []
                        """
                        Good habits often need a solid foundation. To ensure documents stay updated,
                        you can block Github Pull Requests till document changes get approved on FifthTry.
                        """
                    ]
                ]

        githubChecks : E.Element msg
        githubChecks =
            E.column [ maxWidth e.device, E.width E.fill, minHeight e.device, blockPadding e.device 40 0, E.centerX ] [ githubChecksText, githubChecksImage ]

        section3Image : E.Element msg
        section3Image =
            E.image [ E.height (E.px 700), E.paddingEach { edges | left = 100 } ]
                { src = "/static/home/section3.png", description = "FifthTry change requests" }

        section3Text : E.Element msg
        section3Text =
            E.column [ E.width (E.px 700), E.spacing 20, E.centerX ]
                [ E.paragraph [ E.width (E.px 700), EF.size 40, EF.bold ]
                    [ RU.text [] "Diagram source should be part of your documentation" ]
                , E.paragraph [ E.width (E.px 700) ]
                    [ RU.text []
                        """
                        FifthTry supports graphviz, tikz, ditaa, plantuml etc so
                        the source code for your diagram lives with the
                        documentation and can be edited, versioned,
                        approved as part of a single interface.
                        """
                    ]
                ]

        section3 : E.Element msg
        section3 =
            E.row [ E.width E.fill, E.paddingXY 0 100 ] [ section3Image, section3Text ]

        signUpText : E.Element msg
        signUpText =
            let
                w1 =
                    case e.device.class of
                        E.Phone ->
                            E.fill

                        E.Tablet ->
                            E.fill

                        _ ->
                            E.px 600
            in
            E.column [ E.width w1, E.spacing 40, E.paddingEach { edges | bottom = 40, top = 40 } ]
                [ title e.device [ E.width w1 ] "Sign up for early beta access and get lifetime discounted pricing!"
                , E.column [ E.width E.fill, E.centerX ]
                    [ subtitleMid e.device
                        [ EF.color ST.primary ]
                        """
                            FifthTry is in closed beta right now. Sign up now to
                            to get lifetime 50% discounted on pricing.
                        """
                    ]
                ]

        signUpForm : E.Element msg
        signUpForm =
            let
                ( w1, w2 ) =
                    case e.device.class of
                        E.Phone ->
                            ( E.fill, 0 )

                        E.Tablet ->
                            ( E.fill, 0 )

                        _ ->
                            ( E.px 500, 100 )
            in
            E.column [ RU.id "signup-form", E.width w1, E.spacing 20, E.centerX, E.paddingEach { edges | left = w2 } ]
                [ E.column [ E.width E.fill ] [ child ] ]

        signUpSection : E.Element msg
        signUpSection =
            case e.device.class of
                E.Phone ->
                    E.column [ E.width E.fill, E.paddingXY 0 20, Bg.color ST.brand, RU.id "signup" ]
                        [ E.column [ E.centerX, maxWidth e.device, blockPadding e.device 10 10 ] [ signUpText, signUpForm ] ]

                E.Tablet ->
                    E.column [ E.width E.fill, E.paddingXY 0 40, Bg.color ST.brand, RU.id "signup" ]
                        [ E.row [ E.centerX, maxWidth e.device, blockPadding e.device 20 20, E.spacing 40 ] [ signUpText, signUpForm ] ]

                _ ->
                    E.column [ E.width E.fill, E.paddingXY 0 80, Bg.color ST.brand, RU.id "signup" ]
                        [ E.row [ E.centerX, maxWidth e.device ] [ signUpText, signUpForm ] ]

        footerText : E.Element msg
        footerText =
            E.column [ maxWidth e.device, E.centerX ]
                [ ST.p [ EF.color ST.secondary ] <| E.text "© 2021 FifthTry. All rights reserved"
                ]

        footer : E.Element msg
        footer =
            E.row [ E.width E.fill, blockPadding e.device 40 50, Bg.color ST.primary ] [ footerText ]

        top : E.Element msg
        top =
            E.column
                [ E.width E.fill
                , Bg.color ST.brand
                , minHeight e.device
                ]
                [ header, line1, heroSection ]

        testimonials : E.Element msg
        testimonials =
            let
                w1 =
                    case e.device.class of
                        E.Phone ->
                            20

                        E.Tablet ->
                            40

                        _ ->
                            110
            in
            E.column [ E.width E.fill, minHeight e.device, blockPadding e.device 40 0, E.centerX, Bg.color ST.secondaryLightXX ]
                [ RU.column [ E.centerX, maxWidth e.device, E.spacing w1 ]
                    [ testimonialsText e.device, testimonyChild ]
                ]

        main_ : E.Element msg
        main_ =
            E.column
                [ E.width E.fill, E.height E.fill, EF.color S.gray0 ]
                [ top
                , documentationSection
                , changeRequestView
                , githubChecks
                , testimonials
                , signUpSection
                , footer
                ]
    in
    F.e E.row [ E.width E.fill, E.height E.fill ] [ main_ ] e
