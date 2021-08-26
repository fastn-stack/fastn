# FTD Components

## ftd/text

This component is to be used to create new text. It accepts
the following arguments:

text[string]: Text to be displayed.

size[int=12]: Size in pixels.






-- var intro:
fn: ftd.text

[founder|Amit [surname|Upadhyay]] is the CEO of [Fifthtry]. [founder|Deepak
[surname|Angrula]].


--- surname:
underline: true

--- founder:
color: red
background: yellow

--- Fifthtry:
link: https://www.fifthtry.com
