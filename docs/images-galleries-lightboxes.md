# Images, galleries and lightboxes

`Image` wraps native egui image sources with sharp borders, explicit fitting,
caption/source typography, accessible loading/error states and independent
expand/retry controls. `Gallery` lays out keyed thumbnails. `Lightbox` binds a
host-owned selected key to a constrained fullscreen dialog. The default build
uses egui alone; optional `url-images` adds asynchronous HTTP(S) PNG/JPEG support.

## Images and native loaders

```rust
use dutchy_egui::{Image, ImageAspect, ImageFit};

fn picture(ui: &mut egui::Ui, texture: &egui::TextureHandle) {
    let result = Image::new("project-photo", texture, "Canal-side buildings")
        .aspect(ImageAspect::Video)
        .fit(ImageFit::Cover)
        .caption("A sharp, bordered image.")
        .caption_source("Project archive")
        .expandable(true)
        .show(ui);
    if result.expand_requested {
        // Set the application's selected image and remember result.expand's ID.
    }
}
```

The host must retain texture handles. `egui::ImageSource::Texture` works directly.
Enable **`url-images`** to install the built-in HTTP(S) PNG/JPEG texture loader
through `dutchy_egui::install`. See [URL image integration](url-images.md) for
configuration, demo commands, resource bounds and platform limits.
Other URI and encoded-byte sources use **host-installed egui loaders**.
The default feature set installs no codecs or network/file access.
Applications can use their existing media pipeline, implement egui's loader
traits, or install integrations they already use. Supplying encoded bytes alone
does not install a decoder. Missing loaders display explicit failure guidance.
Give byte sources stable, unique URIs with appropriate extensions; follow native
egui cache/forget policies when replacing data behind a URI.

`ImageStatus::{Ready, Loading, Failed(String)}` reports presentation/loader
outcomes. Diagnostics return to the host rather than appearing as implementation
details in the product UI. Loading reserves the image box, marks it busy and
paints a flat placeholder with shared skeleton motion, a rotating square spinner
and a centered “Loading image” label. The spinner has a named busy progress role
and uses the shared Spinner motion token. Feedback is indeterminate; no download
percentage is invented. Reduced motion keeps the spinner/placeholder static;
clipped placeholders schedule no animation frames. Small boxes omit the label
first, then the spinner if it cannot fit safely. Loading feedback disappears on
ready/error without affecting the reserved image box. URI/byte loading is deferred while
the image is clipped; such a presentation may report Loading until revealed.

`retryable(true)` adds a named retry button on failure. `retry_requested` is an
event only: the application decides whether to call `Context::forget_image`,
replace data, or perform an async retry. The component never retries every frame.

## Fitting, captions and controls

Images fill available width by default. `.width(...)` bounds the entire figure,
including caption/source. Unconstrained width falls back to 640 points. Explicit
dimensions clamp to 1–4096 points; nonfinite requests use defaults. Available
width remains authoritative. `.height(...)` overrides aspect-derived box height and keeps the chosen fit
(the web `height` prop crops); pair it with `.fit(ImageFit::Cover)` for that result.

`ImageAspect` offers Original, Square, Video (16:9), Standard (4:3) and Widescreen
(16:10). Original uses intrinsic dimensions, with a 16:9 fallback until known.
Choose a fixed aspect/height when replacement must avoid layout shift. Calling
`.aspect(...)` chooses Contain for Original and Cover for fixed aspects; a later
`.fit(...)` overrides it. Contain shows the full source with flat letterboxing.
Cover crops symmetrically using texture UVs. Neither stretches the source.

Borders are two points, corners zero. Captions use 14-point body text and a
12-point gap. Source credits use uppercase 12-point monospaced text with wider
(0.05 em) tracking. Credit sits at the trailing edge when both fit, otherwise below the
caption. Source-only credits are supported. Geometry comes from shared tokens.
Caption/source text honors enclosing card text-color overrides; loading/error
content retains the image box's own neutral surface colors.

Expansion is a separate, always-visible inverse button with flat accent hover,
pressed feedback, a pointer cursor and a keyboard focus cue. Standalone targets
are 40 points with a 12-point inset; gallery targets are 32 points inset eight.
Names include alternative text, e.g. “Expand Canal houses”. Clicking the image
surface does not expand it. Enter/Space activate controls normally. Pending and
failed sources remain expandable so lazy loading cannot block keyboard access.
`.enabled(false)` disables controls while retaining readable image content and
respects disabled parent UIs. Gaining focus, or a layout move, reveals clipped expand/retry controls once;
retained focus never pulls a scroll area back.

