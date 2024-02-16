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
/**
 * marked v9.1.4 - a markdown parser
 * Copyright (c) 2011-2023, Christopher Jeffrey. (MIT Licensed)
 * https://github.com/markedjs/marked
 */
// Content taken from https://cdn.jsdelivr.net/npm/marked/marked.min.js
!function(e,t){"object"==typeof exports&&"undefined"!=typeof module?t(exports):"function"==typeof define&&define.amd?define(["exports"],t):t((e="undefined"!=typeof globalThis?globalThis:e||self).marked={})}(this,(function(e){"use strict";function t(){return{async:!1,breaks:!1,extensions:null,gfm:!0,hooks:null,pedantic:!1,renderer:null,silent:!1,tokenizer:null,walkTokens:null}}function n(t){e.defaults=t}e.defaults={async:!1,breaks:!1,extensions:null,gfm:!0,hooks:null,pedantic:!1,renderer:null,silent:!1,tokenizer:null,walkTokens:null};const s=/[&<>"']/,r=new RegExp(s.source,"g"),i=/[<>"']|&(?!(#\d{1,7}|#[Xx][a-fA-F0-9]{1,6}|\w+);)/,l=new RegExp(i.source,"g"),o={"&":"&amp;","<":"&lt;",">":"&gt;",'"':"&quot;","'":"&#39;"},a=e=>o[e];function c(e,t){if(t){if(s.test(e))return e.replace(r,a)}else if(i.test(e))return e.replace(l,a);return e}const h=/&(#(?:\d+)|(?:#x[0-9A-Fa-f]+)|(?:\w+));?/gi;const p=/(^|[^\[])\^/g;function u(e,t){e="string"==typeof e?e:e.source,t=t||"";const n={replace:(t,s)=>(s=(s="object"==typeof s&&"source"in s?s.source:s).replace(p,"$1"),e=e.replace(t,s),n),getRegex:()=>new RegExp(e,t)};return n}function g(e){try{e=encodeURI(e).replace(/%25/g,"%")}catch(e){return null}return e}const k={exec:()=>null};function f(e,t){const n=e.replace(/\|/g,((e,t,n)=>{let s=!1,r=t;for(;--r>=0&&"\\"===n[r];)s=!s;return s?"|":" |"})).split(/ \|/);let s=0;if(n[0].trim()||n.shift(),n.length>0&&!n[n.length-1].trim()&&n.pop(),t)if(n.length>t)n.splice(t);else for(;n.length<t;)n.push("");for(;s<n.length;s++)n[s]=n[s].trim().replace(/\\\|/g,"|");return n}function d(e,t,n){const s=e.length;if(0===s)return"";let r=0;for(;r<s;){const i=e.charAt(s-r-1);if(i!==t||n){if(i===t||!n)break;r++}else r++}return e.slice(0,s-r)}function x(e,t,n,s){const r=t.href,i=t.title?c(t.title):null,l=e[1].replace(/\\([\[\]])/g,"$1");if("!"!==e[0].charAt(0)){s.state.inLink=!0;const e={type:"link",raw:n,href:r,title:i,text:l,tokens:s.inlineTokens(l)};return s.state.inLink=!1,e}return{type:"image",raw:n,href:r,title:i,text:c(l)}}class b{options;rules;lexer;constructor(t){this.options=t||e.defaults}space(e){const t=this.rules.block.newline.exec(e);if(t&&t[0].length>0)return{type:"space",raw:t[0]}}code(e){const t=this.rules.block.code.exec(e);if(t){const e=t[0].replace(/^ {1,4}/gm,"");return{type:"code",raw:t[0],codeBlockStyle:"indented",text:this.options.pedantic?e:d(e,"\n")}}}fences(e){const t=this.rules.block.fences.exec(e);if(t){const e=t[0],n=function(e,t){const n=e.match(/^(\s+)(?:```)/);if(null===n)return t;const s=n[1];return t.split("\n").map((e=>{const t=e.match(/^\s+/);if(null===t)return e;const[n]=t;return n.length>=s.length?e.slice(s.length):e})).join("\n")}(e,t[3]||"");return{type:"code",raw:e,lang:t[2]?t[2].trim().replace(this.rules.inline._escapes,"$1"):t[2],text:n}}}heading(e){const t=this.rules.block.heading.exec(e);if(t){let e=t[2].trim();if(/#$/.test(e)){const t=d(e,"#");this.options.pedantic?e=t.trim():t&&!/ $/.test(t)||(e=t.trim())}return{type:"heading",raw:t[0],depth:t[1].length,text:e,tokens:this.lexer.inline(e)}}}hr(e){const t=this.rules.block.hr.exec(e);if(t)return{type:"hr",raw:t[0]}}blockquote(e){const t=this.rules.block.blockquote.exec(e);if(t){const e=d(t[0].replace(/^ *>[ \t]?/gm,""),"\n"),n=this.lexer.state.top;this.lexer.state.top=!0;const s=this.lexer.blockTokens(e);return this.lexer.state.top=n,{type:"blockquote",raw:t[0],tokens:s,text:e}}}list(e){let t=this.rules.block.list.exec(e);if(t){let n=t[1].trim();const s=n.length>1,r={type:"list",raw:"",ordered:s,start:s?+n.slice(0,-1):"",loose:!1,items:[]};n=s?`\\d{1,9}\\${n.slice(-1)}`:`\\${n}`,this.options.pedantic&&(n=s?n:"[*+-]");const i=new RegExp(`^( {0,3}${n})((?:[\t ][^\\n]*)?(?:\\n|$))`);let l="",o="",a=!1;for(;e;){let n=!1;if(!(t=i.exec(e)))break;if(this.rules.block.hr.test(e))break;l=t[0],e=e.substring(l.length);let s=t[2].split("\n",1)[0].replace(/^\t+/,(e=>" ".repeat(3*e.length))),c=e.split("\n",1)[0],h=0;this.options.pedantic?(h=2,o=s.trimStart()):(h=t[2].search(/[^ ]/),h=h>4?1:h,o=s.slice(h),h+=t[1].length);let p=!1;if(!s&&/^ *$/.test(c)&&(l+=c+"\n",e=e.substring(c.length+1),n=!0),!n){const t=new RegExp(`^ {0,${Math.min(3,h-1)}}(?:[*+-]|\\d{1,9}[.)])((?:[ \t][^\\n]*)?(?:\\n|$))`),n=new RegExp(`^ {0,${Math.min(3,h-1)}}((?:- *){3,}|(?:_ *){3,}|(?:\\* *){3,})(?:\\n+|$)`),r=new RegExp(`^ {0,${Math.min(3,h-1)}}(?:\`\`\`|~~~)`),i=new RegExp(`^ {0,${Math.min(3,h-1)}}#`);for(;e;){const a=e.split("\n",1)[0];if(c=a,this.options.pedantic&&(c=c.replace(/^ {1,4}(?=( {4})*[^ ])/g,"  ")),r.test(c))break;if(i.test(c))break;if(t.test(c))break;if(n.test(e))break;if(c.search(/[^ ]/)>=h||!c.trim())o+="\n"+c.slice(h);else{if(p)break;if(s.search(/[^ ]/)>=4)break;if(r.test(s))break;if(i.test(s))break;if(n.test(s))break;o+="\n"+c}p||c.trim()||(p=!0),l+=a+"\n",e=e.substring(a.length+1),s=c.slice(h)}}r.loose||(a?r.loose=!0:/\n *\n *$/.test(l)&&(a=!0));let u,g=null;this.options.gfm&&(g=/^\[[ xX]\] /.exec(o),g&&(u="[ ] "!==g[0],o=o.replace(/^\[[ xX]\] +/,""))),r.items.push({type:"list_item",raw:l,task:!!g,checked:u,loose:!1,text:o,tokens:[]}),r.raw+=l}r.items[r.items.length-1].raw=l.trimEnd(),r.items[r.items.length-1].text=o.trimEnd(),r.raw=r.raw.trimEnd();for(let e=0;e<r.items.length;e++)if(this.lexer.state.top=!1,r.items[e].tokens=this.lexer.blockTokens(r.items[e].text,[]),!r.loose){const t=r.items[e].tokens.filter((e=>"space"===e.type)),n=t.length>0&&t.some((e=>/\n.*\n/.test(e.raw)));r.loose=n}if(r.loose)for(let e=0;e<r.items.length;e++)r.items[e].loose=!0;return r}}html(e){const t=this.rules.block.html.exec(e);if(t){return{type:"html",block:!0,raw:t[0],pre:"pre"===t[1]||"script"===t[1]||"style"===t[1],text:t[0]}}}def(e){const t=this.rules.block.def.exec(e);if(t){const e=t[1].toLowerCase().replace(/\s+/g," "),n=t[2]?t[2].replace(/^<(.*)>$/,"$1").replace(this.rules.inline._escapes,"$1"):"",s=t[3]?t[3].substring(1,t[3].length-1).replace(this.rules.inline._escapes,"$1"):t[3];return{type:"def",tag:e,raw:t[0],href:n,title:s}}}table(e){const t=this.rules.block.table.exec(e);if(t){if(!/[:|]/.test(t[2]))return;const e={type:"table",raw:t[0],header:f(t[1]).map((e=>({text:e,tokens:[]}))),align:t[2].replace(/^\||\| *$/g,"").split("|"),rows:t[3]&&t[3].trim()?t[3].replace(/\n[ \t]*$/,"").split("\n"):[]};if(e.header.length===e.align.length){let t,n,s,r,i=e.align.length;for(t=0;t<i;t++){const n=e.align[t];n&&(/^ *-+: *$/.test(n)?e.align[t]="right":/^ *:-+: *$/.test(n)?e.align[t]="center":/^ *:-+ *$/.test(n)?e.align[t]="left":e.align[t]=null)}for(i=e.rows.length,t=0;t<i;t++)e.rows[t]=f(e.rows[t],e.header.length).map((e=>({text:e,tokens:[]})));for(i=e.header.length,n=0;n<i;n++)e.header[n].tokens=this.lexer.inline(e.header[n].text);for(i=e.rows.length,n=0;n<i;n++)for(r=e.rows[n],s=0;s<r.length;s++)r[s].tokens=this.lexer.inline(r[s].text);return e}}}lheading(e){const t=this.rules.block.lheading.exec(e);if(t)return{type:"heading",raw:t[0],depth:"="===t[2].charAt(0)?1:2,text:t[1],tokens:this.lexer.inline(t[1])}}paragraph(e){const t=this.rules.block.paragraph.exec(e);if(t){const e="\n"===t[1].charAt(t[1].length-1)?t[1].slice(0,-1):t[1];return{type:"paragraph",raw:t[0],text:e,tokens:this.lexer.inline(e)}}}text(e){const t=this.rules.block.text.exec(e);if(t)return{type:"text",raw:t[0],text:t[0],tokens:this.lexer.inline(t[0])}}escape(e){const t=this.rules.inline.escape.exec(e);if(t)return{type:"escape",raw:t[0],text:c(t[1])}}tag(e){const t=this.rules.inline.tag.exec(e);if(t)return!this.lexer.state.inLink&&/^<a /i.test(t[0])?this.lexer.state.inLink=!0:this.lexer.state.inLink&&/^<\/a>/i.test(t[0])&&(this.lexer.state.inLink=!1),!this.lexer.state.inRawBlock&&/^<(pre|code|kbd|script)(\s|>)/i.test(t[0])?this.lexer.state.inRawBlock=!0:this.lexer.state.inRawBlock&&/^<\/(pre|code|kbd|script)(\s|>)/i.test(t[0])&&(this.lexer.state.inRawBlock=!1),{type:"html",raw:t[0],inLink:this.lexer.state.inLink,inRawBlock:this.lexer.state.inRawBlock,block:!1,text:t[0]}}link(e){const t=this.rules.inline.link.exec(e);if(t){const e=t[2].trim();if(!this.options.pedantic&&/^</.test(e)){if(!/>$/.test(e))return;const t=d(e.slice(0,-1),"\\");if((e.length-t.length)%2==0)return}else{const e=function(e,t){if(-1===e.indexOf(t[1]))return-1;let n=0;for(let s=0;s<e.length;s++)if("\\"===e[s])s++;else if(e[s]===t[0])n++;else if(e[s]===t[1]&&(n--,n<0))return s;return-1}(t[2],"()");if(e>-1){const n=(0===t[0].indexOf("!")?5:4)+t[1].length+e;t[2]=t[2].substring(0,e),t[0]=t[0].substring(0,n).trim(),t[3]=""}}let n=t[2],s="";if(this.options.pedantic){const e=/^([^'"]*[^\s])\s+(['"])(.*)\2/.exec(n);e&&(n=e[1],s=e[3])}else s=t[3]?t[3].slice(1,-1):"";return n=n.trim(),/^</.test(n)&&(n=this.options.pedantic&&!/>$/.test(e)?n.slice(1):n.slice(1,-1)),x(t,{href:n?n.replace(this.rules.inline._escapes,"$1"):n,title:s?s.replace(this.rules.inline._escapes,"$1"):s},t[0],this.lexer)}}reflink(e,t){let n;if((n=this.rules.inline.reflink.exec(e))||(n=this.rules.inline.nolink.exec(e))){let e=(n[2]||n[1]).replace(/\s+/g," ");if(e=t[e.toLowerCase()],!e){const e=n[0].charAt(0);return{type:"text",raw:e,text:e}}return x(n,e,n[0],this.lexer)}}emStrong(e,t,n=""){let s=this.rules.inline.emStrong.lDelim.exec(e);if(!s)return;if(s[3]&&n.match(/[\p{L}\p{N}]/u))return;if(!(s[1]||s[2]||"")||!n||this.rules.inline.punctuation.exec(n)){const n=[...s[0]].length-1;let r,i,l=n,o=0;const a="*"===s[0][0]?this.rules.inline.emStrong.rDelimAst:this.rules.inline.emStrong.rDelimUnd;for(a.lastIndex=0,t=t.slice(-1*e.length+s[0].length-1);null!=(s=a.exec(t));){if(r=s[1]||s[2]||s[3]||s[4]||s[5]||s[6],!r)continue;if(i=[...r].length,s[3]||s[4]){l+=i;continue}if((s[5]||s[6])&&n%3&&!((n+i)%3)){o+=i;continue}if(l-=i,l>0)continue;i=Math.min(i,i+l+o);const t=[...e].slice(0,n+s.index+i+1).join("");if(Math.min(n,i)%2){const e=t.slice(1,-1);return{type:"em",raw:t,text:e,tokens:this.lexer.inlineTokens(e)}}const a=t.slice(2,-2);return{type:"strong",raw:t,text:a,tokens:this.lexer.inlineTokens(a)}}}}codespan(e){const t=this.rules.inline.code.exec(e);if(t){let e=t[2].replace(/\n/g," ");const n=/[^ ]/.test(e),s=/^ /.test(e)&&/ $/.test(e);return n&&s&&(e=e.substring(1,e.length-1)),e=c(e,!0),{type:"codespan",raw:t[0],text:e}}}br(e){const t=this.rules.inline.br.exec(e);if(t)return{type:"br",raw:t[0]}}del(e){const t=this.rules.inline.del.exec(e);if(t)return{type:"del",raw:t[0],text:t[2],tokens:this.lexer.inlineTokens(t[2])}}autolink(e){const t=this.rules.inline.autolink.exec(e);if(t){let e,n;return"@"===t[2]?(e=c(t[1]),n="mailto:"+e):(e=c(t[1]),n=e),{type:"link",raw:t[0],text:e,href:n,tokens:[{type:"text",raw:e,text:e}]}}}url(e){let t;if(t=this.rules.inline.url.exec(e)){let e,n;if("@"===t[2])e=c(t[0]),n="mailto:"+e;else{let s;do{s=t[0],t[0]=this.rules.inline._backpedal.exec(t[0])[0]}while(s!==t[0]);e=c(t[0]),n="www."===t[1]?"http://"+t[0]:t[0]}return{type:"link",raw:t[0],text:e,href:n,tokens:[{type:"text",raw:e,text:e}]}}}inlineText(e){const t=this.rules.inline.text.exec(e);if(t){let e;return e=this.lexer.state.inRawBlock?t[0]:c(t[0]),{type:"text",raw:t[0],text:e}}}}const m={newline:/^(?: *(?:\n|$))+/,code:/^( {4}[^\n]+(?:\n(?: *(?:\n|$))*)?)+/,fences:/^ {0,3}(`{3,}(?=[^`\n]*(?:\n|$))|~{3,})([^\n]*)(?:\n|$)(?:|([\s\S]*?)(?:\n|$))(?: {0,3}\1[~`]* *(?=\n|$)|$)/,hr:/^ {0,3}((?:-[\t ]*){3,}|(?:_[ \t]*){3,}|(?:\*[ \t]*){3,})(?:\n+|$)/,heading:/^ {0,3}(#{1,6})(?=\s|$)(.*)(?:\n+|$)/,blockquote:/^( {0,3}> ?(paragraph|[^\n]*)(?:\n|$))+/,list:/^( {0,3}bull)([ \t][^\n]+?)?(?:\n|$)/,html:"^ {0,3}(?:<(script|pre|style|textarea)[\\s>][\\s\\S]*?(?:</\\1>[^\\n]*\\n+|$)|comment[^\\n]*(\\n+|$)|<\\?[\\s\\S]*?(?:\\?>\\n*|$)|<![A-Z][\\s\\S]*?(?:>\\n*|$)|<!\\[CDATA\\[[\\s\\S]*?(?:\\]\\]>\\n*|$)|</?(tag)(?: +|\\n|/?>)[\\s\\S]*?(?:(?:\\n *)+\\n|$)|<(?!script|pre|style|textarea)([a-z][\\w-]*)(?:attribute)*? */?>(?=[ \\t]*(?:\\n|$))[\\s\\S]*?(?:(?:\\n *)+\\n|$)|</(?!script|pre|style|textarea)[a-z][\\w-]*\\s*>(?=[ \\t]*(?:\\n|$))[\\s\\S]*?(?:(?:\\n *)+\\n|$))",def:/^ {0,3}\[(label)\]: *(?:\n *)?([^<\s][^\s]*|<.*?>)(?:(?: +(?:\n *)?| *\n *)(title))? *(?:\n+|$)/,table:k,lheading:/^(?!bull )((?:.|\n(?!\s*?\n|bull ))+?)\n {0,3}(=+|-+) *(?:\n+|$)/,_paragraph:/^([^\n]+(?:\n(?!hr|heading|lheading|blockquote|fences|list|html|table| +\n)[^\n]+)*)/,text:/^[^\n]+/,_label:/(?!\s*\])(?:\\.|[^\[\]\\])+/,_title:/(?:"(?:\\"?|[^"\\])*"|'[^'\n]*(?:\n[^'\n]+)*\n?'|\([^()]*\))/};m.def=u(m.def).replace("label",m._label).replace("title",m._title).getRegex(),m.bullet=/(?:[*+-]|\d{1,9}[.)])/,m.listItemStart=u(/^( *)(bull) */).replace("bull",m.bullet).getRegex(),m.list=u(m.list).replace(/bull/g,m.bullet).replace("hr","\\n+(?=\\1?(?:(?:- *){3,}|(?:_ *){3,}|(?:\\* *){3,})(?:\\n+|$))").replace("def","\\n+(?="+m.def.source+")").getRegex(),m._tag="address|article|aside|base|basefont|blockquote|body|caption|center|col|colgroup|dd|details|dialog|dir|div|dl|dt|fieldset|figcaption|figure|footer|form|frame|frameset|h[1-6]|head|header|hr|html|iframe|legend|li|link|main|menu|menuitem|meta|nav|noframes|ol|optgroup|option|p|param|section|source|summary|table|tbody|td|tfoot|th|thead|title|tr|track|ul",m._comment=/<!--(?!-?>)[\s\S]*?(?:-->|$)/,m.html=u(m.html,"i").replace("comment",m._comment).replace("tag",m._tag).replace("attribute",/ +[a-zA-Z:_][\w.:-]*(?: *= *"[^"\n]*"| *= *'[^'\n]*'| *= *[^\s"'=<>`]+)?/).getRegex(),m.lheading=u(m.lheading).replace(/bull/g,m.bullet).getRegex(),m.paragraph=u(m._paragraph).replace("hr",m.hr).replace("heading"," {0,3}#{1,6}(?:\\s|$)").replace("|lheading","").replace("|table","").replace("blockquote"," {0,3}>").replace("fences"," {0,3}(?:`{3,}(?=[^`\\n]*\\n)|~{3,})[^\\n]*\\n").replace("list"," {0,3}(?:[*+-]|1[.)]) ").replace("html","</?(?:tag)(?: +|\\n|/?>)|<(?:script|pre|style|textarea|!--)").replace("tag",m._tag).getRegex(),m.blockquote=u(m.blockquote).replace("paragraph",m.paragraph).getRegex(),m.normal={...m},m.gfm={...m.normal,table:"^ *([^\\n ].*)\\n {0,3}((?:\\| *)?:?-+:? *(?:\\| *:?-+:? *)*(?:\\| *)?)(?:\\n((?:(?! *\\n|hr|heading|blockquote|code|fences|list|html).*(?:\\n|$))*)\\n*|$)"},m.gfm.table=u(m.gfm.table).replace("hr",m.hr).replace("heading"," {0,3}#{1,6}(?:\\s|$)").replace("blockquote"," {0,3}>").replace("code"," {4}[^\\n]").replace("fences"," {0,3}(?:`{3,}(?=[^`\\n]*\\n)|~{3,})[^\\n]*\\n").replace("list"," {0,3}(?:[*+-]|1[.)]) ").replace("html","</?(?:tag)(?: +|\\n|/?>)|<(?:script|pre|style|textarea|!--)").replace("tag",m._tag).getRegex(),m.gfm.paragraph=u(m._paragraph).replace("hr",m.hr).replace("heading"," {0,3}#{1,6}(?:\\s|$)").replace("|lheading","").replace("table",m.gfm.table).replace("blockquote"," {0,3}>").replace("fences"," {0,3}(?:`{3,}(?=[^`\\n]*\\n)|~{3,})[^\\n]*\\n").replace("list"," {0,3}(?:[*+-]|1[.)]) ").replace("html","</?(?:tag)(?: +|\\n|/?>)|<(?:script|pre|style|textarea|!--)").replace("tag",m._tag).getRegex(),m.pedantic={...m.normal,html:u("^ *(?:comment *(?:\\n|\\s*$)|<(tag)[\\s\\S]+?</\\1> *(?:\\n{2,}|\\s*$)|<tag(?:\"[^\"]*\"|'[^']*'|\\s[^'\"/>\\s]*)*?/?> *(?:\\n{2,}|\\s*$))").replace("comment",m._comment).replace(/tag/g,"(?!(?:a|em|strong|small|s|cite|q|dfn|abbr|data|time|code|var|samp|kbd|sub|sup|i|b|u|mark|ruby|rt|rp|bdi|bdo|span|br|wbr|ins|del|img)\\b)\\w+(?!:|[^\\w\\s@]*@)\\b").getRegex(),def:/^ *\[([^\]]+)\]: *<?([^\s>]+)>?(?: +(["(][^\n]+[")]))? *(?:\n+|$)/,heading:/^(#{1,6})(.*)(?:\n+|$)/,fences:k,lheading:/^(.+?)\n {0,3}(=+|-+) *(?:\n+|$)/,paragraph:u(m.normal._paragraph).replace("hr",m.hr).replace("heading"," *#{1,6} *[^\n]").replace("lheading",m.lheading).replace("blockquote"," {0,3}>").replace("|fences","").replace("|list","").replace("|html","").getRegex()};const w={escape:/^\\([!"#$%&'()*+,\-./:;<=>?@\[\]\\^_`{|}~])/,autolink:/^<(scheme:[^\s\x00-\x1f<>]*|email)>/,url:k,tag:"^comment|^</[a-zA-Z][\\w:-]*\\s*>|^<[a-zA-Z][\\w-]*(?:attribute)*?\\s*/?>|^<\\?[\\s\\S]*?\\?>|^<![a-zA-Z]+\\s[\\s\\S]*?>|^<!\\[CDATA\\[[\\s\\S]*?\\]\\]>",link:/^!?\[(label)\]\(\s*(href)(?:\s+(title))?\s*\)/,reflink:/^!?\[(label)\]\[(ref)\]/,nolink:/^!?\[(ref)\](?:\[\])?/,reflinkSearch:"reflink|nolink(?!\\()",emStrong:{lDelim:/^(?:\*+(?:((?!\*)[punct])|[^\s*]))|^_+(?:((?!_)[punct])|([^\s_]))/,rDelimAst:/^[^_*]*?__[^_*]*?\*[^_*]*?(?=__)|[^*]+(?=[^*])|(?!\*)[punct](\*+)(?=[\s]|$)|[^punct\s](\*+)(?!\*)(?=[punct\s]|$)|(?!\*)[punct\s](\*+)(?=[^punct\s])|[\s](\*+)(?!\*)(?=[punct])|(?!\*)[punct](\*+)(?!\*)(?=[punct])|[^punct\s](\*+)(?=[^punct\s])/,rDelimUnd:/^[^_*]*?\*\*[^_*]*?_[^_*]*?(?=\*\*)|[^_]+(?=[^_])|(?!_)[punct](_+)(?=[\s]|$)|[^punct\s](_+)(?!_)(?=[punct\s]|$)|(?!_)[punct\s](_+)(?=[^punct\s])|[\s](_+)(?!_)(?=[punct])|(?!_)[punct](_+)(?!_)(?=[punct])/},code:/^(`+)([^`]|[^`][\s\S]*?[^`])\1(?!`)/,br:/^( {2,}|\\)\n(?!\s*$)/,del:k,text:/^(`+|[^`])(?:(?= {2,}\n)|[\s\S]*?(?:(?=[\\<!\[`*_]|\b_|$)|[^ ](?= {2,}\n)))/,punctuation:/^((?![*_])[\spunctuation])/,_punctuation:"\\p{P}$+<=>`^|~"};w.punctuation=u(w.punctuation,"u").replace(/punctuation/g,w._punctuation).getRegex(),w.blockSkip=/\[[^[\]]*?\]\([^\(\)]*?\)|`[^`]*?`|<[^<>]*?>/g,w.anyPunctuation=/\\[punct]/g,w._escapes=/\\([punct])/g,w._comment=u(m._comment).replace("(?:--\x3e|$)","--\x3e").getRegex(),w.emStrong.lDelim=u(w.emStrong.lDelim,"u").replace(/punct/g,w._punctuation).getRegex(),w.emStrong.rDelimAst=u(w.emStrong.rDelimAst,"gu").replace(/punct/g,w._punctuation).getRegex(),w.emStrong.rDelimUnd=u(w.emStrong.rDelimUnd,"gu").replace(/punct/g,w._punctuation).getRegex(),w.anyPunctuation=u(w.anyPunctuation,"gu").replace(/punct/g,w._punctuation).getRegex(),w._escapes=u(w._escapes,"gu").replace(/punct/g,w._punctuation).getRegex(),w._scheme=/[a-zA-Z][a-zA-Z0-9+.-]{1,31}/,w._email=/[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+(@)[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)+(?![-_])/,w.autolink=u(w.autolink).replace("scheme",w._scheme).replace("email",w._email).getRegex(),w._attribute=/\s+[a-zA-Z:_][\w.:-]*(?:\s*=\s*"[^"]*"|\s*=\s*'[^']*'|\s*=\s*[^\s"'=<>`]+)?/,w.tag=u(w.tag).replace("comment",w._comment).replace("attribute",w._attribute).getRegex(),w._label=/(?:\[(?:\\.|[^\[\]\\])*\]|\\.|`[^`]*`|[^\[\]\\`])*?/,w._href=/<(?:\\.|[^\n<>\\])+>|[^\s\x00-\x1f]*/,w._title=/"(?:\\"?|[^"\\])*"|'(?:\\'?|[^'\\])*'|\((?:\\\)?|[^)\\])*\)/,w.link=u(w.link).replace("label",w._label).replace("href",w._href).replace("title",w._title).getRegex(),w.reflink=u(w.reflink).replace("label",w._label).replace("ref",m._label).getRegex(),w.nolink=u(w.nolink).replace("ref",m._label).getRegex(),w.reflinkSearch=u(w.reflinkSearch,"g").replace("reflink",w.reflink).replace("nolink",w.nolink).getRegex(),w.normal={...w},w.pedantic={...w.normal,strong:{start:/^__|\*\*/,middle:/^__(?=\S)([\s\S]*?\S)__(?!_)|^\*\*(?=\S)([\s\S]*?\S)\*\*(?!\*)/,endAst:/\*\*(?!\*)/g,endUnd:/__(?!_)/g},em:{start:/^_|\*/,middle:/^()\*(?=\S)([\s\S]*?\S)\*(?!\*)|^_(?=\S)([\s\S]*?\S)_(?!_)/,endAst:/\*(?!\*)/g,endUnd:/_(?!_)/g},link:u(/^!?\[(label)\]\((.*?)\)/).replace("label",w._label).getRegex(),reflink:u(/^!?\[(label)\]\s*\[([^\]]*)\]/).replace("label",w._label).getRegex()},w.gfm={...w.normal,escape:u(w.escape).replace("])","~|])").getRegex(),_extended_email:/[A-Za-z0-9._+-]+(@)[a-zA-Z0-9-_]+(?:\.[a-zA-Z0-9-_]*[a-zA-Z0-9])+(?![-_])/,url:/^((?:ftp|https?):\/\/|www\.)(?:[a-zA-Z0-9\-]+\.?)+[^\s<]*|^email/,_backpedal:/(?:[^?!.,:;*_'"~()&]+|\([^)]*\)|&(?![a-zA-Z0-9]+;$)|[?!.,:;*_'"~)]+(?!$))+/,del:/^(~~?)(?=[^\s~])([\s\S]*?[^\s~])\1(?=[^~]|$)/,text:/^([`~]+|[^`~])(?:(?= {2,}\n)|(?=[a-zA-Z0-9.!#$%&'*+\/=?_`{\|}~-]+@)|[\s\S]*?(?:(?=[\\<!\[`*~_]|\b_|https?:\/\/|ftp:\/\/|www\.|$)|[^ ](?= {2,}\n)|[^a-zA-Z0-9.!#$%&'*+\/=?_`{\|}~-](?=[a-zA-Z0-9.!#$%&'*+\/=?_`{\|}~-]+@)))/},w.gfm.url=u(w.gfm.url,"i").replace("email",w.gfm._extended_email).getRegex(),w.breaks={...w.gfm,br:u(w.br).replace("{2,}","*").getRegex(),text:u(w.gfm.text).replace("\\b_","\\b_| {2,}\\n").replace(/\{2,\}/g,"*").getRegex()};class _{tokens;options;state;tokenizer;inlineQueue;constructor(t){this.tokens=[],this.tokens.links=Object.create(null),this.options=t||e.defaults,this.options.tokenizer=this.options.tokenizer||new b,this.tokenizer=this.options.tokenizer,this.tokenizer.options=this.options,this.tokenizer.lexer=this,this.inlineQueue=[],this.state={inLink:!1,inRawBlock:!1,top:!0};const n={block:m.normal,inline:w.normal};this.options.pedantic?(n.block=m.pedantic,n.inline=w.pedantic):this.options.gfm&&(n.block=m.gfm,this.options.breaks?n.inline=w.breaks:n.inline=w.gfm),this.tokenizer.rules=n}static get rules(){return{block:m,inline:w}}static lex(e,t){return new _(t).lex(e)}static lexInline(e,t){return new _(t).inlineTokens(e)}lex(e){let t;for(e=e.replace(/\r\n|\r/g,"\n"),this.blockTokens(e,this.tokens);t=this.inlineQueue.shift();)this.inlineTokens(t.src,t.tokens);return this.tokens}blockTokens(e,t=[]){let n,s,r,i;for(e=this.options.pedantic?e.replace(/\t/g,"    ").replace(/^ +$/gm,""):e.replace(/^( *)(\t+)/gm,((e,t,n)=>t+"    ".repeat(n.length)));e;)if(!(this.options.extensions&&this.options.extensions.block&&this.options.extensions.block.some((s=>!!(n=s.call({lexer:this},e,t))&&(e=e.substring(n.raw.length),t.push(n),!0)))))if(n=this.tokenizer.space(e))e=e.substring(n.raw.length),1===n.raw.length&&t.length>0?t[t.length-1].raw+="\n":t.push(n);else if(n=this.tokenizer.code(e))e=e.substring(n.raw.length),s=t[t.length-1],!s||"paragraph"!==s.type&&"text"!==s.type?t.push(n):(s.raw+="\n"+n.raw,s.text+="\n"+n.text,this.inlineQueue[this.inlineQueue.length-1].src=s.text);else if(n=this.tokenizer.fences(e))e=e.substring(n.raw.length),t.push(n);else if(n=this.tokenizer.heading(e))e=e.substring(n.raw.length),t.push(n);else if(n=this.tokenizer.hr(e))e=e.substring(n.raw.length),t.push(n);else if(n=this.tokenizer.blockquote(e))e=e.substring(n.raw.length),t.push(n);else if(n=this.tokenizer.list(e))e=e.substring(n.raw.length),t.push(n);else if(n=this.tokenizer.html(e))e=e.substring(n.raw.length),t.push(n);else if(n=this.tokenizer.def(e))e=e.substring(n.raw.length),s=t[t.length-1],!s||"paragraph"!==s.type&&"text"!==s.type?this.tokens.links[n.tag]||(this.tokens.links[n.tag]={href:n.href,title:n.title}):(s.raw+="\n"+n.raw,s.text+="\n"+n.raw,this.inlineQueue[this.inlineQueue.length-1].src=s.text);else if(n=this.tokenizer.table(e))e=e.substring(n.raw.length),t.push(n);else if(n=this.tokenizer.lheading(e))e=e.substring(n.raw.length),t.push(n);else{if(r=e,this.options.extensions&&this.options.extensions.startBlock){let t=1/0;const n=e.slice(1);let s;this.options.extensions.startBlock.forEach((e=>{s=e.call({lexer:this},n),"number"==typeof s&&s>=0&&(t=Math.min(t,s))})),t<1/0&&t>=0&&(r=e.substring(0,t+1))}if(this.state.top&&(n=this.tokenizer.paragraph(r)))s=t[t.length-1],i&&"paragraph"===s.type?(s.raw+="\n"+n.raw,s.text+="\n"+n.text,this.inlineQueue.pop(),this.inlineQueue[this.inlineQueue.length-1].src=s.text):t.push(n),i=r.length!==e.length,e=e.substring(n.raw.length);else if(n=this.tokenizer.text(e))e=e.substring(n.raw.length),s=t[t.length-1],s&&"text"===s.type?(s.raw+="\n"+n.raw,s.text+="\n"+n.text,this.inlineQueue.pop(),this.inlineQueue[this.inlineQueue.length-1].src=s.text):t.push(n);else if(e){const t="Infinite loop on byte: "+e.charCodeAt(0);if(this.options.silent){console.error(t);break}throw new Error(t)}}return this.state.top=!0,t}inline(e,t=[]){return this.inlineQueue.push({src:e,tokens:t}),t}inlineTokens(e,t=[]){let n,s,r,i,l,o,a=e;if(this.tokens.links){const e=Object.keys(this.tokens.links);if(e.length>0)for(;null!=(i=this.tokenizer.rules.inline.reflinkSearch.exec(a));)e.includes(i[0].slice(i[0].lastIndexOf("[")+1,-1))&&(a=a.slice(0,i.index)+"["+"a".repeat(i[0].length-2)+"]"+a.slice(this.tokenizer.rules.inline.reflinkSearch.lastIndex))}for(;null!=(i=this.tokenizer.rules.inline.blockSkip.exec(a));)a=a.slice(0,i.index)+"["+"a".repeat(i[0].length-2)+"]"+a.slice(this.tokenizer.rules.inline.blockSkip.lastIndex);for(;null!=(i=this.tokenizer.rules.inline.anyPunctuation.exec(a));)a=a.slice(0,i.index)+"++"+a.slice(this.tokenizer.rules.inline.anyPunctuation.lastIndex);for(;e;)if(l||(o=""),l=!1,!(this.options.extensions&&this.options.extensions.inline&&this.options.extensions.inline.some((s=>!!(n=s.call({lexer:this},e,t))&&(e=e.substring(n.raw.length),t.push(n),!0)))))if(n=this.tokenizer.escape(e))e=e.substring(n.raw.length),t.push(n);else if(n=this.tokenizer.tag(e))e=e.substring(n.raw.length),s=t[t.length-1],s&&"text"===n.type&&"text"===s.type?(s.raw+=n.raw,s.text+=n.text):t.push(n);else if(n=this.tokenizer.link(e))e=e.substring(n.raw.length),t.push(n);else if(n=this.tokenizer.reflink(e,this.tokens.links))e=e.substring(n.raw.length),s=t[t.length-1],s&&"text"===n.type&&"text"===s.type?(s.raw+=n.raw,s.text+=n.text):t.push(n);else if(n=this.tokenizer.emStrong(e,a,o))e=e.substring(n.raw.length),t.push(n);else if(n=this.tokenizer.codespan(e))e=e.substring(n.raw.length),t.push(n);else if(n=this.tokenizer.br(e))e=e.substring(n.raw.length),t.push(n);else if(n=this.tokenizer.del(e))e=e.substring(n.raw.length),t.push(n);else if(n=this.tokenizer.autolink(e))e=e.substring(n.raw.length),t.push(n);else if(this.state.inLink||!(n=this.tokenizer.url(e))){if(r=e,this.options.extensions&&this.options.extensions.startInline){let t=1/0;const n=e.slice(1);let s;this.options.extensions.startInline.forEach((e=>{s=e.call({lexer:this},n),"number"==typeof s&&s>=0&&(t=Math.min(t,s))})),t<1/0&&t>=0&&(r=e.substring(0,t+1))}if(n=this.tokenizer.inlineText(r))e=e.substring(n.raw.length),"_"!==n.raw.slice(-1)&&(o=n.raw.slice(-1)),l=!0,s=t[t.length-1],s&&"text"===s.type?(s.raw+=n.raw,s.text+=n.text):t.push(n);else if(e){const t="Infinite loop on byte: "+e.charCodeAt(0);if(this.options.silent){console.error(t);break}throw new Error(t)}}else e=e.substring(n.raw.length),t.push(n);return t}}class y{options;constructor(t){this.options=t||e.defaults}code(e,t,n){const s=(t||"").match(/^\S*/)?.[0];return e=e.replace(/\n$/,"")+"\n",s?'<pre><code class="language-'+c(s)+'">'+(n?e:c(e,!0))+"</code></pre>\n":"<pre><code>"+(n?e:c(e,!0))+"</code></pre>\n"}blockquote(e){return`<blockquote>\n${e}</blockquote>\n`}html(e,t){return e}heading(e,t,n){return`<h${t}>${e}</h${t}>\n`}hr(){return"<hr>\n"}list(e,t,n){const s=t?"ol":"ul";return"<"+s+(t&&1!==n?' start="'+n+'"':"")+">\n"+e+"</"+s+">\n"}listitem(e,t,n){return`<li>${e}</li>\n`}checkbox(e){return"<input "+(e?'checked="" ':"")+'disabled="" type="checkbox">'}paragraph(e){return`<p>${e}</p>\n`}table(e,t){return t&&(t=`<tbody>${t}</tbody>`),"<table>\n<thead>\n"+e+"</thead>\n"+t+"</table>\n"}tablerow(e){return`<tr>\n${e}</tr>\n`}tablecell(e,t){const n=t.header?"th":"td";return(t.align?`<${n} align="${t.align}">`:`<${n}>`)+e+`</${n}>\n`}strong(e){return`<strong>${e}</strong>`}em(e){return`<em>${e}</em>`}codespan(e){return`<code>${e}</code>`}br(){return"<br>"}del(e){return`<del>${e}</del>`}link(e,t,n){const s=g(e);if(null===s)return n;let r='<a href="'+(e=s)+'"';return t&&(r+=' title="'+t+'"'),r+=">"+n+"</a>",r}image(e,t,n){const s=g(e);if(null===s)return n;let r=`<img src="${e=s}" alt="${n}"`;return t&&(r+=` title="${t}"`),r+=">",r}text(e){return e}}class ${strong(e){return e}em(e){return e}codespan(e){return e}del(e){return e}html(e){return e}text(e){return e}link(e,t,n){return""+n}image(e,t,n){return""+n}br(){return""}}class z{options;renderer;textRenderer;constructor(t){this.options=t||e.defaults,this.options.renderer=this.options.renderer||new y,this.renderer=this.options.renderer,this.renderer.options=this.options,this.textRenderer=new $}static parse(e,t){return new z(t).parse(e)}static parseInline(e,t){return new z(t).parseInline(e)}parse(e,t=!0){let n="";for(let s=0;s<e.length;s++){const r=e[s];if(this.options.extensions&&this.options.extensions.renderers&&this.options.extensions.renderers[r.type]){const e=r,t=this.options.extensions.renderers[e.type].call({parser:this},e);if(!1!==t||!["space","hr","heading","code","table","blockquote","list","html","paragraph","text"].includes(e.type)){n+=t||"";continue}}switch(r.type){case"space":continue;case"hr":n+=this.renderer.hr();continue;case"heading":{const e=r;n+=this.renderer.heading(this.parseInline(e.tokens),e.depth,this.parseInline(e.tokens,this.textRenderer).replace(h,((e,t)=>"colon"===(t=t.toLowerCase())?":":"#"===t.charAt(0)?"x"===t.charAt(1)?String.fromCharCode(parseInt(t.substring(2),16)):String.fromCharCode(+t.substring(1)):"")));continue}case"code":{const e=r;n+=this.renderer.code(e.text,e.lang,!!e.escaped);continue}case"table":{const e=r;let t="",s="";for(let t=0;t<e.header.length;t++)s+=this.renderer.tablecell(this.parseInline(e.header[t].tokens),{header:!0,align:e.align[t]});t+=this.renderer.tablerow(s);let i="";for(let t=0;t<e.rows.length;t++){const n=e.rows[t];s="";for(let t=0;t<n.length;t++)s+=this.renderer.tablecell(this.parseInline(n[t].tokens),{header:!1,align:e.align[t]});i+=this.renderer.tablerow(s)}n+=this.renderer.table(t,i);continue}case"blockquote":{const e=r,t=this.parse(e.tokens);n+=this.renderer.blockquote(t);continue}case"list":{const e=r,t=e.ordered,s=e.start,i=e.loose;let l="";for(let t=0;t<e.items.length;t++){const n=e.items[t],s=n.checked,r=n.task;let o="";if(n.task){const e=this.renderer.checkbox(!!s);i?n.tokens.length>0&&"paragraph"===n.tokens[0].type?(n.tokens[0].text=e+" "+n.tokens[0].text,n.tokens[0].tokens&&n.tokens[0].tokens.length>0&&"text"===n.tokens[0].tokens[0].type&&(n.tokens[0].tokens[0].text=e+" "+n.tokens[0].tokens[0].text)):n.tokens.unshift({type:"text",text:e+" "}):o+=e+" "}o+=this.parse(n.tokens,i),l+=this.renderer.listitem(o,r,!!s)}n+=this.renderer.list(l,t,s);continue}case"html":{const e=r;n+=this.renderer.html(e.text,e.block);continue}case"paragraph":{const e=r;n+=this.renderer.paragraph(this.parseInline(e.tokens));continue}case"text":{let i=r,l=i.tokens?this.parseInline(i.tokens):i.text;for(;s+1<e.length&&"text"===e[s+1].type;)i=e[++s],l+="\n"+(i.tokens?this.parseInline(i.tokens):i.text);n+=t?this.renderer.paragraph(l):l;continue}default:{const e='Token with "'+r.type+'" type was not found.';if(this.options.silent)return console.error(e),"";throw new Error(e)}}}return n}parseInline(e,t){t=t||this.renderer;let n="";for(let s=0;s<e.length;s++){const r=e[s];if(this.options.extensions&&this.options.extensions.renderers&&this.options.extensions.renderers[r.type]){const e=this.options.extensions.renderers[r.type].call({parser:this},r);if(!1!==e||!["escape","html","link","image","strong","em","codespan","br","del","text"].includes(r.type)){n+=e||"";continue}}switch(r.type){case"escape":{const e=r;n+=t.text(e.text);break}case"html":{const e=r;n+=t.html(e.text);break}case"link":{const e=r;n+=t.link(e.href,e.title,this.parseInline(e.tokens,t));break}case"image":{const e=r;n+=t.image(e.href,e.title,e.text);break}case"strong":{const e=r;n+=t.strong(this.parseInline(e.tokens,t));break}case"em":{const e=r;n+=t.em(this.parseInline(e.tokens,t));break}case"codespan":{const e=r;n+=t.codespan(e.text);break}case"br":n+=t.br();break;case"del":{const e=r;n+=t.del(this.parseInline(e.tokens,t));break}case"text":{const e=r;n+=t.text(e.text);break}default:{const e='Token with "'+r.type+'" type was not found.';if(this.options.silent)return console.error(e),"";throw new Error(e)}}}return n}}class T{options;constructor(t){this.options=t||e.defaults}static passThroughHooks=new Set(["preprocess","postprocess"]);preprocess(e){return e}postprocess(e){return e}}class R{defaults={async:!1,breaks:!1,extensions:null,gfm:!0,hooks:null,pedantic:!1,renderer:null,silent:!1,tokenizer:null,walkTokens:null};options=this.setOptions;parse=this.#e(_.lex,z.parse);parseInline=this.#e(_.lexInline,z.parseInline);Parser=z;parser=z.parse;Renderer=y;TextRenderer=$;Lexer=_;lexer=_.lex;Tokenizer=b;Hooks=T;constructor(...e){this.use(...e)}walkTokens(e,t){let n=[];for(const s of e)switch(n=n.concat(t.call(this,s)),s.type){case"table":{const e=s;for(const s of e.header)n=n.concat(this.walkTokens(s.tokens,t));for(const s of e.rows)for(const e of s)n=n.concat(this.walkTokens(e.tokens,t));break}case"list":{const e=s;n=n.concat(this.walkTokens(e.items,t));break}default:{const e=s;this.defaults.extensions?.childTokens?.[e.type]?this.defaults.extensions.childTokens[e.type].forEach((s=>{n=n.concat(this.walkTokens(e[s],t))})):e.tokens&&(n=n.concat(this.walkTokens(e.tokens,t)))}}return n}use(...e){const t=this.defaults.extensions||{renderers:{},childTokens:{}};return e.forEach((e=>{const n={...e};if(n.async=this.defaults.async||n.async||!1,e.extensions&&(e.extensions.forEach((e=>{if(!e.name)throw new Error("extension name required");if("renderer"in e){const n=t.renderers[e.name];t.renderers[e.name]=n?function(...t){let s=e.renderer.apply(this,t);return!1===s&&(s=n.apply(this,t)),s}:e.renderer}if("tokenizer"in e){if(!e.level||"block"!==e.level&&"inline"!==e.level)throw new Error("extension level must be 'block' or 'inline'");const n=t[e.level];n?n.unshift(e.tokenizer):t[e.level]=[e.tokenizer],e.start&&("block"===e.level?t.startBlock?t.startBlock.push(e.start):t.startBlock=[e.start]:"inline"===e.level&&(t.startInline?t.startInline.push(e.start):t.startInline=[e.start]))}"childTokens"in e&&e.childTokens&&(t.childTokens[e.name]=e.childTokens)})),n.extensions=t),e.renderer){const t=this.defaults.renderer||new y(this.defaults);for(const n in e.renderer){const s=e.renderer[n],r=n,i=t[r];t[r]=(...e)=>{let n=s.apply(t,e);return!1===n&&(n=i.apply(t,e)),n||""}}n.renderer=t}if(e.tokenizer){const t=this.defaults.tokenizer||new b(this.defaults);for(const n in e.tokenizer){const s=e.tokenizer[n],r=n,i=t[r];t[r]=(...e)=>{let n=s.apply(t,e);return!1===n&&(n=i.apply(t,e)),n}}n.tokenizer=t}if(e.hooks){const t=this.defaults.hooks||new T;for(const n in e.hooks){const s=e.hooks[n],r=n,i=t[r];T.passThroughHooks.has(n)?t[r]=e=>{if(this.defaults.async)return Promise.resolve(s.call(t,e)).then((e=>i.call(t,e)));const n=s.call(t,e);return i.call(t,n)}:t[r]=(...e)=>{let n=s.apply(t,e);return!1===n&&(n=i.apply(t,e)),n}}n.hooks=t}if(e.walkTokens){const t=this.defaults.walkTokens,s=e.walkTokens;n.walkTokens=function(e){let n=[];return n.push(s.call(this,e)),t&&(n=n.concat(t.call(this,e))),n}}this.defaults={...this.defaults,...n}})),this}setOptions(e){return this.defaults={...this.defaults,...e},this}#e(e,t){return(n,s)=>{const r={...s},i={...this.defaults,...r};!0===this.defaults.async&&!1===r.async&&(i.silent||console.warn("marked(): The async option was set to true by an extension. The async: false option sent to parse will be ignored."),i.async=!0);const l=this.#t(!!i.silent,!!i.async);if(null==n)return l(new Error("marked(): input parameter is undefined or null"));if("string"!=typeof n)return l(new Error("marked(): input parameter is of type "+Object.prototype.toString.call(n)+", string expected"));if(i.hooks&&(i.hooks.options=i),i.async)return Promise.resolve(i.hooks?i.hooks.preprocess(n):n).then((t=>e(t,i))).then((e=>i.walkTokens?Promise.all(this.walkTokens(e,i.walkTokens)).then((()=>e)):e)).then((e=>t(e,i))).then((e=>i.hooks?i.hooks.postprocess(e):e)).catch(l);try{i.hooks&&(n=i.hooks.preprocess(n));const s=e(n,i);i.walkTokens&&this.walkTokens(s,i.walkTokens);let r=t(s,i);return i.hooks&&(r=i.hooks.postprocess(r)),r}catch(e){return l(e)}}}#t(e,t){return n=>{if(n.message+="\nPlease report this to https://github.com/markedjs/marked.",e){const e="<p>An error occurred:</p><pre>"+c(n.message+"",!0)+"</pre>";return t?Promise.resolve(e):e}if(t)return Promise.reject(n);throw n}}}const S=new R;function A(e,t){return S.parse(e,t)}A.options=A.setOptions=function(e){return S.setOptions(e),A.defaults=S.defaults,n(A.defaults),A},A.getDefaults=t,A.defaults=e.defaults,A.use=function(...e){return S.use(...e),A.defaults=S.defaults,n(A.defaults),A},A.walkTokens=function(e,t){return S.walkTokens(e,t)},A.parseInline=S.parseInline,A.Parser=z,A.parser=z.parse,A.Renderer=y,A.TextRenderer=$,A.Lexer=_,A.lexer=_.lex,A.Tokenizer=b,A.Hooks=T,A.parse=A;const I=A.options,E=A.setOptions,Z=A.use,q=A.walkTokens,L=A.parseInline,D=A,P=z.parse,v=_.lex;e.Hooks=T,e.Lexer=_,e.Marked=R,e.Parser=z,e.Renderer=y,e.TextRenderer=$,e.Tokenizer=b,e.getDefaults=t,e.lexer=v,e.marked=A,e.options=I,e.parse=D,e.parseInline=L,e.parser=P,e.setOptions=E,e.use=Z,e.walkTokens=q}));
const fastn = (function (fastn) {
    class Closure {
        #cached_value;
        #node;
        #property;
        #formula;
        #inherited;
        constructor(func, execute = true) {
            if (execute) {
                this.#cached_value = func();
            }
            this.#formula = func;
        }

        get() {
            return this.#cached_value;
        }
        getFormula() {
            return this.#formula;
        }
        addNodeProperty(node, property, inherited) {
            this.#node = node;
            this.#property = property;
            this.#inherited = inherited;
            this.updateUi();

            return this;
        }
        update() {
            this.#cached_value = this.#formula();
            this.updateUi();
        }
        getNode() {
            return this.#node;
        }
        updateUi() {
            if (
                !this.#node ||
                this.#property === null ||
                this.#property === undefined ||
                !this.#node.getNode()
            ) {
                return;
            }

            this.#node.setStaticProperty(
                this.#property,
                this.#cached_value,
                this.#inherited,
            );
        }
    }

    class Mutable {
        #value;
        #old_closure;
        #closures;
        #closureInstance;
        constructor(val) {
            this.#value = null;
            this.#old_closure = null;
            this.#closures = [];
            this.#closureInstance = fastn.closure(() =>
                this.#closures.forEach((closure) => closure.update()),
            );
            this.set(val);
        }
        get(key) {
            if (
                !fastn_utils.isNull(key) &&
                (this.#value instanceof RecordInstance ||
                    this.#value instanceof MutableList ||
                    this.#value instanceof Mutable)
            ) {
                return this.#value.get(key);
            }
            return this.#value;
        }
        setWithoutUpdate(value) {
            if (this.#old_closure) {
                this.#value.removeClosure(this.#old_closure);
            }

            if (this.#value instanceof RecordInstance) {
                // this.#value.replace(value); will replace the record type
                // variable instance created which we don't want.
                // color: red
                // color if { something }: $orange-green
                // The `this.#value.replace(value);` will replace the value of
                // `orange-green` with `{light: red, dark: red}`
                this.#value = value;
            } else {
                this.#value = value;
            }

            if (this.#value instanceof Mutable) {
                this.#old_closure = fastn.closureWithoutExecute(() =>
                    this.#closureInstance.update(),
                );
                this.#value.addClosure(this.#old_closure);
            } else {
                this.#old_closure = null;
            }
        }
        set(value) {
            this.setWithoutUpdate(value);

            this.#closureInstance.update();
        }
        // we have to unlink all nodes, else they will be kept in memory after the node is removed from DOM
        unlinkNode(node) {
            this.#closures = this.#closures.filter(
                (closure) => closure.getNode() !== node,
            );
        }
        addClosure(closure) {
            this.#closures.push(closure);
        }
        removeClosure(closure) {
            this.#closures = this.#closures.filter((c) => c !== closure);
        }
        equalMutable(other) {
            if (!fastn_utils.deepEqual(this.get(), other.get())) {
                return false;
            }
            const thisClosures = this.#closures;
            const otherClosures = other.#closures;

            return thisClosures === otherClosures;
        }
        getClone() {
            return new Mutable(fastn_utils.clone(this.#value));
        }
    }

    class Proxy {
        #differentiator;
        #cached_value;
        #closures;
        #closureInstance;
        constructor(targets, differentiator) {
            this.#differentiator = differentiator;
            this.#cached_value = this.#differentiator().get();
            this.#closures = [];

            let proxy = this;
            for (let idx in targets) {
                targets[idx].addClosure(
                    new Closure(function () {
                        proxy.update();
                        proxy.#closures.forEach((closure) => closure.update());
                    }),
                );
                targets[idx].addClosure(this);
            }
        }
        addClosure(closure) {
            this.#closures.push(closure);
        }
        removeClosure(closure) {
            this.#closures = this.#closures.filter((c) => c !== closure);
        }
        update() {
            this.#cached_value = this.#differentiator().get();
        }
        get(key) {
            if (
                !!key &&
                (this.#cached_value instanceof RecordInstance ||
                    this.#cached_value instanceof MutableList ||
                    this.#cached_value instanceof Mutable)
            ) {
                return this.#cached_value.get(key);
            }
            return this.#cached_value;
        }
        set(value) {
            // Todo: Optimization removed. Reuse optimization later again
            /*if (fastn_utils.deepEqual(this.#cached_value, value)) {
                return;
            }*/
            this.#differentiator().set(value);
        }
    }

    class MutableList {
        #list;
        #watchers;
        #closures;
        constructor(list) {
            this.#list = [];
            for (let idx in list) {
                this.#list.push({
                    item: fastn.wrapMutable(list[idx]),
                    index: new Mutable(parseInt(idx)),
                });
            }
            this.#watchers = [];
            this.#closures = [];
        }
        addClosure(closure) {
            this.#closures.push(closure);
        }
        unlinkNode(node) {
            this.#closures = this.#closures.filter(
                (closure) => closure.getNode() !== node,
            );
        }
        forLoop(root, dom_constructor) {
            let l = fastn_dom.forLoop(root, dom_constructor, this);
            this.#watchers.push(l);
            return l;
        }
        getList() {
            return this.#list;
        }
        getLength() {
            return this.#list.length;
        }
        get(idx) {
            if (fastn_utils.isNull(idx)) {
                return this.getList();
            }
            return this.#list[idx];
        }
        set(index, value) {
            if (value === undefined) {
                value = index;
                if (!(value instanceof MutableList)) {
                    if (!Array.isArray(value)) {
                        value = [value];
                    }
                    value = new MutableList(value);
                }

                let list = value.#list;
                this.#list = [];
                for (let i in list) {
                    this.#list.push(list[i]);
                }

                for (let i in this.#watchers) {
                    this.#watchers[i].createAllNode();
                }
            } else {
                index = fastn_utils.getFlattenStaticValue(index);
                this.#list[index].item.set(value);
            }

            this.#closures.forEach((closure) => closure.update());
        }
        insertAt(index, value) {
            index = fastn_utils.getFlattenStaticValue(index);
            let mutable = fastn.wrapMutable(value);
            this.#list.splice(index, 0, {
                item: mutable,
                index: new Mutable(index),
            });
            // for every item after the inserted item, update the index
            for (let i = index + 1; i < this.#list.length; i++) {
                this.#list[i].index.set(i);
            }

            for (let i in this.#watchers) {
                this.#watchers[i].createNode(index);
            }
            this.#closures.forEach((closure) => closure.update());
        }
        push(value) {
            this.insertAt(this.#list.length, value);
        }
        deleteAt(index) {
            index = fastn_utils.getFlattenStaticValue(index);
            this.#list.splice(index, 1);
            // for every item after the deleted item, update the index
            for (let i = index; i < this.#list.length; i++) {
                this.#list[i].index.set(i);
            }

            for (let i in this.#watchers) {
                let forLoop = this.#watchers[i];
                forLoop.deleteNode(index);
            }
            this.#closures.forEach((closure) => closure.update());
        }
        clearAll() {
            this.#list = [];
            for (let i in this.#watchers) {
                this.#watchers[i].deleteAllNode();
            }
            this.#closures.forEach((closure) => closure.update());
        }
        pop() {
            this.deleteAt(this.#list.length - 1);
        }
        getClone() {
            let current_list = this.#list;
            let new_list = [];
            for (let idx in current_list) {
                new_list.push(fastn_utils.clone(current_list[idx].item));
            }
            return new MutableList(new_list);
        }
    }

    fastn.mutable = function (val) {
        return new Mutable(val);
    };

    fastn.closure = function (func) {
        return new Closure(func);
    };

    fastn.closureWithoutExecute = function (func) {
        return new Closure(func, false);
    };

    fastn.formula = function (deps, func) {
        let closure = fastn.closure(func);
        let mutable = new Mutable(closure.get());
        for (let idx in deps) {
            if (fastn_utils.isNull(deps[idx]) || !deps[idx].addClosure) {
                continue;
            }
            deps[idx].addClosure(
                new Closure(function () {
                    closure.update();
                    mutable.set(closure.get());
                }),
            );
        }

        return mutable;
    };

    fastn.proxy = function (targets, differentiator) {
        return new Proxy(targets, differentiator);
    };

    fastn.wrapMutable = function (obj) {
        if (
            !(obj instanceof Mutable) &&
            !(obj instanceof RecordInstance) &&
            !(obj instanceof MutableList)
        ) {
            obj = new Mutable(obj);
        }
        return obj;
    };

    fastn.mutableList = function (list) {
        return new MutableList(list);
    };

    class RecordInstance {
        #fields;
        #closures;
        constructor(obj) {
            this.#fields = {};
            this.#closures = [];

            for (let key in obj) {
                if (obj[key] instanceof fastn.mutableClass) {
                    this.#fields[key] = fastn.mutable(null);
                    this.#fields[key].setWithoutUpdate(obj[key]);
                } else {
                    this.#fields[key] = fastn.mutable(obj[key]);
                }
            }
        }
        getAllFields() {
            return this.#fields;
        }
        getClonedFields() {
            let clonedFields = {};
            for (let key in this.#fields) {
                let field_value = this.#fields[key];
                if (
                    field_value instanceof fastn.recordInstanceClass ||
                    field_value instanceof fastn.mutableClass ||
                    field_value instanceof fastn.mutableListClass
                ) {
                    clonedFields[key] = this.#fields[key].getClone();
                } else {
                    clonedFields[key] = this.#fields[key];
                }
            }
            return clonedFields;
        }
        addClosure(closure) {
            this.#closures.push(closure);
        }
        unlinkNode(node) {
            this.#closures = this.#closures.filter(
                (closure) => closure.getNode() !== node,
            );
        }
        get(key) {
            return this.#fields[key];
        }
        set(key, value) {
            if (value === undefined) {
                value = key;
                if (!(value instanceof RecordInstance)) {
                    value = new RecordInstance(value);
                }

                let fields = {};
                for (let key in value.#fields) {
                    fields[key] = value.#fields[key];
                }

                this.#fields = fields;
            } else if (this.#fields[key] === undefined) {
                this.#fields[key] = fastn.mutable(null);
                this.#fields[key].setWithoutUpdate(value);
            } else {
                this.#fields[key].set(value);
            }
            this.#closures.forEach((closure) => closure.update());
        }
        setAndReturn(key, value) {
            this.set(key, value);
            return this;
        }
        replace(obj) {
            for (let key in this.#fields) {
                if (!(key in obj.#fields)) {
                    throw new Error(
                        "RecordInstance.replace: key " +
                            key +
                            " not present in new object",
                    );
                }
                this.#fields[key] = fastn.wrapMutable(obj.#fields[key]);
            }
            this.#closures.forEach((closure) => closure.update());
        }
        toObject() {
            return Object.fromEntries(
                Object.entries(this.#fields).map(([key, value]) => [
                    key,
                    fastn_utils.getFlattenStaticValue(value),
                ]),
            );
        }
        getClone() {
            let current_fields = this.#fields;
            let cloned_fields = {};
            for (let key in current_fields) {
                let value = fastn_utils.clone(current_fields[key]);
                if (value instanceof fastn.mutableClass) {
                    value = value.get();
                }
                cloned_fields[key] = value;
            }
            return new RecordInstance(cloned_fields);
        }
    }

    class Module {
        #name;
        #global;
        constructor(name, global) {
            this.#name = name;
            this.#global = global;
        }

        getName() {
            return this.#name;
        }

        get(function_name) {
            return this.#global[`${this.#name}__${function_name}`];
        }
    }

    fastn.recordInstance = function (obj) {
        return new RecordInstance(obj);
    };

    fastn.color = function (r, g, b) {
        return `rgb(${r},${g},${b})`;
    };

    fastn.mutableClass = Mutable;
    fastn.mutableListClass = MutableList;
    fastn.recordInstanceClass = RecordInstance;
    fastn.module = function (name, global) {
        return new Module(name, global);
    };
    fastn.moduleClass = Module;

    return fastn;
})({});
let fastn_dom = {};

