//! Dutchy components, implemented with egui primitives.

mod accordion;
mod actions;
mod alert;
mod avatar;
mod badge;
mod breadcrumbs;
mod button;
mod card;
mod chip;
mod close;
mod code_block;
mod command_palette;
mod dropdown;
mod empty_state;
pub mod forms;
mod kbd;
mod key_recorder;
mod list;
mod loading_motion;
#[cfg(feature = "markdown")]
mod markdown;
mod media;
pub(crate) mod menu;
mod menu_bar;
mod nav_icon;
mod navigation;
mod overlay;
mod pagination;
mod popup_focus;
mod progress;
mod scribble;
pub(crate) mod search;
mod searchable_select;
mod skeleton;
mod spinner;
mod split_button;
mod stats;
mod status_bar;
mod stepper;
mod table;
mod tabs;
mod toast;
mod tooltip;
mod tree_view;
mod upload;

pub use accordion::{
    Accordion, AccordionItem, AccordionResponse, AccordionState, AccordionVariant,
};
pub use actions::{
    IconButton, ToggleButton, Toolbar, ToolbarEvent, ToolbarItem, ToolbarOrientation,
    ToolbarResponse,
};
pub use alert::{Alert, AlertResponse, AlertVariant};
pub use avatar::{Avatar, AvatarVariant, Presence};
pub use badge::{Badge, BadgeVariant};
pub use breadcrumbs::{
    BreadcrumbIconPainter, BreadcrumbItem, BreadcrumbSeparator, Breadcrumbs, BreadcrumbsResponse,
};
pub use button::{Button, ButtonVariant};
pub use card::{Card, CardVariant};
pub use chip::{Chip, ChipResponse};
pub use code_block::{CodeBlock, CodeHighlighter, CodeSpan};
pub use command_palette::{CommandPalette, CommandPaletteResponse};
pub use dropdown::{Dropdown, DropdownAlign, DropdownResponse};
pub use empty_state::{EmptyState, EmptyStateIcon, EmptyStateResponse, EmptyStateVariant};
pub use kbd::Kbd;
pub use key_recorder::{KeyRecorder, KeyRecorderResponse};
#[cfg(feature = "markdown")]
pub use markdown::{LinkPolicy, Markdown, MarkdownLink, MarkdownResponse};
pub use menu::{Menu, MenuEvent, MenuIconPainter, MenuItem, MenuItemVariant, MenuResponse};
pub use menu_bar::{MenuBar, MenuBarItem, MenuBarResponse, MenuBarState};
pub use pagination::{Pagination, PaginationResponse, PaginationVariant};
pub use progress::{Progress, ProgressVariant};
pub use search::SearchState;
pub use searchable_select::{SearchableSelect, SearchableSelectResponse};
pub use skeleton::{Skeleton, SkeletonGroup, SkeletonVariant};
pub use spinner::{Spinner, SpinnerSize};
pub use split_button::{SplitButton, SplitButtonResponse};
pub use stats::{
    Stat, StatSize, StatVariant, StatsGrid, StatsLayout, Trend, TrendDirection, TrendSentiment,
};
pub use status_bar::{StatusAlign, StatusBar, StatusBarResponse, StatusItem, StatusTone};
pub use stepper::{Step, Stepper, StepperResponse, StepperState, StepperVariant};
pub use tabs::{Tab, Tabs, TabsResponse, TabsVariant};
pub use tooltip::{Tooltip, TooltipPosition};
pub use upload::{
    FileInfo, FileItem, FileList, FileListResponse, FileRejection, FileRejectionReason, FileRules,
    FileStatus, FileUpload, FileUploadResponse, FileUploadVariant, FileValidation, file_size,
};

pub use overlay::{Drawer, DrawerSide, Modal, OverlayDismissal, OverlayOptions, OverlayResponse};

pub use toast::{
    Toast, ToastAppearance, ToastDismissal, ToastEvent, ToastId, ToastPosition, ToastQueue,
    ToastResponse, ToastVariant, Toasts, ToastsResponse,
};

pub use table::{
    DataTable, DataTableResponse, TableAlign, TableColumn, TableColumnWidth, TableDensity,
    TableHeader, TableRowResponse, TableSelection, TableSort, TableSortDirection,
};

pub use media::{
    Gallery, GalleryItem, GalleryResponse, Image, ImageAspect, ImageFit, ImageResponse,
    ImageStatus, Lightbox, LightboxResponse, MediaCard, MediaCardResponse,
};

pub use scribble::{Scribble, ScribbleAction, ScribbleResponse};

pub use list::{
    List, ListItem, ListResponse, ListRow, ListRowResponse, ListSelectionMode, ListState,
};

pub use tree_view::{TreeNode, TreeRow, TreeState, TreeView, TreeViewResponse};
