use super::{Toast, ToastDismissal, ToastEvent, ToastId};
#[derive(Debug)]
pub(super) struct Entry<T> {
    pub id: ToastId,
    pub toast: Toast<T>,
    pub remaining: Option<f64>,
    pub running: bool,
    pub entrance: Option<f64>,
}
impl<T> Entry<T> {
    pub fn tick(&mut self, delta: f64, running: bool) -> bool {
        if running
            && self.running
            && let Some(remaining) = &mut self.remaining
        {
            *remaining = (*remaining - delta).max(0.0);
        }
        self.running = running;
        running && self.remaining == Some(0.0)
    }
}
/// Application-owned queue. Push once per host event, not once per drawing frame.
/// Pending entries are retained in FIFO order without aging. No action payload needs Clone.
#[derive(Debug)]
pub struct ToastQueue<T = ()> {
    pub(super) entries: Vec<Entry<T>>,
    next: u64,
    last_time: Option<f64>,
}
impl<T> Default for ToastQueue<T> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            next: 0,
            last_time: None,
        }
    }
}
impl<T> ToastQueue<T> {
    /// Empty host-owned notification queue.
    pub fn new() -> Self {
        Self::default()
    }
    /// Number of visible and pending notifications.
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    /// Whether no notifications remain.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    /// Enqueue owned content and return a stable live-entry key.
    pub fn push(&mut self, toast: Toast<T>) -> ToastId {
        let mut id = ToastId(self.next);
        while self.entries.iter().any(|entry| entry.id == id) {
            self.next = self.next.wrapping_add(1);
            id = ToastId(self.next);
        }
        self.next = self.next.wrapping_add(1);
        self.entries.push(Entry {
            id,
            remaining: toast.timeout.map(|duration| duration.as_secs_f64()),
            toast,
            running: false,
            entrance: None,
        });
        id
    }
    /// Replace content while retaining identity/focus. Restarts its visible lifetime.
    pub fn replace(&mut self, id: ToastId, toast: Toast<T>) -> Option<Toast<T>> {
        let entry = self.entries.iter_mut().find(|entry| entry.id == id)?;
        entry.remaining = toast.timeout.map(|duration| duration.as_secs_f64());
        entry.running = false;
        Some(std::mem::replace(&mut entry.toast, toast))
    }
    /// Remove explicitly, returning owned content. No automatic dismissal event is emitted.
    pub fn dismiss(&mut self, id: ToastId) -> Option<Toast<T>> {
        let index = self.entries.iter().position(|entry| entry.id == id)?;
        Some(self.entries.remove(index).toast)
    }
    /// Remove all notifications, returning their owned content in queue order.
    pub fn clear(&mut self) -> Vec<Toast<T>> {
        self.entries.drain(..).map(|entry| entry.toast).collect()
    }
    pub(super) fn delta(&mut self, time: f64) -> f64 {
        let time = if time.is_finite() {
            time
        } else {
            self.last_time.unwrap_or(0.0)
        };
        let delta = self
            .last_time
            .map_or(0.0, |before| (time - before).max(0.0));
        self.last_time = Some(time);
        delta
    }
    pub(super) fn pause(&mut self) {
        for entry in &mut self.entries {
            entry.running = false;
        }
    }
    pub(super) fn finish(
        &mut self,
        requests: Vec<(ToastId, Option<ToastDismissal>)>,
    ) -> Vec<ToastEvent<T>> {
        requests
            .into_iter()
            .filter_map(|(id, reason)| {
                let toast = self.dismiss(id)?;
                Some(match reason {
                    Some(reason) => ToastEvent::Dismissed { id, reason },
                    None => ToastEvent::Action {
                        id,
                        action: toast.action?.1,
                    },
                })
            })
            .collect()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn visible_time_only_advances_between_running_frames() {
        let mut queue: ToastQueue = ToastQueue::new();
        let id = queue.push(Toast::new("Message"));
        let entry = &mut queue.entries[0];
        assert!(!entry.tick(100.0, true));
        assert!(!entry.tick(1.0, true));
        assert_eq!(entry.remaining, Some(4.0));
        assert!(!entry.tick(10.0, false));
        assert!(!entry.tick(10.0, true));
        assert!(entry.tick(4.0, true));
        assert!(
            queue
                .replace(id, Toast::new("Updated").timeout(None))
                .is_some()
        );
        assert_eq!(queue.entries[0].remaining, None);
        assert!(!queue.entries[0].tick(f64::MAX, true));
    }
    #[test]
    fn clock_and_ids_are_bounded_and_outcomes_consume_payloads_once() {
        let mut queue = ToastQueue::new();
        queue.next = u64::MAX;
        let first = queue.push(Toast::new("First").action("Undo", String::from("undo")));
        let second = queue.push(Toast::new("Second"));
        assert_ne!(first, second);
        assert_eq!(queue.delta(2.0), 0.0);
        assert_eq!(queue.delta(f64::NAN), 0.0);
        assert_eq!(queue.delta(1.0), 0.0);
        assert_eq!(queue.delta(3.0), 2.0);
        assert_eq!(
            queue.finish(vec![(first, None), (first, None)]),
            [ToastEvent::Action {
                id: first,
                action: "undo".into()
            }]
        );
        assert_eq!(queue.clear().len(), 1);
        assert!(queue.is_empty());
    }
}
