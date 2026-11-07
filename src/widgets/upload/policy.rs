use std::path::Path;

/// Host-supplied file metadata. Contents and native/browser handles stay in the host.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileInfo {
    /// Display name, normally the basename rather than an absolute local path.
    pub name: String,
    /// Known byte size; unknown is distinct from an empty file.
    pub bytes: Option<u64>,
    /// MIME type, if supplied by the host/browser.
    pub mime: Option<String>,
}
impl FileInfo {
    /// Metadata with a name and unknown size/type; performs no I/O.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            bytes: None,
            mime: None,
        }
    }
    /// Extract only the display name. Does not stat or read the path.
    pub fn from_path(path: &Path) -> Self {
        Self::new(path.file_name().unwrap_or_default().to_string_lossy())
    }
    /// Supply a known byte size.
    pub fn size(mut self, bytes: u64) -> Self {
        self.bytes = Some(bytes);
        self
    }
    /// Supply a known MIME type.
    pub fn mime(mut self, mime: impl Into<String>) -> Self {
        self.mime = Some(mime.into());
        self
    }
}
/// Metadata rejection; hosts still validate content before processing/transferring.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileRejectionReason {
    /// Name has no non-whitespace characters.
    MissingName,
    /// No configured extension or MIME rule matched.
    UnsupportedType,
    /// A size limit exists, but byte size is unknown.
    UnknownSize,
    /// Byte size exceeds the configured limit.
    TooLarge {
        /// Maximum bytes.
        max_bytes: u64,
    },
    /// Existing and accepted files exhaust the count limit.
    TooMany {
        /// Maximum file count.
        max_files: usize,
    },
}
impl std::fmt::Display for FileRejectionReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingName => f.write_str("File name is missing"),
            Self::UnsupportedType => f.write_str("File type is not accepted"),
            Self::UnknownSize => f.write_str("File size is unavailable"),
            Self::TooLarge { max_bytes } => {
                write!(f, "File exceeds {}", file_size(Some(*max_bytes)))
            }
            Self::TooMany { max_files } => write!(f, "Choose at most {max_files} files"),
        }
    }
}
/// Rejected index in the submitted batch and its reason.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileRejection {
    /// Index in the candidate slice, not the retained file list.
    pub index: usize,
    /// Rejection explanation.
    pub reason: FileRejectionReason,
}
/// Ordered metadata-validation result; the host chooses what to retain.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileValidation {
    /// Accepted candidate indices in original order.
    pub accepted: Vec<usize>,
    /// Rejected indices in original order.
    pub rejected: Vec<FileRejection>,
}
/// Pure metadata policy shared by picked and dropped batches. No I/O or sniffing.
/// Extensions and MIME types are an OR list, like a browser `accept` attribute.
#[derive(Clone, Debug, Default)]
pub struct FileRules {
    extensions: Vec<String>,
    mime_types: Vec<String>,
    max_bytes: Option<u64>,
    max_files: Option<usize>,
}
impl FileRules {
    /// Unrestricted metadata policy.
    pub fn new() -> Self {
        Self::default()
    }
    /// Accepted extensions, case-insensitive, with or without a leading dot.
    /// Compound extensions such as `tar.gz` are supported. Replaces prior values.
    pub fn extensions(mut self, extensions: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        self.extensions = extensions
            .into_iter()
            .map(|x| {
                x.as_ref()
                    .trim()
                    .trim_start_matches('.')
                    .to_ascii_lowercase()
            })
            .filter(|x| !x.is_empty())
            .collect();
        self
    }
    /// Accepted exact MIME types or top-level wildcards such as `image/*`.
    /// Replaces prior values; unknown MIME can still match a configured extension.
    pub fn mime_types(mut self, types: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        self.mime_types = types
            .into_iter()
            .map(|x| x.as_ref().trim().to_ascii_lowercase())
            .filter(|x| !x.is_empty())
            .collect();
        self
    }
    /// Maximum bytes per file. Unknown sizes are rejected when this is set.
    pub fn max_bytes(mut self, bytes: u64) -> Self {
        self.max_bytes = Some(bytes);
        self
    }
    /// Maximum retained count, including `existing_count` supplied to validate.
    pub fn max_files(mut self, count: usize) -> Self {
        self.max_files = Some(count);
        self
    }
    /// Validate in order. Rejected candidates do not consume remaining capacity.
    pub fn validate(&self, files: &[FileInfo], existing_count: usize) -> FileValidation {
        let mut result = FileValidation::default();
        for (index, file) in files.iter().enumerate() {
            let count = existing_count.saturating_add(result.accepted.len());
            match self.rejection(file, count) {
                Some(reason) => result.rejected.push(FileRejection { index, reason }),
                None => result.accepted.push(index),
            }
        }
        result
    }
    fn rejection(&self, file: &FileInfo, count: usize) -> Option<FileRejectionReason> {
        if file.name.trim().is_empty() {
            return Some(FileRejectionReason::MissingName);
        }
        if !self.accepts(file) {
            return Some(FileRejectionReason::UnsupportedType);
        }
        if let Some(max_bytes) = self.max_bytes {
            match file.bytes {
                None => return Some(FileRejectionReason::UnknownSize),
                Some(bytes) if bytes > max_bytes => {
                    return Some(FileRejectionReason::TooLarge { max_bytes });
                }
                _ => {}
            }
        }
        self.max_files
            .filter(|max| count >= *max)
            .map(|max_files| FileRejectionReason::TooMany { max_files })
    }
    fn accepts(&self, file: &FileInfo) -> bool {
        if self.extensions.is_empty() && self.mime_types.is_empty() {
            return true;
        }
        let name = file.name.to_ascii_lowercase();
        if self
            .extensions
            .iter()
            .any(|ext| name.ends_with(&format!(".{ext}")))
        {
            return true;
        }
        let Some(mime) = &file.mime else {
            return false;
        };
        let mime = mime
            .split(';')
            .next()
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        self.mime_types.iter().any(|rule| {
            rule == &mime
                || rule.strip_suffix("/*").is_some_and(|prefix| {
                    mime.strip_prefix(prefix)
                        .is_some_and(|rest| rest.starts_with('/') && rest.len() > 1)
                })
        })
    }
}
/// Decimal byte units for file metadata; unknown size remains explicit.
pub fn file_size(bytes: Option<u64>) -> String {
    let Some(bytes) = bytes else {
        return "Size unknown".into();
    };
    let mut value = bytes as f64;
    let mut unit = "B";
    for next in ["KB", "MB", "GB", "TB", "PB", "EB"] {
        // Choose the unit after rounding, so 999,950 bytes read "1.0 MB".
        if value < 999.95 {
            break;
        }
        value /= 1000.0;
        unit = next;
    }
    if unit == "B" {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {unit}")
    }
}
