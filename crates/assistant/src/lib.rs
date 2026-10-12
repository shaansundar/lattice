//! Lattice's built-in AI assistant.
//!
//! A locally hosted, fine-tuned model works on the open workbook through the same tool layer external agents use
//! (`lattice_mcp::call_tool`), so the app, the training data and the evals share one set of semantics:
//!
//! - [`prompt`]: the fixed system prompt (with the model's 15 tools) and the per-request workbook snapshot.
//! - [`observe`]: tool results rendered as the compact text the model was trained on.
//! - [`parse`]: Gemma-4 tool calls and thinking channel out of the model's output.
//! - [`docs`]: the `lattice_docs` tool over the documentation bundled into this build.
//! - [`harness`]: one model tool call → guardrails → the engine → one observation.
//!
//! Everything here mirrors the Python reference harness the model was trained against; the golden fixtures in
//! `tests/golden/` keep them byte-identical.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
#![forbid(unsafe_code)]

pub mod agent;
pub mod chat;
pub mod docs;
pub mod harness;
pub mod install;
pub mod observe;
pub mod parse;
pub mod prompt;
pub mod pyfmt;
pub mod router;
pub mod runtime;

pub use harness::{Harness, ToolHost, is_risky};
pub use install::with_session;
pub use parse::{ToolCall, parse_tool_calls, split_thinking};
