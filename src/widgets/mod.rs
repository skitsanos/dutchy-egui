//! Dutchy components, implemented with egui primitives.

mod alert;
mod badge;
mod button;
mod card;
pub mod forms;
mod loading_motion;
mod progress;
mod skeleton;
mod spinner;

pub use alert::{Alert, AlertResponse, AlertVariant};
pub use badge::{Badge, BadgeVariant};
pub use button::{Button, ButtonVariant};
pub use card::{Card, CardVariant};
pub use progress::{Progress, ProgressVariant};
pub use skeleton::{Skeleton, SkeletonGroup, SkeletonVariant};
pub use spinner::{Spinner, SpinnerSize};
