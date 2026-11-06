/// Host-owned expansion state. Unknown/temporarily hidden keys are retained.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AccordionState<T> {
    /// At most one open item; `None` closes all items.
    Single(Option<T>),
    /// Independently open items. Prefer unique keys in the initial vector.
    Multiple(Vec<T>),
}

impl<T> Default for AccordionState<T> {
    fn default() -> Self {
        Self::Multiple(Vec::new())
    }
}

impl<T: PartialEq> AccordionState<T> {
    /// Whether the item is currently expanded.
    pub fn is_open(&self, value: &T) -> bool {
        match self {
            Self::Single(selected) => selected.as_ref() == Some(value),
            Self::Multiple(open) => open.contains(value),
        }
    }
    /// Toggle an item, respecting the state's single/multiple policy.
    pub fn toggle(&mut self, value: T) {
        match self {
            Self::Single(selected) => {
                *selected = if selected.as_ref() == Some(&value) {
                    None
                } else {
                    Some(value)
                };
            }
            Self::Multiple(open) => {
                if open.contains(&value) {
                    open.retain(|key| key != &value);
                } else {
                    open.push(value);
                }
            }
        }
    }
}
