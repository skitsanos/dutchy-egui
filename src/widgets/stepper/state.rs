use super::Step;
/// Host-owned linear workflow position, separate from its field values or work.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum StepperState<T> {
    /// No step has started.
    #[default]
    NotStarted,
    /// This keyed step is current; preceding steps are displayed as completed.
    Current(T),
    /// Every step is completed. No current step remains.
    Complete,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Status {
    Completed,
    Current,
    Upcoming,
}
impl Status {
    pub fn name(self) -> &'static str {
        match self {
            Self::Completed => "Completed",
            Self::Current => "Current",
            Self::Upcoming => "Upcoming",
        }
    }
}
pub(super) struct Model {
    pub current: Option<usize>,
    pub total: usize,
    pub complete: bool,
    pub progress: f64,
    pub summary: String,
}
impl Model {
    pub fn new<T: PartialEq>(state: &StepperState<T>, steps: &[Step<'_, T>]) -> Self {
        let total = steps.len();
        let current = if let StepperState::Current(key) = state {
            steps.iter().position(|step| &step.value == key)
        } else {
            None
        };
        let complete = matches!(state, StepperState::Complete);
        let progress = if total == 0 {
            0.0
        } else if complete {
            100.0
        } else if total > 1 {
            current.map_or(0.0, |index| index as f64 / (total - 1) as f64 * 100.0)
        } else {
            0.0
        };
        let summary = if total == 0 {
            "No steps".into()
        } else {
            match state {
                StepperState::NotStarted => format!("Not started · {total} steps"),
                StepperState::Complete => format!("All steps completed · {total} steps"),
                StepperState::Current(_) => current.map_or_else(
                    || "Current step unavailable".into(),
                    |index| format!("Step {} of {total}", index + 1),
                ),
            }
        };
        Self {
            current,
            total,
            complete,
            progress,
            summary,
        }
    }
    pub fn status(&self, index: usize) -> Status {
        if self.complete || self.current.is_some_and(|current| index < current) {
            Status::Completed
        } else if self.current == Some(index) {
            Status::Current
        } else {
            Status::Upcoming
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn positions_and_completion_have_finite_explicit_boundaries() {
        let steps = [
            Step::new(1, "Account"),
            Step::new(2, "Details"),
            Step::new(3, "Review"),
        ];
        for (state, current, progress) in [
            (StepperState::NotStarted, None, 0.0),
            (StepperState::Current(1), Some(0), 0.0),
            (StepperState::Current(2), Some(1), 50.0),
            (StepperState::Current(3), Some(2), 100.0),
            (StepperState::Complete, None, 100.0),
            (StepperState::Current(99), None, 0.0),
        ] {
            let model = Model::new(&state, &steps);
            assert_eq!(model.current, current);
            assert_eq!(model.progress, progress);
            assert!(model.progress.is_finite());
        }
        let model = Model::new(&StepperState::Current(2), &steps);
        assert_eq!(
            [model.status(0), model.status(1), model.status(2)],
            [Status::Completed, Status::Current, Status::Upcoming]
        );
    }
    #[test]
    fn empty_and_single_step_do_not_divide_by_zero_or_imply_completion() {
        let empty: [Step<'_, u8>; 0] = [];
        assert_eq!(Model::new(&StepperState::Complete, &empty).progress, 0.0);
        let single = [Step::new(1, "Only step")];
        assert_eq!(Model::new(&StepperState::Current(1), &single).progress, 0.0);
        assert_eq!(Model::new(&StepperState::Complete, &single).progress, 100.0);
    }
}
