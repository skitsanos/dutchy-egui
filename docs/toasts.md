# Toast notifications

`ToastQueue<T>` is owned by the application. It stores notification content,
optional typed action payloads, IDs, and transient lifetime state. `Toasts`
presents the queue without a modal layer or a full-screen input target. New
notifications never request keyboard focus. The host executes actions, keeps
business records, and decides what belongs in a notification history.

```rust
use dutchy_egui::{Toast, ToastEvent, ToastQueue, Toasts};

struct App {
    notices: ToastQueue<&'static str>,
    archives: usize,
}

impl App {
    fn archive(&mut self) {
        self.archives += 1;
        self.notices.push(
            Toast::new("The item was archived.")
                .title("Archived")
                .action("Undo", "undo-archive")
                .timeout(None),
        );
    }

    fn notifications(&mut self, ctx: &egui::Context) {
        for event in Toasts::new("app-notifications", &mut self.notices).show(ctx).events {
            if let ToastEvent::Action { action: "undo-archive", .. } = event {
                self.archives = self.archives.saturating_sub(1);
            }
        }
    }
}
```

Push **once per host event**, rather than on every drawing frame. Call the stack
once per frame, including when empty, so timing and focus lifecycle remain
current. Give each queue one stable, globally unique stack ID. Layout/sizing
passes can redraw it; the clock counts each sampled time interval once.

## Content and outcomes

`Toast::new(message)` defaults to informational, solid, enabled, dismissible,
announced, and a five-second lifetime. Builders set `title`, `variant`,
`appearance`, `action(label, payload)`, `timeout`, `dismissible`, `enabled`, or
`announce`. `ToastVariant` reuses the existing four-value `AlertVariant` intent
type; its toast announcement policy is described below.

An action removes the entry and returns `ToastEvent::Action { id, action }` once.
Payloads need no `Clone`. The host performs the operation. Manual close and
expiry return `ToastEvent::Dismissed` with `CloseButton` or `Expired`. Clicking the
message itself performs no action. Enter/Space activate native controls; there
is no global Escape handler or notification shortcut to intercept host input.

`push` returns a `ToastId` unique among live entries. `replace(id, toast)` keeps
identity/focus and restarts the visible lifetime; content can change without
stealing focus. `dismiss(id)` and `clear()` return owned content without generating
automatic dismissal events. `len` includes visible and waiting entries. The host
owns queue capacity, deduplication, retention, persistence, and any durable log;
there is no silent pending-entry eviction or background notification service.

`ToastsResponse` supplies optional stack geometry, rendered entry/control
responses, pending count, and events. Entries removed this frame can still be in
`visible`, because they were painted before processing activation/expiry.

## Position and presentation

`ToastPosition::{TopLeft, TopRight, BottomLeft, BottomRight}` selects the corner;
BottomRight is the default. Preferred width is 384 points, clamped to the viewport
with 24-point gutters. Entries have 12-point gaps and oldest-first reading order.
Three slots are visible by default; `max_visible(n)` clamps to 1–4. Additional
entries wait in FIFO order without aging. Long stacks/content scroll within the
available height. Keyboard focus reveals an off-screen control once, without animation; manual
scrolling is never undone.

`ToastAppearance::Solid` follows the live source: inverse information fill or
solid status color, uppercase bold 14-point text, 24x16 padding, and a translucent
4-point edge. `Outline` follows the documented alternative: background fill,
2-point severity outline, 16-point padding, normal-case title/body, and muted
supporting text. Existing sharp status symbols and close feedback are reused.
Optional actions use native interaction, wrapped labels, shared small-control
geometry, and surface-aware text/borders/focus. No gradients, radius, or blurred
shadows are introduced. Source-palette contrast limits remain in
[DS-003](issues/DS-003.md); applications can override semantic colors.

Entry fades over the shared 300ms motion token; reduced motion makes it immediate.
The fade advances by the queue's guarded clock delta, so a stalled or rewound
host clock pauses it without hiding the toast.
There is no slide or exit animation. Removing an entry immediately frees its
slot. Settled persistent/paused entries request no lifetime or animation frames.

## Lifetime and accessibility

`timeout(None)` persists until a user or host removes the toast. Finite requested
durations, including zero, clamp to a four-second minimum. A queued notification
starts its lifetime only when shown. Expiry pauses while hovered, a control is
focused/pressed, any part of the toast is clipped, the window is inactive, or the
native modal layer is active. `.paused(true)` explicitly pauses all lifetimes
while keeping controls usable. Resume does not charge the paused interval.
Expiry schedules the next deadline rather than polling every frame.

Native egui modality is reported from the preceding pass. Toast areas use the
Middle order, below modal/popup Foreground layers even during that transition.
For same-event timing policy, a host can also pass its own open flag to `paused`.
Inactive/modal stacks are hidden and retained. Keep calling the stack globally;
conditionally omitting it leaves its clock unsampled. It is in-app UI, not an OS
notification, and follows egui's input time/focus state.

Each toast is a named live region containing its native action/close controls.
Errors are assertive alerts; information, success, and warnings are polite status
messages. `announce(false)` uses a non-live group for history/static samples.
Countdown changes are not announced. Titles/messages retain their accessible
content; severity is also included in the region name. Close buttons are named
“Dismiss {title or severity}”. Arrival/content replacement never requests focus.

Removing a focused toast moves focus to another surviving toast control, then
the last outside control. `return_focus_to(actual_widget_id)` supplies a preferred
surviving host destination. Removing a toast while focus is elsewhere preserves
that focus. Missing/disabled targets are skipped. These policies also apply to
host removal/clear when the stack keeps being called. There is no screen-reader
acceptance claim from metadata tests alone.

## Reference and scope

Reviewed `docs/01-design-system/components/toast.md`,
`src/components/Toast/index.tsx`, `public/assets/js/toast.js`, and the registry at
web revision `104e4dab59fd94130b44fb276736c6be731e1ebf`. The source's solid live
style and outlined documented style differ; both are supported explicitly.
Manual close and typed actions extend the live JavaScript using the documented
interaction guidance. Pause/queue/focus policies avoid losing unread or focused
messages. CSS slide/exit effects and soft shadows are intentionally absent.
This is an original repository-owned implementation using egui only; no widget
crate or third-party implementation was copied.
