# fastn-section Grammar

This document provides the complete grammar reference for the fastn-section parser module. The grammar is presented using Extended Backus-Naur Form (EBNF) notation.

**Important Note:** The fastn-section parser is the first stage of the fastn parsing pipeline. It accepts a broad syntax that may be further validated and potentially rejected by subsequent parsing stages. This grammar documents what fastn-section accepts, not necessarily what constitutes valid fastn code.

## Notation

- `::=` - Definition
- `|` - Alternation (or)
- `[]` - Optional (0 or 1)
- `{}` - Repetition (0 or more)
- `()` - Grouping
- `""` - Literal string
- `<>` - Non-terminal

## Document Structure

```ebnf
<document> ::= [<module_doc>] {<spaces>} {<section> {<newlines>} {<spaces>}}

<module_doc> ::= <module_doc_line> {<module_doc_line>}

<module_doc_line> ::= {<spaces>} ";-;" {<char_except_newline>} <newline>
```

**Example:**
```ftd
;-; This is module documentation
;-; It describes the purpose of this module
;-; And appears once at the top of the file

-- foo: First section
```

## Section

```ebnf
<section> ::= [<doc_comment>] {<spaces>} ["/"] <section_init> {<spaces>} [<caption>] <newline>
              [<headers>] [<double_newline> <body>]

<section_init> ::= "--" {<spaces>} [<visibility>] {<spaces>} [<kind>] {<spaces>} 
                   <identifier_reference> [<function_marker>] ":"

<function_marker> ::= "(" {<spaces>} {<comment>} {<spaces>} ")"

<caption> ::= <header_value>
```

**Examples:**
```ftd
-- foo: Simple section

-- string message: Typed section
header: value

-- public list<int> items(): Function section

-- ftd.text: Component invocation
color: red

Body content here
```

## Headers

```ebnf
<headers> ::= {<header> <newline>}

<header> ::= {<spaces>} [<doc_comment>] ["/"] {<spaces>} [<visibility>] {<spaces>} 
             [<kind>] {<spaces>} <identifier> [<condition>] ":" {<spaces>} [<header_value>]

<condition> ::= {<spaces>} "if" {<spaces>} "{" {<spaces>} <condition_expr> {<spaces>} "}"

<condition_expr> ::= {<char_except_brace>}

<header_value> ::= <tes_list_till_newline>
```

**Examples:**
```ftd
name: John
public string email: john@example.com
list<string> tags: admin, moderator
/disabled: true
empty:

# Conditional headers
color: black                           # Default value
color if { dark-mode }: white         # Conditional value
size if { mobile }: small
size if { tablet }: medium
size if { desktop }: large
```

### Conditional Headers

Headers can have conditional values based on conditions. Multiple headers with the same name but different conditions will coalesce into a single header with multiple conditional values:

```ftd
-- ftd.text: Hello World
color: black                         # Default/unconditional value
color if { dark-mode }: white       # When dark-mode is true
color if { high-contrast }: yellow  # When high-contrast is true

# These three headers will be merged into one header with three conditional values
```

## Body

```ebnf
<body> ::= <tes_list>
```

The body contains free-form content that continues until the next section marker or end of document.

## Text-Expression-Section (Tes)

The Tes grammar handles mixed text and expressions within header values and body content.

```ebnf
<tes_list> ::= {<tes>}

<tes> ::= <text>
        | <expression>
        | <inline_section>

<text> ::= {<char>}+

<expression> ::= "{" <tes_list> "}"
               | "${" <tes_list> "}"

<inline_section> ::= "{" {<spaces>} "--" <section> "}"
```

**Examples:**
```ftd
Plain text
Text with {expression} embedded
Dollar ${expression} syntax
Nested {outer {inner} text} expressions
Complex {-- inline: section} content
Recursive ${outer ${inner ${deep}}} structures
```

### Expression Nesting

Expressions can be arbitrarily nested:
```ftd
{level1 {level2 {level3}}}
${dollar {mixed ${nested}}}
```

### Inline Sections

Inline sections are expressions that start with `--`:
```ftd
{-- component: inline content}
{-- foo: caption
header: value}
```

## Identifiers

```ebnf
<identifier> ::= <unicode_letter> {<identifier_char>}

<identifier_char> ::= <unicode_letter>
                    | <unicode_digit>
                    | "-"
                    | "_"
```

**Valid identifiers:**
```
foo
snake_case
kebab-case
_private
item123
नाम
名前
```

## Identifier References

```ebnf
<identifier_reference> ::= <dotted_ref>
                         | <absolute_ref>

<dotted_ref> ::= <identifier> {"." <identifier>}

<absolute_ref> ::= <identifier> "#" [<identifier> "/"] <identifier>
```

