/// File read/write tools — sandboxed to the workspace BASE_DIR.
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};
use crate::config;

fn safe_path(rel: &str) -> Result<PathBuf> {
    let base = &config::get().base_dir;
    let resolved = base.join(rel).canonicalize()
        .unwrap_or_else(|_| base.join(rel));
    if !resolved.starts_with(base) {
        bail!("path outside workspace");
    }
    Ok(resolved)
}

pub fn file_read(path: &str) -> Result<String> {
    let p = safe_path(path)?;
    Ok(std::fs::read_to_string(p)?)
}

pub fn file_write(path: &str, content: &str) -> Result<String> {
    let p = safe_path(path)?;
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&p, content)?;
    Ok(format!("wrote {} chars to {path}", content.len()))
}
