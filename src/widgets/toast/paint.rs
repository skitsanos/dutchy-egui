use super::{Toast, ToastAppearance, ToastId, ToastResponse, ToastVariant, controls};
use crate::{
    Text, TextSize, Theme, Weight,
    tokens::{border, space, toast as geometry},
    widgets::alert::icon,
};
use egui::{Color32, Frame, Id, Margin, Ui, UiBuilder};
struct Colors {
    fill: Color32,
    text: Color32,
    accent: Color32,
}
fn colors<T>(toast: &Toast<T>, theme: Theme) -> Colors {
    let p = theme.palette;
    let accent = match toast.variant {
        ToastVariant::Info => p.foreground,
        ToastVariant::Success => p.success,
        ToastVariant::Warning => p.warning,
        ToastVariant::Error => p.destructive,
    };
    if toast.appearance == ToastAppearance::Outline {
        return Colors {
            fill: p.background,
            text: p.foreground,
            accent: if toast.variant == ToastVariant::Info {
                p.primary
            } else {
                accent
            },
        };
    }
    Colors {
        fill: accent,
        text: if toast.variant == ToastVariant::Info {
            p.background
        } else {
            p.on_status
        },
        accent,
    }
}
pub(super) fn row<T>(
    ui: &mut Ui,
    root: Id,
    id: ToastId,
    toast: &Toast<T>,
    opacity: f32,
) -> ToastResponse {
    let colors = colors(toast, Theme::from_context(ui.ctx()));
    let solid = toast.appearance == ToastAppearance::Solid;
    let mut margin = if solid {
        Margin::symmetric(geometry::PADDING_X, geometry::PADDING_Y)
    } else {
        Margin::same(geometry::OUTLINE_PADDING)
    };
    if solid {
        margin.left += border::ACCENT as i8;
    }
    let stroke = if solid {
        egui::Stroke::NONE
    } else {
        egui::Stroke::new(border::STANDARD, colors.accent)
    };
    let result = ui.scope_builder(UiBuilder::new().scope_id(root.with(id)), |ui| {
        ui.multiply_opacity(opacity);
        if !toast.enabled {
            ui.disable();
        }
        Frame::NONE
            .fill(colors.fill)
            .stroke(stroke)
            .inner_margin(margin)
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.spacing_mut().item_spacing.x = geometry::CONTENT_GAP;
                let reserve = crate::tokens::surface::STATUS_ICON
                    + geometry::CONTENT_GAP
                    + if toast.dismissible {
                        crate::ControlSize::Small.height() + geometry::CONTENT_GAP
                    } else {
                        0.0
                    };
                let width = (ui.available_width() - reserve).max(0.0);
                ui.horizontal_top(|ui| {
                    icon::status(
                        ui,
                        toast.variant,
                        if solid { colors.text } else { colors.accent },
                    );
                    let action = ui
                        .vertical(|ui| {
                            ui.set_width(width);
                            ui.spacing_mut().item_spacing.y = space::XS;
                            if let Some(title) = &toast.title {
                                ui.add(
                                    Text::new(title)
                                        .size(TextSize::Sm)
                                        .weight(Weight::Bold)
                                        .uppercase(solid)
                                        .color(colors.text),
                                );
                            }
                            ui.add(
                                Text::new(&toast.message)
                                    .size(TextSize::Sm)
                                    .weight(if solid { Weight::Bold } else { Weight::Regular })
                                    .uppercase(solid)
                                    .color(if solid {
                                        colors.text
                                    } else {
                                        Theme::from_context(ui.ctx()).palette.muted_foreground
                                    }),
                            );
                            toast.action.as_ref().map(|(label, _)| {
                                ui.add_space(geometry::ACTION_GAP);
                                controls::action(ui, root.with((id, "action")), label, colors.text)
                            })
                        })
                        .inner;
                    let dismiss = toast.dismissible.then(|| {
                        icon::dismiss(
                            ui,
                            &format!(
                                "Dismiss {}",
                                toast.title.as_deref().unwrap_or(toast.variant.name())
                            ),
                            colors
                                .text
                                .gamma_multiply(crate::tokens::close::IDLE_OPACITY),
                            colors.text,
                        )
                    });
                    (dismiss, action)
                })
                .inner
            })
    });
    if solid {
        let mut edge = result.response.rect;
        edge.max.x = edge.min.x + border::ACCENT;
        ui.painter().rect_filled(
            edge,
            0,
            Theme::from_context(ui.ctx())
                .palette
                .background
                .gamma_multiply(geometry::EDGE_OPACITY * opacity),
        );
    }
    metadata(&result.response, toast);
    ToastResponse {
        id,
        response: result.response,
        close: result.inner.inner.0,
        action: result.inner.inner.1,
    }
}
fn metadata<T>(response: &egui::Response, toast: &Toast<T>) {
    use egui::accesskit::{Live, Role};
    response.ctx.accesskit_node_builder(response.id, |node| {
        let (role, live) = if !toast.announce {
            (Role::Group, Live::Off)
        } else if toast.variant == ToastVariant::Error {
            (Role::Alert, Live::Assertive)
        } else {
            (Role::Status, Live::Polite)
        };
        node.set_role(role);
        node.set_live(live);
        node.set_label(toast.name());
        node.set_description(toast.message.clone());
    });
}
