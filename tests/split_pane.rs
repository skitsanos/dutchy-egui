//! Split geometry, keyboard/assistive input, pointer drags and child state.
mod common;
#[path = "split_pane_support/geometry.rs"]
mod geometry;
#[path = "split_pane_support/interaction.rs"]
mod interaction;
#[path = "split_pane_support/lifecycle.rs"]
mod lifecycle;
#[cfg(feature = "bundled-fonts")]
#[path = "split_pane_support/rendering.rs"]
mod rendering;
#[path = "split_pane_support/mod.rs"]
mod support;
