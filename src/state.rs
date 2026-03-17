use std::sync::Arc;

use tokio::sync::Mutex;

use crate::error::Result;
use crate::store::Store;
use crate::types::{Annotation, CreateAnnotation, UpdateAnnotation};

/// Thread-safe wrapper around the annotation store for Tauri managed state.
#[derive(Clone)]
pub struct InstrucktState {
    store: Arc<Mutex<Store>>,
}

impl InstrucktState {
    pub fn new(store: Store) -> Self {
        Self {
            store: Arc::new(Mutex::new(store)),
        }
    }

    pub async fn read_all(&self) -> Result<Vec<Annotation>> {
        let store = self.store.lock().await;
        store.read_all()
    }

    pub async fn create_annotation(&self, input: CreateAnnotation) -> Result<Annotation> {
        let store = self.store.lock().await;
        store.create_annotation(input)
    }

    pub async fn update_annotation(&self, id: &str, input: UpdateAnnotation) -> Result<Annotation> {
        let store = self.store.lock().await;
        store.update_annotation(id, input)
    }

    pub async fn delete_annotation(&self, id: &str) -> Result<()> {
        let store = self.store.lock().await;
        store.delete_annotation(id)
    }
}
