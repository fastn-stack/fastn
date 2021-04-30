import Element as E
import Element.Background as Bg
import Element.Border as EB
import Element.Font as EF
import Element.Input as EI
import F
import Html as H


anonPage : msg -> element -> F.Element msg
anonPage sign child =
	F.e E.row [E.width E.fill, E.height E.fill] [leftBorder, main]


anonPage : F.Element msg
anonPage =
	F.e E.column [] [leftBorder, main]


