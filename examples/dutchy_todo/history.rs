//! Undo actions belong to one store lifetime, even if numeric IDs are reused.
use crate::store::{RemovedProject, Store, Task, TaskId};
use std::sync::Arc;

/// Single-use payload returned by an undo toast.
pub enum Undo {
    /// Put a deleted task back.
    Delete(Task),
    /// Reopen a completed task.
    Complete(TaskId),
    /// Put a deleted project and its tasks back.
    DeleteProject(RemovedProject),
    /// Restore the store that an import replaced.
    Import(Store),
}

/// An action with the identity of the store that offered it.
pub struct UndoRequest {
    scope: Arc<()>,
    action: Undo,
}

/// Old requests retain their identity, preventing reuse across replacements.
#[derive(Default)]
pub struct History(Arc<()>);

impl History {
    pub fn request(&self, action: Undo) -> UndoRequest {
        UndoRequest {
            scope: Arc::clone(&self.0),
            action,
        }
    }

    pub fn take(&self, request: UndoRequest) -> Option<Undo> {
        if Arc::ptr_eq(&self.0, &request.scope) {
            Some(request.action)
        } else {
            None
        }
    }

    pub fn invalidate(&mut self) {
        self.0 = Arc::new(());
    }
}
