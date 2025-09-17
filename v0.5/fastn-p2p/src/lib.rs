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

mod coordination;
mod globals;
mod macros;

// Export client and server modules (new modular API)
pub mod client;
pub mod server;

// Re-export essential types from fastn-net that users need
pub use fastn_net::{Graceful, Protocol};
// Note: PeerStreamSenders is intentionally NOT exported - users should use global singletons

// Re-export procedural macros
pub use fastn_p2p_macros::main;

// Global singleton access - graceful is completely encapsulated in coordination module
pub use coordination::{cancelled, shutdown, spawn};
pub use globals::{graceful, pool};

// Legacy top-level exports (for backward compatibility)
pub use client::{CallError, call};
pub use server::{
    GetInputError, HandleRequestError, ListenerAlreadyActiveError, ListenerNotFoundError, Request,
    ResponseHandle, SendError, active_listener_count, active_listeners, is_listening, listen,
    stop_listening, Session,
};
