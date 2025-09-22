/**
 * https://github.com/PrismJS/prism/releases/tag/v1.29.0 - a syntax highlighting library
 * Copyright (c) 2012 Lea Verou (MIT Licensed)
 * https://github.com/PrismJS/prism
 * https://github.com/PrismJS/prism/tree/11c54624ee4f0e36ec3607c16d74969c8264a79d
 */
// Content taken from https://raw.githubusercontent.com/PrismJS/prism/11c54624ee4f0e36ec3607c16d74969c8264a79d/components/prism-diff.min.js
!function(e){e.languages.diff={coord:[/^(?:\*{3}|-{3}|\+{3}).*$/m,/^@@.*@@$/m,/^\d.*$/m]};var n={"deleted-sign":"-","deleted-arrow":"<","inserted-sign":"+","inserted-arrow":">",unchanged:" ",diff:"!"};Object.keys(n).forEach((function(a){var i=n[a],r=[];/^\w+$/.test(a)||r.push(/\w+/.exec(a)[0]),"diff"===a&&r.push("bold"),e.languages.diff[a]={pattern:RegExp("^(?:["+i+"].*(?:\r\n?|\n|(?![\\s\\S])))+","m"),alias:r,inside:{line:{pattern:/(.)(?=[\s\S]).*(?:\r\n?|\n)?/,lookbehind:!0},prefix:{pattern:/[\s\S]/,alias:/\w+/.exec(a)[0]}}}})),Object.defineProperty(e.languages.diff,"PREFIXES",{value:n})}(Prism);