fastn_dom.styleClasses = "";

fastn_dom.InternalClass = {
    FT_COLUMN: "ft_column",
    FT_ROW: "ft_row",
    FT_FULL_SIZE: "ft_full_size",
};

fastn_dom.codeData = {
    availableThemes: {},
    addedCssFile: [],
};

fastn_dom.externalCss = new Set();
fastn_dom.externalJs = new Set();

// Todo: Object (key, value) pair (counter type key)
fastn_dom.webComponent = [];

fastn_dom.commentNode = "comment";
fastn_dom.wrapperNode = "wrapper";
fastn_dom.commentMessage = "***FASTN***";
fastn_dom.webComponentArgument = "args";

fastn_dom.classes = {};
fastn_dom.unsanitised_classes = {};
fastn_dom.class_count = 0;
fastn_dom.propertyMap = {
    "align-items": "ali",
    "align-self": "as",
    "background-color": "bgc",
    "background-image": "bgi",
    "background-position": "bgp",
    "background-repeat": "bgr",
    "background-size": "bgs",
    "border-bottom-color": "bbc",
    "border-bottom-left-radius": "bblr",
    "border-bottom-right-radius": "bbrr",
    "border-bottom-style": "bbs",
    "border-bottom-width": "bbw",
    "border-color": "bc",
    "border-left-color": "blc",
    "border-left-style": "bls",
    "border-left-width": "blw",
    "border-radius": "br",
    "border-right-color": "brc",
    "border-right-style": "brs",
    "border-right-width": "brw",
    "border-style": "bs",
    "border-top-color": "btc",
    "border-top-left-radius": "btlr",
    "border-top-right-radius": "btrr",
    "border-top-style": "bts",
    "border-top-width": "btw",
    "border-width": "bw",
    bottom: "b",
    color: "c",
    shadow: "sh",
    "text-shadow": "tsh",
    cursor: "cur",
    display: "d",
    "flex-wrap": "fw",
    "font-style": "fst",
    "font-weight": "fwt",
    gap: "g",
    height: "h",
    "justify-content": "jc",
    left: "l",
    link: "lk",
    "link-color": "lkc",
    margin: "m",
    "margin-bottom": "mb",
    "margin-horizontal": "mh",
    "margin-left": "ml",
    "margin-right": "mr",
    "margin-top": "mt",
    "margin-vertical": "mv",
    "max-height": "mxh",
    "max-width": "mxw",
    "min-height": "mnh",
    "min-width": "mnw",
    opacity: "op",
    overflow: "o",
    "overflow-x": "ox",
    "overflow-y": "oy",
    "object-fit": "of",
    padding: "p",
    "padding-bottom": "pb",
    "padding-horizontal": "ph",
    "padding-left": "pl",
    "padding-right": "pr",
    "padding-top": "pt",
    "padding-vertical": "pv",
    position: "pos",
    resize: "res",
    role: "rl",
    right: "r",
    sticky: "s",
    "text-align": "ta",
    "text-decoration": "td",
    "text-transform": "tt",
    top: "t",
    width: "w",
    "z-index": "z",
    "-webkit-box-orient": "wbo",
    "-webkit-line-clamp": "wlc",
    "backdrop-filter": "bdf",
    "mask-image": "mi",
    "-webkit-mask-image": "wmi",
    "mask-size": "ms",
    "-webkit-mask-size": "wms",
    "mask-repeat": "mre",
    "-webkit-mask-repeat": "wmre",
    "mask-position": "mp",
    "-webkit-mask-position": "wmp",
    "fetch-priority": "ftp",
};

