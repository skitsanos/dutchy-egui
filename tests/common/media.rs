use egui::{
    Color32, ColorImage, Context, TextureHandle, TextureOptions,
    load::{LoadError, SizeHint, SizedTexture, TextureLoadResult, TextureLoader, TexturePoll},
};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Ready,
    Pending,
    Failed,
}
pub struct Probe {
    pub outcome: Outcome,
    pub loads: usize,
    pub forgets: usize,
}
impl Default for Probe {
    fn default() -> Self {
        Self {
            outcome: Outcome::Ready,
            loads: 0,
            forgets: 0,
        }
    }
}
pub type Shared = Arc<Mutex<Probe>>;
pub fn install(ctx: &Context, probe: Shared) {
    let mut image = ColorImage::filled([160, 80], Color32::GREEN);
    for y in 0..80 {
        for x in 0..40 {
            image.pixels[y * 160 + x] = Color32::RED;
            image.pixels[y * 160 + x + 120] = Color32::BLUE;
        }
    }
    let texture = ctx.load_texture("Fixture image", image, TextureOptions::NEAREST);
    ctx.add_texture_loader(Arc::new(Loader { probe, texture }));
}
struct Loader {
    probe: Shared,
    texture: TextureHandle,
}
impl TextureLoader for Loader {
    fn id(&self) -> &str {
        concat!(module_path!(), "::Loader")
    }
    fn load(&self, _: &Context, uri: &str, _: TextureOptions, _: SizeHint) -> TextureLoadResult {
        if !uri.starts_with("fixture://") {
            return Err(LoadError::NotSupported);
        }
        let mut probe = self.probe.lock().unwrap();
        probe.loads += 1;
        match probe.outcome {
            Outcome::Ready => Ok(TexturePoll::Ready {
                texture: SizedTexture::from_handle(&self.texture),
            }),
            Outcome::Pending => Ok(TexturePoll::Pending {
                size: Some(egui::vec2(160.0, 80.0)),
            }),
            Outcome::Failed => Err(LoadError::Loading("Fixture decoder failed".into())),
        }
    }
    fn forget(&self, _: &str) {
        self.probe.lock().unwrap().forgets += 1;
    }
    fn forget_all(&self) {}
    fn byte_size(&self) -> usize {
        160 * 80 * 4
    }
}
