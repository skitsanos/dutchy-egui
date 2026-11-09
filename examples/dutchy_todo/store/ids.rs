//! Monotonic identifiers reserve the maximum value as the exhausted state.
use std::fmt;

/// The identifier has no valid successor, or no unused identifier remains.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IdExhausted;

impl fmt::Display for IdExhausted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("No unused identifiers remain.")
    }
}
impl std::error::Error for IdExhausted {}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Counter(u64);

impl Counter {
    pub fn allocate(&mut self) -> Result<u64, IdExhausted> {
        let id = self.0;
        self.0 = id.checked_add(1).ok_or(IdExhausted)?;
        Ok(id)
    }

    pub fn reserve(&mut self, id: u64) -> Result<(), IdExhausted> {
        self.0 = self.0.max(id.checked_add(1).ok_or(IdExhausted)?);
        Ok(())
    }
}
