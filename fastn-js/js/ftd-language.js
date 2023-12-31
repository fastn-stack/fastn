/* ftd-language.js */

Prism.languages.ftd = {
    comment: [
        {
            pattern: /\/--\s*((?!--)[\S\s])*/g,
            greedy: true,
            alias: "section-comment",
        },
        {
            pattern: /[\s]*\/[\w]+(:).*\n/g,
            greedy: true,
            alias: "header-comment",
        },
        {
            pattern: /(;;).*\n/g,
            greedy: true,
            alias: "inline-or-line-comment",
        },
    ],
    /*
    -- [section-type] <section-name>: [caption]
    [header-type] <header>: [value]

    [block headers]

    [body] -> string

    [children]

    [-- end: <section-name>]
    */
    string: {
        pattern: /^[ \t\n]*--\s+(.*)(\n(?![ \n\t]*--).*)*/g,
        inside: {
            /* section-identifier */
            "section-identifier": /([ \t\n])*--\s+/g,
            /* [section type] <section name>: */
            punctuation: {
                pattern: /^(.*):/g,
                inside: {
                    "semi-colon": /:/g,
                    keyword: /^(component|record|end|or-type)/g,
                    "value-type": /^(integer|boolean|decimal|string)/g,
                    "kernel-type": /\s*ftd[\S]+/g,
                    "type-modifier": {
                        pattern: /(\s)+list(?=\s)/g,
                        lookbehind: true,
                    },
                    "section-name": {
                        pattern: /(\s)*.+/g,
                        lookbehind: true,
                    },
                },
            },
            /* section caption */
            "section-caption": /^.+(?=\n)*/g,
            /* header name: header value */
            regex: {
                pattern: /(?!--\s*).*[:]\s*(.*)(\n)*/g,
                inside: {
                    /* if condition on component */
                    "header-condition": /\s*if\s*:(.)+/g,
                    /* header event */
                    event: /\s*\$on(.)+\$(?=:)/g,
                    /* header processor */
                    processor: /\s*\$[^:]+\$(?=:)/g,
                    /* header name => [header-type] <name> [header-condition] */
                    regex: {
                        pattern: /[^:]+(?=:)/g,
                        inside: {
                            /* [header-condition]  */
                            "header-condition": /if\s*{.+}/g,
                            /* [header-type] <name> */
                            tag: {
                                pattern: /(.)+(?=if)?/g,
                                inside: {
                                    "kernel-type": /^\s*ftd[\S]+/g,
                                    "header-type":
                                        /^(record|caption|body|caption or body|body or caption|integer|boolean|decimal|string)/g,
                                    "type-modifier": {
                                        pattern: /(\s)+list(?=\s)/g,
                                        lookbehind: true,
                                    },
                                    "header-name": {
                                        pattern: /(\s)*(.)+/g,
                                        lookbehind: true,
                                    },
                                },
                            },
                        },
                    },
                    /* semicolon */
                    "semi-colon": /:/g,
                    /* header value (if any) */
                    "header-value": {
                        pattern: /(\s)*(.+)/g,
                        lookbehind: true,
                    },
                },
            },
        },
    },
};
