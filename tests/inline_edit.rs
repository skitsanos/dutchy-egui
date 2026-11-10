//! Explicit rename requests, retained drafts, native input and focused rendering.
mod common;
#[path = "inline_edit_support/composition.rs"]
mod composition;
#[path = "inline_edit_support/input_edges.rs"]
mod input_edges;
#[path = "inline_edit_support/interaction.rs"]
mod interaction;
#[path = "inline_edit_support/lifecycle.rs"]
mod lifecycle;
#[cfg(feature = "bundled-fonts")]
#[path = "inline_edit_support/rendering.rs"]
mod rendering;
#[path = "inline_edit_support/mod.rs"]
mod support;