The accessibility tree has a named Figure, an Image with alternative text and
busy/failure metadata, a FigureCaption, and separate native action buttons. The
figure and the image share the alternative text; some screen readers announce it
twice, which is accepted so the figure stays navigable by name.
Always supply meaningful alternative text. These primitives do not infer it.

## Keyed galleries and viewer binding

```rust
use dutchy_egui::{Gallery, GalleryItem, Lightbox, OverlayOptions};

fn gallery_ui(
    ui: &mut egui::Ui,
    items: &[GalleryItem<'_, u64>],
    selected: &mut Option<u64>,
    opener: &mut Option<egui::Id>,
) {
    let gallery = Gallery::new("project-gallery", items).show(ui);
    if let Some(key) = gallery.open_requested {
        *selected = Some(key);
        *opener = gallery.images.iter()
            .find(|(item, _)| *item == key)
            .and_then(|(_, image)| image.expand.as_ref().map(|button| button.id));
    }
    Lightbox::new("project-viewer", items, selected)
        .options(OverlayOptions::default().return_focus(*opener))
        .show(ui.ctx());
}
```

Call the lightbox **globally each frame**, including while selection is None, so
deferred focus restoration can finish. The example above is compact; persistent
apps should place its lightbox call outside conditional page content.

`GalleryItem::new(key, source, alt)` adds optional caption/source and an enabled
flag. Keys must be unique and immutable. The default gallery has up to three
columns, 160-point desired minima, square cover thumbnails, two-point joined
gaps and an outer border. It adapts down to a single column. Builders set maximum
columns, typed minimum width, token gap, aspect, thumbnail captions, border and
enabled state. Joined galleries (the default micro gap) share one border-colored
surface that shows through the gaps and carry the outer border; spaced galleries
(any larger gap) paint nothing between tiles, and `bordered` then borders each
tile as the web image cards do. Gallery expand controls use the web gallery's
70 % inverse fill; standalone images keep 80 %. Captions are hidden in thumbnails by default but retained for the
viewer. Reordering/reflow preserves item control identities. Empty galleries
provide guidance; gallery opening returns a key without owning selection.

The lightbox mutates only the bound `Option<K>` in response to navigation or
dismissal. The host owns items, sources, selected key, loader policy and business
actions. Previous/Next and Left/Right navigate supplied order, skipping disabled
items. Home/End reach enabled endpoints. Ends disable by default;
`.wrap_navigation(true)` cycles. A singleton has no navigation buttons. Status
metadata announces the current position politely. Loading/error states retain
close/navigation controls, with retry available through `response.image`.

Missing, removed or disabled selected keys remain explicit in a closable
unavailable view; the component never chooses an unrelated image. Reordering
retains the selected key. The host can remove/replace selection at any time.

## Lightbox modality, dismissal and scope

The viewer consistently uses the source's dark neutral surface, with a 95%
backdrop and 24-point viewport clearance. It contains the full image, scrolls
long captions and stays constrained at narrow sizes. Page Up/Page Down scroll
the body; Close remains in the fixed header. This is an in-app viewer, not an
OS fullscreen or external-window API.

Native egui modality blocks the workspace and confines Tab/Shift+Tab. Focus
starts on Close, or the named heading if Close is explicitly hidden. The shared
`OverlayOptions` configures Escape/backdrop dismissal and initial/return focus.
Escape applies only to the top modal after child popups. Clicking the image
surface does not dismiss; backdrop dismissal covers the 24-point clearance
around the stage, where the web closes on any click outside the image. Closing by button/Escape/backdrop clears selection and
reports `OverlayDismissal`; host closure also finishes focus restoration. A
surviving explicit target handles removed openers. Host focus changes elsewhere
are preserved. Overlay focus/dismissal helpers are shared with Modal/Drawer.

Reviewed `docs/01-design-system/components/image.md`,
`src/components/Image/index.tsx`, `public/assets/js/image-expand.js` and the image
showcase at web revision `104e4dab59fd94130b44fb276736c6be731e1ebf`. This is original
repository-owned code. Original raster illustrations keep the default showcase offline. Optional
Picsum mode uses credited online photographs without bundling them. No widget/helper code was copied.
Gradient overlays from later web examples are excluded by Dutchy's flat-fill
contract. Image cards/hero compositions can use the existing layout/Card APIs.

Collections are not virtualized. Bound large datasets and own texture/cache
lifetimes. Zoom/pan, slideshow timers, swipe gestures, masonry, drag reorder,
downloads/export, additional codecs/file loaders and animated-format acceptance
are outside this step. Browser/OS/screen-reader acceptance remains separate from
metadata and rendered tests. See [DS-021](issues/DS-021.md) and
[verification](verification-media.md).
