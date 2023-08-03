// ftd-language.js

Prism.languages.ftd = {
    'keyword': /\b(component|record)\b/g,
    'comment': {
        pattern: /((;;).*)|(^[ \t\n]*\/--\s+(.*))/g,
        greedy: true
    },
    'string': {
        pattern: /^[ \t\n]*--\s+(.*)(\n(?![ \n\t]*--).*)*/g,
        inside: {
            'comment': /^[ \t\n]*--\s+/g,
            'punctuation': {
                pattern: /^(.*):/g,
                inside: {
                    "comment": /:/g,
                    'tag': /\b(component|record|end|or-type)\b/g,
                    "function": /^\S+/g
                }
            },
            'regex': /\b(?!--\s+)(.*?)(?=:)/g,
            'deliminator': /^[ \n\t]+((.*)(\n)*)*/g,
        }
    },
};
