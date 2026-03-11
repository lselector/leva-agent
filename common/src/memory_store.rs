/// Per-session short-term message history (in-memory + disk).
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use anyhow::Result;
use crate::types::ChatMessage;
use crate::config;

const MAX_MESSAGES: usize = 50;

// ---------------------------------------------------------------

#[derive(Clone)]
pub struct SessionStore {
    inner: Arc<RwLock<HashMap<String, Vec<ChatMessage>>>>,
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn sessions_dir() -> std::path::PathBuf {
        config::get().memory_dir.join("sessions")
    }

    fn session_path(sid: &str) -> std::path::PathBuf {
        let safe = sid.replace('/', "_");
        Self::sessions_dir().join(format!("{safe}.json"))
    }

    /// Load session from disk into cache if not already loaded.
    fn load(&self, sid: &str) -> Vec<ChatMessage> {
        {
            let cache = self.inner.read().unwrap();
            if let Some(msgs) = cache.get(sid) {
                return msgs.clone();
            }
        }
        // Not in cache — try disk
        let path = Self::session_path(sid);
        let msgs: Vec<ChatMessage> = if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let mut cache = self.inner.write().unwrap();
        cache.insert(sid.to_string(), msgs.clone());
        msgs
    }

    fn save(&self, sid: &str) {
        let dir = Self::sessions_dir();
        let _ = std::fs::create_dir_all(&dir);
        let path = Self::session_path(sid);
        let cache = self.inner.read().unwrap();
        if let Some(msgs) = cache.get(sid) {
            if let Ok(json) = serde_json::to_string_pretty(msgs) {
                let _ = std::fs::write(path, json);
            }
        }
    }

    pub fn get_messages(&self, sid: &str) -> Vec<ChatMessage> {
        self.load(sid)
    }

    pub fn add_message(&self, sid: &str, role: &str, content: &str) {
        self.load(sid); // ensure in cache
        let mut cache = self.inner.write().unwrap();
        let msgs = cache.entry(sid.to_string()).or_default();
        msgs.push(ChatMessage {
            role: role.to_string(),
            content: serde_json::Value::String(content.to_string()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        });
        if msgs.len() > MAX_MESSAGES {
            let drain_to = msgs.len() - MAX_MESSAGES;
            msgs.drain(0..drain_to);
        }
        drop(cache);
        self.save(sid);
    }

    pub fn list_sessions(&self) -> Vec<String> {
        let dir = Self::sessions_dir();
        if !dir.exists() {
            return Vec::new();
        }
        let mut names: Vec<String> = std::fs::read_dir(&dir)
            .map(|rd| {
                rd.filter_map(|e| {
                    let e = e.ok()?;
                    let name = e.file_name().into_string().ok()?;
                    name.strip_suffix(".json").map(|s| s.to_string())
                })
                .collect()
            })
            .unwrap_or_default();
        names.sort();
        names
    }

    pub fn get_session(&self, sid: &str) -> Option<Vec<ChatMessage>> {
        let path = Self::session_path(sid);
        let cache = self.inner.read().unwrap();
        if cache.contains_key(sid) || path.exists() {
            drop(cache);
            Some(self.load(sid))
        } else {
            None
        }
    }

    pub fn delete_session(&self, sid: &str) -> bool {
        let mut cache = self.inner.write().unwrap();
        cache.remove(sid);
        drop(cache);
        let path = Self::session_path(sid);
        if path.exists() {
            let _ = std::fs::remove_file(&path);
            true
        } else {
            false
        }
    }
}
