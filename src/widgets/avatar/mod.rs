mod initials;

use super::media::geometry;
use crate::{
    ImageFit, Text, TextColor, Theme, Weight,
    tokens::{AvatarSize, Palette, avatar as tokens},
};
use egui::{
    Color32, ImageSource, Rect, Response, Sense, Stroke, StrokeKind, Ui, Widget, WidgetInfo,
    accesskit::Role,
    load::{SizedTexture, TexturePoll},
    pos2, vec2,
};
use std::borrow::Cow;

/// Fill treatment of the initials square.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum AvatarVariant {
    /// Muted surface with foreground initials.
    #[default]
    Muted,
    /// Foreground block with background initials.
    Default,
    /// Brand accent with on-accent initials.
    Primary,
    /// Strong border and transparent fill.
    Outline,
}

impl AvatarVariant {
    /// Every variant in display order.
    pub const ALL: &[Self] = &[Self::Muted, Self::Default, Self::Primary, Self::Outline];

    /// Human-readable variant name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Muted => "Muted",
            Self::Default => "Default",
            Self::Primary => "Primary",
            Self::Outline => "Outline",
        }
    }

    fn colors(self, p: &Palette) -> (Color32, Color32, Stroke) {
        match self {
            Self::Muted => (p.muted, p.foreground, Stroke::NONE),
            Self::Default => (p.foreground, p.background, Stroke::NONE),
            Self::Primary => (p.primary, p.on_primary, Stroke::NONE),
            Self::Outline => (
                Color32::TRANSPARENT,
                p.foreground,
                Stroke::new(tokens::OUTLINE, p.foreground),
            ),
        }
    }
}

/// Host-reported availability, shown as a square marker and described in words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Presence {
    /// Available.
    Online,
    /// Idle or away.
    Away,
    /// Do not disturb.
    Busy,
    /// Not connected.
    Offline,
}

impl Presence {
    /// Every presence state in display order.
    pub const ALL: &[Self] = &[Self::Online, Self::Away, Self::Busy, Self::Offline];

    /// Human-readable state, also the accessible description.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Online => "Online",
            Self::Away => "Away",
            Self::Busy => "Busy",
            Self::Offline => "Offline",
        }
    }

    fn color(self, p: &Palette) -> Color32 {
        match self {
            Self::Online => p.success,
            Self::Away => p.warning,
            Self::Busy => p.destructive,
            Self::Offline => p.muted_foreground,
        }
    }
}

/// Square identity mark: initials from a display name, or an image once it
/// has loaded. Passive: it never takes focus or clicks.
#[derive(Clone, Debug)]
#[must_use = "Add this avatar to a Ui"]
pub struct Avatar<'a> {
    name: Cow<'a, str>,
    initials: Option<Cow<'a, str>>,
    image: Option<ImageSource<'a>>,
    size: AvatarSize,
    variant: AvatarVariant,
    presence: Option<Presence>,
    accessible_name: Option<Cow<'a, str>>,
}

