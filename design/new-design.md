# FTD to HTML

## How can we do this in 0.5?

We go `String, String` (document-id, source code) -> `ftd:p1::Section`
then to `ftd::ast::Ast`. We store in `documents: Map<String, Document>`.

x.ftd
y.ftd

```ftd
-- import: y

-- integer $a: 2

-- integer x: 2 * $y.one

-- integer z: 4 * $y.one
```

```rust
struct State {
    documents: std::collections::BTreeMap<String, Document>,
    types: Map<String, Sourced<Type>>,
}

struct Document {
    aliases: Map<String, String>,
    ast: Vec<ftd::ast::Ast>
}

struct Sourced<T> {
    file: String,
    line: usize,
    value: T,
}

enum Type {
    Integer,
    MutableInteger,
}
```

Once we have `documents`, we can write `document_to_js`.

```json
{
  "x.a": "MutableInteger",
  "x.x": "Integer"
}
```

```js
let foo_bar__x = 2;  // foo/bar.ftd
```

```ftd
-- integer $k: 1
 
-- integer x:
$processor$: <> 
k: $k

-- integer y: $x * 2

-- ftd.integer: $k
$on-click$: { k += 1 }

-- ftd.integer: $y
```

### Type Checking

The generated JS should work if everything is correct, else we will get runtime error. We do
not want runtime errors, so we implement a type check pass on `documents`.

The type checker will create `bag: Map<String, ftd::interpreter::Thing>` containing symbols from
all documents (starting from source document, and only things that are referred from there).

Type checker will go through every Ast in source document, and for each symbol identified, it will
check if it is present in `types`. If not, it will try to add the symbol in the bag.

## How is HTML generated, given ftd source file in 0.4?

First we parse:

```rust
pub fn parse_doc(name: &str, source: &str) -> ftd::interpreter::Result<ftd::interpreter::Document> {
    let mut s = ftd::interpreter::interpret(name, source)?;

    // .. skipped ..
}
```

We then run the interpreter loop:

```rust
pub fn parse_doc(name: &str, source: &str) -> ftd::interpreter::Result<ftd::interpreter::Document> {
    let mut s = ftd::interpreter::interpret(name, source)?;
    loop {
        match s {
            ftd::interpreter::Interpreter::Done { document: doc } => {
                break;
            }
            ftd::interpreter::Interpreter::StuckOnImport {
                module, state: st, ..
            } => {
                s = st.continue_after_import(
                    module.as_str(),
                    document,
                    foreign_variable,
                    foreign_function,
                    0,
                )?;
            }
            ftd::interpreter::Interpreter::StuckOnProcessor {
                state, ast, module, ..
            } => {
                s = state.continue_after_processor(value, ast)?;
            }
            ftd::interpreter::Interpreter::StuckOnForeignVariable {
                state,
                module,
                variable,
                ..
            } => {
                s = state.continue_after_variable(module.as_str(), variable.as_str(), value)?;
            }
        }
    }
    Ok(document)
}
```

We then convert the document to JS using `ftd::js::document_into_js_ast()`.

## Main Journey

```rust
// ftd::p1::Section
pub struct Section {
    pub name: String,
    pub kind: Option<String>,
    pub caption: Option<ftd::p1::Header>,
    pub headers: ftd::p1::Headers,
    pub body: Option<Body>,
    pub sub_sections: Vec<Section>,
    pub is_commented: bool,
    pub line_number: usize,
    pub block_body: bool,
}

pub enum AST {
    // ftd::ast::Ast
    #[serde(rename = "import")]
    Import(ftd::ast::Import),
    #[serde(rename = "record")]
    Record(ftd::ast::Record),
    #[serde(rename = "or-type")]
    OrType(ftd::ast::OrType),
    VariableDefinition(ftd::ast::VariableDefinition),
    VariableInvocation(ftd::ast::VariableInvocation),
    ComponentDefinition(ftd::ast::ComponentDefinition),
    #[serde(rename = "component-invocation")]
    ComponentInvocation(ftd::ast::Component),
    FunctionDefinition(ftd::ast::Function),
    WebComponentDefinition(ftd::ast::WebComponentDefinition),
}

pub struct Document {
    pub data: indexmap::IndexMap<String, ftd::interpreter::Thing>,
    pub name: String,
    pub tree: Vec<ftd::interpreter::Component>,
    pub aliases: ftd::Map<String>,
    pub js: std::collections::HashSet<String>,
    pub css: std::collections::HashSet<String>,
}
```

## P1 Parser