// dynamic-class-css.md
fastn_dom.getClassesAsString = function () {
    return `<style id="styles">
    ${fastn_dom.getClassesAsStringWithoutStyleTag()}
    </style>`;
};

fastn_dom.getClassesAsStringWithoutStyleTag = function () {
    let classes = Object.entries(fastn_dom.classes).map((entry) => {
        return getClassAsString(entry[0], entry[1]);
    });

    /*.ft_text {
        padding: 0;
    }*/
    return classes.join("\n\t");
};

function getClassAsString(className, obj) {
    if (typeof obj.value === "object" && obj.value !== null) {
        let value = "";
        for (let key in obj.value) {
            if (obj.value[key] === undefined || obj.value[key] === null) {
                continue;
            }
            value = `${value} ${key}: ${obj.value[key]}${
                key === "color" ? " !important" : ""
            };`;
        }
        return `${className} { ${value} }`;
    } else {
        return `${className} { ${obj.property}: ${obj.value}${
            obj.property === "color" ? " !important" : ""
        }; }`;
    }
}

fastn_dom.ElementKind = {
    Row: 0,
    Column: 1,
    Integer: 2,
    Decimal: 3,
    Boolean: 4,
    Text: 5,
    Image: 6,
    IFrame: 7,
    // To create parent for dynamic DOM
    Comment: 8,
    CheckBox: 9,
    TextInput: 10,
    ContainerElement: 11,
    Rive: 12,
    Document: 13,
    Wrapper: 14,
    Code: 15,
    // Note: This is called internally, it gives `code` as tagName. This is used
    // along with the Code: 15.
    CodeChild: 16,
    // Note: 'arguments' cant be used as function parameter name bcoz it has
    // internal usage in js functions.
    WebComponent: (webcomponent, args) => {
        return [17, [webcomponent, args]];
    },
    Video: 18,
};

fastn_dom.PropertyKind = {
    Color: 0,
    IntegerValue: 1,
    StringValue: 2,
    DecimalValue: 3,
    BooleanValue: 4,
    Width: 5,
    Padding: 6,
    Height: 7,
    Id: 8,
    BorderWidth: 9,
    BorderStyle: 10,
    Margin: 11,
    Background: 12,
    PaddingHorizontal: 13,
    PaddingVertical: 14,
    PaddingLeft: 15,
    PaddingRight: 16,
    PaddingTop: 17,
    PaddingBottom: 18,
    MarginHorizontal: 19,
    MarginVertical: 20,
    MarginLeft: 21,
    MarginRight: 22,
    MarginTop: 23,
    MarginBottom: 24,
    Role: 25,
    ZIndex: 26,
    Sticky: 27,
    Top: 28,
    Bottom: 29,
    Left: 30,
    Right: 31,
    Overflow: 32,
    OverflowX: 33,
    OverflowY: 34,
    Spacing: 35,
    Wrap: 36,
    TextTransform: 37,
    TextIndent: 38,
    TextAlign: 39,
    LineClamp: 40,
    Opacity: 41,
    Cursor: 42,
    Resize: 43,
    MinHeight: 44,
    MaxHeight: 45,
    MinWidth: 46,
    MaxWidth: 47,
    WhiteSpace: 48,
    BorderTopWidth: 49,
    BorderBottomWidth: 50,
    BorderLeftWidth: 51,
    BorderRightWidth: 52,
    BorderRadius: 53,
    BorderTopLeftRadius: 54,
    BorderTopRightRadius: 55,
    BorderBottomLeftRadius: 56,
    BorderBottomRightRadius: 57,
    BorderStyleVertical: 58,
    BorderStyleHorizontal: 59,
    BorderLeftStyle: 60,
    BorderRightStyle: 61,
    BorderTopStyle: 62,
    BorderBottomStyle: 63,
    BorderColor: 64,
    BorderLeftColor: 65,
    BorderRightColor: 66,
    BorderTopColor: 67,
    BorderBottomColor: 68,
    AlignSelf: 69,
    Classes: 70,
    Anchor: 71,
    Link: 72,
    Children: 73,
    OpenInNewTab: 74,
    TextStyle: 75,
    Region: 76,
    AlignContent: 77,
    Display: 78,
    Checked: 79,
    Enabled: 80,
    TextInputType: 81,
    Placeholder: 82,
    Multiline: 83,
    DefaultTextInputValue: 84,
    Loading: 85,
    Src: 86,
    YoutubeSrc: 87,
    Code: 88,
    ImageSrc: 89,
    Alt: 90,
    DocumentProperties: {
        MetaTitle: 91,
        MetaOGTitle: 92,
        MetaTwitterTitle: 93,
        MetaDescription: 94,
        MetaOGDescription: 95,
        MetaTwitterDescription: 96,
        MetaOGImage: 97,
        MetaTwitterImage: 98,
        MetaThemeColor: 99,
        MetaFacebookDomainVerification: 123,
    },
    Shadow: 100,
    CodeTheme: 101,
    CodeLanguage: 102,
    CodeShowLineNumber: 103,
    Css: 104,
    Js: 105,
    LinkRel: 106,
    InputMaxLength: 107,
    Favicon: 108,
    Fit: 109,
    VideoSrc: 110,
    Autoplay: 111,
    Poster: 112,
    LoopVideo: 113,
    Controls: 114,
    Muted: 115,
    LinkColor: 116,
    TextShadow: 117,
    Selectable: 118,
    BackdropFilter: 119,
    Mask: 120,
    TextInputValue: 121,
    FetchPriority: 122,
};

fastn_dom.Loading = {
    Lazy: "lazy",
    Eager: "eager",
};

fastn_dom.LinkRel = {
    NoFollow: "nofollow",
    Sponsored: "sponsored",
    Ugc: "ugc",
};

fastn_dom.TextInputType = {
    Text: "text",
    Email: "email",
    Password: "password",
    Url: "url",
    DateTime: "datetime",
    Date: "date",
    Time: "time",
    Month: "month",
    Week: "week",
    Color: "color",
    File: "file",
};

fastn_dom.AlignContent = {
    TopLeft: "top-left",
    TopCenter: "top-center",
    TopRight: "top-right",
    Right: "right",
    Left: "left",
    Center: "center",
    BottomLeft: "bottom-left",
    BottomRight: "bottom-right",
    BottomCenter: "bottom-center",
};

fastn_dom.Region = {
    H1: "h1",
    H2: "h2",
    H3: "h3",
    H4: "h4",
    H5: "h5",
    H6: "h6",
};

fastn_dom.Anchor = {
    Window: [1, "fixed"],
    Parent: [2, "absolute"],
    Id: (value) => {
        return [3, value];
    },
};

fastn_dom.DeviceData = {
    Desktop: "desktop",
    Mobile: "mobile",
};

fastn_dom.TextStyle = {
    Underline: "underline",
    Italic: "italic",
    Strike: "line-through",
    Heavy: "900",
    Extrabold: "800",
    Bold: "700",
    SemiBold: "600",
    Medium: "500",
    Regular: "400",
    Light: "300",
    ExtraLight: "200",
    Hairline: "100",
};

fastn_dom.Resizing = {
    FillContainer: "100%",
    HugContent: "fit-content",
    Auto: "auto",
    Fixed: (value) => {
        return value;
    },
};

fastn_dom.Spacing = {
    SpaceEvenly: [1, "space-evenly"],
    SpaceBetween: [2, "space-between"],
    SpaceAround: [3, "space-around"],
    Fixed: (value) => {
        return [4, value];
    },
};

fastn_dom.BorderStyle = {
    Solid: "solid",
    Dashed: "dashed",
    Dotted: "dotted",
    Double: "double",
    Ridge: "ridge",
    Groove: "groove",
    Inset: "inset",
    Outset: "outset",
};

fastn_dom.Fit = {
    none: "none",
    fill: "fill",
    contain: "contain",
    cover: "cover",
    scaleDown: "scale-down",
};

fastn_dom.FetchPriority = {
    auto: "auto",
    high: "high",
    low: "low",
};

fastn_dom.Overflow = {
    Scroll: "scroll",
    Visible: "visible",
    Hidden: "hidden",
    Auto: "auto",
};

fastn_dom.Display = {
    Block: "block",
    Inline: "inline",
    InlineBlock: "inline-block",
};

fastn_dom.AlignSelf = {
    Start: "start",
    Center: "center",
    End: "end",
};

fastn_dom.TextTransform = {
    None: "none",
    Capitalize: "capitalize",
    Uppercase: "uppercase",
    Lowercase: "lowercase",
    Inherit: "inherit",
    Initial: "initial",
};

fastn_dom.TextAlign = {
    Start: "start",
    Center: "center",
    End: "end",
    Justify: "justify",
};

fastn_dom.Cursor = {
    None: "none",
    Default: "default",
    ContextMenu: "context-menu",
    Help: "help",
    Pointer: "pointer",
    Progress: "progress",
    Wait: "wait",
    Cell: "cell",
    CrossHair: "crosshair",
    Text: "text",
    VerticalText: "vertical-text",
    Alias: "alias",
    Copy: "copy",
    Move: "move",
    NoDrop: "no-drop",
    NotAllowed: "not-allowed",
    Grab: "grab",
    Grabbing: "grabbing",
    EResize: "e-resize",
    NResize: "n-resize",
    NeResize: "ne-resize",
    SResize: "s-resize",
    SeResize: "se-resize",
    SwResize: "sw-resize",
    Wresize: "w-resize",
    Ewresize: "ew-resize",
    NsResize: "ns-resize",
    NeswResize: "nesw-resize",
    NwseResize: "nwse-resize",
    ColResize: "col-resize",
    RowResize: "row-resize",
    AllScroll: "all-scroll",
    ZoomIn: "zoom-in",
    ZoomOut: "zoom-out",
};

fastn_dom.Resize = {
    Vertical: "vertical",
    Horizontal: "horizontal",
    Both: "both",
};

fastn_dom.WhiteSpace = {
    Normal: "normal",
    NoWrap: "nowrap",
    Pre: "pre",
    PreLine: "pre-line",
    PreWrap: "pre-wrap",
    BreakSpaces: "break-spaces",
};

fastn_dom.BackdropFilter = {
    Blur: (value) => {
        return [1, value];
    },
    Brightness: (value) => {
        return [2, value];
    },
    Contrast: (value) => {
        return [3, value];
    },
    Grayscale: (value) => {
        return [4, value];
    },
    Invert: (value) => {
        return [5, value];
    },
    Opacity: (value) => {
        return [6, value];
    },
    Sepia: (value) => {
        return [7, value];
    },
    Saturate: (value) => {
        return [8, value];
    },
    Multi: (value) => {
        return [9, value];
    },
};

fastn_dom.BackgroundStyle = {
    Solid: (value) => {
        return [1, value];
    },
    Image: (value) => {
        return [2, value];
    },
    LinearGradient: (value) => {
        return [3, value];
    },
};

fastn_dom.BackgroundRepeat = {
    Repeat: "repeat",
    RepeatX: "repeat-x",
    RepeatY: "repeat-y",
    NoRepeat: "no-repeat",
    Space: "space",
    Round: "round",
};

fastn_dom.BackgroundSize = {
    Auto: "auto",
    Cover: "cover",
    Contain: "contain",
    Length: (value) => {
        return value;
    },
};

fastn_dom.BackgroundPosition = {
    Left: "left",
    Right: "right",
    Center: "center",
    LeftTop: "left top",
    LeftCenter: "left center",
    LeftBottom: "left bottom",
    CenterTop: "center top",
    CenterCenter: "center center",
    CenterBottom: "center bottom",
    RightTop: "right top",
    RightCenter: "right center",
    RightBottom: "right bottom",
    Length: (value) => {
        return value;
    },
};

fastn_dom.LinearGradientDirection = {
    Angle: (value) => {
        return `${value}deg`;
    },
    Turn: (value) => {
        return `${value}turn`;
    },
    Left: "270deg",
    Right: "90deg",
    Top: "0deg",
    Bottom: "180deg",
    TopLeft: "315deg",
    TopRight: "45deg",
    BottomLeft: "225deg",
    BottomRight: "135deg",
};

fastn_dom.FontSize = {
    Px: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () {
                return `${value.get()}px`;
            });
        }
        return `${value}px`;
    },
    Em: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () {
                return `${value.get()}em`;
            });
        }
        return `${value}em`;
    },
    Rem: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () {
                return `${value.get()}rem`;
            });
        }
        return `${value}rem`;
    },
};

fastn_dom.Length = {
    Px: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () {
                return `${fastn_utils.getStaticValue(value)}px`;
            });
        }
        return `${value}px`;
    },
    Em: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () {
                return `${fastn_utils.getStaticValue(value)}em`;
            });
        }
        return `${value}em`;
    },
    Rem: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () {
                return `${fastn_utils.getStaticValue(value)}rem`;
            });
        }
        return `${value}rem`;
    },
    Percent: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () {
                return `${fastn_utils.getStaticValue(value)}%`;
            });
        }
        return `${value}%`;
    },
    Calc: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () {
                return `calc(${fastn_utils.getStaticValue(value)})`;
            });
        }
        return `calc(${value})`;
    },
    Vh: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () {
                return `${fastn_utils.getStaticValue(value)}vh`;
            });
        }
        return `${value}vh`;
    },
    Vw: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () {
                return `${fastn_utils.getStaticValue(value)}vw`;
            });
        }
        return `${value}vw`;
    },
    Dvh: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () {
                return `${fastn_utils.getStaticValue(value)}dvh`;
            });
        }
        return `${value}dvh`;
    },
    Lvh: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () {
                return `${fastn_utils.getStaticValue(value)}lvh`;
            });
        }
        return `${value}lvh`;
    },
    Svh: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () {
                return `${fastn_utils.getStaticValue(value)}svh`;
            });
        }
        return `${value}svh`;
    },

    Vmin: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () {
                return `${fastn_utils.getStaticValue(value)}vmin`;
            });
        }
        return `${value}vmin`;
    },
    Vmax: (value) => {
        if (value instanceof fastn.mutableClass) {
            return fastn.formula([value], function () {
                return `${fastn_utils.getStaticValue(value)}vmax`;
            });
        }
        return `${value}vmax`;
    },
    Responsive: (length) => {
        return new PropertyValueAsClosure(() => {
            if (ftd.device.get() === "desktop") {
                return length.get("desktop");
            } else {
                let mobile = length.get("mobile");
                let desktop = length.get("desktop");
                return mobile ? mobile : desktop;
            }
        }, [ftd.device, length]);
    },
};

fastn_dom.Mask = {
    Image: (value) => {
        return [1, value];
    },
    Multi: (value) => {
        return [2, value];
    },
};

fastn_dom.MaskSize = {
    Auto: "auto",
    Cover: "cover",
    Contain: "contain",
    Fixed: (value) => {
        return value;
    },
};

fastn_dom.MaskRepeat = {
    Repeat: "repeat",
    RepeatX: "repeat-x",
    RepeatY: "repeat-y",
    NoRepeat: "no-repeat",
    Space: "space",
    Round: "round",
};

fastn_dom.MaskPosition = {
    Left: "left",
    Right: "right",
    Center: "center",
    LeftTop: "left top",
    LeftCenter: "left center",
    LeftBottom: "left bottom",
    CenterTop: "center top",
    CenterCenter: "center center",
    CenterBottom: "center bottom",
    RightTop: "right top",
    RightCenter: "right center",
    RightBottom: "right bottom",
    Length: (value) => {
        return value;
    },
};

fastn_dom.Event = {
    Click: 0,
    MouseEnter: 1,
    MouseLeave: 2,
    ClickOutside: 3,
    GlobalKey: (val) => {
        return [4, val];
    },
    GlobalKeySeq: (val) => {
        return [5, val];
    },
    Input: 6,
    Change: 7,
    Blur: 8,
    Focus: 9,
};

class PropertyValueAsClosure {
    closureFunction;
    deps;
    constructor(closureFunction, deps) {
        this.closureFunction = closureFunction;
        this.deps = deps;
    }
}

