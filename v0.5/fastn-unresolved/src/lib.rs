#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn_unresolved;

#[cfg(test)]
mod debug;
mod parser;
mod resolver;
mod utils;

pub use parser::parse;
pub use resolver::ResolutionOutput;

pub type LookupResult = fastn_unresolved::UR<fastn_unresolved::Definition, fastn_type::Definition>;

#[derive(Debug, Clone, Default)]
pub struct Document {
    pub module_doc: Option<fastn_section::Span>,
    pub imports: Vec<fastn_unresolved::Import>,
    pub definitions: Vec<UR<Definition, fastn_type::Definition>>,
    pub content: Vec<UR<ComponentInvocation, fastn_type::ComponentInvocation>>,
    pub errors: Vec<fastn_section::Spanned<fastn_section::Error>>,
    pub warnings: Vec<fastn_section::Spanned<fastn_section::Warning>>,
    pub comments: Vec<fastn_section::Span>,
    pub line_starts: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct Definition {
    pub symbol: Option<string_interner::DefaultSymbol>, // <package-name>/<module-name>#<definition-name>
    pub module: Option<string_interner::DefaultSymbol>,
    pub package: Option<string_interner::DefaultSymbol>,
    pub doc: Option<fastn_section::Span>,
    /// resolving an identifier means making sure it is unique in the document, and performing
    /// other checks.
    pub name: UR<Identifier, Identifier>,
    pub visibility: fastn_section::Visibility,
    pub inner: InnerDefinition,
}

#[derive(Debug, Clone)]
pub enum InnerDefinition {
    Component {
        properties: Vec<UR<Argument, fastn_type::Argument>>,
        body: Vec<UR<ComponentInvocation, fastn_type::ComponentInvocation>>,
    },
    Variable {
        kind: UR<Kind, fastn_type::Kind>,
        properties: Vec<UR<Property, fastn_type::Property>>,
        /// resolved caption goes to properties
        caption: UR<Vec<fastn_section::Tes>, ()>,
        /// resolved body goes to properties
        body: UR<Vec<fastn_section::Tes>, ()>,
    },
    Function {
        arguments: Vec<UR<Argument, fastn_type::Argument>>,
        return_type: Option<UR<Kind, fastn_type::Kind>>,
        /// This one is a little interesting, the number of expressions can be higher than the
        /// number of Tes, this because we can have multiple expressions in a single `Tes`.
        ///
        /// ```ftd
        /// -- integer x():
        ///
        /// foo();
        /// bar()
        ///
        /// -- integer p: x()
        /// ```
        ///
        /// When we are parsing `x`, we will get the body as a single `Tes::Text("foo();\nbar()")`.
        /// In the `body` below we will start with `Vec<UR::UnResolved(Tes::Text("foo();\nbar()"))>`.
        ///
        /// When trying to resolve it, we will first get "stuck" at `foo();` and would have made no
        /// progress in the first pass (we will realize we need definition of `foo` to make progress,
        /// but we haven't yet made any progress.
        ///
        /// After `foo` is resolved, and we are called again, we can fully parse `foo();` statement,
        /// and would get stuck at `bar`. Now we can throw this away and not modify `body` at all,
        /// in which case we will have to reparse `foo();` line once `bar` is available, and if
        /// there are many such so far unknown symbols, we will be doing a lot of re-parsing.
        ///
        /// So the other approach is to modify the body to `Vec<UR::Resolved(<parsed-foo>),
        /// UR::UnResolved(Tes::Text("bar()"))>`. Notice how we have reduced the `Tex::Text()` part
        /// to no longer refer to `foo()`, and only keep the part that is still unresolved.
        body: Vec<UR<fastn_section::Tes, fastn_type::FunctionExpression>>,
        // body: Vec<UR<fastn_section::Tes, fastn_fscript::Expression>>,
    },
    TypeAlias {
        kind: UR<Kind, fastn_type::Kind>,
        /// ```ftd
        /// -- type foo: person
        /// name: foo                  ;; we are updating / setting the default value
        /// ```
        arguments: Vec<UR<Property, fastn_type::Property>>,
    },
    Record {
        properties: Vec<UR<Argument, fastn_type::Argument>>,
    },
    // TODO: OrType(fastn_section::Section),
    // TODO: Module(fastn_section::Section),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    pub module: ModuleName,
    pub alias: Option<Identifier>,
    pub export: Option<Export>,
    pub exposing: Option<Export>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UR<U, R> {
    Resolved(R),
    UnResolved(U),
    NotFound,
    /// if the resolution failed, we need not try to resolve it again, unless dependencies change.
    ///
    /// say when we are processing x.ftd we found out that the symbol foo is invalid, so when we are
    /// processing y.ftd, and we find foo, we can directly say that it is invalid.
    ///
    /// this is the goal, but we do not know why isn't `foo` valid, meaning on what another symbol
    /// does it depend on, so when do we "revalidate" the symbol?
    ///
    /// what if we store the dependencies it failed on, so when any of them changes, we can
    /// revalidate?
    Invalid(Vec<fastn_section::Error>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComponentInvocation {
    pub name: UR<Identifier, Identifier>,
    /// once a caption is resolved, it is set to () here, and moved to properties
    pub caption: UR<Option<fastn_section::HeaderValue>, ()>,
    pub properties: Vec<UR<Property, fastn_type::Property>>,
    /// once the body is resolved, it is set to () here, and moved to properties
    pub body: UR<Vec<fastn_section::Tes>, ()>,
    pub children: Vec<UR<ComponentInvocation, fastn_type::ComponentInvocation>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    pub name: Identifier,
    pub value: Vec<fastn_section::Tes>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Argument {
    pub name: Identifier,
    pub kind: Kind,
    pub visibility: fastn_section::Visibility,
    pub default: Option<fastn_section::Tes>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct PackageName(pub Identifier);

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct ModuleName {
    pub name: Identifier,
    pub package: PackageName,
}

pub type Identifier = fastn_section::Identifier;

#[derive(Debug, Clone, PartialEq)]
pub enum Export {
    All,
    Things(Vec<AliasableIdentifier>),
}

/// is this generic enough?
#[derive(Debug, Clone, PartialEq)]
pub struct AliasableIdentifier {
    pub alias: Option<Identifier>,
    pub name: Identifier,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct SymbolName {
    pub module: ModuleName,
    /// can name contain dots? after we have `-- module foo:` feature it will, but now?
    pub name: Identifier, // name comes after #
}

/// We cannot have kinds of like Record(SymbolName), OrType(SymbolName), because they are not
/// yet "resolved", eg `-- foo x:`, we do not know if `foo` is a record or an or-type.
#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    Integer,
    Decimal,
    String,
    Boolean,
    Option(Box<Kind>),
    // TODO: Map(Kind, Kind),
    List(Box<Kind>),
    Caption(Box<Kind>),
    Body(Box<Kind>),
    CaptionOrBody(Box<Kind>),
    // TODO: Future(Kind),
    // TODO: Result(Kind, Kind),
    Custom(SymbolName),
}
