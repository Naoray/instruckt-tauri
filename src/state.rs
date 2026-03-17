use std::sync::{Arc, Mutex};

use crate::error::Result;
use crate::store::Store;
use crate::types::{Annotation, CreateAnnotation, UpdateAnnotation};

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

    pub fn read_all(&self) -> Result<Vec<Annotation>> {
        let store = self.store.lock()?;
        store.read_all()
    }

    pub fn create_annotation(&self, data: CreateAnnotation) -> Result<Annotation> {
        let store = self.store.lock()?;
        store.create_annotation(data)
    }

    pub fn update_annotation(&self, id: &str, data: UpdateAnnotation) -> Result<Annotation> {
        let store = self.store.lock()?;
        store.update_annotation(id, data)
    }
}
