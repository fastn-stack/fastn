# ftd


## Basic

```
-- amitu/table: Some table
columns: l | c | r 

some body
```

```json
[{
    "section": "amitu/table",
    "caption": "Some table",
    "columns": "l | c | r",
    "body": "some body"
}]
```

- caption is optional, if not passed it should be set to empty string.
- keys can contain alphanumeric, `-` and `_`.


## Body starts after first empty line 

If no key value is provided, there must be one empty line before `--` line and body.

```
-- amitu/table: Some table
key: value

columns: l | c | r 

some body
```

```json
[{
    "section": "amitu/table",
    "caption": "Some table",
    "key": "value",
    "body": "columns: l | c | r\n\nsome body"
}]
```



## With Nested Object

```
-- amitu/table: Some table
columns: l | c | r 

some body

--- something: something caption
s_key: yo

something body
```

```json
[{
    "section": "amitu/table",
    "caption": "Some table",
    "columns": "l | c | r",
    "body": "some body",
    "something": {
        "caption": "something caption",
        "s_key": "yo",
        "body": "something body"
    }
}]
```

## With Arrays:

```
-- amitu/table: Some table
columns: l | c | r 

some body

--- something: something caption
s_key: yo

something body

--- row[]: row 1
r: row1's r

row 1's body

-- row[]: row 2
r: row1's r

row 2's body
```

```json
[{
    "section": "amitu/table",
    "caption": "Some table",
    "columns": "l | c | r",
    "body": "some body",
    "something": {
        "caption": "something caption",
        "s_key": "yo",
        "body": "something body"
    },
    "rows[]": [
        { 
            "caption": "row 1",
            "r": "row1's r",
            "body": "row 1's body"
        },
        { 
            "caption": "row 2",
            "r": "row2's r",
            "body": "row 2's body"
        }
    ]
}]
```

## On Body

- Body should have at least single empty line before and after it.
- Empty lines before and after the body would be removed.

```
-- amitu/table: Some table

some body



```

```json
[{
    "section": "amitu/table",
    "caption": "Some table",
    "body": "some body"
}]
```

## Escaping in Body

If any line body is supposed to start with either `-- ` or `--- `, they would be escaped with "\": `\-- ` and `\--- ` respectively:

```
-- amitu/table: Some table

\-- yo

this is cool

\--- something

yo yo yo


```

```json
[{
    "section": "amitu/table",
    "caption": "Some table",
    "body": "-- yo\n\nthis is cool\n\n--- something\n\nyo yo yo"
}]
```

## FTD Is An Array Of Sections

```
-- section1:
-- section2:

some body
```

```json
[
    { 
        "section": "section1",
        "caption": "",
        "body": ""
    },
    { 
        "section": "section2",
        "caption": "",
        "body": "some1 body"
    }
]
```

## Comments

```
; this is a comment
-- some/section:

; this is our body
the body
\; comments can be escaped with backslash
```

```json
[{
    "section": "some/section",
    "caption": "",
    "body": "the body\n; comments can be escaped with backslash"
}]
```

- Comments can be inserted between key value pairs in header.
