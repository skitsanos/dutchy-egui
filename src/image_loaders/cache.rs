use super::{UrlImageOptions, fetch};
use egui::{
    Context, TextureHandle, TextureOptions,
    load::{LoadError, SizeHint, SizedTexture, TextureLoadResult, TextureLoader, TexturePoll},
};
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

pub(super) const ID: &str = "dutchy-egui::url-images";
type Key = (String, TextureOptions);
struct Entry {
    request: Arc<fetch::Request>,
    texture: Option<TextureHandle>,
    last_used: u64,
}
pub(super) struct Loader {
    options: UrlImageOptions,
    entries: Mutex<HashMap<Key, Entry>>,
    active: Arc<AtomicUsize>,
}
impl Loader {
    pub fn new(options: UrlImageOptions) -> Self {
        Self {
            options,
            entries: Mutex::default(),
            active: Arc::default(),
        }
    }
    fn reserve(&self, entries: &mut HashMap<Key, Entry>, pass: u64) -> Result<bool, LoadError> {
        if self.active.load(Ordering::Acquire) >= self.options.max_concurrent.max(1) {
            return Ok(false);
        }
        if entries.len() >= self.options.max_entries.max(1) {
            // Retain the previous pass too: its meshes may still be rendered.
            let victim = entries
                .iter()
                .filter(|(_, entry)| entry.last_used < pass.saturating_sub(1))
                .min_by_key(|(_, entry)| entry.last_used)
                .map(|(key, _)| key.clone());
            let Some(victim) = victim else {
                return Err(LoadError::Loading(
                    "URL image cache is full; increase max_entries or show fewer images".into(),
                ));
            };
            entries.remove(&victim).unwrap().request.cancel();
        }
        Ok(true)
    }
}
impl TextureLoader for Loader {
    fn id(&self) -> &str {
        ID
    }
    fn load(
        &self,
        ctx: &Context,
        uri: &str,
        options: TextureOptions,
        _: SizeHint,
    ) -> TextureLoadResult {
        if !(uri.starts_with("https://") || uri.starts_with("http://")) {
            return Err(LoadError::NotSupported);
        }
        let key = (uri.to_owned(), options);
        let pass = ctx.cumulative_pass_nr();
        let mut entries = self.entries.lock().unwrap();
        if let Some(entry) = entries.get_mut(&key) {
            entry.last_used = pass;
            if let Some(texture) = &entry.texture {
                return Ok(TexturePoll::Ready {
                    texture: SizedTexture::from_handle(texture),
                });
            }
            let mut result = entry.request.result.lock().unwrap();
            match result.as_ref() {
                None => return Ok(TexturePoll::Pending { size: None }),
                Some(Err(error)) => return Err(LoadError::Loading(error.clone())),
                Some(Ok(_)) => {}
            }
            let image = result.take().unwrap().unwrap();
            let texture = ctx.load_texture(uri, image, options);
            let sized = SizedTexture::from_handle(&texture);
            entry.texture = Some(texture);
            return Ok(TexturePoll::Ready { texture: sized });
        }
        if !self.reserve(&mut entries, pass)? {
            return Ok(TexturePoll::Pending { size: None });
        }
        self.active.fetch_add(1, Ordering::AcqRel);
        let request = Arc::new(fetch::Request::default());
        entries.insert(
            key,
            Entry {
                request: request.clone(),
                texture: None,
                last_used: pass,
            },
        );
        drop(entries);
        fetch::start(ctx.clone(), uri, request, self.active.clone(), self.options);
        Ok(TexturePoll::Pending { size: None })
    }
    fn forget(&self, uri: &str) {
        self.entries.lock().unwrap().retain(|(url, _), entry| {
            if url == uri {
                entry.request.cancel();
                false
            } else {
                true
            }
        });
    }
    fn forget_all(&self) {
        let mut entries = self.entries.lock().unwrap();
        for entry in entries.values() {
            entry.request.cancel();
        }
        entries.clear();
    }
    fn byte_size(&self) -> usize {
        self.entries
            .lock()
            .unwrap()
            .values()
            .map(|entry| {
                entry.texture.as_ref().map_or_else(
                    || {
                        entry
                            .request
                            .result
                            .lock()
                            .unwrap()
                            .as_ref()
                            .and_then(|result| result.as_ref().ok())
                            .map_or(0, |image| image.pixels.len() * 4)
                    },
                    TextureHandle::byte_size,
                )
            })
            .sum()
    }
}