// Node2 -> Intermediate node
// Node -> similar to HTML DOM node (Node2.#node)
class Node2 {
    #node;
    #kind;
    #parent;
    #tagName;
    #rawInnerValue;
    /**
     * This is where we store all the attached closures, so we can free them
     * when we are done.
     */
    #mutables;
    /**
     * This is where we store the extraData related to node. This is
     * especially useful to store data for integrated external library (like
     * rive).
     */
    #extraData;
    #children;
    constructor(parentOrSibiling, kind) {
        this.#kind = kind;
        this.#parent = parentOrSibiling;
        this.#children = [];
        this.#rawInnerValue = null;

        let sibiling = undefined;

        if (parentOrSibiling instanceof ParentNodeWithSibiling) {
            this.#parent = parentOrSibiling.getParent();
            while (this.#parent instanceof ParentNodeWithSibiling) {
                this.#parent = this.#parent.getParent();
            }
            sibiling = parentOrSibiling.getSibiling();
        }

        this.createNode(kind);

        this.#mutables = [];
        this.#extraData = {};
        /*if (!!parent.parent) {
            parent = parent.parent();
        }*/

        if (this.#parent.getNode) {
            this.#parent = this.#parent.getNode();
        }

        if (fastn_utils.isWrapperNode(this.#tagName)) {
            this.#parent = parentOrSibiling;
            return;
        }
        if (sibiling) {
            this.#parent.insertBefore(
                this.#node,
                fastn_utils.nextSibling(sibiling, this.#parent),
            );
        } else {
            this.#parent.appendChild(this.#node);
        }
    }
    createNode(kind) {
        if (kind === fastn_dom.ElementKind.Code) {
            let [node, classes, attributes] = fastn_utils.htmlNode(kind);
            [this.#tagName, this.#node] = fastn_utils.createNodeHelper(
                node,
                classes,
                attributes,
            );
            let codeNode = new Node2(
                this.#node,
                fastn_dom.ElementKind.CodeChild,
            );
            this.#children.push(codeNode);
        } else {
            let [node, classes, attributes] = fastn_utils.htmlNode(kind);
            [this.#tagName, this.#node] = fastn_utils.createNodeHelper(
                node,
                classes,
                attributes,
            );
        }
    }
    getTagName() {
        return this.#tagName;
    }
    getParent() {
        return this.#parent;
    }
    removeAllFaviconLinks() {
        if (doubleBuffering) {
            const links = document.head.querySelectorAll(
                'link[rel="shortcut icon"]',
            );
            links.forEach((link) => {
                link.parentNode.removeChild(link);
            });
        }
    }
    setFavicon(url) {
        if (doubleBuffering) {
            if (url instanceof fastn.recordInstanceClass) url = url.get("src");
            while (true) {
                if (url instanceof fastn.mutableClass) url = url.get();
                else break;
            }

            let link_element = document.createElement("link");
            link_element.rel = "shortcut icon";
            link_element.href = url;

            this.removeAllFaviconLinks();
            document.head.appendChild(link_element);
        }
    }
    updateTextInputValue() {
        if (fastn_utils.isNull(this.#rawInnerValue)) {
            this.attachAttribute("value");
            return;
        }
        if (!ssr && this.#node.tagName.toLowerCase() === "textarea") {
            this.#node.innerHTML = this.#rawInnerValue;
        } else {
            this.attachAttribute("value", this.#rawInnerValue);
        }
    }
    // for attaching inline attributes
    attachAttribute(property, value) {
        // If the value is null, undefined, or false, the attribute will be removed.
        // For example, if attributes like checked, muted, or autoplay have been assigned a "false" value.
        if (fastn_utils.isNull(value)) {
            this.#node.removeAttribute(property);
            return;
        }
        this.#node.setAttribute(property, value);
    }
    removeAttribute(property) {
        this.#node.removeAttribute(property);
    }
    updateTagName(name) {
        if (ssr) {
            this.#node.updateTagName(name);
        } else {
            let newElement = document.createElement(name);
            newElement.innerHTML = this.#node.innerHTML;
            newElement.className = this.#node.className;
            newElement.style = this.#node.style;
            for (var i = 0; i < this.#node.attributes.length; i++) {
                var attr = this.#node.attributes[i];
                newElement.setAttribute(attr.name, attr.value);
            }
            var eventListeners = fastn_utils.getEventListeners(this.#node);
            for (var eventType in eventListeners) {
                newElement[eventType] = eventListeners[eventType];
            }
            this.#parent.replaceChild(newElement, this.#node);
            this.#node = newElement;
        }
    }
    updateToAnchor(url) {
        let node_kind = this.#kind;
        if (ssr) {
            if (node_kind !== fastn_dom.ElementKind.Image) {
                this.updateTagName("a");
                this.attachAttribute("href", url);
            }
            return;
        }
        if (node_kind === fastn_dom.ElementKind.Image) {
            let anchorElement = document.createElement("a");
            anchorElement.href = url;
            anchorElement.appendChild(this.#node);
            this.#parent.appendChild(anchorElement);
            this.#node = anchorElement;
        } else {
            this.updateTagName("a");
            this.#node.href = url;
        }
    }
    updatePositionForNodeById(node_id, value) {
        if (!ssr) {
            const target_node = fastnVirtual.root.querySelector(
                `[id="${node_id}"]`,
            );
            if (!fastn_utils.isNull(target_node))
                target_node.style["position"] = value;
        }
    }
    updateParentPosition(value) {
        if (ssr) {
            let parent = this.#parent;
            if (parent.style) parent.style["position"] = value;
        }
        if (!ssr) {
            let current_node = this.#node;
            if (current_node) {
                let parent_node = current_node.parentNode;
                parent_node.style["position"] = value;
            }
        }
    }
    updateMetaTitle(value) {
        if (!ssr && doubleBuffering) {
            if (!fastn_utils.isNull(value)) window.document.title = value;
        }
    }
    addMetaTagByName(name, value) {
        if (value === null || value === undefined) {
            this.removeMetaTagByName(name);
            return;
        }
        if (!ssr && doubleBuffering) {
            const metaTag = window.document.createElement("meta");
            metaTag.setAttribute("name", name);
            metaTag.setAttribute("content", value);
            document.head.appendChild(metaTag);
        }
    }
    addMetaTagByProperty(property, value) {
        if (value === null || value === undefined) {
            this.removeMetaTagByProperty(property);
            return;
        }
        if (!ssr && doubleBuffering) {
            const metaTag = window.document.createElement("meta");
            metaTag.setAttribute("property", property);
            metaTag.setAttribute("content", value);
            document.head.appendChild(metaTag);
        }
    }
    removeMetaTagByName(name) {
        if (!ssr && doubleBuffering) {
            const metaTags = document.getElementsByTagName("meta");
            for (let i = 0; i < metaTags.length; i++) {
                const metaTag = metaTags[i];
                if (metaTag.getAttribute("name") === name) {
                    metaTag.remove();
                    break;
                }
            }
        }
    }
    removeMetaTagByProperty(property) {
        if (!ssr && doubleBuffering) {
            const metaTags = document.getElementsByTagName("meta");
            for (let i = 0; i < metaTags.length; i++) {
                const metaTag = metaTags[i];
                if (metaTag.getAttribute("property") === property) {
                    metaTag.remove();
                    break;
                }
            }
        }
    }
    // dynamic-class-css
    attachCss(property, value, createClass, className) {
        let propertyShort = fastn_dom.propertyMap[property] || property;
        propertyShort = `__${propertyShort}`;
        let cls = `${propertyShort}-${fastn_dom.class_count}`;
        if (!!className) {
            cls = className;
        } else {
            if (!fastn_dom.unsanitised_classes[cls]) {
                fastn_dom.unsanitised_classes[cls] = ++fastn_dom.class_count;
            }
            cls = `${propertyShort}-${fastn_dom.unsanitised_classes[cls]}`;
        }
        let cssClass = className ? cls : `.${cls}`;

        const obj = { property, value };

        if (value === undefined) {
            if (!ssr) {
                for (const className of this.#node.classList.values()) {
                    if (className.startsWith(`${propertyShort}-`)) {
                        this.#node.classList.remove(className);
                    }
                }
                this.#node.style[property] = null;
            }
            return cls;
        }

        if (!ssr && !doubleBuffering) {
            if (!!className) {
                if (!fastn_dom.classes[cssClass]) {
                    fastn_dom.classes[cssClass] =
                        fastn_dom.classes[cssClass] || obj;
                    fastn_utils.createStyle(cssClass, obj);
                }
                return cls;
            }

            for (const className of this.#node.classList.values()) {
                if (className.startsWith(`${propertyShort}-`)) {
                    this.#node.classList.remove(className);
                }
            }

            if (createClass) {
                if (!fastn_dom.classes[cssClass]) {
                    fastn_dom.classes[cssClass] =
                        fastn_dom.classes[cssClass] || obj;
                    fastn_utils.createStyle(cssClass, obj);
                }
                this.#node.style.removeProperty(property);
                this.#node.classList.add(cls);
            } else if (!fastn_dom.classes[cssClass]) {
                if (typeof value === "object" && value !== null) {
                    for (let key in value) {
                        this.#node.style[key] = value[key];
                    }
                } else {
                    this.#node.style[property] = value;
                }
            } else {
                this.#node.style.removeProperty(property);
                this.#node.classList.add(cls);
            }

            return cls;
        }

        fastn_dom.classes[cssClass] = fastn_dom.classes[cssClass] || obj;

        if (!!className) {
            return cls;
        }

        this.#node.classList.add(cls);
        return cls;
    }
    attachShadow(value) {
        if (fastn_utils.isNull(value)) {
            this.attachCss("box-shadow", value);
            return;
        }

        const color = value.get("color");

        const lightColor = fastn_utils.getStaticValue(color.get("light"));
        const darkColor = fastn_utils.getStaticValue(color.get("dark"));

        const blur = fastn_utils.getStaticValue(value.get("blur"));
        const xOffset = fastn_utils.getStaticValue(value.get("x_offset"));
        const yOffset = fastn_utils.getStaticValue(value.get("y_offset"));
        const spread = fastn_utils.getStaticValue(value.get("spread"));
        const inset = fastn_utils.getStaticValue(value.get("inset"));

        const shadowCommonCss = `${
            inset ? "inset " : ""
        }${xOffset} ${yOffset} ${blur} ${spread}`;
        const lightShadowCss = `${shadowCommonCss} ${lightColor}`;
        const darkShadowCss = `${shadowCommonCss} ${darkColor}`;

        if (lightShadowCss === darkShadowCss) {
            this.attachCss("box-shadow", lightShadowCss, false);
        } else {
            let lightClass = this.attachCss("box-shadow", lightShadowCss, true);
            this.attachCss(
                "box-shadow",
                darkShadowCss,
                true,
                `body.dark .${lightClass}`,
            );
        }
    }
    attachBackdropMultiFilter(value) {
        const filters = {
            blur: fastn_utils.getStaticValue(value.get("blur")),
            brightness: fastn_utils.getStaticValue(value.get("brightness")),
            contrast: fastn_utils.getStaticValue(value.get("contrast")),
            grayscale: fastn_utils.getStaticValue(value.get("grayscale")),
            invert: fastn_utils.getStaticValue(value.get("invert")),
            opacity: fastn_utils.getStaticValue(value.get("opacity")),
            sepia: fastn_utils.getStaticValue(value.get("sepia")),
            saturate: fastn_utils.getStaticValue(value.get("saturate")),
        };

        const filterString = Object.entries(filters)
            .filter(([_, value]) => !fastn_utils.isNull(value))
            .map(([name, value]) => `${name}(${value})`)
            .join(" ");

        this.attachCss("backdrop-filter", filterString, false);
    }
    attachTextShadow(value) {
        if (fastn_utils.isNull(value)) {
            this.attachCss("text-shadow", value);
            return;
        }

        const color = value.get("color");

        const lightColor = fastn_utils.getStaticValue(color.get("light"));
        const darkColor = fastn_utils.getStaticValue(color.get("dark"));

        const blur = fastn_utils.getStaticValue(value.get("blur"));
        const xOffset = fastn_utils.getStaticValue(value.get("x_offset"));
        const yOffset = fastn_utils.getStaticValue(value.get("y_offset"));

        const shadowCommonCss = `${xOffset} ${yOffset} ${blur}`;
        const lightShadowCss = `${shadowCommonCss} ${lightColor}`;
        const darkShadowCss = `${shadowCommonCss} ${darkColor}`;

        if (lightShadowCss === darkShadowCss) {
            this.attachCss("text-shadow", lightShadowCss, false);
        } else {
            let lightClass = this.attachCss("box-shadow", lightShadowCss, true);
            this.attachCss(
                "text-shadow",
                darkShadowCss,
                true,
                `body.dark .${lightClass}`,
            );
        }
    }
    getLinearGradientString(value) {
        var lightGradientString = "";
        var darkGradientString = "";

        let colorsList = value.get("colors").get().getList();
        colorsList.map(function (element) {
            // LinearGradient RecordInstance
            let lg_color = element.item;

            let color = lg_color.get("color").get();
            let lightColor = fastn_utils.getStaticValue(color.get("light"));
            let darkColor = fastn_utils.getStaticValue(color.get("dark"));

            lightGradientString = `${lightGradientString} ${lightColor}`;
            darkGradientString = `${darkGradientString} ${darkColor}`;

            let start = fastn_utils.getStaticValue(lg_color.get("start"));
            if (start !== undefined && start !== null) {
                lightGradientString = `${lightGradientString} ${start}`;
                darkGradientString = `${darkGradientString} ${start}`;
            }

            let end = fastn_utils.getStaticValue(lg_color.get("end"));
            if (end !== undefined && end !== null) {
                lightGradientString = `${lightGradientString} ${end}`;
                darkGradientString = `${darkGradientString} ${end}`;
            }

            let stop_position = fastn_utils.getStaticValue(
                lg_color.get("stop_position"),
            );
            if (stop_position !== undefined && stop_position !== null) {
                lightGradientString = `${lightGradientString}, ${stop_position}`;
                darkGradientString = `${darkGradientString}, ${stop_position}`;
            }

            lightGradientString = `${lightGradientString},`;
            darkGradientString = `${darkGradientString},`;
        });

        lightGradientString = lightGradientString.trim().slice(0, -1);
        darkGradientString = darkGradientString.trim().slice(0, -1);

        return [lightGradientString, darkGradientString];
    }
    attachLinearGradientCss(value) {
        if (fastn_utils.isNull(value)) {
            this.attachCss("background-image", value);
            return;
        }

        const closure = fastn
            .closure(() => {
                let direction = fastn_utils.getStaticValue(
                    value.get("direction"),
                );

                const [lightGradientString, darkGradientString] =
                    this.getLinearGradientString(value);

                if (lightGradientString === darkGradientString) {
                    this.attachCss(
                        "background-image",
                        `linear-gradient(${direction}, ${lightGradientString})`,
                        false,
                    );
                } else {
                    let lightClass = this.attachCss(
                        "background-image",
                        `linear-gradient(${direction}, ${lightGradientString})`,
                        true,
                    );
                    this.attachCss(
                        "background-image",
                        `linear-gradient(${direction}, ${darkGradientString})`,
                        true,
                        `body.dark .${lightClass}`,
                    );
                }
            })
            .addNodeProperty(this, null, inherited);

        const colorsList = value.get("colors").get().getList();

        colorsList.forEach(({ item }) => {
            const color = item.get("color");

            [color.get("light"), color.get("dark")].forEach((variant) => {
                variant.addClosure(closure);
                this.#mutables.push(variant);
            });
        });
    }
    attachBackgroundImageCss(value) {
        if (fastn_utils.isNull(value)) {
            this.attachCss("background-repeat", value);
            this.attachCss("background-position", value);
            this.attachCss("background-size", value);
            this.attachCss("background-image", value);
            return;
        }

        let src = fastn_utils.getStaticValue(value.get("src"));
        let lightValue = fastn_utils.getStaticValue(src.get("light"));
        let darkValue = fastn_utils.getStaticValue(src.get("dark"));

        let position = fastn_utils.getStaticValue(value.get("position"));
        let positionX = null;
        let positionY = null;
        if (position !== null && position instanceof Object) {
            positionX = fastn_utils.getStaticValue(position.get("x"));
            positionY = fastn_utils.getStaticValue(position.get("y"));

            if (positionX !== null) position = `${positionX}`;
            if (positionY !== null) {
                if (positionX === null) position = `0px ${positionY}`;
                else position = `${position} ${positionY}`;
            }
        }
        let repeat = fastn_utils.getStaticValue(value.get("repeat"));
        let size = fastn_utils.getStaticValue(value.get("size"));
        let sizeX = null;
        let sizeY = null;
        if (size !== null && size instanceof Object) {
            sizeX = fastn_utils.getStaticValue(size.get("x"));
            sizeY = fastn_utils.getStaticValue(size.get("y"));

            if (sizeX !== null) size = `${sizeX}`;
            if (sizeY !== null) {
                if (sizeX === null) size = `0px ${sizeY}`;
                else size = `${size} ${sizeY}`;
            }
        }

        if (repeat !== null) this.attachCss("background-repeat", repeat);
        if (position !== null) this.attachCss("background-position", position);
        if (size !== null) this.attachCss("background-size", size);

        if (lightValue === darkValue) {
            this.attachCss("background-image", `url(${lightValue})`, false);
        } else {
            let lightClass = this.attachCss(
                "background-image",
                `url(${lightValue})`,
                true,
            );
            this.attachCss(
                "background-image",
                `url(${darkValue})`,
                true,
                `body.dark .${lightClass}`,
            );
        }
    }
    attachMaskImageCss(value, vendorPrefix) {
        const propertyWithPrefix = vendorPrefix
            ? `${vendorPrefix}-mask-image`
            : "mask-image";

        if (fastn_utils.isNull(value)) {
            this.attachCss(propertyWithPrefix, value);
            return;
        }

        let src = fastn_utils.getStaticValue(value.get("src"));
        let linearGradient = fastn_utils.getStaticValue(
            value.get("linear_gradient"),
        );
        let color = fastn_utils.getStaticValue(value.get("color"));

        const maskLightImageValues = [];
        const maskDarkImageValues = [];

        if (!fastn_utils.isNull(src)) {
            let lightValue = fastn_utils.getStaticValue(src.get("light"));
            let darkValue = fastn_utils.getStaticValue(src.get("dark"));

            const lightUrl = `url(${lightValue})`;
            const darkUrl = `url(${darkValue})`;

            if (!fastn_utils.isNull(linearGradient)) {
                const lightImageValues = [lightUrl];
                const darkImageValues = [darkUrl];

                if (!fastn_utils.isNull(color)) {
                    const lightColor = fastn_utils.getStaticValue(
                        color.get("light"),
                    );
                    const darkColor = fastn_utils.getStaticValue(
                        color.get("dark"),
                    );

                    lightImageValues.push(lightColor);
                    darkImageValues.push(darkColor);
                }
                maskLightImageValues.push(
                    `image(${lightImageValues.join(", ")})`,
                );
                maskDarkImageValues.push(
                    `image(${darkImageValues.join(", ")})`,
                );
            } else {
                maskLightImageValues.push(lightUrl);
                maskDarkImageValues.push(darkUrl);
            }
        }

        if (!fastn_utils.isNull(linearGradient)) {
            let direction = fastn_utils.getStaticValue(
                linearGradient.get("direction"),
            );

            const [lightGradientString, darkGradientString] =
                this.getLinearGradientString(linearGradient);

            maskLightImageValues.push(
                `linear-gradient(${direction}, ${lightGradientString})`,
            );
            maskDarkImageValues.push(
                `linear-gradient(${direction}, ${darkGradientString})`,
            );
        }

        const maskLightImageString = maskLightImageValues.join(", ");
        const maskDarkImageString = maskDarkImageValues.join(", ");

        if (maskLightImageString === maskDarkImageString) {
            this.attachCss(propertyWithPrefix, maskLightImageString, true);
        } else {
            let lightClass = this.attachCss(
                propertyWithPrefix,
                maskLightImageString,
                true,
            );
            this.attachCss(
                propertyWithPrefix,
                maskDarkImageString,
                true,
                `body.dark .${lightClass}`,
            );
        }
    }
    attachMaskSizeCss(value, vendorPrefix) {
        const propertyNameWithPrefix = vendorPrefix
            ? `${vendorPrefix}-mask-size`
            : "mask-size";
        if (fastn_utils.isNull(value)) {
            this.attachCss(propertyNameWithPrefix, value);
        }
        const [size, ...two_values] = ["size", "size_x", "size_y"].map((size) =>
            fastn_utils.getStaticValue(value.get(size)),
        );

        if (!fastn_utils.isNull(size)) {
            this.attachCss(propertyNameWithPrefix, size, true);
        } else {
            const [size_x, size_y] = two_values.map((value) => value || "auto");
            this.attachCss(propertyNameWithPrefix, `${size_x} ${size_y}`, true);
        }
    }
    attachMaskMultiCss(value, vendorPrefix) {
        if (fastn_utils.isNull(value)) {
            this.attachCss("mask-repeat", value);
            this.attachCss("mask-position", value);
            this.attachCss("mask-size", value);
            this.attachCss("mask-image", value);
            return;
        }

        const maskImage = fastn_utils.getStaticValue(value.get("image"));
        this.attachMaskImageCss(maskImage);
        this.attachMaskImageCss(maskImage, vendorPrefix);
        this.attachMaskSizeCss(value);
        this.attachMaskSizeCss(value, vendorPrefix);
        const maskRepeatValue = fastn_utils.getStaticValue(value.get("repeat"));
        if (fastn_utils.isNull(maskRepeatValue)) {
            this.attachCss("mask-repeat", maskRepeatValue, true);
            this.attachCss("-webkit-mask-repeat", maskRepeatValue, true);
        } else {
            this.attachCss("mask-repeat", maskRepeatValue, true);
            this.attachCss("-webkit-mask-repeat", maskRepeatValue, true);
        }
        const maskPositionValue = fastn_utils.getStaticValue(
            value.get("position"),
        );
        if (fastn_utils.isNull(maskPositionValue)) {
            this.attachCss("mask-position", maskPositionValue, true);
            this.attachCss("-webkit-mask-position", maskPositionValue, true);
        } else {
            this.attachCss("mask-position", maskPositionValue, true);
            this.attachCss("-webkit-mask-position", maskPositionValue, true);
        }
    }
    attachExternalCss(css) {
        if (!ssr) {
            let css_tag = document.createElement("link");
            css_tag.rel = "stylesheet";
            css_tag.type = "text/css";
            css_tag.href = css;

            let head =
                document.head || document.getElementsByTagName("head")[0];
            if (!fastn_dom.externalCss.has(css)) {
                head.appendChild(css_tag);
                fastn_dom.externalCss.add(css);
            }
        }
    }
    attachExternalJs(js) {
        if (!ssr) {
            let js_tag = document.createElement("script");
            js_tag.src = js;

            let head =
                document.head || document.getElementsByTagName("head")[0];
            if (!fastn_dom.externalJs.has(js)) {
                head.appendChild(js_tag);
                fastn_dom.externalCss.add(js);
            }
        }
    }
    attachColorCss(property, value, visited) {
        if (fastn_utils.isNull(value)) {
            this.attachCss(property, value);
            return;
        }
        value = value instanceof fastn.mutableClass ? value.get() : value;

        const lightValue = value.get("light");
        const darkValue = value.get("dark");

        const closure = fastn
            .closure(() => {
                let lightValueStatic = fastn_utils.getStaticValue(lightValue);
                let darkValueStatic = fastn_utils.getStaticValue(darkValue);

                if (lightValueStatic === darkValueStatic) {
                    this.attachCss(property, lightValueStatic, false);
                } else {
                    let lightClass = this.attachCss(
                        property,
                        lightValueStatic,
                        true,
                    );
                    this.attachCss(
                        property,
                        darkValueStatic,
                        true,
                        `body.dark .${lightClass}`,
                    );
                    if (visited) {
                        this.attachCss(
                            property,
                            lightValueStatic,
                            true,
                            `.${lightClass}:visited`,
                        );
                        this.attachCss(
                            property,
                            darkValueStatic,
                            true,
                            `body.dark  .${lightClass}:visited`,
                        );
                    }
                }
            })
            .addNodeProperty(this, null, inherited);

        [lightValue, darkValue].forEach((modeValue) => {
            modeValue.addClosure(closure);
            this.#mutables.push(modeValue);
        });
    }
    attachRoleCss(value) {
        if (fastn_utils.isNull(value)) {
            this.attachCss("role", value);
            return;
        }
        value.addClosure(
            fastn
                .closure(() => {
                    let desktopValue = value.get("desktop");
                    let mobileValue = value.get("mobile");
                    if (
                        fastn_utils.sameResponsiveRole(
                            desktopValue,
                            mobileValue,
                        )
                    ) {
                        this.attachCss(
                            "role",
                            fastn_utils.getRoleValues(desktopValue),
                            true,
                        );
                    } else {
                        let desktopClass = this.attachCss(
                            "role",
                            fastn_utils.getRoleValues(desktopValue),
                            true,
                        );
                        this.attachCss(
                            "role",
                            fastn_utils.getRoleValues(mobileValue),
                            true,
                            `body.mobile .${desktopClass}`,
                        );
                    }
                })
                .addNodeProperty(this, null, inherited),
        );
        this.#mutables.push(value);
    }
    attachTextStyles(styles) {
        if (fastn_utils.isNull(styles)) {
            this.attachCss("font-style", styles);
            this.attachCss("font-weight", styles);
            this.attachCss("text-decoration", styles);
            return;
        }
        for (var s of styles) {
            switch (s) {
                case "italic":
                    this.attachCss("font-style", s);
                    break;
                case "underline":
                case "line-through":
                    this.attachCss("text-decoration", s);
                    break;
                default:
                    this.attachCss("font-weight", s);
            }
        }
    }
    attachAlignContent(value, node_kind) {
        if (fastn_utils.isNull(value)) {
            this.attachCss("align-items", value);
            this.attachCss("justify-content", value);
            return;
        }
        if (node_kind === fastn_dom.ElementKind.Column) {
            switch (value) {
                case "top-left":
                    this.attachCss("justify-content", "start");
                    this.attachCss("align-items", "start");
                    break;
                case "top-center":
                    this.attachCss("justify-content", "start");
                    this.attachCss("align-items", "center");
                    break;
                case "top-right":
                    this.attachCss("justify-content", "start");
                    this.attachCss("align-items", "end");
                    break;
                case "left":
                    this.attachCss("justify-content", "center");
                    this.attachCss("align-items", "start");
                    break;
                case "center":
                    this.attachCss("justify-content", "center");
                    this.attachCss("align-items", "center");
                    break;
                case "right":
                    this.attachCss("justify-content", "center");
                    this.attachCss("align-items", "end");
                    break;
                case "bottom-left":
                    this.attachCss("justify-content", "end");
                    this.attachCss("align-items", "left");
                    break;
                case "bottom-center":
                    this.attachCss("justify-content", "end");
                    this.attachCss("align-items", "center");
                    break;
                case "bottom-right":
                    this.attachCss("justify-content", "end");
                    this.attachCss("align-items", "end");
                    break;
            }
        }

        if (node_kind === fastn_dom.ElementKind.Row) {
            switch (value) {
                case "top-left":
                    this.attachCss("justify-content", "start");
                    this.attachCss("align-items", "start");
                    break;
                case "top-center":
                    this.attachCss("justify-content", "center");
                    this.attachCss("align-items", "start");
                    break;
                case "top-right":
                    this.attachCss("justify-content", "end");
                    this.attachCss("align-items", "start");
                    break;
                case "left":
                    this.attachCss("justify-content", "start");
                    this.attachCss("align-items", "center");
                    break;
                case "center":
                    this.attachCss("justify-content", "center");
                    this.attachCss("align-items", "center");
                    break;
                case "right":
                    this.attachCss("justify-content", "right");
                    this.attachCss("align-items", "center");
                    break;
                case "bottom-left":
                    this.attachCss("justify-content", "start");
                    this.attachCss("align-items", "end");
                    break;
                case "bottom-center":
                    this.attachCss("justify-content", "center");
                    this.attachCss("align-items", "end");
                    break;
                case "bottom-right":
                    this.attachCss("justify-content", "end");
                    this.attachCss("align-items", "end");
                    break;
            }
        }
    }
    attachLinkColor(value) {
        ftd.dark_mode.addClosure(
            fastn
                .closure(() => {
                    if (!ssr) {
                        const anchors =
                            this.#node.tagName.toLowerCase() === "a"
                                ? [this.#node]
                                : Array.from(this.#node.querySelectorAll("a"));
                        let propertyShort = `__${fastn_dom.propertyMap["link-color"]}`;

                        if (fastn_utils.isNull(value)) {
                            anchors.forEach((a) => {
                                a.classList.values().forEach((className) => {
                                    if (
                                        className.startsWith(
                                            `${propertyShort}-`,
                                        )
                                    ) {
                                        a.classList.remove(className);
                                    }
                                });
                            });
                        } else {
                            const lightValue = fastn_utils.getStaticValue(
                                value.get("light"),
                            );
                            const darkValue = fastn_utils.getStaticValue(
                                value.get("dark"),
                            );
                            let cls = `${propertyShort}-${JSON.stringify(
                                lightValue,
                            )}`;

                            if (!fastn_dom.unsanitised_classes[cls]) {
                                fastn_dom.unsanitised_classes[cls] =
                                    ++fastn_dom.class_count;
                            }

                            cls = `${propertyShort}-${fastn_dom.unsanitised_classes[cls]}`;

                            const cssClass = `.${cls}`;

                            if (!fastn_dom.classes[cssClass]) {
                                const obj = {
                                    property: "color",
                                    value: lightValue,
                                };
                                fastn_dom.classes[cssClass] =
                                    fastn_dom.classes[cssClass] || obj;
                                let styles = document.getElementById("styles");
                                styles.innerHTML = `${
                                    styles.innerHTML
                                }${getClassAsString(cssClass, obj)}\n`;
                            }

                            if (lightValue !== darkValue) {
                                const obj = {
                                    property: "color",
                                    value: darkValue,
                                };
                                let darkCls = `body.dark ${cssClass}`;
                                if (!fastn_dom.classes[darkCls]) {
                                    fastn_dom.classes[darkCls] =
                                        fastn_dom.classes[darkCls] || obj;
                                    let styles =
                                        document.getElementById("styles");
                                    styles.innerHTML = `${
                                        styles.innerHTML
                                    }${getClassAsString(darkCls, obj)}\n`;
                                }
                            }

                            anchors.forEach((a) => a.classList.add(cls));
                        }
                    }
                })
                .addNodeProperty(this, null, inherited),
        );
        this.#mutables.push(ftd.dark_mode);
    }
    setStaticProperty(kind, value, inherited) {
        // value can be either static or mutable
        let staticValue = fastn_utils.getStaticValue(value);
        if (kind === fastn_dom.PropertyKind.Children) {
            if (fastn_utils.isWrapperNode(this.#tagName)) {
                let parentWithSibiling = this.#parent;
                if (Array.isArray(staticValue)) {
                    staticValue.forEach((func, index) => {
                        if (index !== 0) {
                            parentWithSibiling = new ParentNodeWithSibiling(
                                this.#parent.getParent(),
                                this.#children[index - 1],
                            );
                        }
                        this.#children.push(
                            fastn_utils.getStaticValue(func.item)(
                                parentWithSibiling,
                                inherited,
                            ),
                        );
                    });
                } else {
                    this.#children.push(
                        staticValue(parentWithSibiling, inherited),
                    );
                }
            } else {
                if (Array.isArray(staticValue)) {
                    staticValue.forEach((func) =>
                        this.#children.push(
                            fastn_utils.getStaticValue(func.item)(
                                this,
                                inherited,
                            ),
                        ),
                    );
                } else {
                    this.#children.push(staticValue(this, inherited));
                }
            }
        } else if (kind === fastn_dom.PropertyKind.Id) {
            this.#node.id = staticValue;
        } else if (kind === fastn_dom.PropertyKind.BreakpointWidth) {
            if (fastn_utils.isNull(staticValue)) {
                return;
            }
            ftd.breakpoint_width.set(fastn_utils.getStaticValue(staticValue));
        } else if (kind === fastn_dom.PropertyKind.Css) {
            let css_list = staticValue.map((obj) =>
                fastn_utils.getStaticValue(obj.item),
            );
            css_list.forEach((css) => {
                this.attachExternalCss(css);
            });
        } else if (kind === fastn_dom.PropertyKind.Js) {
            let js_list = staticValue.map((obj) =>
                fastn_utils.getStaticValue(obj.item),
            );
            js_list.forEach((js) => {
                this.attachExternalJs(js);
            });
        } else if (kind === fastn_dom.PropertyKind.Width) {
            this.attachCss("width", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Height) {
            fastn_utils.resetFullHeight();
            this.attachCss("height", staticValue);
            fastn_utils.setFullHeight();
        } else if (kind === fastn_dom.PropertyKind.Padding) {
            this.attachCss("padding", staticValue);
        } else if (kind === fastn_dom.PropertyKind.PaddingHorizontal) {
            this.attachCss("padding-left", staticValue);
            this.attachCss("padding-right", staticValue);
        } else if (kind === fastn_dom.PropertyKind.PaddingVertical) {
            this.attachCss("padding-top", staticValue);
            this.attachCss("padding-bottom", staticValue);
        } else if (kind === fastn_dom.PropertyKind.PaddingLeft) {
            this.attachCss("padding-left", staticValue);
        } else if (kind === fastn_dom.PropertyKind.PaddingRight) {
            this.attachCss("padding-right", staticValue);
        } else if (kind === fastn_dom.PropertyKind.PaddingTop) {
            this.attachCss("padding-top", staticValue);
        } else if (kind === fastn_dom.PropertyKind.PaddingBottom) {
            this.attachCss("padding-bottom", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Margin) {
            this.attachCss("margin", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MarginHorizontal) {
            this.attachCss("margin-left", staticValue);
            this.attachCss("margin-right", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MarginVertical) {
            this.attachCss("margin-top", staticValue);
            this.attachCss("margin-bottom", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MarginLeft) {
            this.attachCss("margin-left", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MarginRight) {
            this.attachCss("margin-right", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MarginTop) {
            this.attachCss("margin-top", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MarginBottom) {
            this.attachCss("margin-bottom", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderWidth) {
            this.attachCss("border-width", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderTopWidth) {
            this.attachCss("border-top-width", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderBottomWidth) {
            this.attachCss("border-bottom-width", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderLeftWidth) {
            this.attachCss("border-left-width", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderRightWidth) {
            this.attachCss("border-right-width", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderRadius) {
            this.attachCss("border-radius", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderTopLeftRadius) {
            this.attachCss("border-top-left-radius", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderTopRightRadius) {
            this.attachCss("border-top-right-radius", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderBottomLeftRadius) {
            this.attachCss("border-bottom-left-radius", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderBottomRightRadius) {
            this.attachCss("border-bottom-right-radius", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderStyle) {
            this.attachCss("border-style", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderStyleVertical) {
            this.attachCss("border-top-style", staticValue);
            this.attachCss("border-bottom-style", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderStyleHorizontal) {
            this.attachCss("border-left-style", staticValue);
            this.attachCss("border-right-style", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderLeftStyle) {
            this.attachCss("border-left-style", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderRightStyle) {
            this.attachCss("border-right-style", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderTopStyle) {
            this.attachCss("border-top-style", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderBottomStyle) {
            this.attachCss("border-bottom-style", staticValue);
        } else if (kind === fastn_dom.PropertyKind.ZIndex) {
            this.attachCss("z-index", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Shadow) {
            this.attachShadow(staticValue);
        } else if (kind === fastn_dom.PropertyKind.TextShadow) {
            this.attachTextShadow(staticValue);
        } else if (kind === fastn_dom.PropertyKind.BackdropFilter) {
            if (fastn_utils.isNull(staticValue)) {
                this.attachCss("backdrop-filter", staticValue);
                return;
            }

            let backdropType = staticValue[0];
            switch (backdropType) {
                case 1:
                    this.attachCss(
                        "backdrop-filter",
                        `blur(${fastn_utils.getStaticValue(staticValue[1])})`,
                    );
                    break;
                case 2:
                    this.attachCss(
                        "backdrop-filter",
                        `brightness(${fastn_utils.getStaticValue(
                            staticValue[1],
                        )})`,
                    );
                    break;
                case 3:
                    this.attachCss(
                        "backdrop-filter",
                        `contrast(${fastn_utils.getStaticValue(
                            staticValue[1],
                        )})`,
                    );
                    break;
                case 4:
                    this.attachCss(
                        "backdrop-filter",
                        `greyscale(${fastn_utils.getStaticValue(
                            staticValue[1],
                        )})`,
                    );
                    break;
                case 5:
                    this.attachCss(
                        "backdrop-filter",
                        `invert(${fastn_utils.getStaticValue(staticValue[1])})`,
                    );
                    break;
                case 6:
                    this.attachCss(
                        "backdrop-filter",
                        `opacity(${fastn_utils.getStaticValue(
                            staticValue[1],
                        )})`,
                    );
                    break;
                case 7:
                    this.attachCss(
                        "backdrop-filter",
                        `sepia(${fastn_utils.getStaticValue(staticValue[1])})`,
                    );
                    break;
                case 8:
                    this.attachCss(
                        "backdrop-filter",
                        `saturate(${fastn_utils.getStaticValue(
                            staticValue[1],
                        )})`,
                    );
                    break;
                case 9:
                    this.attachBackdropMultiFilter(staticValue[1]);
                    break;
            }
        } else if (kind === fastn_dom.PropertyKind.Mask) {
            if (fastn_utils.isNull(staticValue)) {
                this.attachCss("mask-image", staticValue);
                return;
            }

            const [backgroundType, value] = staticValue;

            switch (backgroundType) {
                case fastn_dom.Mask.Image()[0]:
                    this.attachMaskImageCss(value);
                    this.attachMaskImageCss(value, "-webkit");
                    break;
                case fastn_dom.Mask.Multi()[0]:
                    this.attachMaskMultiCss(value);
                    this.attachMaskMultiCss(value, "-webkit");
                    break;
            }
        } else if (kind === fastn_dom.PropertyKind.Classes) {
            fastn_utils.removeNonFastnClasses(this);
            if (!fastn_utils.isNull(staticValue)) {
                let cls = staticValue.map((obj) =>
                    fastn_utils.getStaticValue(obj.item),
                );
                cls.forEach((c) => {
                    this.#node.classList.add(c);
                });
            }
        } else if (kind === fastn_dom.PropertyKind.Anchor) {
            // todo: this needs fixed for anchor.id = v
            // need to change position of element with id = v to relative
            if (fastn_utils.isNull(staticValue)) {
                this.attachCss("position", staticValue);
                return;
            }

            let anchorType = staticValue[0];
            switch (anchorType) {
                case 1:
                    this.attachCss("position", staticValue[1]);
                    break;
                case 2:
                    this.attachCss("position", staticValue[1]);
                    this.updateParentPosition("relative");
                    break;
                case 3:
                    const parent_node_id = staticValue[1];
                    this.attachCss("position", "absolute");
                    this.updatePositionForNodeById(parent_node_id, "relative");
                    break;
            }
        } else if (kind === fastn_dom.PropertyKind.Sticky) {
            // sticky is boolean type
            switch (staticValue) {
                case "true":
                case true:
                    this.attachCss("position", "sticky");
                    break;
                case "false":
                case false:
                    this.attachCss("position", "static");
                    break;
                default:
                    this.attachCss("position", staticValue);
            }
        } else if (kind === fastn_dom.PropertyKind.Top) {
            this.attachCss("top", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Bottom) {
            this.attachCss("bottom", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Left) {
            this.attachCss("left", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Right) {
            this.attachCss("right", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Overflow) {
            this.attachCss("overflow", staticValue);
        } else if (kind === fastn_dom.PropertyKind.OverflowX) {
            this.attachCss("overflow-x", staticValue);
        } else if (kind === fastn_dom.PropertyKind.OverflowY) {
            this.attachCss("overflow-y", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Spacing) {
            if (fastn_utils.isNull(staticValue)) {
                this.attachCss("justify-content", staticValue);
                this.attachCss("gap", staticValue);
                return;
            }

            let spacingType = staticValue[0];
            switch (spacingType) {
                case fastn_dom.Spacing.SpaceEvenly[0]:
                case fastn_dom.Spacing.SpaceBetween[0]:
                case fastn_dom.Spacing.SpaceAround[0]:
                    this.attachCss("justify-content", staticValue[1]);
                    break;
                case fastn_dom.Spacing.Fixed()[0]:
                    this.attachCss(
                        "gap",
                        fastn_utils.getStaticValue(staticValue[1]),
                    );
                    break;
            }
        } else if (kind === fastn_dom.PropertyKind.Wrap) {
            // sticky is boolean type
            switch (staticValue) {
                case "true":
                case true:
                    this.attachCss("flex-wrap", "wrap");
                    break;
                case "false":
                case false:
                    this.attachCss("flex-wrap", "no-wrap");
                    break;
                default:
                    this.attachCss("flex-wrap", staticValue);
            }
        } else if (kind === fastn_dom.PropertyKind.TextTransform) {
            this.attachCss("text-transform", staticValue);
        } else if (kind === fastn_dom.PropertyKind.TextIndent) {
            this.attachCss("text-indent", staticValue);
        } else if (kind === fastn_dom.PropertyKind.TextAlign) {
            this.attachCss("text-align", staticValue);
        } else if (kind === fastn_dom.PropertyKind.LineClamp) {
            // -webkit-line-clamp: staticValue
            // display: -webkit-box, overflow: hidden
            // -webkit-box-orient: vertical
            this.attachCss("-webkit-line-clamp", staticValue);
            this.attachCss("display", "-webkit-box");
            this.attachCss("overflow", "hidden");
            this.attachCss("-webkit-box-orient", "vertical");
        } else if (kind === fastn_dom.PropertyKind.Opacity) {
            this.attachCss("opacity", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Cursor) {
            this.attachCss("cursor", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Resize) {
            // overflow: auto, resize: staticValue
            this.attachCss("resize", staticValue);
            this.attachCss("overflow", "auto");
        } else if (kind === fastn_dom.PropertyKind.Selectable) {
            if (staticValue === false) {
                this.attachCss("user-select", "none");
            } else {
                this.attachCss("user-select", null);
            }
        } else if (kind === fastn_dom.PropertyKind.MinHeight) {
            this.attachCss("min-height", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MaxHeight) {
            this.attachCss("max-height", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MinWidth) {
            this.attachCss("min-width", staticValue);
        } else if (kind === fastn_dom.PropertyKind.MaxWidth) {
            this.attachCss("max-width", staticValue);
        } else if (kind === fastn_dom.PropertyKind.WhiteSpace) {
            this.attachCss("white-space", staticValue);
        } else if (kind === fastn_dom.PropertyKind.AlignSelf) {
            this.attachCss("align-self", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderColor) {
            this.attachColorCss("border-color", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderLeftColor) {
            this.attachColorCss("border-left-color", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderRightColor) {
            this.attachColorCss("border-right-color", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderTopColor) {
            this.attachColorCss("border-top-color", staticValue);
        } else if (kind === fastn_dom.PropertyKind.BorderBottomColor) {
            this.attachColorCss("border-bottom-color", staticValue);
        } else if (kind === fastn_dom.PropertyKind.LinkColor) {
            this.attachLinkColor(staticValue);
        } else if (kind === fastn_dom.PropertyKind.Color) {
            this.attachColorCss("color", staticValue, true);
        } else if (kind === fastn_dom.PropertyKind.Background) {
            if (fastn_utils.isNull(staticValue)) {
                this.attachColorCss("background-color", staticValue);
                this.attachBackgroundImageCss(staticValue);
                this.attachLinearGradientCss(staticValue);
                return;
            }

            let backgroundType = staticValue[0];
            switch (backgroundType) {
                case fastn_dom.BackgroundStyle.Solid()[0]:
                    this.attachColorCss("background-color", staticValue[1]);
                    break;
                case fastn_dom.BackgroundStyle.Image()[0]:
                    this.attachBackgroundImageCss(staticValue[1]);
                    break;
                case fastn_dom.BackgroundStyle.LinearGradient()[0]:
                    this.attachLinearGradientCss(staticValue[1]);
                    break;
            }
        } else if (kind === fastn_dom.PropertyKind.Display) {
            this.attachCss("display", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Checked) {
            switch (staticValue) {
                case "true":
                case true:
                    this.attachAttribute("checked", "");
                    break;
                case "false":
                case false:
                    this.removeAttribute("checked");
                    break;
                default:
                    this.attachAttribute("checked", staticValue);
            }
            if (!ssr) this.#node.checked = staticValue;
        } else if (kind === fastn_dom.PropertyKind.Enabled) {
            switch (staticValue) {
                case "false":
                case false:
                    this.attachAttribute("disabled", "");
                    break;
                case "true":
                case true:
                    this.removeAttribute("disabled");
                    break;
                default:
                    this.attachAttribute("disabled", staticValue);
            }
        } else if (kind === fastn_dom.PropertyKind.TextInputType) {
            this.attachAttribute("type", staticValue);
        } else if (kind === fastn_dom.PropertyKind.TextInputValue) {
            this.#rawInnerValue = staticValue;
            this.updateTextInputValue();
        } else if (kind === fastn_dom.PropertyKind.DefaultTextInputValue) {
            if (!fastn_utils.isNull(this.#rawInnerValue)) {
                return;
            }
            this.#rawInnerValue = staticValue;
            this.updateTextInputValue();
        } else if (kind === fastn_dom.PropertyKind.InputMaxLength) {
            this.attachAttribute("maxlength", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Placeholder) {
            this.attachAttribute("placeholder", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Multiline) {
            switch (staticValue) {
                case "true":
                case true:
                    this.updateTagName("textarea");
                    break;
                case "false":
                case false:
                    this.updateTagName("input");
                    break;
            }
            this.updateTextInputValue();
        } else if (kind === fastn_dom.PropertyKind.Link) {
            // Changing node type to `a` for link
            // todo: needs fix for image links
            if (fastn_utils.isNull(staticValue)) {
                return;
            }
            this.updateToAnchor(staticValue);
        } else if (kind === fastn_dom.PropertyKind.LinkRel) {
            if (fastn_utils.isNull(staticValue)) {
                this.removeAttribute("rel");
            }
            let rel_list = staticValue.map((obj) =>
                fastn_utils.getStaticValue(obj.item),
            );
            this.attachAttribute("rel", rel_list.join(" "));
        } else if (kind === fastn_dom.PropertyKind.OpenInNewTab) {
            // open_in_new_tab is boolean type
            switch (staticValue) {
                case "true":
                case true:
                    this.attachAttribute("target", "_blank");
                    break;
                default:
                    this.attachAttribute("target", staticValue);
            }
        } else if (kind === fastn_dom.PropertyKind.TextStyle) {
            let styles = staticValue?.map((obj) =>
                fastn_utils.getStaticValue(obj.item),
            );
            this.attachTextStyles(styles);
        } else if (kind === fastn_dom.PropertyKind.Region) {
            this.updateTagName(staticValue);
            if (this.#node.innerHTML) {
                this.#node.id = fastn_utils.slugify(this.#rawInnerValue);
            }
        } else if (kind === fastn_dom.PropertyKind.AlignContent) {
            let node_kind = this.#kind;
            this.attachAlignContent(staticValue, node_kind);
        } else if (kind === fastn_dom.PropertyKind.Loading) {
            this.attachAttribute("loading", staticValue);
        } else if (kind === fastn_dom.PropertyKind.Src) {
            this.attachAttribute("src", staticValue);
        } else if (kind === fastn_dom.PropertyKind.ImageSrc) {
            ftd.dark_mode.addClosure(
                fastn
                    .closure(() => {
                        if (fastn_utils.isNull(staticValue)) {
                            this.attachAttribute("src", staticValue);
                            return;
                        }
                        const is_dark_mode = ftd.dark_mode.get();
                        const src = staticValue.get(
                            is_dark_mode ? "dark" : "light",
                        );
                        if (!ssr) {
                            let image_node = this.#node;
                            if (image_node.nodeName.toLowerCase() === "a") {
                                let childNodes = image_node.childNodes;
                                childNodes.forEach(function (child) {
                                    if (child.nodeName.toLowerCase() === "img")
                                        image_node = child;
                                });
                            }
                            image_node.setAttribute(
                                "src",
                                fastn_utils.getStaticValue(src),
                            );
                        } else {
                            this.attachAttribute(
                                "src",
                                fastn_utils.getStaticValue(src),
                            );
                        }
                    })
                    .addNodeProperty(this, null, inherited),
            );
            this.#mutables.push(ftd.dark_mode);
        } else if (kind === fastn_dom.PropertyKind.Alt) {
            this.attachAttribute("alt", staticValue);
        } else if (kind === fastn_dom.PropertyKind.VideoSrc) {
            ftd.dark_mode.addClosure(
                fastn
                    .closure(() => {
                        if (fastn_utils.isNull(staticValue)) {
                            this.attachAttribute("src", staticValue);
                            return;
                        }
                        const is_dark_mode = ftd.dark_mode.get();
                        const src = staticValue.get(
                            is_dark_mode ? "dark" : "light",
                        );

                        this.attachAttribute(
                            "src",
                            fastn_utils.getStaticValue(src),
                        );
                    })
                    .addNodeProperty(this, null, inherited),
            );
            this.#mutables.push(ftd.dark_mode);
        } else if (kind === fastn_dom.PropertyKind.Autoplay) {
            if (staticValue) {
                this.attachAttribute("autoplay", staticValue);
            } else {
                this.removeAttribute("autoplay");
            }
        } else if (kind === fastn_dom.PropertyKind.Muted) {
            if (staticValue) {
                this.attachAttribute("muted", staticValue);
            } else {
                this.removeAttribute("muted");
            }
        } else if (kind === fastn_dom.PropertyKind.Controls) {
            if (staticValue) {
                this.attachAttribute("controls", staticValue);
            } else {
                this.removeAttribute("controls");
            }
        } else if (kind === fastn_dom.PropertyKind.LoopVideo) {
            if (staticValue) {
                this.attachAttribute("loop", staticValue);
            } else {
                this.removeAttribute("loop");
            }
        } else if (kind === fastn_dom.PropertyKind.Poster) {
            ftd.dark_mode.addClosure(
                fastn
                    .closure(() => {
                        if (fastn_utils.isNull(staticValue)) {
                            this.attachAttribute("poster", staticValue);
                            return;
                        }
                        const is_dark_mode = ftd.dark_mode.get();
                        const posterSrc = staticValue.get(
                            is_dark_mode ? "dark" : "light",
                        );

                        this.attachAttribute(
                            "poster",
                            fastn_utils.getStaticValue(posterSrc),
                        );
                    })
                    .addNodeProperty(this, null, inherited),
            );
            this.#mutables.push(ftd.dark_mode);
        } else if (kind === fastn_dom.PropertyKind.Fit) {
            this.attachCss("object-fit", staticValue);
        } else if (kind === fastn_dom.PropertyKind.FetchPriority) {
            this.attachAttribute("fetchpriority", staticValue);
        } else if (kind === fastn_dom.PropertyKind.YoutubeSrc) {
            if (fastn_utils.isNull(staticValue)) {
                this.attachAttribute("src", staticValue);
                return;
            }
            const id_pattern = "^([a-zA-Z0-9_-]{11})$";
            let id = staticValue.match(id_pattern);
            if (!fastn_utils.isNull(id)) {
                this.attachAttribute(
                    "src",
                    `https:\/\/youtube.com/embed/${id[0]}`,
                );
            } else {
                this.attachAttribute("src", staticValue);
            }
        } else if (kind === fastn_dom.PropertyKind.Role) {
            this.attachRoleCss(staticValue);
        } else if (kind === fastn_dom.PropertyKind.Code) {
            if (!fastn_utils.isNull(staticValue)) {
                let { modifiedText, highlightedLines } =
                    fastn_utils.findAndRemoveHighlighter(staticValue);
                if (highlightedLines.length !== 0) {
                    this.attachAttribute("data-line", highlightedLines);
                }
                staticValue = modifiedText;
            }
            let codeNode = this.#children[0].getNode();
            let codeText = fastn_utils.escapeHtmlInCode(staticValue);
            codeNode.innerHTML = codeText;
            this.#extraData.code = this.#extraData.code
                ? this.#extraData.code
                : {};
            fastn_utils.highlightCode(codeNode, this.#extraData.code);
        } else if (kind === fastn_dom.PropertyKind.CodeShowLineNumber) {
            if (staticValue) {
                this.#node.classList.add("line-numbers");
            } else {
                this.#node.classList.remove("line-numbers");
            }
        } else if (kind === fastn_dom.PropertyKind.CodeTheme) {
            this.#extraData.code = this.#extraData.code
                ? this.#extraData.code
                : {};
            if (fastn_utils.isNull(staticValue)) {
                if (!fastn_utils.isNull(this.#extraData.code.theme)) {
                    this.#node.classList.remove(this.#extraData.code.theme);
                }
                return;
            }
            if (!ssr) {
                fastn_utils.addCodeTheme(staticValue);
            }
            staticValue = fastn_utils.getStaticValue(staticValue);
            let theme = staticValue.replace(".", "-");
            if (this.#extraData.code.theme !== theme) {
                let codeNode = this.#children[0].getNode();
                this.#node.classList.remove(this.#extraData.code.theme);
                codeNode.classList.remove(this.#extraData.code.theme);
                this.#extraData.code.theme = theme;
                this.#node.classList.add(theme);
                codeNode.classList.add(theme);
                fastn_utils.highlightCode(codeNode, this.#extraData.code);
            }
        } else if (kind === fastn_dom.PropertyKind.CodeLanguage) {
            let language = `language-${staticValue}`;
            this.#extraData.code = this.#extraData.code
                ? this.#extraData.code
                : {};
            if (this.#extraData.code.language) {
                this.#node.classList.remove(language);
            }
            this.#extraData.code.language = language;
            this.#node.classList.add(language);
            let codeNode = this.#children[0].getNode();
            codeNode.classList.add(language);
            fastn_utils.highlightCode(codeNode, this.#extraData.code);
        } else if (kind === fastn_dom.PropertyKind.Favicon) {
            if (fastn_utils.isNull(staticValue)) return;
            this.setFavicon(staticValue);
        } else if (
            kind === fastn_dom.PropertyKind.DocumentProperties.MetaTitle
        ) {
            this.updateMetaTitle(staticValue);
        } else if (
            kind === fastn_dom.PropertyKind.DocumentProperties.MetaOGTitle
        ) {
            this.addMetaTagByProperty("og:title", staticValue);
        } else if (
            kind === fastn_dom.PropertyKind.DocumentProperties.MetaTwitterTitle
        ) {
            this.addMetaTagByName("twitter:title", staticValue);
        } else if (
            kind === fastn_dom.PropertyKind.DocumentProperties.MetaDescription
        ) {
            this.addMetaTagByName("description", staticValue);
        } else if (
            kind === fastn_dom.PropertyKind.DocumentProperties.MetaOGDescription
        ) {
            this.addMetaTagByProperty("og:description", staticValue);
        } else if (
            kind ===
            fastn_dom.PropertyKind.DocumentProperties.MetaTwitterDescription
        ) {
            this.addMetaTagByName("twitter:description", staticValue);
        } else if (
            kind === fastn_dom.PropertyKind.DocumentProperties.MetaOGImage
        ) {
            // staticValue is of ftd.raw-image-src RecordInstance type
            if (fastn_utils.isNull(staticValue)) {
                this.removeMetaTagByProperty("og:image");
                return;
            }
            this.addMetaTagByProperty(
                "og:image",
                fastn_utils.getStaticValue(staticValue.get("src")),
            );
        } else if (
            kind === fastn_dom.PropertyKind.DocumentProperties.MetaTwitterImage
        ) {
            // staticValue is of ftd.raw-image-src RecordInstance type
            if (fastn_utils.isNull(staticValue)) {
                this.removeMetaTagByName("twitter:image");
                return;
            }
            this.addMetaTagByName(
                "twitter:image",
                fastn_utils.getStaticValue(staticValue.get("src")),
            );
        } else if (
            kind === fastn_dom.PropertyKind.DocumentProperties.MetaThemeColor
        ) {
            // staticValue is of ftd.color RecordInstance type
            if (fastn_utils.isNull(staticValue)) {
                this.removeMetaTagByName("theme-color");
                return;
            }
            this.addMetaTagByName(
                "theme-color",
                fastn_utils.getStaticValue(staticValue.get("light")),
            );
        } else if (
            kind ===
            fastn_dom.PropertyKind.DocumentProperties
                .MetaFacebookDomainVerification
        ) {
            if (fastn_utils.isNull(staticValue)) {
                this.removeMetaTagByName("facebook-domain-verification");
                return;
            }
            this.addMetaTagByName(
                "facebook-domain-verification",
                fastn_utils.getStaticValue(staticValue),
            );
        } else if (
            kind === fastn_dom.PropertyKind.IntegerValue ||
            kind === fastn_dom.PropertyKind.DecimalValue ||
            kind === fastn_dom.PropertyKind.BooleanValue
        ) {
            this.#node.innerHTML = staticValue;
            this.#rawInnerValue = staticValue;
        } else if (kind === fastn_dom.PropertyKind.StringValue) {
            this.#rawInnerValue = staticValue;
            staticValue = fastn_utils.markdown_inline(
                fastn_utils.escapeHtmlInMarkdown(staticValue),
            );
            staticValue = fastn_utils.process_post_markdown(
                this.#node,
                staticValue,
            );
            if (!fastn_utils.isNull(staticValue)) {
                this.#node.innerHTML = staticValue;
            } else {
                this.#node.innerHTML = "";
            }
        } else {
            throw "invalid fastn_dom.PropertyKind: " + kind;
        }
    }
    setProperty(kind, value, inherited) {
        if (value instanceof fastn.mutableClass) {
            this.setDynamicProperty(
                kind,
                [value],
                () => {
                    return value.get();
                },
                inherited,
            );
        } else if (value instanceof PropertyValueAsClosure) {
            this.setDynamicProperty(
                kind,
                value.deps,
                value.closureFunction,
                inherited,
            );
        } else {
            this.setStaticProperty(kind, value, inherited);
        }
    }
    setDynamicProperty(kind, deps, func, inherited) {
        let closure = fastn
            .closure(func)
            .addNodeProperty(this, kind, inherited);
        for (let dep in deps) {
            if (fastn_utils.isNull(deps[dep]) || !deps[dep].addClosure) {
                continue;
            }
            deps[dep].addClosure(closure);
            this.#mutables.push(deps[dep]);
        }
    }
    getNode() {
        return this.#node;
    }
    getExtraData() {
        return this.#extraData;
    }
    getChildren() {
        return this.#children;
    }
    mergeFnCalls(current, newFunc) {
        return () => {
            if (current instanceof Function) current();
            if (newFunc instanceof Function) newFunc();
        };
    }
    addEventHandler(event, func) {
        if (event === fastn_dom.Event.Click) {
            let onclickEvents = this.mergeFnCalls(this.#node.onclick, func);
            if (fastn_utils.isNull(this.#node.onclick))
                this.attachCss("cursor", "pointer");
            this.#node.onclick = onclickEvents;
        } else if (event === fastn_dom.Event.MouseEnter) {
            let mouseEnterEvents = this.mergeFnCalls(
                this.#node.onmouseenter,
                func,
            );
            this.#node.onmouseenter = mouseEnterEvents;
        } else if (event === fastn_dom.Event.MouseLeave) {
            let mouseLeaveEvents = this.mergeFnCalls(
                this.#node.onmouseleave,
                func,
            );
            this.#node.onmouseleave = mouseLeaveEvents;
        } else if (event === fastn_dom.Event.ClickOutside) {
            ftd.clickOutsideEvents.push([this, func]);
        } else if (!!event[0] && event[0] === fastn_dom.Event.GlobalKey()[0]) {
            ftd.globalKeyEvents.push([this, func, event[1]]);
        } else if (
            !!event[0] &&
            event[0] === fastn_dom.Event.GlobalKeySeq()[0]
        ) {
            ftd.globalKeySeqEvents.push([this, func, event[1]]);
        } else if (event === fastn_dom.Event.Input) {
            let onInputEvents = this.mergeFnCalls(this.#node.oninput, func);
            this.#node.oninput = onInputEvents;
        } else if (event === fastn_dom.Event.Change) {
            let onChangeEvents = this.mergeFnCalls(this.#node.onchange, func);
            this.#node.onchange = onChangeEvents;
        } else if (event === fastn_dom.Event.Blur) {
            let onBlurEvents = this.mergeFnCalls(this.#node.onblur, func);
            this.#node.onblur = onBlurEvents;
        } else if (event === fastn_dom.Event.Focus) {
            let onFocusEvents = this.mergeFnCalls(this.#node.onfocus, func);
            this.#node.onfocus = onFocusEvents;
        }
    }
    destroy() {
        for (let i = 0; i < this.#mutables.length; i++) {
            this.#mutables[i].unlinkNode(this);
        }
        // Todo: We don't need this condition as after destroying this node
        //  ConditionalDom reset this.#conditionUI to null or some different
        //  value. Not sure why this is still needed.
        if (!fastn_utils.isNull(this.#node)) {
            this.#node.remove();
        }
        this.#mutables = [];
        this.#parent = null;
        this.#node = null;
    }
}

class ConditionalDom {
    #marker;
    #parent;
    #node_constructor;
    #condition;
    #mutables;
    #conditionUI;

    constructor(parent, deps, condition, node_constructor) {
        this.#marker = fastn_dom.createKernel(
            parent,
            fastn_dom.ElementKind.Comment,
        );
        this.#parent = parent;

        this.#conditionUI = null;
        let closure = fastn.closure(() => {
            fastn_utils.resetFullHeight();
            if (condition()) {
                if (this.#conditionUI) {
                    let conditionUI = fastn_utils.flattenArray(
                        this.#conditionUI,
                    );
                    while (conditionUI.length > 0) {
                        let poppedElement = conditionUI.pop();
                        poppedElement.destroy();
                    }
                }
                this.#conditionUI = node_constructor(
                    new ParentNodeWithSibiling(this.#parent, this.#marker),
                );
                if (
                    !Array.isArray(this.#conditionUI) &&
                    fastn_utils.isWrapperNode(this.#conditionUI.getTagName())
                ) {
                    this.#conditionUI = this.#conditionUI.getChildren();
                }
            } else if (this.#conditionUI) {
                let conditionUI = fastn_utils.flattenArray(this.#conditionUI);
                while (conditionUI.length > 0) {
                    let poppedElement = conditionUI.pop();
                    poppedElement.destroy();
                }
                this.#conditionUI = null;
            }
            fastn_utils.setFullHeight();
        });
        deps.forEach((dep) => {
            if (!fastn_utils.isNull(dep) && dep.addClosure) {
                dep.addClosure(closure);
            }
        });

        this.#node_constructor = node_constructor;
        this.#condition = condition;
        this.#mutables = [];
    }

    getParent() {
        let nodes = [this.#marker];
        if (this.#conditionUI) {
            nodes.push(this.#conditionUI);
        }
        return nodes;
    }
}

fastn_dom.createKernel = function (parent, kind) {
    return new Node2(parent, kind);
};

fastn_dom.conditionalDom = function (
    parent,
    deps,
    condition,
    node_constructor,
) {
    return new ConditionalDom(parent, deps, condition, node_constructor);
};

class ParentNodeWithSibiling {
    #parent;
    #sibiling;
    constructor(parent, sibiling) {
        this.#parent = parent;
        this.#sibiling = sibiling;
    }
    getParent() {
        return this.#parent;
    }
    getSibiling() {
        return this.#sibiling;
    }
}

class ForLoop {
    #node_constructor;
    #list;
    #wrapper;
    #parent;
    #nodes;
    constructor(parent, node_constructor, list) {
        this.#wrapper = fastn_dom.createKernel(
            parent,
            fastn_dom.ElementKind.Comment,
        );
        this.#parent = parent;
        this.#node_constructor = node_constructor;
        this.#list = list;
        this.#nodes = [];

        fastn_utils.resetFullHeight();
        for (let idx in list.getList()) {
            this.createNode(idx, false);
        }
        fastn_utils.setFullHeight();
    }
    createNode(index, resizeBodyHeight = true) {
        if (resizeBodyHeight) {
            fastn_utils.resetFullHeight();
        }
        let parentWithSibiling = new ParentNodeWithSibiling(
            this.#parent,
            this.#wrapper,
        );
        if (index !== 0) {
            parentWithSibiling = new ParentNodeWithSibiling(
                this.#parent,
                this.#nodes[index - 1],
            );
        }
        let v = this.#list.get(index);
        let node = this.#node_constructor(parentWithSibiling, v.item, v.index);
        this.#nodes.splice(index, 0, node);
        if (resizeBodyHeight) {
            fastn_utils.setFullHeight();
        }
        return node;
    }
    createAllNode() {
        fastn_utils.resetFullHeight();
        this.deleteAllNode(false);
        for (let idx in this.#list.getList()) {
            this.createNode(idx, false);
        }
        fastn_utils.setFullHeight();
    }
    deleteAllNode(resizeBodyHeight = true) {
        if (resizeBodyHeight) {
            fastn_utils.resetFullHeight();
        }
        while (this.#nodes.length > 0) {
            this.#nodes.pop().destroy();
        }
        if (resizeBodyHeight) {
            fastn_utils.setFullHeight();
        }
    }
    getWrapper() {
        return this.#wrapper;
    }
    deleteNode(index) {
        fastn_utils.resetFullHeight();
        let node = this.#nodes.splice(index, 1)[0];
        node.destroy();
        fastn_utils.setFullHeight();
    }
    getParent() {
        return this.#parent;
    }
}

fastn_dom.forLoop = function (parent, node_constructor, list) {
    return new ForLoop(parent, node_constructor, list);
};
let fastn_utils = {
    htmlNode(kind) {
        let node = "div";
        let css = [];
        let attributes = {};
        if (kind === fastn_dom.ElementKind.Column) {
            css.push(fastn_dom.InternalClass.FT_COLUMN);
        } else if (kind === fastn_dom.ElementKind.Document) {
            css.push(fastn_dom.InternalClass.FT_COLUMN);
            css.push(fastn_dom.InternalClass.FT_FULL_SIZE);
        } else if (kind === fastn_dom.ElementKind.Row) {
            css.push(fastn_dom.InternalClass.FT_ROW);
        } else if (kind === fastn_dom.ElementKind.IFrame) {
            node = "iframe";
            // To allow fullscreen support
            // Reference: https://stackoverflow.com/questions/27723423/youtube-iframe-embed-full-screen
            attributes["allowfullscreen"] = "";
        } else if (kind === fastn_dom.ElementKind.Image) {
            node = "img";
        } else if (kind === fastn_dom.ElementKind.Video) {
            node = "video";
        } else if (
            kind === fastn_dom.ElementKind.ContainerElement ||
            kind === fastn_dom.ElementKind.Text
        ) {
            node = "div";
        } else if (kind === fastn_dom.ElementKind.Rive) {
            node = "canvas";
        } else if (kind === fastn_dom.ElementKind.CheckBox) {
            node = "input";
            attributes["type"] = "checkbox";
        } else if (kind === fastn_dom.ElementKind.TextInput) {
            node = "input";
        } else if (kind === fastn_dom.ElementKind.Comment) {
            node = fastn_dom.commentNode;
        } else if (kind === fastn_dom.ElementKind.Wrapper) {
            node = fastn_dom.wrapperNode;
        } else if (kind === fastn_dom.ElementKind.Code) {
            node = "pre";
        } else if (kind === fastn_dom.ElementKind.CodeChild) {
            node = "code";
        } else if (kind[0] === fastn_dom.ElementKind.WebComponent()[0]) {
            let [webcomponent, args] = kind[1];
            node = `${webcomponent}`;
            fastn_dom.webComponent.push(args);
            attributes[fastn_dom.webComponentArgument] =
                fastn_dom.webComponent.length - 1;
        }
        return [node, css, attributes];
    },
    createStyle(cssClass, obj) {
        if (doubleBuffering) {
            fastn_dom.styleClasses = `${
                fastn_dom.styleClasses
            }${getClassAsString(cssClass, obj)}\n`;
        } else {
            let styles = document.getElementById("styles");
            let newClasses = getClassAsString(cssClass, obj);
            let textNode = document.createTextNode(newClasses);
            if (styles.styleSheet) {
                styles.styleSheet.cssText = newClasses;
            } else {
                styles.appendChild(textNode);
            }
        }
    },
    getStaticValue(obj) {
        if (obj instanceof fastn.mutableClass) {
            return this.getStaticValue(obj.get());
        } else if (obj instanceof fastn.mutableListClass) {
            return obj.getList();
        } /*
        Todo: Make this work
        else if (obj instanceof fastn.recordInstanceClass) {
            return obj.getAllFields();
        }*/ else {
            return obj;
        }
    },
    getInheritedValues(default_args, inherited, function_args) {
        let record_fields = {
            colors: ftd.default_colors.getClone().setAndReturn("is_root", true),
            types: ftd.default_types.getClone().setAndReturn("is_root", true),
        };
        Object.assign(record_fields, default_args);
        let fields = {};
        if (inherited instanceof fastn.recordInstanceClass) {
            fields = inherited.getClonedFields();
            if (fastn_utils.getStaticValue(fields["colors"].get("is_root"))) {
                delete fields.colors;
            }
            if (fastn_utils.getStaticValue(fields["types"].get("is_root"))) {
                delete fields.types;
            }
        }
        Object.assign(record_fields, fields);
        Object.assign(record_fields, function_args);
        return fastn.recordInstance({
            ...record_fields,
        });
    },
    removeNonFastnClasses(node) {
        let classList = node.getNode().classList;
        let extraCodeData = node.getExtraData().code;
        let iterativeClassList = classList;
        if (ssr) {
            iterativeClassList = iterativeClassList.getClasses();
        }
        const internalClassNames = Object.values(fastn_dom.InternalClass);
        const classesToRemove = [];

        for (const className of iterativeClassList) {
            if (
                !className.startsWith("__") &&
                !internalClassNames.includes(className) &&
                className !== extraCodeData?.language &&
                className !== extraCodeData?.theme
            ) {
                classesToRemove.push(className);
            }
        }

        for (const classNameToRemove of classesToRemove) {
            classList.remove(classNameToRemove);
        }
    },
    staticToMutables(obj) {
        if (
            !(obj instanceof fastn.mutableClass) &&
            !(obj instanceof fastn.mutableListClass) &&
            !(obj instanceof fastn.recordInstanceClass)
        ) {
            if (Array.isArray(obj)) {
                let list = [];
                for (let index in obj) {
                    list.push(fastn_utils.staticToMutables(obj[index]));
                }
                return fastn.mutableList(list);
            } else if (obj instanceof Object) {
                let fields = {};
                for (let objKey in obj) {
                    fields[objKey] = fastn_utils.staticToMutables(obj[objKey]);
                }
                return fastn.recordInstance(fields);
            } else {
                return fastn.mutable(obj);
            }
        } else {
            return obj;
        }
    },
    getFlattenStaticValue(obj) {
        let staticValue = fastn_utils.getStaticValue(obj);
        if (Array.isArray(staticValue)) {
            return staticValue.map((func) =>
                fastn_utils.getFlattenStaticValue(func.item),
            );
        } /*
        Todo: Make this work
        else if (typeof staticValue === 'object' && fastn_utils.isNull(staticValue)) {
            return Object.fromEntries(
                Object.entries(staticValue).map(([k,v]) =>
                    [k, fastn_utils.getFlattenStaticValue(v)]
                )
            );
        }*/
        return staticValue;
    },
    getter(value) {
        if (value instanceof fastn.mutableClass) {
            return value.get();
        } else {
            return value;
        }
    },
    // Todo: Merge getterByKey with getter
    getterByKey(value, index) {
        if (
            value instanceof fastn.mutableClass ||
            value instanceof fastn.recordInstanceClass
        ) {
            return value.get(index);
        } else if (value instanceof fastn.mutableListClass) {
            return value.get(index).item;
        } else {
            return value;
        }
    },
    setter(variable, value) {
        if (!fastn_utils.isNull(variable) && variable.set) {
            variable.set(value);
            return true;
        }
        return false;
    },
    defaultPropertyValue(_propertyValue) {
        return null;
    },
    sameResponsiveRole(desktop, mobile) {
        return (
            desktop.get("font_family") === mobile.get("font_family") &&
            desktop.get("letter_spacing") === mobile.get("letter_spacing") &&
            desktop.get("line_height") === mobile.get("line_height") &&
            desktop.get("size") === mobile.get("size") &&
            desktop.get("weight") === mobile.get("weight")
        );
    },
    getRoleValues(value) {
        let font_families = fastn_utils.getStaticValue(
            value.get("font_family"),
        );
        if (Array.isArray(font_families))
            font_families = font_families
                .map((obj) => fastn_utils.getStaticValue(obj.item))
                .join(", ");
        return {
            "font-family": font_families,
            "letter-spacing": fastn_utils.getStaticValue(
                value.get("letter_spacing"),
            ),
            "font-size": fastn_utils.getStaticValue(value.get("size")),
            "font-weight": fastn_utils.getStaticValue(value.get("weight")),
            "line-height": fastn_utils.getStaticValue(value.get("line_height")),
        };
    },
    clone(value) {
        if (value === null || value === undefined) {
            return value;
        }
        if (
            value instanceof fastn.mutableClass ||
            value instanceof fastn.mutableListClass
        ) {
            return value.getClone();
        }
        if (value instanceof fastn.recordInstanceClass) {
            return value.getClone();
        }
        return value;
    },
    getListItem(value) {
        if (value === undefined) {
            return null;
        }
        if (value instanceof Object && value.hasOwnProperty("item")) {
            value = value.item;
        }
        return value;
    },
    getEventKey(event) {
        if (65 <= event.keyCode && event.keyCode <= 90) {
            return String.fromCharCode(event.keyCode).toLowerCase();
        } else {
            return event.key;
        }
    },
    createNestedObject(currentObject, path, value) {
        const properties = path.split(".");

        for (let i = 0; i < properties.length - 1; i++) {
            let property = fastn_utils.private.addUnderscoreToStart(
                properties[i],
            );
            if (currentObject instanceof fastn.recordInstanceClass) {
                if (currentObject.get(property) === undefined) {
                    currentObject.set(property, fastn.recordInstance({}));
                }
                currentObject = currentObject.get(property).get();
            } else {
                if (!currentObject.hasOwnProperty(property)) {
                    currentObject[property] = fastn.recordInstance({});
                }
                currentObject = currentObject[property];
            }
        }

        const innermostProperty = properties[properties.length - 1];
        if (currentObject instanceof fastn.recordInstanceClass) {
            currentObject.set(innermostProperty, value);
        } else {
            currentObject[innermostProperty] = value;
        }
    },
    /**
     * Takes an input string and processes it as inline markdown using the
     * 'marked' library. The function removes the last occurrence of
     * wrapping <p> tags (i.e. <p> tag found at the end) from the result and
     * adjusts spaces around the content.
     *
     * @param {string} i - The input string to be processed as inline markdown.
     * @returns {string} - The processed string with inline markdown.
     */
    markdown_inline(i) {
        if (fastn_utils.isNull(i)) return;
        i = i.toString();
        const { space_before, space_after } = fastn_utils.private.spaces(i);
        const o = (() => {
            let g = fastn_utils.private.replace_last_occurrence(
                marked.parse(i),
                "<p>",
                "",
            );
            g = fastn_utils.private.replace_last_occurrence(g, "</p>", "");
            return g;
        })();
        return `${fastn_utils.private.repeated_space(
            space_before,
        )}${o}${fastn_utils.private.repeated_space(space_after)}`.replace(
            /\n+$/,
            "",
        );
    },

    process_post_markdown(node, body) {
        if (!ssr) {
            const divElement = document.createElement("div");
            divElement.innerHTML = body;

            const current_node = node;
            const colorClasses = Array.from(current_node.classList).filter(
                (className) => className.startsWith("__c"),
            );
            const roleClasses = Array.from(current_node.classList).filter(
                (className) => className.startsWith("__rl"),
            );
            const tableElements = Array.from(
                divElement.getElementsByTagName("table"),
            );
            const codeElements = Array.from(
                divElement.getElementsByTagName("code"),
            );

            tableElements.forEach((table) => {
                colorClasses.forEach((colorClass) => {
                    table.classList.add(colorClass);
                });
            });

            codeElements.forEach((code) => {
                roleClasses.forEach((roleClass) => {
                    var roleCls = "." + roleClass;
                    let role = fastn_dom.classes[roleCls];
                    let roleValue = role["value"];
                    let fontFamily = roleValue["font-family"];
                    code.style.fontFamily = fontFamily;
                });
            });

            body = divElement.innerHTML;
        }
        return body;
    },
    isNull(a) {
        return a === null || a === undefined;
    },
    isCommentNode(node) {
        return node === fastn_dom.commentNode;
    },
    isWrapperNode(node) {
        return node === fastn_dom.wrapperNode;
    },
    nextSibling(node, parent) {
        // For Conditional DOM
        while (Array.isArray(node)) {
            node = node[node.length - 1];
        }
        if (node.nextSibling) {
            return node.nextSibling;
        }
        if (node.getNode && node.getNode().nextSibling !== undefined) {
            return node.getNode().nextSibling;
        }
        return parent.getChildren().indexOf(node.getNode()) + 1;
    },
    createNodeHelper(node, classes, attributes) {
        let tagName = node;
        let element = fastnVirtual.document.createElement(node);
        for (let key in attributes) {
            element.setAttribute(key, attributes[key]);
        }
        for (let c in classes) {
            element.classList.add(classes[c]);
        }

        return [tagName, element];
    },
    addCssFile(url) {
        // Create a new link element
        const linkElement = document.createElement("link");

        // Set the attributes of the link element
        linkElement.rel = "stylesheet";
        linkElement.href = url;

        // Append the link element to the head section of the document
        document.head.appendChild(linkElement);
    },
    addCodeTheme(theme) {
        if (!fastn_dom.codeData.addedCssFile.includes(theme)) {
            let themeCssUrl = fastn_dom.codeData.availableThemes[theme];
            fastn_utils.addCssFile(themeCssUrl);
            fastn_dom.codeData.addedCssFile.push(theme);
        }
    },
    /**
     * Searches for highlighter occurrences in the text, removes them,
     * and returns the modified text along with highlighted line numbers.
     *
     * @param {string} text - The input text to process.
     * @returns {{ modifiedText: string, highlightedLines: number[] }}
     *   Object containing modified text and an array of highlighted line numbers.
     *
     * @example
     * const text = `/-- ftd.text: Hello ;; hello
     *
     * -- some-component: caption-value
     * attr-name: attr-value ;; <hl>
     *
     *
     * -- other-component: caption-value ;; <hl>
     * attr-name: attr-value`;
     *
     * const result = findAndRemoveHighlighter(text);
     * console.log(result.modifiedText);
     * console.log(result.highlightedLines);
     */
    findAndRemoveHighlighter(text) {
        const lines = text.split("\n");
        const highlighter = ";; <hl>";
        const result = {
            modifiedText: "",
            highlightedLines: "",
        };

        let highlightedLines = [];
        for (let i = 0; i < lines.length; i++) {
            const line = lines[i];
            const highlighterIndex = line.indexOf(highlighter);

            if (highlighterIndex !== -1) {
                highlightedLines.push(i + 1); // Adding 1 to convert to human-readable line numbers
                result.modifiedText +=
                    line.substring(0, highlighterIndex) +
                    line.substring(highlighterIndex + highlighter.length) +
                    "\n";
            } else {
                result.modifiedText += line + "\n";
            }
        }

        result.highlightedLines =
            fastn_utils.private.mergeNumbers(highlightedLines);

        return result;
    },
    getNodeValue(node) {
        return node.getNode().value;
    },
    getNodeCheckedState(node) {
        return node.getNode().checked;
    },
    setFullHeight() {
        if (!ssr) {
            document.body.style.height = `max(${document.documentElement.scrollHeight}px, 100%)`;
        }
    },
    resetFullHeight() {
        if (!ssr) {
            document.body.style.height = `100%`;
        }
    },
    highlightCode(codeElement, extraCodeData) {
        if (
            !ssr &&
            !fastn_utils.isNull(extraCodeData.language) &&
            !fastn_utils.isNull(extraCodeData.theme)
        ) {
            Prism.highlightElement(codeElement);
        }
    },

    //Taken from: https://byby.dev/js-slugify-string
    slugify(str) {
        return String(str)
            .normalize("NFKD") // split accented characters into their base characters and diacritical marks
            .replace(".", "-")
            .replace(/[\u0300-\u036f]/g, "") // remove all the accents, which happen to be all in the \u03xx UNICODE block.
            .trim() // trim leading or trailing whitespace
            .toLowerCase() // convert to lowercase
            .replace(/[^a-z0-9 -]/g, "") // remove non-alphanumeric characters
            .replace(/\s+/g, "-") // replace spaces with hyphens
            .replace(/-+/g, "-"); // remove consecutive hyphens
    },

    getEventListeners(node) {
        return {
            onclick: node.onclick,
            onmouseleave: node.onmouseleave,
            onmouseenter: node.onmouseenter,
            oninput: node.oninput,
            onblur: node.onblur,
            onfocus: node.onfocus,
        };
    },

    flattenArray(arr) {
        return fastn_utils.private.flattenArray([arr]);
    },
    toSnakeCase(value) {
        return value
            .trim()
            .split("")
            .map((v, i) => {
                const lowercased = v.toLowerCase();
                if (v == " ") {
                    return "_";
                }
                if (v != lowercased && i > 0) {
                    return `_${lowercased}`;
                }
                return lowercased;
            })
            .join("");
    },
    escapeHtmlInCode(str) {
        return str.replace(/[<]/g, "&lt;");
    },

    escapeHtmlInMarkdown(str) {
        if (typeof str !== "string") {
            return str;
        }

        let result = "";
        let ch_map = {
            "<": "&lt;",
            ">": "&gt;",
            "&": "&amp;",
            '"': "&quot;",
            "'": "&#39;",
            "/": "&#47;",
        };
        let foundBackTick = false;
        for (var i = 0; i < str.length; i++) {
            let current = str[i];
            if (current === "`") {
                foundBackTick = !foundBackTick;
            }
            // Ignore escaping html inside backtick (as marked function
            // escape html for backtick content):
            // For instance: In `hello <title>`, `<` and `>` should not be
            // escaped. (`foundBackTick`)
            // Also the `/` which is followed by `<` should be escaped.
            // For instance: `</` should be escaped but `http://` should not
            // be escaped. (`(current === '/' && !(i > 0 && str[i-1] === "<"))`)
            if (
                foundBackTick ||
                (current === "/" && !(i > 0 && str[i - 1] === "<"))
            ) {
                result += current;
                continue;
            }
            result += ch_map[current] ?? current;
        }
        return result;
    },

    // Used to initialize __args__ inside component and UDF js functions
    getArgs(default_args, passed_args) {
        // Note: arguments as variable name not allowed in strict mode
        let args = default_args;
        for (var arg in passed_args) {
            if (!default_args.hasOwnProperty(arg)) {
                args[arg] = passed_args[arg];
                continue;
            }
            if (
                default_args.hasOwnProperty(arg) &&
                fastn_utils.getStaticValue(passed_args[arg]) !== undefined
            ) {
                args[arg] = passed_args[arg];
            }
        }
        return args;
    },

    /**
     * Replaces the children of `document.body` with the children from
     * newChildrenWrapper and updates the styles based on the
     * `fastn_dom.styleClasses`.
     *
     * @param {HTMLElement} newChildrenWrapper - The wrapper element
     * containing the new children.
     */
    replaceBodyStyleAndChildren(newChildrenWrapper) {
        // Update styles based on `fastn_dom.styleClasses`
        let styles = document.getElementById("styles");
        styles.innerHTML = fastn_dom.getClassesAsStringWithoutStyleTag();

        // Replace the children of document.body with the children from
        // newChildrenWrapper
        fastn_utils.private.replaceChildren(document.body, newChildrenWrapper);
    },
};

fastn_utils.private = {
    flattenArray(arr) {
        return arr.reduce((acc, item) => {
            return acc.concat(
                Array.isArray(item)
                    ? fastn_utils.private.flattenArray(item)
                    : item,
            );
        }, []);
    },
    /**
     * Helper function for `fastn_utils.markdown_inline` to find the number of
     * spaces before and after the content.
     *
     * @param {string} s - The input string.
     * @returns {Object} - An object with 'space_before' and 'space_after' properties
     * representing the number of spaces before and after the content.
     */
    spaces(s) {
        let space_before = 0;
        for (let i = 0; i < s.length; i++) {
            if (s[i] !== " ") {
                space_before = i;
                break;
            }
            space_before = i + 1;
        }
        if (space_before === s.length) {
            return { space_before, space_after: 0 };
        }

        let space_after = 0;
        for (let i = s.length - 1; i >= 0; i--) {
            if (s[i] !== " ") {
                space_after = s.length - 1 - i;
                break;
            }
            space_after = i + 1;
        }

        return { space_before, space_after };
    },
    /**
     * Helper function for `fastn_utils.markdown_inline` to replace the last
     * occurrence of a substring in a string.
     *
     * @param {string} s - The input string.
     * @param {string} old_word - The substring to be replaced.
     * @param {string} new_word - The replacement substring.
     * @returns {string} - The string with the last occurrence of 'old_word' replaced by 'new_word'.
     */
    replace_last_occurrence(s, old_word, new_word) {
        if (!s.includes(old_word)) {
            return s;
        }

        const idx = s.lastIndexOf(old_word);
        return s.slice(0, idx) + new_word + s.slice(idx + old_word.length);
    },
    /**
     * Helper function for `fastn_utils.markdown_inline` to generate a string
     * containing a specified number of spaces.
     *
     * @param {number} n - The number of spaces to be generated.
     * @returns {string} - A string with 'n' spaces concatenated together.
     */
    repeated_space(n) {
        return Array.from({ length: n }, () => " ").join("");
    },
    /**
     * Merges consecutive numbers in a comma-separated list into ranges.
     *
     * @param {string} input - Comma-separated list of numbers.
     * @returns {string} Merged number ranges.
     *
     * @example
     * const input = '1,2,3,5,6,7,8,9,11';
     * const output = mergeNumbers(input);
     * console.log(output); // Output: '1-3,5-9,11'
     */
    mergeNumbers(numbers) {
        if (numbers.length === 0) {
            return "";
        }
        const mergedRanges = [];

        let start = numbers[0];
        let end = numbers[0];

        for (let i = 1; i < numbers.length; i++) {
            if (numbers[i] === end + 1) {
                end = numbers[i];
            } else {
                if (start === end) {
                    mergedRanges.push(start.toString());
                } else {
                    mergedRanges.push(`${start}-${end}`);
                }
                start = end = numbers[i];
            }
        }

        if (start === end) {
            mergedRanges.push(start.toString());
        } else {
            mergedRanges.push(`${start}-${end}`);
        }

        return mergedRanges.join(",");
    },
    addUnderscoreToStart(text) {
        if (/^\d/.test(text)) {
            return "_" + text;
        }
        return text;
    },

    /**
     * Replaces the children of a parent element with the children from a
     * new children wrapper.
     *
     * @param {HTMLElement} parent - The parent element whose children will
     * be replaced.
     * @param {HTMLElement} newChildrenWrapper - The wrapper element
     * containing the new children.
     * @returns {void}
     */
    replaceChildren(parent, newChildrenWrapper) {
        // Remove existing children of the parent
        var children = parent.children;
        // Loop through the direct children and remove those with tagName 'div'
        for (var i = children.length - 1; i >= 0; i--) {
            var child = children[i];
            if (child.tagName === "DIV") {
                parent.removeChild(child);
            }
        }

        // Cut and append the children from newChildrenWrapper to the parent
        while (newChildrenWrapper.firstChild) {
            parent.appendChild(newChildrenWrapper.firstChild);
        }
    },

    // Cookie related functions ----------------------------------------------
    setCookie(cookieName, cookieValue) {
        cookieName = fastn_utils.getStaticValue(cookieName);
        cookieValue = fastn_utils.getStaticValue(cookieValue);

        // Default expiration period of 30 days
        var expires = "";
        var expirationDays = 30;
        if (expirationDays) {
            var date = new Date();
            date.setTime(date.getTime() + expirationDays * 24 * 60 * 60 * 1000);
            expires = "; expires=" + date.toUTCString();
        }

        document.cookie =
            cookieName +
            "=" +
            encodeURIComponent(cookieValue) +
            expires +
            "; path=/";
    },
    getCookie(cookieName) {
        cookieName = fastn_utils.getStaticValue(cookieName);
        var name = cookieName + "=";
        var decodedCookie = decodeURIComponent(document.cookie);
        var cookieArray = decodedCookie.split(";");

        for (var i = 0; i < cookieArray.length; i++) {
            var cookie = cookieArray[i].trim();
            if (cookie.indexOf(name) === 0) {
                return cookie.substring(name.length, cookie.length);
            }
        }

        return "None";
    },
};

/*Object.prototype.get = function(index) {
    return this[index];
}*/
let fastnVirtual = {};

let id_counter = 0;
let ssr = false;
let doubleBuffering = false;

class ClassList {
    #classes = [];
    add(item) {
        this.#classes.push(item);
    }

    remove(itemToRemove) {
        this.#classes.filter((item) => item !== itemToRemove);
    }
    toString() {
        return this.#classes.join(" ");
    }
    getClasses() {
        return this.#classes;
    }
}

class Node {
    id;
    #dataId;
    #tagName;
    #children;
    #attributes;
    constructor(id, tagName) {
        this.#tagName = tagName;
        this.#dataId = id;
        this.classList = new ClassList();
        this.#children = [];
        this.#attributes = {};
        this.innerHTML = "";
        this.style = {};
        this.onclick = null;
        this.id = null;
    }
    appendChild(c) {
        this.#children.push(c);
    }

    insertBefore(node, index) {
        this.#children.splice(index, 0, node);
    }

    getChildren() {
        return this.#children;
    }

    setAttribute(attribute, value) {
        this.#attributes[attribute] = value;
    }

    getAttribute(attribute) {
        return this.#attributes[attribute];
    }

    removeAttribute(attribute) {
        if (attribute in this.#attributes) delete this.#attributes[attribute];
    }

    // Caution: This is only supported in ssr mode
    updateTagName(tagName) {
        this.#tagName = tagName;
    }
    // Caution: This is only supported in ssr mode
    toHtmlAsString() {
        const openingTag = `<${
            this.#tagName
        }${this.getDataIdString()}${this.getIdString()}${this.getAttributesString()}${this.getClassString()}${this.getStyleString()}>`;
        const closingTag = `</${this.#tagName}>`;
        const innerHTML = this.innerHTML;
        const childNodes = this.#children
            .map((child) => child.toHtmlAsString())
            .join("");

        return `${openingTag}${innerHTML}${childNodes}${closingTag}`;
    }
    // Caution: This is only supported in ssr mode
    getDataIdString() {
        return ` data-id="${this.#dataId}"`;
    }
    // Caution: This is only supported in ssr mode
    getIdString() {
        return fastn_utils.isNull(this.id) ? "" : ` id="${this.id}"`;
    }
    // Caution: This is only supported in ssr mode
    getClassString() {
        const classList = this.classList.toString();
        return classList ? ` class="${classList}"` : "";
    }
    // Caution: This is only supported in ssr mode
    getStyleString() {
        const styleProperties = Object.entries(this.style)
            .map(([prop, value]) => `${prop}:${value}`)
            .join(";");
        return styleProperties ? ` style="${styleProperties}"` : "";
    }
    // Caution: This is only supported in ssr mode
    getAttributesString() {
        const nodeAttributes = Object.entries(this.#attributes)
            .map(([attribute, value]) => {
                if (value !== undefined && value !== null && value !== "") {
                    return `${attribute}=\"${value}\"`;
                }
                return `${attribute}`;
            })
            .join(" ");
        return nodeAttributes ? ` ${nodeAttributes}` : "";
    }
}

class Document2 {
    createElement(tagName) {
        id_counter++;

        if (ssr) {
            return new Node(id_counter, tagName);
        }

        if (tagName === "body") {
            return window.document.body;
        }

        if (fastn_utils.isWrapperNode(tagName)) {
            return window.document.createComment(fastn_dom.commentMessage);
        }
        if (fastn_utils.isCommentNode(tagName)) {
            return window.document.createComment(fastn_dom.commentMessage);
        }
        return window.document.createElement(tagName);
    }
}

fastnVirtual.document = new Document2();

function addClosureToBreakpointWidth() {
    let closure = fastn.closureWithoutExecute(function () {
        let current = ftd.get_device();
        let lastDevice = ftd.device.get();
        if (current === lastDevice) {
            return;
        }
        console.log("last_device", lastDevice, "current_device", current);
        ftd.device.set(current);
    });

    ftd.breakpoint_width.addClosure(closure);
}

fastnVirtual.doubleBuffer = function (main) {
    addClosureToBreakpointWidth();
    let parent = document.createElement("div");
    let current_device = ftd.get_device();
    ftd.device = fastn.mutable(current_device);
    doubleBuffering = true;
    fastnVirtual.root = parent;
    main(parent);
    fastn_utils.replaceBodyStyleAndChildren(parent);
    doubleBuffering = false;
    fastnVirtual.root = document.body;
};

fastnVirtual.ssr = function (main) {
    ssr = true;
    let body = fastnVirtual.document.createElement("body");
    main(body);
    ssr = false;
    id_counter = 0;
    return body.toHtmlAsString() + fastn_dom.getClassesAsString();
};
class MutableVariable {
    #value;
    constructor(value) {
        this.#value = value;
    }

    get() {
        return fastn_utils.getStaticValue(this.#value);
    }

    set(value) {
        this.#value.set(value);
    }
    // Todo: Remove closure when node is removed.
    on_change(func) {
        this.#value.addClosure(fastn.closureWithoutExecute(func));
    }
}

class MutableListVariable {
    #value;
    constructor(value) {
        this.#value = value;
    }
    get() {
        return fastn_utils.getStaticValue(this.#value);
    }
    set(index, list) {
        if (list === undefined) {
            this.#value.set(fastn_utils.staticToMutables(index));
            return;
        }
        this.#value.set(index, fastn_utils.staticToMutables(list));
    }
    insertAt(index, value) {
        this.#value.insertAt(index, fastn_utils.staticToMutables(value));
    }
    deleteAt(index) {
        this.#value.deleteAt(index);
    }
    push(value) {
        this.#value.push(value);
    }
    pop() {
        this.#value.pop();
    }
    clearAll() {
        this.#value.clearAll();
    }
    on_change(func) {
        this.#value.addClosure(fastn.closureWithoutExecute(func));
    }
}

class RecordVariable {
    #value;
    constructor(value) {
        this.#value = value;
    }

    get() {
        return fastn_utils.getStaticValue(this.#value);
    }

    set(record) {
        this.#value.set(fastn_utils.staticToMutables(record));
    }

    on_change(func) {
        this.#value.addClosure(fastn.closureWithoutExecute(func));
    }
}
class StaticVariable {
    #value;
    #closures;
    constructor(value) {
        this.#value = value;
        this.#closures = [];
        if (this.#value instanceof fastn.mutableClass) {
            this.#value.addClosure(
                fastn.closure(() =>
                    this.#closures.forEach((closure) => closure.update()),
                ),
            );
        }
    }

    get() {
        return fastn_utils.getStaticValue(this.#value);
    }

    on_change(func) {
        if (this.#value instanceof fastn.mutableClass) {
            this.#value.addClosure(fastn.closure(func));
        }
    }
}

fastn.webComponentVariable = {
    mutable: (value) => {
        return new MutableVariable(value);
    },
    mutableList: (value) => {
        return new MutableListVariable(value);
    },
    static: (value) => {
        return new StaticVariable(value);
    },
    record: (value) => {
        return new RecordVariable(value);
    },
};
const ftd = (function () {
    const exports = {};

    const riveNodes = {};

    const global = {};

    const onLoadListeners = new Set();

    let fastnLoaded = false;

    exports.global = global;

    exports.riveNodes = riveNodes;

    exports.is_empty = (value) => {
        value = fastn_utils.getFlattenStaticValue(value);
        return fastn_utils.isNull(value) || value.length === 0;
    };

    exports.len = (data) => {
        if (!!data && data instanceof fastn.mutableListClass) {
            if (data.getLength) return data.getLength();
            return -1;
        }
        if (!!data && data instanceof fastn.mutableClass) {
            let inner_data = data.get();
            return exports.len(inner_data);
        }
        if (!!data && data.length) {
            return data.length;
        }
        return -2;
    };

    exports.copy_to_clipboard = (args) => {
        let text = args.a;
        if (text instanceof fastn.mutableClass)
            text = fastn_utils.getStaticValue(text);
        if (text.startsWith("\\", 0)) {
            text = text.substring(1);
        }
        if (!navigator.clipboard) {
            fallbackCopyTextToClipboard(text);
            return;
        }
        navigator.clipboard.writeText(text).then(
            function () {
                console.log("Async: Copying to clipboard was successful!");
            },
            function (err) {
                console.error("Async: Could not copy text: ", err);
            },
        );
    };

    // Todo: Implement this (Remove highlighter)
    exports.clean_code = (args) => args.a;

    exports.set_rive_boolean = (args, node) => {
        if (!!args.rive) {
            let riveNode = riveNodes[`${args.rive}__${exports.device.get()}`];
            node = riveNode ? riveNode : node;
        }
        let riveConst = node.getExtraData().rive;
        const stateMachineName = riveConst.stateMachineNames[0];
        const inputs = riveConst.stateMachineInputs(stateMachineName);
        const bumpTrigger = inputs.find((i) => i.name === args.input);
        bumpTrigger.value = args.value;
    };

    exports.toggle_rive_boolean = (args, node) => {
        if (!!args.rive) {
            let riveNode = riveNodes[`${args.rive}__${exports.device.get()}`];
            node = riveNode ? riveNode : node;
        }
        let riveConst = node.getExtraData().rive;
        const stateMachineName = riveConst.stateMachineNames[0];
        const inputs = riveConst.stateMachineInputs(stateMachineName);
        const trigger = inputs.find((i) => i.name === args.input);
        trigger.value = !trigger.value;
    };

    exports.set_rive_integer = (args, node) => {
        if (!!args.rive) {
            let riveNode = riveNodes[`${args.rive}__${exports.device.get()}`];
            node = riveNode ? riveNode : node;
        }
        let riveConst = node.getExtraData().rive;
        const stateMachineName = riveConst.stateMachineNames[0];
        const inputs = riveConst.stateMachineInputs(stateMachineName);
        const trigger = inputs.find((i) => i.name === args.input);
        trigger.value = args.value;
    };

    exports.fire_rive = (args, node) => {
        if (!!args.rive) {
            let riveNode = riveNodes[`${args.rive}__${exports.device.get()}`];
            node = riveNode ? riveNode : node;
        }
        let riveConst = node.getExtraData().rive;
        const stateMachineName = riveConst.stateMachineNames[0];
        const inputs = riveConst.stateMachineInputs(stateMachineName);
        const trigger = inputs.find((i) => i.name === args.input);
        trigger.fire();
    };

    exports.play_rive = (args, node) => {
        if (!!args.rive) {
            let riveNode = riveNodes[`${args.rive}__${exports.device.get()}`];
            node = riveNode ? riveNode : node;
        }
        node.getExtraData().rive.play(args.input);
    };

    exports.pause_rive = (args, node) => {
        if (!!args.rive) {
            let riveNode = riveNodes[`${args.rive}__${exports.device.get()}`];
            node = riveNode ? riveNode : node;
        }
        node.getExtraData().rive.pause(args.input);
    };

    exports.toggle_play_rive = (args, node) => {
        if (!!args.rive) {
            let riveNode = riveNodes[`${args.rive}__${exports.device.get()}`];
            node = riveNode ? riveNode : node;
        }
        let riveConst = node.getExtraData().rive;
        riveConst.playingAnimationNames.includes(args.input)
            ? riveConst.pause(args.input)
            : riveConst.play(args.input);
    };

    exports.get = (value, index) => {
        return fastn_utils.getStaticValue(
            fastn_utils.getterByKey(value, index),
        );
    };

    exports.component_data = (component) => {
        let attributesIndex = component.getAttribute(
            fastn_dom.webComponentArgument,
        );
        let attributes = fastn_dom.webComponent[attributesIndex];
        return Object.fromEntries(
            Object.entries(attributes).map(([k, v]) => {
                // Todo: check if argument is mutable reference or not
                if (v instanceof fastn.mutableClass) {
                    v = fastn.webComponentVariable.mutable(v);
                } else if (v instanceof fastn.mutableListClass) {
                    v = fastn.webComponentVariable.mutableList(v);
                } else if (v instanceof fastn.recordInstanceClass) {
                    v = fastn.webComponentVariable.record(v);
                } else {
                    v = fastn.webComponentVariable.static(v);
                }
                return [k, v];
            }),
        );
    };

    exports.append = function (list, item) {
        list.push(item);
    };
    exports.pop = function (list) {
        list.pop();
    };
    exports.insert_at = function (list, index, item) {
        list.insertAt(index, item);
    };
    exports.delete_at = function (list, index) {
        list.deleteAt(index);
    };
    exports.clear_all = function (list) {
        list.clearAll();
    };
    exports.clear = exports.clear_all;
    exports.set_list = function (list, value) {
        list.set(value);
    };

    exports.http = function (url, opts, ...body) {
        if ((!opts) instanceof fastn.recordInstanceClass) {
            console.info(`opts must be a record instance of
                -- record ftd.http-options:
                string method: GET
                string redirect: manual
                string fastn-module:
            `);
            throw new Error("invalid opts");
        }

        let method = opts.get("method").get();
        let fastn_module = opts.get("fastn_module").get();
        let redirect = opts.get("redirect").get();

        if (!["manual", "follow", "error"].includes(redirect)) {
            throw new Error(
                `redirect must be one of "manual", "follow", "error"`,
            );
        }

        if (url instanceof fastn.mutableClass) url = url.get();
        method = method.trim().toUpperCase();
        let request_json = {};

        const init = {
            method,
            headers: { "Content-Type": "application/json" },
            json: null,
            redirect,
        };

        if (method === "GET") {
            console.warn("Method `GET` is not yet supported.");
            return;
        }


        if (body && method !== "GET") {
            if (body[0] instanceof fastn.recordInstanceClass) {
                if (body.length !== 1) {
                    console.warn(
                        "body is a record instance, but has more than 1 element, ignoring",
                    );
                }
                request_json = body[0].toObject();
            } else {
                let json = body[0];
                if (
                    body.length !== 1 ||
                    (body[0].length === 2 && Array.isArray(body[0]))
                ) {
                    let new_json = {};
                    // @ts-ignore
                    for (let [header, value] of Object.entries(body)) {
                        let [key, val] =
                            value.length === 2 ? value : [header, value];
                        new_json[key] = fastn_utils.getStaticValue(val);
                    }
                    json = new_json;
                }
                request_json = json;
            }
        }

        init.body = JSON.stringify(request_json);

        let json;
        fetch(url, init)
            .then((res) => {
                if (res.redirected) {
                    window.location.href = res.url;
                    return;
                }

                if (!res.ok) {
                    return new Error("[http]: Request failed", res);
                }

                return res.json();
            })
            .then((response) => {
                console.log("[http]: Response OK", response);
                if (response.redirect) {
                    window.location.href = response.redirect;
                } else if (!!response && !!response.reload) {
                    window.location.reload();
                } else {
                    let data = {};
                    if (!!response.errors) {
                        for (let key of Object.keys(response.errors)) {
                            let value = response.errors[key];
                            if (Array.isArray(value)) {
                                // django returns a list of strings
                                value = value.join(" ");
                            }
                            // also django does not append `-error`
                            key = key + "-error";
                            key = fastn_module + "#" + key;
                            data[key] = value;
                        }
                    }
                    if (!!response.data) {
                        if (Object.keys(data).length !== 0) {
                            console.log(
                                "both .errors and .data are present in response, ignoring .data",
                            );
                        } else {
                            data = response.data;
                        }
                    }
                    for (let ftd_variable of Object.keys(data)) {
                        // @ts-ignore
                        window.ftd.set_value(ftd_variable, data[ftd_variable]);
                    }
                }
            })
            .catch(console.error);
        return json;
    };

    exports.navigate = function (url, request_data) {
        let query_parameters = new URLSearchParams();
        if (request_data instanceof fastn.recordInstanceClass) {
            // @ts-ignore
            for (let [header, value] of Object.entries(
                request_data.toObject(),
            )) {
                let [key, val] = value.length === 2 ? value : [header, value];
                query_parameters.set(key, val);
            }
        }
        let query_string = query_parameters.toString();
        if (query_string) {
            window.location.href = url + "?" + query_parameters.toString();
        } else {
            window.location.href = url;
        }
    };

    exports.toggle_dark_mode = function () {
        const is_dark_mode = exports.get(exports.dark_mode);
        if (is_dark_mode) {
            enable_light_mode();
        } else {
            enable_dark_mode();
        }
    };

    exports.local_storage = {
        _get_key(key) {
            if (key instanceof fastn.mutableClass) {
                key = key.get();
            }
            const packageNamePrefix = __fastn_package_name__
                ? `${__fastn_package_name__}_`
                : "";
            const snakeCaseKey = fastn_utils.toSnakeCase(key);

            return `${packageNamePrefix}${snakeCaseKey}`;
        },
        set(key, value) {
            key = this._get_key(key);
            value = fastn_utils.getFlattenStaticValue(value);
            localStorage.setItem(
                key,
                value && typeof value === "object"
                    ? JSON.stringify(value)
                    : value,
            );
        },
        get(key) {
            key = this._get_key(key);
            if (ssr) {
                return;
            }
            const item = localStorage.getItem(key);
            if (!item) {
                return;
            }
            try {
                const obj = JSON.parse(item);

                return fastn_utils.staticToMutables(obj);
            } catch {
                return item;
            }
        },
        delete(key) {
            key = this._get_key(key);
            localStorage.removeItem(key);
        },
    };

    exports.on_load = (listener) => {
        if (typeof listener !== "function") {
            throw new Error("listener must be a function");
        }

        if (fastnLoaded) {
            listener();
            return;
        }

        onLoadListeners.add(listener);
    };

    exports.emit_on_load = () => {
        if (fastnLoaded) return;

        fastnLoaded = true;
        onLoadListeners.forEach((listener) => listener());
    };

    // LEGACY

    function legacyNameToJS(s) {
        let name = s.toString();

        if (name[0].charCodeAt(0) >= 48 && name[0].charCodeAt(0) <= 57) {
            name = "_" + name;
        }

        return name
            .replaceAll("#", "__")
            .replaceAll("-", "_")
            .replaceAll(":", "___")
            .replaceAll(",", "$")
            .replaceAll("\\", "/")
            .replaceAll("/", "_")
            .replaceAll(".", "_");
    }

    function getDocNameAndRemaining(s) {
        let part1 = "";
        let patternToSplitAt = s;

        const split1 = s.split("#");
        if (split1.length === 2) {
            part1 = split1[0] + "#";
            patternToSplitAt = split1[1];
        }

        const split2 = patternToSplitAt.split(".");
        if (split2.length === 2) {
            return [part1 + split2[0], split2[1]];
        } else {
            return [s, null];
        }
    }

    function isMutable(obj) {
        return (
            obj instanceof fastn.mutableClass ||
            obj instanceof fastn.mutableListClass ||
            obj instanceof fastn.recordInstanceClass
        );
    }

    exports.set_value = function (variable, value) {
        const [var_name, remaining] = getDocNameAndRemaining(variable);
        let name = legacyNameToJS(var_name);
        if (global[name] === undefined) {
            console.log(
                `[ftd-legacy]: ${variable} is not in global map, ignoring`,
            );
            return;
        }
        const mutable = global[name];
        if (!isMutable(mutable)) {
            console.log(`[ftd-legacy]: ${variable} is not a mutable, ignoring`);
            return;
        }
        if (remaining) {
            mutable.get(remaining).set(value);
        } else {
            mutable.set(value);
        }
    };

    exports.get_value = function (variable) {
        const [var_name, remaining] = getDocNameAndRemaining(variable);
        let name = legacyNameToJS(var_name);
        if (global[name] === undefined) {
            console.log(
                `[ftd-legacy]: ${variable} is not in global map, ignoring`,
            );
            return;
        }
        const value = global[name];
        if (isMutable(value)) {
            if (remaining) {
                return value.get(remaining);
            } else {
                return value.get();
            }
        } else {
            return value;
        }
    };

    // Language related functions ---------------------------------------------
    exports.set_current_language = function (language) {
        language = fastn_utils.getStaticValue(language);
        fastn_utils.private.setCookie("fastn-lang", language);
        location.reload();
    };

    exports.get_current_language = function () {
        return fastn_utils.private.getCookie("fastn-lang");
    };

    return exports;
})();

const len = ftd.len;

const global = ftd.global;
ftd.clickOutsideEvents = [];
ftd.globalKeyEvents = [];
ftd.globalKeySeqEvents = [];

ftd.get_device = function () {
    const MOBILE_CLASS = "mobile";
    // not at all sure about this function logic.
    let width = window.innerWidth;
    // In the future, we may want to have more than one break points, and
    // then we may also want the theme builders to decide where the
    // breakpoints should go. we should be able to fetch fpm variables
    // here, or maybe simply pass the width, user agent etc. to fpm and
    // let people put the checks on width user agent etc., but it would
    // be good if we can standardize few breakpoints. or maybe we should
    // do both, some standard breakpoints and pass the raw data.
    // we would then rename this function to detect_device() which will
    // return one of "desktop", "mobile". and also maybe have another
    // function detect_orientation(), "landscape" and "portrait" etc.,
    // and instead of setting `ftd#mobile: boolean` we set `ftd#device`
    // and `ftd#view-port-orientation` etc.
    let mobile_breakpoint = fastn_utils.getStaticValue(
        ftd.breakpoint_width.get("mobile"),
    );
    if (width <= mobile_breakpoint) {
        document.body.classList.add(MOBILE_CLASS);
        return fastn_dom.DeviceData.Mobile;
    }
    if (document.body.classList.contains(MOBILE_CLASS)) {
        document.body.classList.remove(MOBILE_CLASS);
    }
    return fastn_dom.DeviceData.Desktop;
};

ftd.post_init = function () {
    const DARK_MODE_COOKIE = "fastn-dark-mode";
    const COOKIE_SYSTEM_LIGHT = "system-light";
    const COOKIE_SYSTEM_DARK = "system-dark";
    const COOKIE_DARK_MODE = "dark";
    const COOKIE_LIGHT_MODE = "light";
    const DARK_MODE_CLASS = "dark";
    let last_device = ftd.device.get();

    window.onresize = function () {
        initialise_device();
    };
    function initialise_click_outside_events() {
        document.addEventListener("click", function (event) {
            ftd.clickOutsideEvents.forEach(([ftdNode, func]) => {
                let node = ftdNode.getNode();
                if (
                    !!node &&
                    node.style.display !== "none" &&
                    !node.contains(event.target)
                ) {
                    func();
                }
            });
        });
    }
    function initialise_global_key_events() {
        let globalKeys = {};
        let buffer = [];
        let lastKeyTime = Date.now();

        document.addEventListener("keydown", function (event) {
            let eventKey = fastn_utils.getEventKey(event);
            globalKeys[eventKey] = true;
            const currentTime = Date.now();
            if (currentTime - lastKeyTime > 1000) {
                buffer = [];
            }
            lastKeyTime = currentTime;
            if (
                (event.target.nodeName === "INPUT" ||
                    event.target.nodeName === "TEXTAREA") &&
                eventKey !== "ArrowDown" &&
                eventKey !== "ArrowUp" &&
                eventKey !== "ArrowRight" &&
                eventKey !== "ArrowLeft" &&
                event.target.nodeName === "INPUT" &&
                eventKey !== "Enter"
            ) {
                return;
            }
            buffer.push(eventKey);

            ftd.globalKeyEvents.forEach(([_ftdNode, func, array]) => {
                let globalKeysPresent = array.reduce(
                    (accumulator, currentValue) =>
                        accumulator && !!globalKeys[currentValue],
                    true,
                );
                if (
                    globalKeysPresent &&
                    buffer.join(",").includes(array.join(","))
                ) {
                    func();
                    globalKeys[eventKey] = false;
                    buffer = [];
                }
                return;
            });

            ftd.globalKeySeqEvents.forEach(([_ftdNode, func, array]) => {
                if (buffer.join(",").includes(array.join(","))) {
                    func();
                    globalKeys[eventKey] = false;
                    buffer = [];
                }
                return;
            });
        });

        document.addEventListener("keyup", function (event) {
            globalKeys[fastn_utils.getEventKey(event)] = false;
        });
    }
    function initialise_device() {
        let current = ftd.get_device();
        if (current === last_device) {
            return;
        }
        console.log("last_device", last_device, "current_device", current);
        ftd.device.set(current);
        last_device = current;
    }

    /*
        ftd.dark-mode behaviour:

        ftd.dark-mode is a boolean, default false, it tells the UI to show
        the UI in dark or light mode. Themes should use this variable to decide
        which mode to show in UI.

        ftd.follow-system-dark-mode, boolean, default true, keeps track if
        we are reading the value of `dark-mode` from system preference, or user
        has overridden the system preference.

        These two variables must not be set by ftd code directly, but they must
        use `$on-click$: message-host enable-dark-mode`, to ignore system
        preference and use dark mode. `$on-click$: message-host
        disable-dark-mode` to ignore system preference and use light mode and
        `$on-click$: message-host follow-system-dark-mode` to ignore user
        preference and start following system preference.

        we use a cookie: `ftd-dark-mode` to store the preference. The cookie can
        have three values:

           cookie missing /          user wants us to honour system preference
               system-light          and currently its light.

           system-dark               follow system and currently its dark.

           light:                    user prefers light

           dark:                     user prefers light

        We use cookie instead of localstorage so in future `fpm-repo` can see
        users preferences up front and renders the HTML on service wide
        following user's preference.

     */
    window.enable_dark_mode = function () {
        // TODO: coalesce the two set_bool-s into one so there is only one DOM
        //       update
        ftd.dark_mode.set(true);
        ftd.follow_system_dark_mode.set(false);
        ftd.system_dark_mode.set(system_dark_mode());
        document.body.classList.add(DARK_MODE_CLASS);
        set_cookie(DARK_MODE_COOKIE, COOKIE_DARK_MODE);
    };
    window.enable_light_mode = function () {
        // TODO: coalesce the two set_bool-s into one so there is only one DOM
        //       update
        ftd.dark_mode.set(false);
        ftd.follow_system_dark_mode.set(false);
        ftd.system_dark_mode.set(system_dark_mode());
        if (document.body.classList.contains(DARK_MODE_CLASS)) {
            document.body.classList.remove(DARK_MODE_CLASS);
        }
        set_cookie(DARK_MODE_COOKIE, COOKIE_LIGHT_MODE);
    };
    window.enable_system_mode = function () {
        // TODO: coalesce the two set_bool-s into one so there is only one DOM
        //       update
        let systemMode = system_dark_mode();
        ftd.follow_system_dark_mode.set(true);
        ftd.system_dark_mode.set(systemMode);
        if (systemMode) {
            ftd.dark_mode.set(true);
            document.body.classList.add(DARK_MODE_CLASS);
            set_cookie(DARK_MODE_COOKIE, COOKIE_SYSTEM_DARK);
        } else {
            ftd.dark_mode.set(false);
            if (document.body.classList.contains(DARK_MODE_CLASS)) {
                document.body.classList.remove(DARK_MODE_CLASS);
            }
            set_cookie(DARK_MODE_COOKIE, COOKIE_SYSTEM_LIGHT);
        }
    };
    function set_cookie(name, value) {
        document.cookie = name + "=" + value + "; path=/";
    }
    function system_dark_mode() {
        return !!(
            window.matchMedia &&
            window.matchMedia("(prefers-color-scheme: dark)").matches
        );
    }
    function initialise_dark_mode() {
        update_dark_mode();
        start_watching_dark_mode_system_preference();
    }
    function get_cookie(name, def) {
        // source: https://stackoverflow.com/questions/5639346/
        let regex = document.cookie.match(
            "(^|;)\\s*" + name + "\\s*=\\s*([^;]+)",
        );
        return regex !== null ? regex.pop() : def;
    }
    function update_dark_mode() {
        let current_dark_mode_cookie = get_cookie(
            DARK_MODE_COOKIE,
            COOKIE_SYSTEM_LIGHT,
        );
        switch (current_dark_mode_cookie) {
            case COOKIE_SYSTEM_LIGHT:
            case COOKIE_SYSTEM_DARK:
                window.enable_system_mode();
                break;
            case COOKIE_LIGHT_MODE:
                window.enable_light_mode();
                break;
            case COOKIE_DARK_MODE:
                window.enable_dark_mode();
                break;
            default:
                console_log("cookie value is wrong", current_dark_mode_cookie);
                window.enable_system_mode();
        }
    }
    function start_watching_dark_mode_system_preference() {
        window
            .matchMedia("(prefers-color-scheme: dark)")
            .addEventListener("change", update_dark_mode);
    }
    initialise_device();
    initialise_dark_mode();
    initialise_click_outside_events();
    initialise_global_key_events();
    fastn_utils.resetFullHeight();
    fastn_utils.setFullHeight();
};

window.ftd = ftd;

ftd.toggle = function (args) {
  let __fastn_super_package_name__ = __fastn_package_name__;
  __fastn_package_name__ = "fastn_community_github_io_business_card_demo";
  try {
    let __args__ = fastn_utils.getArgs({
    }, args);
    let fastn_utils_val___args___a = fastn_utils.clone(!fastn_utils.getStaticValue(__args__.a));
    if (fastn_utils_val___args___a instanceof fastn.mutableClass) {
      fastn_utils_val___args___a = fastn_utils_val___args___a.get();
    }
    if (!fastn_utils.setter(__args__.a, fastn_utils_val___args___a)) {
      __args__.a = fastn_utils_val___args___a;
    }
  } finally {
    __fastn_package_name__ = __fastn_super_package_name__;
  }
}
ftd.increment = function (args) {
  let __fastn_super_package_name__ = __fastn_package_name__;
  __fastn_package_name__ = "fastn_community_github_io_business_card_demo";
  try {
    let __args__ = fastn_utils.getArgs({
    }, args);
    let fastn_utils_val___args___a = fastn_utils.clone(fastn_utils.getStaticValue(__args__.a) + 1);
    if (fastn_utils_val___args___a instanceof fastn.mutableClass) {
      fastn_utils_val___args___a = fastn_utils_val___args___a.get();
    }
    if (!fastn_utils.setter(__args__.a, fastn_utils_val___args___a)) {
      __args__.a = fastn_utils_val___args___a;
    }
  } finally {
    __fastn_package_name__ = __fastn_super_package_name__;
  }
}
ftd.increment_by = function (args) {
  let __fastn_super_package_name__ = __fastn_package_name__;
  __fastn_package_name__ = "fastn_community_github_io_business_card_demo";
  try {
    let __args__ = fastn_utils.getArgs({
    }, args);
    let fastn_utils_val___args___a = fastn_utils.clone(fastn_utils.getStaticValue(__args__.a) + fastn_utils.getStaticValue(__args__.v));
    if (fastn_utils_val___args___a instanceof fastn.mutableClass) {
      fastn_utils_val___args___a = fastn_utils_val___args___a.get();
    }
    if (!fastn_utils.setter(__args__.a, fastn_utils_val___args___a)) {
      __args__.a = fastn_utils_val___args___a;
    }
  } finally {
    __fastn_package_name__ = __fastn_super_package_name__;
  }
}
ftd.decrement = function (args) {
  let __fastn_super_package_name__ = __fastn_package_name__;
  __fastn_package_name__ = "fastn_community_github_io_business_card_demo";
  try {
    let __args__ = fastn_utils.getArgs({
    }, args);
    let fastn_utils_val___args___a = fastn_utils.clone(fastn_utils.getStaticValue(__args__.a) - 1);
    if (fastn_utils_val___args___a instanceof fastn.mutableClass) {
      fastn_utils_val___args___a = fastn_utils_val___args___a.get();
    }
    if (!fastn_utils.setter(__args__.a, fastn_utils_val___args___a)) {
      __args__.a = fastn_utils_val___args___a;
    }
  } finally {
    __fastn_package_name__ = __fastn_super_package_name__;
  }
}
ftd.decrement_by = function (args) {
  let __fastn_super_package_name__ = __fastn_package_name__;
  __fastn_package_name__ = "fastn_community_github_io_business_card_demo";
  try {
    let __args__ = fastn_utils.getArgs({
    }, args);
    let fastn_utils_val___args___a = fastn_utils.clone(fastn_utils.getStaticValue(__args__.a) - fastn_utils.getStaticValue(__args__.v));
    if (fastn_utils_val___args___a instanceof fastn.mutableClass) {
      fastn_utils_val___args___a = fastn_utils_val___args___a.get();
    }
    if (!fastn_utils.setter(__args__.a, fastn_utils_val___args___a)) {
      __args__.a = fastn_utils_val___args___a;
    }
  } finally {
    __fastn_package_name__ = __fastn_super_package_name__;
  }
}
ftd.enable_light_mode = function (args) {
  let __fastn_super_package_name__ = __fastn_package_name__;
  __fastn_package_name__ = "fastn_community_github_io_business_card_demo";
  try {
    let __args__ = fastn_utils.getArgs({
    }, args);
    return (enable_light_mode());
  } finally {
    __fastn_package_name__ = __fastn_super_package_name__;
  }
}
ftd.enable_dark_mode = function (args) {
  let __fastn_super_package_name__ = __fastn_package_name__;
  __fastn_package_name__ = "fastn_community_github_io_business_card_demo";
  try {
    let __args__ = fastn_utils.getArgs({
    }, args);
    return (enable_dark_mode());
  } finally {
    __fastn_package_name__ = __fastn_super_package_name__;
  }
}
ftd.enable_system_mode = function (args) {
  let __fastn_super_package_name__ = __fastn_package_name__;
  __fastn_package_name__ = "fastn_community_github_io_business_card_demo";
  try {
    let __args__ = fastn_utils.getArgs({
    }, args);
    return (enable_system_mode());
  } finally {
    __fastn_package_name__ = __fastn_super_package_name__;
  }
}
ftd.set_bool = function (args) {
  let __fastn_super_package_name__ = __fastn_package_name__;
  __fastn_package_name__ = "fastn_community_github_io_business_card_demo";
  try {
    let __args__ = fastn_utils.getArgs({
    }, args);
    let fastn_utils_val___args___a = fastn_utils.clone(__args__.v);
    if (fastn_utils_val___args___a instanceof fastn.mutableClass) {
      fastn_utils_val___args___a = fastn_utils_val___args___a.get();
    }
    if (!fastn_utils.setter(__args__.a, fastn_utils_val___args___a)) {
      __args__.a = fastn_utils_val___args___a;
    }
  } finally {
    __fastn_package_name__ = __fastn_super_package_name__;
  }
}
ftd.set_boolean = function (args) {
  let __fastn_super_package_name__ = __fastn_package_name__;
  __fastn_package_name__ = "fastn_community_github_io_business_card_demo";
  try {
    let __args__ = fastn_utils.getArgs({
    }, args);
    let fastn_utils_val___args___a = fastn_utils.clone(__args__.v);
    if (fastn_utils_val___args___a instanceof fastn.mutableClass) {
      fastn_utils_val___args___a = fastn_utils_val___args___a.get();
    }
    if (!fastn_utils.setter(__args__.a, fastn_utils_val___args___a)) {
      __args__.a = fastn_utils_val___args___a;
    }
  } finally {
    __fastn_package_name__ = __fastn_super_package_name__;
  }
}
ftd.set_string = function (args) {
  let __fastn_super_package_name__ = __fastn_package_name__;
  __fastn_package_name__ = "fastn_community_github_io_business_card_demo";
  try {
    let __args__ = fastn_utils.getArgs({
    }, args);
    let fastn_utils_val___args___a = fastn_utils.clone(__args__.v);
    if (fastn_utils_val___args___a instanceof fastn.mutableClass) {
      fastn_utils_val___args___a = fastn_utils_val___args___a.get();
    }
    if (!fastn_utils.setter(__args__.a, fastn_utils_val___args___a)) {
      __args__.a = fastn_utils_val___args___a;
    }
  } finally {
    __fastn_package_name__ = __fastn_super_package_name__;
  }
}
ftd.set_integer = function (args) {
  let __fastn_super_package_name__ = __fastn_package_name__;
  __fastn_package_name__ = "fastn_community_github_io_business_card_demo";
  try {
    let __args__ = fastn_utils.getArgs({
    }, args);
    let fastn_utils_val___args___a = fastn_utils.clone(__args__.v);
    if (fastn_utils_val___args___a instanceof fastn.mutableClass) {
      fastn_utils_val___args___a = fastn_utils_val___args___a.get();
    }
    if (!fastn_utils.setter(__args__.a, fastn_utils_val___args___a)) {
      __args__.a = fastn_utils_val___args___a;
    }
  } finally {
    __fastn_package_name__ = __fastn_super_package_name__;
  }
}
ftd.dark_mode = fastn.mutable(false);
ftd.empty = "";
ftd.space = " ";
ftd.nbsp = "&nbsp;";
ftd.non_breaking_space = "&nbsp;";
ftd.system_dark_mode = fastn.mutable(false);
ftd.follow_system_dark_mode = fastn.mutable(true);
ftd.font_display = fastn.mutable("sans-serif");
ftd.font_copy = fastn.mutable("sans-serif");
ftd.font_code = fastn.mutable("sans-serif");
ftd.default_types = function () {
  let record = fastn.recordInstance({
  });
  record.set("heading_large", function () {
    let record = fastn.recordInstance({
    });
    record.set("desktop", function () {
      let record = fastn.recordInstance({
      });
      record.set("size", fastn_dom.FontSize.Px(50));
      record.set("line_height", fastn_dom.FontSize.Px(65));
      record.set("letter_spacing", null);
      record.set("weight", 400);
      record.set("font_family", ftd.font_display);
      return record;
    }());
    record.set("mobile", function () {
      let record = fastn.recordInstance({
      });
      record.set("size", fastn_dom.FontSize.Px(36));
      record.set("line_height", fastn_dom.FontSize.Px(54));
      record.set("letter_spacing", null);
      record.set("weight", 400);
      record.set("font_family", ftd.font_display);
      return record;
    }());
    return record;
  }());
  record.set("heading_medium", function () {
    let record = fastn.recordInstance({
    });
    record.set("desktop", function () {
      let record = fastn.recordInstance({
      });
      record.set("size", fastn_dom.FontSize.Px(38));
      record.set("line_height", fastn_dom.FontSize.Px(57));
      record.set("letter_spacing", null);
      record.set("weight", 400);
      record.set("font_family", ftd.font_display);
      return record;
    }());
    record.set("mobile", function () {
      let record = fastn.recordInstance({
      });
      record.set("size", fastn_dom.FontSize.Px(26));
      record.set("line_height", fastn_dom.FontSize.Px(40));
      record.set("letter_spacing", null);
      record.set("weight", 400);
      record.set("font_family", ftd.font_display);
      return record;
    }());
    return record;
  }());
  record.set("heading_small", function () {
    let record = fastn.recordInstance({
    });
    record.set("desktop", function () {
      let record = fastn.recordInstance({
      });
      record.set("size", fastn_dom.FontSize.Px(24));
      record.set("line_height", fastn_dom.FontSize.Px(31));
      record.set("letter_spacing", null);
      record.set("weight", 400);
      record.set("font_family", ftd.font_display);
      return record;
    }());
    record.set("mobile", function () {
      let record = fastn.recordInstance({
      });
      record.set("size", fastn_dom.FontSize.Px(22));
      record.set("line_height", fastn_dom.FontSize.Px(29));
      record.set("letter_spacing", null);
      record.set("weight", 400);
      record.set("font_family", ftd.font_display);
      return record;
    }());
    return record;
  }());
  record.set("heading_hero", function () {
    let record = fastn.recordInstance({
    });
    record.set("desktop", function () {
      let record = fastn.recordInstance({
      });
      record.set("size", fastn_dom.FontSize.Px(80));
      record.set("line_height", fastn_dom.FontSize.Px(104));
      record.set("letter_spacing", null);
      record.set("weight", 400);
      record.set("font_family", ftd.font_display);
      return record;
    }());
    record.set("mobile", function () {
      let record = fastn.recordInstance({
      });
      record.set("size", fastn_dom.FontSize.Px(48));
      record.set("line_height", fastn_dom.FontSize.Px(64));
      record.set("letter_spacing", null);
      record.set("weight", 400);
      record.set("font_family", ftd.font_display);
      return record;
    }());
    return record;
  }());
  record.set("heading_tiny", function () {
    let record = fastn.recordInstance({
    });
    record.set("desktop", function () {
      let record = fastn.recordInstance({
      });
      record.set("size", fastn_dom.FontSize.Px(20));
      record.set("line_height", fastn_dom.FontSize.Px(26));
      record.set("letter_spacing", null);
      record.set("weight", 400);
      record.set("font_family", ftd.font_display);
      return record;
    }());
    record.set("mobile", function () {
      let record = fastn.recordInstance({
      });
      record.set("size", fastn_dom.FontSize.Px(18));
      record.set("line_height", fastn_dom.FontSize.Px(24));
      record.set("letter_spacing", null);
      record.set("weight", 400);
      record.set("font_family", ftd.font_display);
      return record;
    }());
    return record;
  }());
  record.set("copy_small", function () {
    let record = fastn.recordInstance({
    });
    record.set("desktop", function () {
      let record = fastn.recordInstance({
      });
      record.set("size", fastn_dom.FontSize.Px(14));
      record.set("line_height", fastn_dom.FontSize.Px(24));
      record.set("letter_spacing", null);
      record.set("weight", 400);
      record.set("font_family", ftd.font_copy);
      return record;
    }());
    record.set("mobile", function () {
      let record = fastn.recordInstance({
      });
      record.set("size", fastn_dom.FontSize.Px(12));
      record.set("line_height", fastn_dom.FontSize.Px(16));
      record.set("letter_spacing", null);
      record.set("weight", 400);
      record.set("font_family", ftd.font_copy);
      return record;
    }());
    return record;
  }());
  record.set("copy_regular", function () {
    let record = fastn.recordInstance({
    });
    record.set("desktop", function () {
      let record = fastn.recordInstance({
      });
      record.set("size", fastn_dom.FontSize.Px(18));
      record.set("line_height", fastn_dom.FontSize.Px(30));
      record.set("letter_spacing", null);
      record.set("weight", 400);
      record.set("font_family", ftd.font_copy);
      return record;
    }());
    record.set("mobile", function () {
      let record = fastn.recordInstance({
      });
      record.set("size", fastn_dom.FontSize.Px(16));
      record.set("line_height", fastn_dom.FontSize.Px(24));
      record.set("letter_spacing", null);
      record.set("weight", 400);
      record.set("font_family", ftd.font_copy);
      return record;
    }());
    return record;
  }());
  record.set("copy_large", function () {
    let record = fastn.recordInstance({
    });
    record.set("desktop", function () {
      let record = fastn.recordInstance({
      });
      record.set("size", fastn_dom.FontSize.Px(22));
      record.set("line_height", fastn_dom.FontSize.Px(34));
      record.set("letter_spacing", null);
      record.set("weight", 400);
      record.set("font_family", ftd.font_copy);
      return record;
    }());
    record.set("mobile", function () {
      let record = fastn.recordInstance({
      });
      record.set("size", fastn_dom.FontSize.Px(18));
      record.set("line_height", fastn_dom.FontSize.Px(28));
      record.set("letter_spacing", null);
      record.set("weight", 400);
      record.set("font_family", ftd.font_copy);
      return record;
    }());
    return record;
  }());
  record.set("fine_print", function () {
    let record = fastn.recordInstance({
    });
    record.set("desktop", function () {
      let record = fastn.recordInstance({
      });
      record.set("size", fastn_dom.FontSize.Px(12));
      record.set("line_height", fastn_dom.FontSize.Px(16));
      record.set("letter_spacing", null);
      record.set("weight", 400);
      record.set("font_family", ftd.font_code);
      return record;
    }());
    record.set("mobile", function () {
      let record = fastn.recordInstance({
      });
      record.set("size", fastn_dom.FontSize.Px(12));
      record.set("line_height", fastn_dom.FontSize.Px(16));
      record.set("letter_spacing", null);
      record.set("weight", 400);
      record.set("font_family", ftd.font_code);
      return record;
    }());
    return record;
  }());
  record.set("blockquote", function () {
    let record = fastn.recordInstance({
    });
    record.set("desktop", function () {
      let record = fastn.recordInstance({
      });
      record.set("size", fastn_dom.FontSize.Px(16));
      record.set("line_height", fastn_dom.FontSize.Px(21));
      record.set("letter_spacing", null);
      record.set("weight", 400);
      record.set("font_family", ftd.font_code);
      return record;
    }());
    record.set("mobile", function () {
      let record = fastn.recordInstance({
      });
      record.set("size", fastn_dom.FontSize.Px(16));
      record.set("line_height", fastn_dom.FontSize.Px(21));
      record.set("letter_spacing", null);
      record.set("weight", 400);
      record.set("font_family", ftd.font_code);
      return record;
    }());
    return record;
  }());
  record.set("source_code", function () {
    let record = fastn.recordInstance({
    });
    record.set("desktop", function () {
      let record = fastn.recordInstance({
      });
      record.set("size", fastn_dom.FontSize.Px(18));
      record.set("line_height", fastn_dom.FontSize.Px(30));
      record.set("letter_spacing", null);
      record.set("weight", 400);
      record.set("font_family", ftd.font_code);
      return record;
    }());
    record.set("mobile", function () {
      let record = fastn.recordInstance({
      });
      record.set("size", fastn_dom.FontSize.Px(16));
      record.set("line_height", fastn_dom.FontSize.Px(21));
      record.set("letter_spacing", null);
      record.set("weight", 400);
      record.set("font_family", ftd.font_code);
      return record;
    }());
    return record;
  }());
  record.set("button_small", function () {
    let record = fastn.recordInstance({
    });
    record.set("desktop", function () {
      let record = fastn.recordInstance({
      });
      record.set("size", fastn_dom.FontSize.Px(14));
      record.set("line_height", fastn_dom.FontSize.Px(19));
      record.set("letter_spacing", null);
      record.set("weight", 400);
      record.set("font_family", ftd.font_display);
      return record;
    }());
    record.set("mobile", function () {
      let record = fastn.recordInstance({
      });
      record.set("size", fastn_dom.FontSize.Px(14));
      record.set("line_height", fastn_dom.FontSize.Px(19));
      record.set("letter_spacing", null);
      record.set("weight", 400);
      record.set("font_family", ftd.font_display);
      return record;
    }());
    return record;
  }());
  record.set("button_medium", function () {
    let record = fastn.recordInstance({
    });
    record.set("desktop", function () {
      let record = fastn.recordInstance({
      });
      record.set("size", fastn_dom.FontSize.Px(16));
      record.set("line_height", fastn_dom.FontSize.Px(21));
      record.set("letter_spacing", null);
      record.set("weight", 400);
      record.set("font_family", ftd.font_display);
      return record;
    }());
    record.set("mobile", function () {
      let record = fastn.recordInstance({
      });
      record.set("size", fastn_dom.FontSize.Px(16));
      record.set("line_height", fastn_dom.FontSize.Px(21));
      record.set("letter_spacing", null);
      record.set("weight", 400);
      record.set("font_family", ftd.font_display);
      return record;
    }());
    return record;
  }());
  record.set("button_large", function () {
    let record = fastn.recordInstance({
    });
    record.set("desktop", function () {
      let record = fastn.recordInstance({
      });
      record.set("size", fastn_dom.FontSize.Px(18));
      record.set("line_height", fastn_dom.FontSize.Px(24));
      record.set("letter_spacing", null);
      record.set("weight", 400);
      record.set("font_family", ftd.font_display);
      return record;
    }());
    record.set("mobile", function () {
      let record = fastn.recordInstance({
      });
      record.set("size", fastn_dom.FontSize.Px(18));
      record.set("line_height", fastn_dom.FontSize.Px(24));
      record.set("letter_spacing", null);
      record.set("weight", 400);
      record.set("font_family", ftd.font_display);
      return record;
    }());
    return record;
  }());
  record.set("link", function () {
    let record = fastn.recordInstance({
    });
    record.set("desktop", function () {
      let record = fastn.recordInstance({
      });
      record.set("size", fastn_dom.FontSize.Px(14));
      record.set("line_height", fastn_dom.FontSize.Px(19));
      record.set("letter_spacing", null);
      record.set("weight", 400);
      record.set("font_family", ftd.font_display);
      return record;
    }());
    record.set("mobile", function () {
      let record = fastn.recordInstance({
      });
      record.set("size", fastn_dom.FontSize.Px(14));
      record.set("line_height", fastn_dom.FontSize.Px(19));
      record.set("letter_spacing", null);
      record.set("weight", 400);
      record.set("font_family", ftd.font_display);
      return record;
    }());
    return record;
  }());
  record.set("label_large", function () {
    let record = fastn.recordInstance({
    });
    record.set("desktop", function () {
      let record = fastn.recordInstance({
      });
      record.set("size", fastn_dom.FontSize.Px(14));
      record.set("line_height", fastn_dom.FontSize.Px(19));
      record.set("letter_spacing", null);
      record.set("weight", 400);
      record.set("font_family", ftd.font_display);
      return record;
    }());
    record.set("mobile", function () {
      let record = fastn.recordInstance({
      });
      record.set("size", fastn_dom.FontSize.Px(14));
      record.set("line_height", fastn_dom.FontSize.Px(19));
      record.set("letter_spacing", null);
      record.set("weight", 400);
      record.set("font_family", ftd.font_display);
      return record;
    }());
    return record;
  }());
  record.set("label_small", function () {
    let record = fastn.recordInstance({
    });
    record.set("desktop", function () {
      let record = fastn.recordInstance({
      });
      record.set("size", fastn_dom.FontSize.Px(12));
      record.set("line_height", fastn_dom.FontSize.Px(16));
      record.set("letter_spacing", null);
      record.set("weight", 400);
      record.set("font_family", ftd.font_display);
      return record;
    }());
    record.set("mobile", function () {
      let record = fastn.recordInstance({
      });
      record.set("size", fastn_dom.FontSize.Px(12));
      record.set("line_height", fastn_dom.FontSize.Px(16));
      record.set("letter_spacing", null);
      record.set("weight", 400);
      record.set("font_family", ftd.font_display);
      return record;
    }());
    return record;
  }());
  return record;
}();
ftd.default_colors = function () {
  let record = fastn.recordInstance({
  });
  record.set("background", function () {
    let record = fastn.recordInstance({
    });
    record.set("base", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#e7e7e4");
      record.set("dark", "#18181b");
      return record;
    }());
    record.set("step_1", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#f3f3f3");
      record.set("dark", "#141414");
      return record;
    }());
    record.set("step_2", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#c9cece");
      record.set("dark", "#585656");
      return record;
    }());
    record.set("overlay", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "rgba(0, 0, 0, 0.8)");
      record.set("dark", "rgba(0, 0, 0, 0.8)");
      return record;
    }());
    record.set("code", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#F5F5F5");
      record.set("dark", "#21222C");
      return record;
    }());
    return record;
  }());
  record.set("border", function () {
    let record = fastn.recordInstance({
    });
    record.set("light", "#434547");
    record.set("dark", "#434547");
    return record;
  }());
  record.set("border_strong", function () {
    let record = fastn.recordInstance({
    });
    record.set("light", "#919192");
    record.set("dark", "#919192");
    return record;
  }());
  record.set("text", function () {
    let record = fastn.recordInstance({
    });
    record.set("light", "#584b42");
    record.set("dark", "#a8a29e");
    return record;
  }());
  record.set("text_strong", function () {
    let record = fastn.recordInstance({
    });
    record.set("light", "#141414");
    record.set("dark", "#ffffff");
    return record;
  }());
  record.set("shadow", function () {
    let record = fastn.recordInstance({
    });
    record.set("light", "#007f9b");
    record.set("dark", "#007f9b");
    return record;
  }());
  record.set("scrim", function () {
    let record = fastn.recordInstance({
    });
    record.set("light", "#007f9b");
    record.set("dark", "#007f9b");
    return record;
  }());
  record.set("cta_primary", function () {
    let record = fastn.recordInstance({
    });
    record.set("base", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#2dd4bf");
      record.set("dark", "#2dd4bf");
      return record;
    }());
    record.set("hover", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#2c9f90");
      record.set("dark", "#2c9f90");
      return record;
    }());
    record.set("pressed", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#2cc9b5");
      record.set("dark", "#2cc9b5");
      return record;
    }());
    record.set("disabled", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "rgba(44, 201, 181, 0.1)");
      record.set("dark", "rgba(44, 201, 181, 0.1)");
      return record;
    }());
    record.set("focused", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#2cbfac");
      record.set("dark", "#2cbfac");
      return record;
    }());
    record.set("border", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#2b8074");
      record.set("dark", "#2b8074");
      return record;
    }());
    record.set("border_disabled", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#65b693");
      record.set("dark", "#65b693");
      return record;
    }());
    record.set("text", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#feffff");
      record.set("dark", "#feffff");
      return record;
    }());
    record.set("text_disabled", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#65b693");
      record.set("dark", "#65b693");
      return record;
    }());
    return record;
  }());
  record.set("cta_secondary", function () {
    let record = fastn.recordInstance({
    });
    record.set("base", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#4fb2df");
      record.set("dark", "#4fb2df");
      return record;
    }());
    record.set("hover", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#40afe1");
      record.set("dark", "#40afe1");
      return record;
    }());
    record.set("pressed", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#4fb2df");
      record.set("dark", "#4fb2df");
      return record;
    }());
    record.set("disabled", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "rgba(79, 178, 223, 0.1)");
      record.set("dark", "rgba(79, 178, 223, 0.1)");
      return record;
    }());
    record.set("focused", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#4fb1df");
      record.set("dark", "#4fb1df");
      return record;
    }());
    record.set("border", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#209fdb");
      record.set("dark", "#209fdb");
      return record;
    }());
    record.set("border_disabled", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#65b693");
      record.set("dark", "#65b693");
      return record;
    }());
    record.set("text", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#584b42");
      record.set("dark", "#ffffff");
      return record;
    }());
    record.set("text_disabled", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#65b693");
      record.set("dark", "#65b693");
      return record;
    }());
    return record;
  }());
  record.set("cta_tertiary", function () {
    let record = fastn.recordInstance({
    });
    record.set("base", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#556375");
      record.set("dark", "#556375");
      return record;
    }());
    record.set("hover", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#c7cbd1");
      record.set("dark", "#c7cbd1");
      return record;
    }());
    record.set("pressed", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#3b4047");
      record.set("dark", "#3b4047");
      return record;
    }());
    record.set("disabled", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "rgba(85, 99, 117, 0.1)");
      record.set("dark", "rgba(85, 99, 117, 0.1)");
      return record;
    }());
    record.set("focused", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#e0e2e6");
      record.set("dark", "#e0e2e6");
      return record;
    }());
    record.set("border", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#e2e4e7");
      record.set("dark", "#e2e4e7");
      return record;
    }());
    record.set("border_disabled", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#65b693");
      record.set("dark", "#65b693");
      return record;
    }());
    record.set("text", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#ffffff");
      record.set("dark", "#ffffff");
      return record;
    }());
    record.set("text_disabled", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#65b693");
      record.set("dark", "#65b693");
      return record;
    }());
    return record;
  }());
  record.set("cta_danger", function () {
    let record = fastn.recordInstance({
    });
    record.set("base", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#1C1B1F");
      record.set("dark", "#1C1B1F");
      return record;
    }());
    record.set("hover", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#1C1B1F");
      record.set("dark", "#1C1B1F");
      return record;
    }());
    record.set("pressed", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#1C1B1F");
      record.set("dark", "#1C1B1F");
      return record;
    }());
    record.set("disabled", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#1C1B1F");
      record.set("dark", "#1C1B1F");
      return record;
    }());
    record.set("focused", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#1C1B1F");
      record.set("dark", "#1C1B1F");
      return record;
    }());
    record.set("border", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#1C1B1F");
      record.set("dark", "#1C1B1F");
      return record;
    }());
    record.set("border_disabled", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#feffff");
      record.set("dark", "#feffff");
      return record;
    }());
    record.set("text", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#1C1B1F");
      record.set("dark", "#1C1B1F");
      return record;
    }());
    record.set("text_disabled", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#feffff");
      record.set("dark", "#feffff");
      return record;
    }());
    return record;
  }());
  record.set("accent", function () {
    let record = fastn.recordInstance({
    });
    record.set("primary", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#2dd4bf");
      record.set("dark", "#2dd4bf");
      return record;
    }());
    record.set("secondary", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#4fb2df");
      record.set("dark", "#4fb2df");
      return record;
    }());
    record.set("tertiary", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#c5cbd7");
      record.set("dark", "#c5cbd7");
      return record;
    }());
    return record;
  }());
  record.set("error", function () {
    let record = fastn.recordInstance({
    });
    record.set("base", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#f5bdbb");
      record.set("dark", "#311b1f");
      return record;
    }());
    record.set("text", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#c62a21");
      record.set("dark", "#c62a21");
      return record;
    }());
    record.set("border", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#df2b2b");
      record.set("dark", "#df2b2b");
      return record;
    }());
    return record;
  }());
  record.set("success", function () {
    let record = fastn.recordInstance({
    });
    record.set("base", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#e3f0c4");
      record.set("dark", "#405508ad");
      return record;
    }());
    record.set("text", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#467b28");
      record.set("dark", "#479f16");
      return record;
    }());
    record.set("border", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#3d741f");
      record.set("dark", "#3d741f");
      return record;
    }());
    return record;
  }());
  record.set("info", function () {
    let record = fastn.recordInstance({
    });
    record.set("base", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#c4edfd");
      record.set("dark", "#15223a");
      return record;
    }());
    record.set("text", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#205694");
      record.set("dark", "#1f6feb");
      return record;
    }());
    record.set("border", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#205694");
      record.set("dark", "#205694");
      return record;
    }());
    return record;
  }());
  record.set("warning", function () {
    let record = fastn.recordInstance({
    });
    record.set("base", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#fbefba");
      record.set("dark", "#544607a3");
      return record;
    }());
    record.set("text", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#966220");
      record.set("dark", "#d07f19");
      return record;
    }());
    record.set("border", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#966220");
      record.set("dark", "#966220");
      return record;
    }());
    return record;
  }());
  record.set("custom", function () {
    let record = fastn.recordInstance({
    });
    record.set("one", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#ed753a");
      record.set("dark", "#ed753a");
      return record;
    }());
    record.set("two", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#f3db5f");
      record.set("dark", "#f3db5f");
      return record;
    }());
    record.set("three", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#8fdcf8");
      record.set("dark", "#8fdcf8");
      return record;
    }());
    record.set("four", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#7a65c7");
      record.set("dark", "#7a65c7");
      return record;
    }());
    record.set("five", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#eb57be");
      record.set("dark", "#eb57be");
      return record;
    }());
    record.set("six", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#ef8dd6");
      record.set("dark", "#ef8dd6");
      return record;
    }());
    record.set("seven", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#7564be");
      record.set("dark", "#7564be");
      return record;
    }());
    record.set("eight", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#d554b3");
      record.set("dark", "#d554b3");
      return record;
    }());
    record.set("nine", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#ec8943");
      record.set("dark", "#ec8943");
      return record;
    }());
    record.set("ten", function () {
      let record = fastn.recordInstance({
      });
      record.set("light", "#da7a4a");
      record.set("dark", "#da7a4a");
      return record;
    }());
    return record;
  }());
  return record;
}();
ftd.breakpoint_width = function () {
  let record = fastn.recordInstance({
  });
  record.set("mobile", 768);
  return record;
}();
ftd.device = fastn.mutable(fastn_dom.DeviceData.Mobile);
let inherited = function () {
  let record = fastn.recordInstance({
  });
  record.set("colors", ftd.default_colors.getClone().setAndReturn("is_root", true));
  record.set("types", ftd.default_types.getClone().setAndReturn("is_root", true));
  return record;
}();
