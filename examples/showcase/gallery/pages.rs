#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Page {
    #[default]
    Overview,
    Typography,
    Forms,
    Cards,
    Controls,
    Colors,
    Alerts,
    Progress,
    Skeletons,
    Stats,
    EmptyStates,
    Tooltips,
    Layout,
    Accordions,
    Tabs,
    Menus,
    Dropdowns,
    Breadcrumbs,
    Pagination,
    Stepper,
    Modals,
    Drawers,
    Toasts,
    DataTables,
    Images,
    Galleries,
    Lightboxes,
    FileUploads,
    Scribble,
    Buttons,
    Badges,
    Motion,
    Tokens,
}

impl Page {
    /// Foundation pages lead the navigation in reading order.
    pub(crate) const FOUNDATION: [Self; 3] = [Self::Overview, Self::Typography, Self::Tokens];

    /// Component pages follow, sorted by display name.
    pub(crate) const COMPONENTS: [Self; 30] = [
        Self::Accordions,
        Self::Alerts,
        Self::Badges,
        Self::Breadcrumbs,
        Self::Buttons,
        Self::Cards,
        Self::Colors,
        Self::DataTables,
        Self::Drawers,
        Self::Dropdowns,
        Self::EmptyStates,
        Self::FileUploads,
        Self::Controls,
        Self::Forms,
        Self::Galleries,
        Self::Images,
        Self::Layout,
        Self::Lightboxes,
        Self::Menus,
        Self::Modals,
        Self::Motion,
        Self::Pagination,
        Self::Progress,
        Self::Scribble,
        Self::Skeletons,
        Self::Stats,
        Self::Stepper,
        Self::Tabs,
        Self::Toasts,
        Self::Tooltips,
    ];

    /// Every page in navigation order.
    pub(crate) fn all() -> impl Iterator<Item = Self> {
        Self::FOUNDATION.into_iter().chain(Self::COMPONENTS)
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Typography => "Typography",
            Self::Forms => "Forms",
            Self::Cards => "Cards",
            Self::Controls => "Form controls",
            Self::Colors => "Color picker",
            Self::Alerts => "Alerts",
            Self::Progress => "Progress",
            Self::Skeletons => "Skeletons",
            Self::Stats => "Stats",
            Self::EmptyStates => "Empty states",
            Self::Tooltips => "Tooltips",
            Self::Layout => "Layout",
            Self::Accordions => "Accordions",
            Self::Tabs => "Tabs",
            Self::Menus => "Menus",
            Self::Dropdowns => "Dropdowns",
            Self::Breadcrumbs => "Breadcrumbs",
            Self::Pagination => "Pagination",
            Self::Stepper => "Stepper",
            Self::Modals => "Modals",
            Self::Drawers => "Drawers",
            Self::Toasts => "Toasts",
            Self::DataTables => "Data tables",
            Self::Images => "Images",
            Self::Galleries => "Galleries",
            Self::Lightboxes => "Lightboxes",
            Self::Buttons => "Buttons",
            Self::Badges => "Badges",
            Self::Motion => "Motion",
            Self::Tokens => "Tokens",
            Self::FileUploads => "File uploads",
            Self::Scribble => "Scribble",
        }
    }
}
