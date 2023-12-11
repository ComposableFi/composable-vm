#![cfg_attr(not(test), deny(clippy::disallowed_methods, clippy::disallowed_types))]
extern crate alloc;

pub use cvm_runtime::gateway as msg;

pub mod analyzer;
pub mod assets;
pub mod auth;
pub mod batch;
pub mod contract;
pub mod error;
pub mod events;
pub mod exchange;
pub mod executor;
mod network;
mod prelude;
pub mod state;
pub mod router;