impl<'a> Avatar<'a> {
    /// An avatar for a display name, which is also its accessible name.
    pub fn new(name: impl Into<Cow<'a, str>>) -> Self {
        Self {
            name: name.into(),
            initials: None,
            image: None,
            size: AvatarSize::default(),
            variant: AvatarVariant::default(),
            presence: None,
            accessible_name: None,
        }
    }
    /// Initials to show as written, replacing the derived ones.
    pub fn initials(mut self, initials: impl Into<Cow<'a, str>>) -> Self {
        self.initials = Some(initials.into());
        self
    }
    /// Image cropped to the square once loaded; initials show until then and
    /// when it fails. Uses the context's installed image loaders.
    pub fn image(mut self, source: impl Into<ImageSource<'a>>) -> Self {
        self.image = Some(source.into());
        self
    }
    /// Side tier.
    pub fn size(mut self, size: AvatarSize) -> Self {
        self.size = size;
        self
    }
    /// Initials fill treatment.
    pub fn variant(mut self, variant: AvatarVariant) -> Self {
        self.variant = variant;
        self
    }
    /// Presence marker and description.
    pub fn presence(mut self, presence: Presence) -> Self {
        self.presence = Some(presence);
        self
    }
    /// Accessible name, overriding the display name.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.accessible_name = Some(name.into());
        self
    }

    /// Poll the image, only while the avatar is visible.
    fn texture(&self, ui: &Ui, rect: Rect) -> Image {
        let Some(source) = &self.image else {
            return Image::None;
        };
        if !ui.is_rect_visible(rect) {
            return Image::None;
        }
        match egui::Image::new(source.clone())
            .fit_to_exact_size(rect.size())
            .load_for_size(ui.ctx(), rect.size())
        {
            Ok(TexturePoll::Ready { texture }) if geometry::valid(texture.size) => {
                Image::Ready(texture)
            }
            Ok(TexturePoll::Pending { .. }) => Image::Loading,
            _ => Image::None,
        }
    }

    fn describe(&self, response: &Response, loading: bool) {
        let name = match &self.accessible_name {
            Some(name) => name.to_string(),
            None if self.name.trim().is_empty() => "Avatar".to_owned(),
            None => self.name.to_string(),
        };
        response.widget_info(|| WidgetInfo::labeled(Role::Image, response.enabled(), &name));
        let presence = self.presence.map(Presence::name);
        response.ctx.accesskit_node_builder(response.id, |node| {
            node.set_label(name.clone());
            if let Some(presence) = presence {
                node.set_description(presence);
            }
            if loading {
                node.set_busy();
            }
        });
    }

    fn paint(&self, ui: &Ui, rect: Rect, image: &Image) {
        let p = Theme::from_context(ui.ctx()).palette;
        let (fill, text, stroke) = self.variant.colors(&p);
        ui.painter().rect_filled(rect, 0, fill);
        if let Image::Ready(texture) = image {
            let (destination, uv) = geometry::placement(rect, texture.size, ImageFit::Cover);
            ui.painter()
                .with_clip_rect(ui.clip_rect().intersect(rect))
                .image(texture.id, destination, uv, Color32::WHITE);
        } else {
            self.paint_initials(ui, rect, text);
        }
        if stroke != Stroke::NONE {
            ui.painter()
                .rect_stroke(rect, 0, stroke, StrokeKind::Inside);
        }
        if let Some(presence) = self.presence {
            let side = self.size.marker();
            let marker =
                Rect::from_min_size(rect.right_bottom() - vec2(side, side), vec2(side, side));
            let ring = Rect::from_min_max(
                marker.min - vec2(tokens::MARKER_RING, tokens::MARKER_RING),
                marker.max,
            );
            ui.painter().rect_filled(ring, 0, p.background);
            ui.painter().rect_filled(marker, 0, presence.color(&p));
        }
    }

    fn paint_initials(&self, ui: &Ui, rect: Rect, color: Color32) {
        let initials = self
            .initials
            .as_deref()
            .map_or_else(|| initials::initials(&self.name), str::to_owned);
        if initials.is_empty() {
            return;
        }
        let theme = Theme::from_context(ui.ctx());
        let job = Text::new(initials)
            .size(self.size.text_size())
            .weight(Weight::Bold)
            .color(TextColor::Custom(color))
            .layout_job(&theme, f32::INFINITY);
        let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
        let position = pos2(
            rect.center().x - galley.size().x / 2.0,
            rect.center().y - galley.size().y / 2.0,
        );
        ui.painter()
            .with_clip_rect(ui.clip_rect().intersect(rect))
            .galley(position, galley, color);
    }
}

/// Image state for one frame.
enum Image {
    None,
    Loading,
    Ready(SizedTexture),
}

impl Widget for Avatar<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let side = self.size.side();
        let (rect, response) = ui.allocate_exact_size(vec2(side, side), Sense::hover());
        let image = self.texture(ui, rect);
        self.describe(&response, matches!(image, Image::Loading));
        if ui.is_rect_visible(rect) {
            self.paint(ui, rect, &image);
        }
        response
    }
}
