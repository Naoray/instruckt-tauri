use std::sync::{Arc, Mutex};

use crate::store::Store;

pub struct InstrucktState {
    pub store: Arc<Mutex<Store>>,
}

impl InstrucktState {
    pub fn new(store: Store) -> Self {
        Self {
            store: Arc::new(Mutex::new(store)),
        }
    }
}
