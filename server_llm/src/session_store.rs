/// AppState wrapping the shared SessionStore.
use std::sync::Arc;
use common::memory_store::SessionStore;

#[derive(Clone)]
pub struct AppState {
    pub store: Arc<SessionStore>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            store: Arc::new(SessionStore::new()),
        }
    }
}
