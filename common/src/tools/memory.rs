/// Read/write/search across the 3-layer memory system.
use anyhow::Result;
use chrono::Local;
use std::path::{Path, PathBuf};
use crate::config;

// ---------------------------------------------------------------
// Helpers

fn today_file() -> PathBuf {
    let cfg = config::get();
    let _ = std::fs::create_dir_all(&cfg.memory_dir);
    let today = Local::now().format("%Y-%m-%d").to_string();
    cfg.memory_dir.join(format!("{today}.md"))
}

fn sanitize(name: &str) -> String {
    let clean: String = name
        .trim()
        .to_lowercase()
        .replace(' ', "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    if clean.is_empty() { "untitled".to_string() } else { clean }
}

// ---------------------------------------------------------------
// Layer 2: daily memory

pub fn memory_append(text: &str) -> Result<String> {
    let f = today_file();
    let today = Local::now().format("%Y-%m-%d").to_string();
    let prefix = if f.exists() { String::new() } else { format!("# {today}\n\n") };
    let mut content = std::fs::read_to_string(&f).unwrap_or_default();
    content.push_str(&prefix);
    content.push_str(&format!("- {text}\n"));
    std::fs::write(&f, &content)?;
    Ok("ok".to_string())
}

pub fn memory_search(query: &str) -> Result<String> {
    let cfg = config::get();
    if !cfg.memory_dir.exists() {
        return Ok("no memory yet".to_string());
    }
    let mut results = Vec::new();
    for entry in walkdir_md(&cfg.memory_dir) {
        let text = std::fs::read_to_string(&entry).unwrap_or_default();
        for line in text.lines() {
            if line.to_lowercase().contains(&query.to_lowercase()) {
                let rel = entry.strip_prefix(&cfg.memory_dir).unwrap_or(&entry);
                results.push(format!("{}: {line}", rel.display()));
            }
        }
    }
    Ok(if results.is_empty() { "no matches".to_string() } else { results.join("\n") })
}

// ---------------------------------------------------------------
// Layer 1: soul

pub fn soul_read() -> Result<String> {
    let cfg = config::get();
    if !cfg.soul_dir.exists() {
        return Ok("no soul files found".to_string());
    }
    let mut parts = Vec::new();
    let mut files: Vec<_> = std::fs::read_dir(&cfg.soul_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |x| x == "md"))
        .collect();
    files.sort_by_key(|e| e.file_name());
    for entry in files {
        let text = std::fs::read_to_string(entry.path()).unwrap_or_default();
        let name = entry.file_name().to_string_lossy().to_string();
        parts.push(format!("--- {name} ---\n{text}"));
    }
    Ok(if parts.is_empty() { "empty soul".to_string() } else { parts.join("\n\n") })
}

// ---------------------------------------------------------------
// Layer 2: topics

pub fn memory_topic_write(topic: &str, content: &str) -> Result<String> {
    let cfg = config::get();
    std::fs::create_dir_all(&cfg.memory_topics_dir)?;
    let fname = sanitize(topic);
    let path = cfg.memory_topics_dir.join(format!("{fname}.md"));
    std::fs::write(&path, content)?;
    Ok(format!("wrote topic: {fname}.md"))
}

pub fn memory_topic_read(topic: &str) -> Result<String> {
    let cfg = config::get();
    let fname = sanitize(topic);
    let path = cfg.memory_topics_dir.join(format!("{fname}.md"));
    if !path.exists() {
        return Ok(format!("topic '{topic}' not found"));
    }
    Ok(std::fs::read_to_string(path)?)
}

pub fn memory_topic_list() -> Result<String> {
    let cfg = config::get();
    if !cfg.memory_topics_dir.exists() {
        return Ok("no topics yet".to_string());
    }
    let mut names: Vec<_> = std::fs::read_dir(&cfg.memory_topics_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |x| x == "md"))
        .map(|e| e.path().file_stem().unwrap_or_default().to_string_lossy().to_string())
        .collect();
    names.sort();
    Ok(if names.is_empty() { "no topics yet".to_string() } else { names.join("\n") })
}

// ---------------------------------------------------------------
// Layer 3: reference

pub fn reference_read(name: &str) -> Result<String> {
    let cfg = config::get();
    let fname = sanitize(name);
    let path = cfg.reference_dir.join(format!("{fname}.md"));
    if !path.exists() {
        return Ok(format!("reference '{name}' not found"));
    }
    Ok(std::fs::read_to_string(path)?)
}

pub fn reference_write(name: &str, content: &str) -> Result<String> {
    let cfg = config::get();
    std::fs::create_dir_all(&cfg.reference_dir)?;
    let fname = sanitize(name);
    let path = cfg.reference_dir.join(format!("{fname}.md"));
    std::fs::write(&path, content)?;
    Ok(format!("wrote reference: {fname}.md"))
}

pub fn reference_list() -> Result<String> {
    let cfg = config::get();
    if !cfg.reference_dir.exists() {
        return Ok("no references yet".to_string());
    }
    let mut names: Vec<_> = std::fs::read_dir(&cfg.reference_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |x| x == "md"))
        .map(|e| e.path().file_stem().unwrap_or_default().to_string_lossy().to_string())
        .collect();
    names.sort();
    Ok(if names.is_empty() { "no references yet".to_string() } else { names.join("\n") })
}

pub fn reference_search(query: &str) -> Result<String> {
    let cfg = config::get();
    if !cfg.reference_dir.exists() {
        return Ok("no references yet".to_string());
    }
    let mut results = Vec::new();
    for entry in walkdir_md(&cfg.reference_dir) {
        let text = std::fs::read_to_string(&entry).unwrap_or_default();
        for line in text.lines() {
            if line.to_lowercase().contains(&query.to_lowercase()) {
                let rel = entry.strip_prefix(&cfg.reference_dir).unwrap_or(&entry);
                results.push(format!("{}: {line}", rel.display()));
            }
        }
    }
    Ok(if results.is_empty() { "no matches".to_string() } else { results.join("\n") })
}

// ---------------------------------------------------------------
// Walk a directory recursively returning .md files

fn walkdir_md(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    fn recurse(dir: &Path, out: &mut Vec<PathBuf>) {
        if let Ok(rd) = std::fs::read_dir(dir) {
            let mut entries: Vec<_> = rd.filter_map(|e| e.ok()).collect();
            entries.sort_by_key(|e| e.file_name());
            for entry in entries {
                let path = entry.path();
                if path.is_dir() {
                    recurse(&path, out);
                } else if path.extension().map_or(false, |x| x == "md") {
                    out.push(path);
                }
            }
        }
    }
    recurse(dir, &mut out);
    out
}
