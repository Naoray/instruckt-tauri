use tauri::{command, State};

use crate::error::Result;
use crate::state::InstrucktState;
use crate::types::{Annotation, CreateAnnotation, UpdateAnnotation};

#[command]
pub async fn get_annotations(state: State<'_, InstrucktState>) -> Result<Vec<Annotation>> {
    let store = state.store.clone();
    tokio::task::spawn_blocking(move || {
        let store = store.lock()?;
        store.read_all()
    })
    .await?
}

#[command]
pub async fn create_annotation(
    state: State<'_, InstrucktState>,
    data: CreateAnnotation,
) -> Result<Annotation> {
    let store = state.store.clone();
    tokio::task::spawn_blocking(move || {
        let store = store.lock()?;
        store.create_annotation(data)
    })
    .await?
}

#[command]
pub async fn update_annotation(
    state: State<'_, InstrucktState>,
    id: String,
    data: UpdateAnnotation,
) -> Result<Annotation> {
    let store = state.store.clone();
    tokio::task::spawn_blocking(move || {
        let store = store.lock()?;
        store.update_annotation(&id, data)
    })
    .await?
}
