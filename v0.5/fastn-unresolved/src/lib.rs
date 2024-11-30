#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn_unresolved;

#[cfg(test)]
mod debug;
mod parser;
pub mod resolver;
mod utils;

#[cfg(test)]
pub(crate) use debug::JIDebug;
pub use parser::parse;

pub type URD = fastn_unresolved::UR<fastn_unresolved::Definition, fastn_resolved::Definition>;
pub type URCI = fastn_unresolved::UR<
    fastn_unresolved::ComponentInvocation,
    fastn_resolved::ComponentInvocation,
>;
pub type URIS = fastn_unresolved::UR<fastn_section::IdentifierReference, fastn_unresolved::Symbol>;

pub type Bag = std::collections::HashMap<fastn_unresolved::Symbol, fastn_unresolved::URD>;

#[derive(Default)]
pub struct Arena {
    pub interner: string_interner::DefaultStringInterner,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Symbol {
    // 8 bytes
    /// this store the <package>/<module>#<name> of the symbol
    interned: string_interner::DefaultSymbol, // u32
    /// length of the <package> part of the symbol
    package_len: u16,
    /// length of the <module> part of the symbol
    module_len: u16,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Module {
    // 6 bytes
    /// this store the <package>/<module>#<name> of the symbol
    interned: string_interner::DefaultSymbol, // u32
    /// length of the <package> part of the symbol
    package_len: u16,
}

#[derive(Debug, Clone)]
pub struct Document {
    pub module: fastn_unresolved::Module,
    pub module_doc: Option<fastn_section::Span>,
    pub definitions: Vec<URD>,
    pub content: Vec<URCI>,
    pub errors: Vec<fastn_section::Spanned<fastn_section::Error>>,
    pub warnings: Vec<fastn_section::Spanned<fastn_section::Warning>>,
    pub comments: Vec<fastn_section::Span>,
    pub line_starts: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct Definition {
    pub symbol: Option<fastn_unresolved::Symbol>, // <package-name>/<module-name>#<definition-name>
    /// we will keep the builtins not as ScopeFrame, but as plain hashmap.
    /// we have two scopes at this level, the auto-imports, and scope of all symbols explicitly
    /// imported/defined in the document this definition exists in.
    pub doc: Option<fastn_section::Span>,
    /// resolving an identifier means making sure it is unique in the document, and performing
    /// other checks.
    pub name: UR<fastn_section::Identifier, fastn_section::Identifier>,
    pub visibility: fastn_section::Visibility,
    pub inner: InnerDefinition,
}

#[derive(Debug, Clone)]
pub enum InnerDefinition {
    SymbolAlias(fastn_unresolved::Symbol),
    ModuleAlias(fastn_unresolved::Module),
    Component {
        arguments: Vec<UR<Argument, fastn_resolved::Argument>>,
        body: Vec<URCI>,
    },
    Variable {
        kind: UR<Kind, fastn_resolved::Kind>,
        properties: Vec<UR<Property, fastn_resolved::Property>>,
        /// resolved caption goes to properties
        caption: UR<Vec<fastn_section::Tes>, ()>,
        /// resolved body goes to properties
        body: UR<Vec<fastn_section::Tes>, ()>,
    },
    Function {
        arguments: Vec<UR<Argument, fastn_resolved::Argument>>,
        /// `None` means `void`. The `void` keyword is implied in fastn code:
        /// ```ftd
        /// -- foo(): ;; function with void return type
        ///
        /// ;; function body
        /// ```
        return_type: Option<UR<Kind, fastn_resolved::Kind>>,
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
        body: Vec<UR<fastn_section::Tes, fastn_resolved::FunctionExpression>>,
        // body: Vec<UR<fastn_section::Tes, fastn_fscript::Expression>>,
    },
    // TypeAlias {
    //     kind: UR<Kind, fastn_resolved::Kind>,
    //     /// ```ftd
    //     /// -- type foo: person
    //     /// name: foo                  ;; we are updating / setting the default value
    //     /// ```
    //     arguments: Vec<UR<Property, fastn_resolved::Property>>,
    // },
    Record {
        arguments: Vec<UR<Argument, fastn_resolved::Argument>>,
    },
    // TODO: OrType(fastn_section::Section),
    // TODO: Module(fastn_section::Section),
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
    Invalid(fastn_section::Error),
    InvalidN(Vec<fastn_section::Error>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComponentInvocation {
    /// this contains a symbol that is the module where this component invocation happened.
    ///
    /// all local symbols are resolved with respect to the module.
    pub module: fastn_unresolved::Module,
    pub name: URIS,
    /// once a caption is resolved, it is set to () here, and moved to properties
    pub caption: UR<Option<fastn_section::HeaderValue>, ()>,
    pub properties: Vec<UR<Property, fastn_resolved::Property>>,
    /// once the body is resolved, it is set to () here, and moved to properties
    pub body: UR<Vec<fastn_section::Tes>, ()>,
    pub children: Vec<UR<ComponentInvocation, fastn_resolved::ComponentInvocation>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    pub name: fastn_section::Identifier,
    pub value: Vec<fastn_section::Tes>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Argument {
    pub name: fastn_section::Identifier,
    pub doc: Option<fastn_section::Span>,
    pub kind: Kind,
    pub visibility: fastn_section::Visibility,
    pub default: Option<fastn_section::Tes>,
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
    Custom(fastn_unresolved::Symbol),
}

pub enum FromSectionKindError {
    InvalidKind,
}

impl TryFrom<fastn_section::Kind> for Kind {
    type Error = FromSectionKindError;

    fn try_from(kind: fastn_section::Kind) -> Result<Self, Self::Error> {
        let ident = match kind.to_identifier_reference() {
            Some(ident) => ident,
            None => return Err(FromSectionKindError::InvalidKind),
        };

        match ident {
            fastn_section::IdentifierReference::Local(v) => match v.str() {
                "integer" => Ok(Kind::Integer),
                _ => todo!(),
            },
            _ => unreachable!(),
        }
    }
}
