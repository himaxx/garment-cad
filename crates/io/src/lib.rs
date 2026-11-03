//! File import/export and plot output.

pub mod hpgl;
pub mod richpeace;
pub mod sheet;
pub mod svg;

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("file not recognised: {0}")]
    NotRecognised(String),
    #[error("unsupported file content: {0}")]
    Unsupported(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Import a pattern file, choosing the reader from the file extension.
pub fn import_path(path: &std::path::Path) -> Result<gcad_model::Style, ImportError> {
    let bytes = std::fs::read(path)?;
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("style");
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("dgs") => richpeace::dgs::read(&bytes, stem),
        other => Err(ImportError::NotRecognised(format!(
            "unsupported extension {other:?}"
        ))),
    }
}
