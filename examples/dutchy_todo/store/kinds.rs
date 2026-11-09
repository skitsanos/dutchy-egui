//! Value kinds shared by the store and the views: task priority and the
//! list filter, each with labels, exchange keys and Segmented options.

use super::Task;
use dutchy_egui::SelectOption;

/// Task urgency, shown as a badge and used for ordering.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Priority {
    /// Can wait.
    Low,
    /// Ordinary work.
    #[default]
    Medium,
    /// Needs attention first.
    High,
}

impl Priority {
    /// Low to high, for choice controls.
    pub const ALL: [Self; 3] = [Self::Low, Self::Medium, Self::High];

    /// Visible label.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
        }
    }

    /// Stable text form for the exchange format.
    pub const fn key(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }

    /// Parse the exchange-format key.
    pub fn parse(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|priority| priority.key() == key)
    }

    /// Choices for a Segmented control.
    pub fn options() -> Vec<SelectOption<'static, Self>> {
        Self::ALL
            .into_iter()
            .map(|priority| SelectOption::new(priority, priority.name()))
            .collect()
    }
}

/// Which tasks the list shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Filter {
    /// Every task in the project.
    #[default]
    All,
    /// Tasks still to do.
    Active,
    /// Completed tasks.
    Done,
}

impl Filter {
    /// Segment order.
    pub const ALL: [Self; 3] = [Self::All, Self::Active, Self::Done];

    /// Visible label.
    pub const fn name(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Active => "Active",
            Self::Done => "Done",
        }
    }

    /// Choices for a Segmented control.
    pub fn options() -> Vec<SelectOption<'static, Self>> {
        Self::ALL
            .into_iter()
            .map(|filter| SelectOption::new(filter, filter.name()))
            .collect()
    }

    pub(super) fn admits(self, task: &Task) -> bool {
        match self {
            Self::All => true,
            Self::Active => !task.done,
            Self::Done => task.done,
        }
    }
}
