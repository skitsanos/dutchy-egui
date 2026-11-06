//! Dutchy components, implemented with egui primitives.

mod accordion;
mod alert;
mod badge;
mod breadcrumbs;
mod button;
mod card;
mod close;
mod command_palette;
mod dropdown;
mod empty_state;
pub mod forms;
mod loading_motion;
pub(crate) mod menu;
mod nav_icon;
mod navigation;
mod pagination;
mod progress;
pub(crate) mod search;
mod searchable_select;
mod skeleton;
mod spinner;
mod stats;
mod stepper;
mod tabs;
mod tooltip;

pub use accordion::{
    Accordion, AccordionItem, AccordionResponse, AccordionState, AccordionVariant,
};
pub use alert::{Alert, AlertResponse, AlertVariant};
pub use badge::{Badge, BadgeVariant};
pub use breadcrumbs::{
    BreadcrumbIconPainter, BreadcrumbItem, BreadcrumbSeparator, Breadcrumbs, BreadcrumbsResponse,
};
pub use button::{Button, ButtonVariant};
pub use card::{Card, CardVariant};
pub use command_palette::{CommandPalette, CommandPaletteResponse};
pub use dropdown::{Dropdown, DropdownAlign, DropdownResponse};
pub use empty_state::{EmptyState, EmptyStateIcon, EmptyStateResponse, EmptyStateVariant};
pub use menu::{Menu, MenuIconPainter, MenuItem, MenuItemVariant, MenuResponse};
pub use pagination::{Pagination, PaginationResponse, PaginationVariant};
pub use progress::{Progress, ProgressVariant};
pub use search::SearchState;
pub use searchable_select::{SearchableSelect, SearchableSelectResponse};
pub use skeleton::{Skeleton, SkeletonGroup, SkeletonVariant};
pub use spinner::{Spinner, SpinnerSize};
pub use stats::{
    Stat, StatSize, StatVariant, StatsGrid, StatsLayout, Trend, TrendDirection, TrendSentiment,
};
pub use stepper::{Step, Stepper, StepperResponse, StepperState, StepperVariant};
pub use tabs::{Tab, Tabs, TabsResponse, TabsVariant};
pub use tooltip::{Tooltip, TooltipPosition};
