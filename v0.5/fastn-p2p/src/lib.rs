//! # fastn-p2p: High-Level Type-Safe P2P Communication
//!
//! This crate provides a high-level, type-safe API for P2P communication in the fastn ecosystem.
//! It builds on top of `fastn-net` but exposes only the essential, locked-down APIs that 
//! reduce the possibility of bugs through strong typing and compile-time verification.
//!
//! ## Design Philosophy
//!
//! - **Type Safety First**: All communication uses strongly-typed REQUEST/RESPONSE/ERROR contracts
//! - **Minimal Surface Area**: Only essential APIs are exposed to reduce complexity
//! - **Bug Prevention**: API design makes common mistakes impossible or unlikely
//! - **Ergonomic**: High-level APIs handle boilerplate automatically
//!
//! ## Usage Patterns
//!
//! ## API Overview
//!
//! ### Client Side
//! ```rust,ignore
//! // Type-safe P2P calls with shared error types
//! type EchoResult = Result<EchoResponse, EchoError>;
//! let result: EchoResult = fastn_p2p::call(/*...*/).await?;
//! ```
//!
//! ### Server Side  
//! ```rust,ignore
//! // High-level request handling with automatic response management
//! let stream = fastn_p2p::listen(/*...*/)?;
//! request.handle(|req: EchoRequest| async move { /*...*/ }).await?;
//! ```

extern crate self as fastn_p2p;

pub mod client;
pub mod globals;
pub mod macros;
pub mod server;

// Re-export essential types from fastn-net that users need
pub use fastn_net::{Graceful, PeerStreamSenders, Protocol};

// Global singleton access
pub use globals::{graceful, pool};

// Client API - clean, simple naming (only expose simple version)
pub use client::{call, CallError};

// Server API - clean, simple naming  
pub use server::{
    listen,
    stop_listening,
    is_listening,
    active_listener_count,
    active_listeners,
    Request,
    ResponseHandle,
    GetInputError,
    SendError,
    HandleRequestError,
    ListenerAlreadyActiveError,
    ListenerNotFoundError,
};

