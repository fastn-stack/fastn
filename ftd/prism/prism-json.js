/**
 * https://github.com/PrismJS/prism/releases/tag/v1.29.0 - a syntax highlighting library
 * Copyright (c) 2012 Lea Verou (MIT Licensed)
 * https://github.com/PrismJS/prism
 * https://github.com/PrismJS/prism/commit/e2630d890e9ced30a79cdf9ef272601ceeaedccf
 */
// Content taken from https://raw.githubusercontent.com/PrismJS/prism/master/components/prism-json.min.js
Prism.languages.json={property:{pattern:/(^|[^\\])"(?:\\.|[^\\"\r\n])*"(?=\s*:)/,lookbehind:!0,greedy:!0},string:{pattern:/(^|[^\\])"(?:\\.|[^\\"\r\n])*"(?!\s*:)/,lookbehind:!0,greedy:!0},comment:{pattern:/\/\/.*|\/\*[\s\S]*?(?:\*\/|$)/,greedy:!0},number:/-?\b\d+(?:\.\d+)?(?:e[+-]?\d+)?\b/i,punctuation:/[{}[\],]/,operator:/:/,boolean:/\b(?:false|true)\b/,null:{pattern:/\bnull\b/,alias:"keyword"}},Prism.languages.webmanifest=Prism.languages.json;