**Examples:**
```
foo                  // Simple reference
a.b.c               // Dotted reference (can be imported or local module)
module.component    // Two-part dotted reference
package#item        // Absolute reference
pkg#mod/comp       // Absolute with module
```

## Types (Kind)

```ebnf
<kind> ::= <identifier_reference> [<generic_args>]

<generic_args> ::= "<" {<spaces_and_comments>} [<kind_list>] {<spaces_and_comments>} ">"

<kind_list> ::= <kind> {<spaces_and_comments>} {"," {<spaces_and_comments>} <kind>}
```

**Examples:**
```
string
integer
list<string>
map<string, int>
custom<T1, T2, T3>
nested<list<map<string, int>>>
imported.Type
module.CustomType<T>
package#Type
pkg#mod/Type<A, B>
```

## Kinded Names

```ebnf
<kinded_name> ::= [<kind>] {<spaces>} <identifier>
```

**Examples:**
```
foo                    // Name only
string message         // Type and name
list<int> items       // Generic type and name
custom.Type data      // Imported type and name
```

## Kinded References

```ebnf
<kinded_reference> ::= [<kind>] {<spaces>} <identifier_reference>
```

**Examples:**
```
module.component              // Reference only
string ftd.text              // Type and reference
list<int> pkg#items          // Generic type and absolute reference
map<K,V> a.b.c              // Generic type with dotted reference
```

## Visibility

```ebnf
<visibility> ::= "public" [<visibility_scope>]
               | "private"

<visibility_scope> ::= "<" {<spaces_and_comments>} <scope> {<spaces_and_comments>} ">"

<scope> ::= "package" | "module"
```

**Examples:**
```
public
private
public<package>
public<module>
```

## Doc Comments

```ebnf
<doc_comment> ::= <doc_line> {<doc_line>}

<doc_line> ::= {<spaces>} ";;;" {<char_except_newline>} <newline>
```

**Example:**
```ftd
;;; This is documentation
;;; It can span multiple lines
;;; And provides information about the following element
```

## Whitespace and Comments

```ebnf
<spaces> ::= {" " | "\t"}
<newline> ::= "\n" | "\r\n"
<newlines> ::= {<newline>}
<double_newline> ::= <newline> <newline>

<comment> ::= ";;" {<char_except_newline>}
<spaces_and_comments> ::= {<spaces> | <comment> | <newline>}
```

## Complete Examples

### Module with Documentation

```ftd
;-; fastn UI Component Library
;-; Version: 1.0.0
;-; This module provides reusable UI components

-- public component button: Click Me
type: primary
enabled: true

Renders a clickable button
```

### Basic Section with Headers and Body

```ftd
;;; User information component
-- public component user-card: John Doe
;;; Email address
public string email: john@example.com
private integer age: 30
list<string> roles: admin, moderator

This is the body of the user-card component.
It can contain {expressions} and ${dollar expressions}.
```

### Nested Structures

```ftd
-- container: Main
child<widget> items: nested

Body with complex expressions:
- Simple: {value}
- Nested: {outer {inner}}
- Mixed: ${dollar {regular}}
- Inline section: {-- note: Important}
```

### Function Declaration

```ftd
-- public function calculate(): Result
integer x: 10
integer y: 20

{-- compute: ${x} + ${y}}
```

### Commented Elements

```ftd
/-- disabled-feature: Not active
/setting: old-value

-- active-feature: Enabled
setting: new-value
```

### Conditional Headers

Headers can have conditional values that depend on runtime conditions:

```ftd
-- ftd.text: Responsive Text
;; Default values (no condition)
color: black
size: 16px

;; Conditional values
color if { dark-mode }: white
color if { high-contrast }: yellow
size if { mobile }: 14px
size if { tablet }: 16px
size if { desktop }: 18px

;; Complex conditions (parsed as opaque text by fastn-section)
background if { dark-mode && high-contrast }: #333
opacity if { hover || focus }: 0.8
```

**Conditional Header Coalescing:**

When multiple headers have the same name with different conditions, they are coalesced into a single header with multiple conditional values:

```ftd
;; These three header lines:
color: black
color if { dark-mode }: white
color if { high-contrast }: yellow

;; Result in one Header with three ConditionalValue entries:
;; Header {
;;   name: "color",
;;   values: [
;;     ConditionalValue { condition: None, value: "black" },
;;     ConditionalValue { condition: Some("dark-mode"), value: "white" },
;;     ConditionalValue { condition: Some("high-contrast"), value: "yellow" }
;;   ]
;; }
```

**Note:** The fastn-section parser treats conditions as opaque text. The actual condition evaluation and logic is handled by later stages of the fastn compiler.