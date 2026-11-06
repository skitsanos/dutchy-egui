//! Dutchy components, implemented with egui primitives.

mod accordion;
mod alert;
mod badge;
mod button;
mod card;
mod empty_state;
pub mod forms;
mod loading_motion;
mod navigation;
mod progress;
mod skeleton;
mod spinner;
mod stats;
mod tabs;
mod tooltip;

pub use accordion::{
    Accordion, AccordionItem, AccordionResponse, AccordionState, AccordionVariant,
};
pub use alert::{Alert, AlertResponse, AlertVariant};
pub use badge::{Badge, BadgeVariant};
pub use button::{Button, ButtonVariant};
pub use card::{Card, CardVariant};
pub use empty_state::{EmptyState, EmptyStateIcon, EmptyStateResponse, EmptyStateVariant};
pub use progress::{Progress, ProgressVariant};
pub use skeleton::{Skeleton, SkeletonGroup, SkeletonVariant};
pub use spinner::{Spinner, SpinnerSize};
pub use stats::{
    Stat, StatSize, StatVariant, StatsGrid, StatsLayout, Trend, TrendDirection, TrendSentiment,
};
pub use tabs::{Tab, Tabs, TabsResponse, TabsVariant};
pub use tooltip::{Tooltip, TooltipPosition};
