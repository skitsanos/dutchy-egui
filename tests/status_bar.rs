//! StatusBar layout, overflow priority, activation and semantics.
mod common;
#[path = "status_bar_support/interaction.rs"]
mod interaction;
#[path = "status_bar_support/layout.rs"]
mod layout;
#[cfg(feature = "bundled-fonts")]
#[path = "status_bar_support/rendering.rs"]
mod rendering;
#[path = "status_bar_support/semantics.rs"]
mod semantics;
#[path = "status_bar_support/mod.rs"]
mod support;