```rust
pub struct Body {
    pub line_number: usize,
    pub value: String,
}

pub struct Headers(pub Vec<Header>);

pub enum Header {
    KV(ftd::p1::header::KV),
    Section(ftd::p1::header::SectionInfo),
    BlockRecordHeader(ftd::p1::header::BlockRecordHeader),
}

pub struct KV {
    pub line_number: usize,
    pub key: String,
    pub kind: Option<String>,
    pub value: Option<String>,
    pub condition: Option<String>,
    pub access_modifier: AccessModifier,
    pub source: KVSource,
}

pub struct SectionInfo {
    pub line_number: usize,
    pub key: String,
    pub kind: Option<String>,
    pub section: Vec<ftd::p1::Section>,
    pub condition: Option<String>,
}

pub struct BlockRecordHeader {
    pub key: String,
    pub kind: Option<String>,
    pub caption: Option<String>,
    pub body: (Option<String>, Option<usize>),
    pub fields: Vec<Header>,
    pub condition: Option<String>,
    pub line_number: usize,
}
```

## AST

```rust
pub struct ParsedDocument {
    pub name: String,
    pub ast: Vec<ftd::ast::AST>,
    pub processing_imports: bool,
    pub doc_aliases: ftd::Map<String>,
    pub re_exports: ReExport,
    pub exposings: ftd::Map<String>,
    pub foreign_variable: Vec<String>,
    pub foreign_function: Vec<String>,
}


pub struct Import {
    pub module: String,
    pub alias: String,
    #[serde(rename = "line-number")]
    pub line_number: usize,
    pub exports: Option<Export>,
    pub exposing: Option<Exposing>,
}

pub struct Record {
    pub name: String,
    pub fields: Vec<Field>,
    pub line_number: usize,
}
```

## Interpreter

```rust
pub struct InterpreterState {
    pub id: String,
    pub bag: indexmap::IndexMap<String, ftd::interpreter::Thing>,
    pub js: std::collections::HashSet<String>,
    pub css: std::collections::HashSet<String>,
    pub to_process: ToProcess,
    pub pending_imports: PendingImports,
    pub parsed_libs: ftd::Map<ParsedDocument>,
    pub instructions: Vec<ftd::interpreter::Component>,
}

pub enum Interpreter {
    StuckOnImport {
        module: String,
        state: InterpreterState,
        caller_module: String,
    },
    Done {
        document: Document,
    },
    StuckOnProcessor {
        state: InterpreterState,
        ast: ftd::ast::AST,
        module: String,
        processor: String,
        caller_module: String,
    },
    StuckOnForeignVariable {
        state: InterpreterState,
        module: String,
        variable: String,
        caller_module: String,
    },
}

pub struct Document {
    pub data: indexmap::IndexMap<String, ftd::interpreter::Thing>,
    pub name: String,
    pub tree: Vec<ftd::interpreter::Component>,
    pub aliases: ftd::Map<String>,
    pub js: std::collections::HashSet<String>,
    pub css: std::collections::HashSet<String>,
}

pub struct Component {
    pub id: Option<String>,
    pub name: String,
    pub properties: Vec<Property>,
    pub iteration: Box<Option<Loop>>,
    pub condition: Box<Option<ftd::interpreter::Expression>>,
    pub events: Vec<Event>,
    pub children: Vec<Component>,
    pub source: ComponentSource,
    pub line_number: usize,
}
```

## JS

```rust
pub struct JSAstData {
    // fastn_js::JSAstData
    /// This contains asts of things (other than `ftd`) and instructions/tree
    pub asts: Vec<fastn_js::Ast>,
    /// This contains external scripts provided by user and also `ftd`
    /// internally supports (like rive).
    pub scripts: Vec<String>,
}

pub enum Ast {
    // fastn_js::Ast
    Component(fastn_js::Component),
    UDF(fastn_js::UDF),
    // user defined function
    StaticVariable(fastn_js::StaticVariable),
    MutableVariable(fastn_js::MutableVariable),
    MutableList(fastn_js::MutableList),
    RecordInstance(fastn_js::RecordInstance),
    OrType(fastn_js::OrType),
    Export { from: String, to: String },
}

pub struct Component {
    pub name: String,
    pub params: Vec<String>,
    pub args: Vec<(String, fastn_js::SetPropertyValue, bool)>,
    // Vec<(name, value, is_mutable)>
    pub body: Vec<fastn_js::ComponentStatement>,
}

pub enum SetPropertyValue {
    Reference(String),
    Value(fastn_js::Value),
    Formula(fastn_js::Formula),
    Clone(String),
}
```
