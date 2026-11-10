//! Tree hierarchy geometry, combined with the shared List row tokens.
use super::space;
/// Horizontal step for one hierarchy level.
pub const INDENT: f32 = space::XL;
/// Reserved disclosure hit slot (also reserved for leaves).
pub const DISCLOSURE_SLOT: f32 = space::XXL;
/// Chevron extent.
pub const DISCLOSURE_SIZE: f32 = space::MD;
