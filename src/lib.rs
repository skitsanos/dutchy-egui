//! Dutchy Design System for egui.
//!
//! Install the theme and bundled typefaces once, then compose normal egui UIs
//! with Dutchy widgets. Applications own their state.

pub use egui;

pub mod calendar;
pub mod commands;
pub mod fonts;
pub mod fuzzy;
pub mod icons;
#[cfg(feature = "url-images")]
pub mod image_loaders;
pub mod interaction;
pub mod layout;
pub mod scribble;
pub mod text;
pub mod theme;
pub mod tokens;
pub mod validation;
pub mod widgets;

pub use commands::{Command, CommandRegistry, CommandRequest, CommandScope, ShortcutConflict};
pub use fuzzy::{FuzzyMatch, fuzzy_match};
pub use icons::{Icon, IconPainter};
pub use layout::{
    Container, CrossAlign, Flex, FlexDirection, ResponsiveGrid, SplitPane, SplitPaneResponse,
};
pub use scribble::{
    ScribbleBrush, ScribbleDrawing, ScribbleError, ScribbleExportOptions, ScribbleSmoothing,
    ScribbleSnapshot, ScribbleStroke, ScribbleWidth,
};
pub use text::{Text, TextColor};
pub use theme::Theme;
pub use tokens::{
    Accent, AvatarSize, CardPadding, ColumnWidth, ContainerWidth, ControlSize, DrawerWidth, Family,
    Leading, MenuWidth, ModalWidth, Mode, ProgressSize, Spacing, TextSize, Tracking, Weight,
};
pub use widgets::forms::{
    Checkbox, ColorPicker, ColorPreset, DatePicker, Field, InlineEdit, InlineEditEvent,
    InlineEditResponse, InlineEditState, Input, NumberInput, PathBrowseRequest, PathInput,
    PathInputResponse, RadioGroup, SearchInput, Segmented, Select, SelectOption, Slider, Switch,
    TagInput, TagInputRequest, TagInputResponse, TagItem, Textarea, TimePicker,
};
pub use widgets::{
    Accordion, AccordionItem, AccordionResponse, AccordionState, AccordionVariant, Alert,
    AlertResponse, AlertVariant, Badge, BadgeVariant, BreadcrumbIconPainter, BreadcrumbItem,
    BreadcrumbSeparator, Breadcrumbs, BreadcrumbsResponse, Button, ButtonVariant, Card,
    CardVariant, Chip, ChipResponse, CommandPalette, CommandPaletteResponse, Drawer, DrawerSide,
    Dropdown, DropdownAlign, DropdownResponse, EmptyState, EmptyStateIcon, EmptyStateResponse,
    EmptyStateVariant, Kbd, Menu, MenuBar, MenuBarItem, MenuBarResponse, MenuBarState, MenuEvent,
    MenuIconPainter, MenuItem, MenuItemVariant, MenuResponse, Modal, OverlayDismissal,
    OverlayOptions, OverlayResponse, Pagination, PaginationResponse, PaginationVariant, Progress,
    ProgressVariant, SearchState, SearchableSelect, SearchableSelectResponse, Skeleton,
    SkeletonGroup, SkeletonVariant, Spinner, SpinnerSize, Stat, StatSize, StatVariant, StatsGrid,
    StatsLayout, Step, Stepper, StepperResponse, StepperState, StepperVariant, Tab, Tabs,
    TabsResponse, TabsVariant, Tooltip, TooltipPosition, Trend, TrendDirection, TrendSentiment,
};
pub use widgets::{
    DataTable, DataTableResponse, TableAlign, TableColumn, TableColumnWidth, TableDensity,
    TableHeader, TableRowResponse, TableSelection, TableSort, TableSortDirection,
};
pub use widgets::{FilterChipGroup, FilterChipGroupResponse, FilterChipItem, FilterChipRequest};
pub use widgets::{KeyRecorder, KeyRecorderResponse, SplitButton, SplitButtonResponse};
pub use widgets::{MultiSelect, MultiSelectRequest, MultiSelectResponse};
pub use widgets::{
    Toast, ToastAppearance, ToastDismissal, ToastEvent, ToastId, ToastPosition, ToastQueue,
    ToastResponse, ToastVariant, Toasts, ToastsResponse,
};

/// Install bundled fonts once and apply a theme. Does not change display scaling.
///
/// ```
/// let ctx = egui::Context::default();
/// dutchy_egui::install(&ctx, dutchy_egui::Theme::default());
/// ```
pub fn install(ctx: &egui::Context, theme: Theme) {
    fonts::install(ctx);
    #[cfg(feature = "url-images")]
    image_loaders::install(ctx);
    theme.apply(ctx);
}

pub use widgets::{
    FileInfo, FileItem, FileList, FileListResponse, FileRejection, FileRejectionReason, FileRules,
    FileStatus, FileUpload, FileUploadResponse, FileUploadVariant, FileValidation, file_size,
};
pub use widgets::{
    Gallery, GalleryItem, GalleryResponse, Image, ImageAspect, ImageFit, ImageResponse,
    ImageStatus, Lightbox, LightboxResponse, MediaCard, MediaCardResponse,
};

pub use widgets::{
    IconButton, ToggleButton, Toolbar, ToolbarEvent, ToolbarItem, ToolbarOrientation,
    ToolbarResponse,
};
pub use widgets::{Scribble, ScribbleAction, ScribbleResponse};

pub use widgets::{CodeBlock, CodeHighlighter, CodeSpan};
#[cfg(feature = "markdown")]
pub use widgets::{LinkPolicy, Markdown, MarkdownLink, MarkdownResponse};

pub use widgets::{
    List, ListItem, ListResponse, ListRow, ListRowResponse, ListSelectionMode, ListState,
};

pub use widgets::{TreeNode, TreeRow, TreeState, TreeView, TreeViewResponse};

pub use widgets::{Avatar, AvatarVariant, Presence};
pub use widgets::{StatusAlign, StatusBar, StatusBarResponse, StatusItem, StatusTone};
