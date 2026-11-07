pub(crate) mod jobs;
pub mod sample;
use super::section;
use dutchy_egui::{
    Alert, AlertVariant, Button, ButtonVariant, Checkbox, ControlSize, Scribble, ScribbleAction,
    ScribbleBrush, ScribbleDrawing, ScribbleExportOptions, ScribbleResponse, ScribbleWidth,
    Spinner, Text, TextColor,
    tokens::{border, scribble as tokens, space},
};
use egui::{Context, TextureHandle, Ui};
#[derive(Clone, Copy)]
struct PickerFocus {
    opener: egui::Id,
    previous: Option<egui::Id>,
    frame: u64,
}
#[derive(Default)]
pub struct SketchState {
    pub drawing: ScribbleDrawing,
    pub disabled: bool,
    pub read_only: bool,
    accent_ink: bool,
    bold: bool,
    paper: bool,
    pending: Option<jobs::Task>,
    preview: Option<(u64, TextureHandle)>,
    message: String,
    error: Option<String>,
    focus_return: Option<PickerFocus>,
}
impl SketchState {
    pub fn tick(&mut self, ctx: &Context) {
        self.tick_job(ctx);
        self.restore_focus(ctx);
    }
    fn restore_focus(&mut self, ctx: &Context) {
        let Some(focus) = self.focus_return else {
            return;
        };
        if ctx.cumulative_frame_nr() <= focus.frame {
            return;
        }
        let current = ctx.memory(|memory| memory.focused());
        if current.is_some() && current != Some(focus.opener) && current != focus.previous {
            // Preserve an explicit focus change while the worker was running.
            self.focus_return = None;
        } else if self.pending.is_none() && ctx.input(|input| input.focused) {
            // Export controls stay disabled until encoding/writing completes.
            ctx.memory_mut(|memory| memory.request_focus(focus.opener));
            self.focus_return = None;
        }
    }
    fn tick_job(&mut self, ctx: &Context) {
        let Some(task) = &self.pending else {
            return;
        };
        match task.try_recv() {
            Ok(Ok(product)) => {
                self.pending = None;
                self.error = None;
                match product {
                    jobs::Product::Preview { revision, image } => {
                        self.preview = Some((
                            revision,
                            ctx.load_texture(
                                "Drawing export preview",
                                image,
                                egui::TextureOptions::LINEAR,
                            ),
                        ));
                        self.message = "Preview generated locally.".into();
                    }
                    jobs::Product::Saved { name, bytes } => {
                        self.message = format!(
                            "Saved {name} · {}",
                            dutchy_egui::file_size(Some(bytes as u64))
                        );
                    }
                }
            }
            Ok(Err(error)) => {
                self.pending = None;
                self.error = Some(error);
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.pending = None;
                self.error = Some("The export did not finish. Try again.".into());
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
        }
    }
    pub fn show(&mut self, ui: &mut Ui) {
        ui.add(Text::heading("Make your mark."));
        ui.add(Text::new("Draw with a pointer or keyboard. The application retains the strokes; exports are generated locally."));
        ui.horizontal_wrapped(|ui| {
            ui.add(Checkbox::new(
                "disable-drawing",
                &mut self.disabled,
                "Disable drawing controls",
            ));
            ui.add(Checkbox::new(
                "readonly-drawing",
                &mut self.read_only,
                "Read-only drawing",
            ));
            ui.add(Checkbox::new(
                "accent-ink",
                &mut self.accent_ink,
                "Use accent ink",
            ));
            ui.add(Checkbox::new("bold-ink", &mut self.bold, "Use bold pen"));
            ui.add(Checkbox::new(
                "paper-export",
                &mut self.paper,
                "Include paper in export",
            ));
        });
        if ui
            .add(
                Button::new("Load sample drawing")
                    .variant(ButtonVariant::Outline)
                    .size(ControlSize::Small)
                    .enabled(!self.disabled && !self.read_only && self.pending.is_none()),
            )
            .clicked()
        {
            self.drawing = sample::drawing();
            self.preview = None;
            self.message = "Sample drawing loaded.".into();
        }
        section(ui, "01", "Pointer / keyboard / controlled history", |ui| {
            let color = if self.accent_ink {
                dutchy_egui::Theme::from_context(ui.ctx()).palette.primary
            } else {
                tokens::INK
            };
            let brush = ScribbleBrush::new().color(color).width(if self.bold {
                ScribbleWidth::Bold
            } else {
                ScribbleWidth::Regular
            });
            let response = Scribble::new("main-pad", &mut self.drawing)
                .label("Project drawing")
                .brush(brush)
                .enabled(!self.disabled)
                .read_only(self.read_only)
                .exports_enabled(self.pending.is_none())
                .show(ui);
            self.handle(ui, response);
            ui.add(
                Text::label(format!(
                    "{} strokes · {} points",
                    self.drawing.strokes().len(),
                    self.drawing.point_count()
                ))
                .color(TextColor::Muted),
            );
        });
        if self.pending.is_some() {
            ui.horizontal(|ui| {
                ui.add(Spinner::new());
                ui.add(Text::new("Generating export…"));
            });
        }
        if !self.message.is_empty() {
            ui.add(Text::new(&self.message).color(TextColor::Muted));
        }
        if let Some(error) = &self.error {
            Alert::new("drawing-error", error.as_str())
                .title("Drawing action could not finish")
                .variant(AlertVariant::Error)
                .show(ui);
        }
        section(
            ui,
            "02",
            "Snapshot preview / transparent or paper export",
            |ui| {
                if let Some((revision, texture)) = &self.preview {
                    if *revision != self.drawing.revision() {
                        ui.add(
                            Text::new(
                                "The drawing changed. Generate another preview to refresh it.",
                            )
                            .color(TextColor::Muted),
                        );
                    }
                    let p = dutchy_egui::Theme::from_context(ui.ctx()).palette;
                    egui::Frame::NONE
                        .fill(tokens::PAPER)
                        .stroke(egui::Stroke::new(border::STANDARD, p.border))
                        .inner_margin(space::MD)
                        .show(ui, |ui| {
                            let width = ui.available_width().min(tokens::WIDTH as f32);
                            ui.add(
                                egui::Image::from_texture(texture)
                                    .fit_to_exact_size(egui::vec2(width, width / 3.0))
                                    .bg_fill(tokens::PAPER)
                                    .alt_text("Exported drawing preview"),
                            );
                        });
                } else {
                    ui.add(Text::new(
                        "Choose Preview after drawing to inspect a completed snapshot.",
                    ));
                }
                ui.add(Text::new("The preview sits on light paper. The export background follows the checkbox above; frames, controls and the pen cursor are excluded."));
            },
        );
        section(ui, "03", "Same host data / read-only view", |ui| {
            Scribble::new("readonly-pad", &mut self.drawing)
                .label("Read-only copy")
                .read_only(true)
                .controls(false)
                .instructions(false)
                .show(ui);
        });
    }
    fn handle(&mut self, ui: &Ui, response: ScribbleResponse) {
        if let Some(error) = &response.error {
            self.error = Some(error.to_string());
        }
        for action in response.actions.iter().copied() {
            match action {
                ScribbleAction::Undo => {
                    self.drawing.undo();
                    response.canvas.request_focus();
                }
                ScribbleAction::Redo => {
                    self.drawing.redo();
                    response.canvas.request_focus();
                }
                ScribbleAction::Clear => {
                    self.drawing.clear();
                    response.canvas.request_focus();
                }
                ScribbleAction::Preview => {
                    self.pending = Some(jobs::preview(
                        ui.ctx().clone(),
                        self.drawing.snapshot(),
                        self.options(1.0),
                    ));
                }
                ScribbleAction::ExportSvg => self.save(ui, &response, jobs::Format::Svg),
                #[cfg(feature = "png-export")]
                ScribbleAction::ExportPng => self.save(ui, &response, jobs::Format::Png),
            }
        }
    }
    fn options(&self, scale: f32) -> ScribbleExportOptions {
        ScribbleExportOptions {
            scale,
            background: self.paper.then_some(tokens::PAPER),
        }
    }
    fn save(&mut self, ui: &Ui, response: &ScribbleResponse, format: jobs::Format) {
        let task = jobs::choose_path(format).map(|path| {
            jobs::save(
                ui.ctx().clone(),
                self.drawing.snapshot(),
                self.options(2.0),
                format,
                path,
            )
        });
        let id = response
            .action_response
            .as_ref()
            .unwrap_or(&response.canvas)
            .id;
        self.finish_picker(ui.ctx(), id, task);
    }
    pub(crate) fn finish_picker(
        &mut self,
        ctx: &Context,
        opener: egui::Id,
        task: Option<jobs::Task>,
    ) {
        self.pending = task;
        self.focus_return = Some(PickerFocus {
            opener,
            previous: ctx.memory(|memory| memory.focused()),
            frame: ctx.cumulative_frame_nr(),
        });
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        ctx.request_repaint();
    }
}
