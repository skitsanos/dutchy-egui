//! Deliberately pending host loader for the loading-state demonstration.
use egui::{
    Context, TextureOptions,
    load::{LoadError, SizeHint, TextureLoadResult, TextureLoader, TexturePoll},
};
pub struct Pending;
impl TextureLoader for Pending {
    fn id(&self) -> &str {
        concat!(module_path!(), "::Pending")
    }
    fn load(&self, _: &Context, uri: &str, _: TextureOptions, _: SizeHint) -> TextureLoadResult {
        if uri == "showcase://pending" {
            Ok(TexturePoll::Pending { size: None })
        } else {
            Err(LoadError::NotSupported)
        }
    }
    fn forget(&self, _: &str) {}
    fn forget_all(&self) {}
    fn byte_size(&self) -> usize {
        0
    }
}
