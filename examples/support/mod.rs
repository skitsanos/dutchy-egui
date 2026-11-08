//! Shared scaffolding for the scenario examples: the theme switcher, shell
//! frames, a status strip, deterministic fictional data and opt-in capture
//! hooks for native verification runs.
//!
//! Each example includes this module with `#[path = "../support/mod.rs"]` and
//! uses the parts it needs, so unused items are expected here.
#![allow(dead_code)]

pub mod capture;
pub mod data;
pub mod shell;
pub mod theme;
