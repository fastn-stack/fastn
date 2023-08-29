// ftd-language.js

Prism.languages.ftd = {
    'comment': [
        {
            'pattern': /\/--[\s]*.+/g,
            'greedy': true,
            'alias': "section-comment",
        },
        {
            "pattern": /[\s]*\/[\w]+(:).*\n/g,
            "greedy": true,
            "alias": "header-comment"
        },
        {
            'pattern': /(;;)[\w\s]*\n/g,
            'greedy': true,
            'alias': "inline-or-line-comment",
        }
    ],
    /*
    -- [section-type] <section-name>: [caption]
    [header-type] <header>: [value]

    [block headers]

    [body] -> string

    [children]

    [-- end: <section-name>]
    */
    'string': {
        'pattern': /^[ \t\n]*--\s+(.*)(\n(?![ \n\t]*--).*)*/g,
        'inside': {
            // section-identifier
            'comment': /^[ \t\n]*--\s+/g,
            // [section type] <section name>:
            'punctuation': {
                'pattern': /^(.*):/g,
                'inside': {
                    "comment": /:/g,
                    'tag': /^\b(component|record|end|or-type)\b/g,
                    "function": /^\s*\S+/g,
                }
            },
            // header name
            'regex': {
                'pattern': /\b(?!--\s+)(.*?)(?=:)/g,
            },
            // header value
            'deliminator': /^[ \n\t]+(.*)(\n)/g,
        },
    },
};

