use tauri::{command, State};

use crate::error::Result;
use crate::state::InstrucktState;
use crate::types::{Annotation, CreateAnnotation, UpdateAnnotation};

#[command]
pub async fn get_annotations(state: State<'_, InstrucktState>) -> Result<Vec<Annotation>> {
    let state = state.inner().clone();
    tokio::task::spawn_blocking(move || state.read_all())
        .await?
}

#[command]
pub async fn create_annotation(
    state: State<'_, InstrucktState>,
    data: CreateAnnotation,
) -> Result<Annotation> {
    let state = state.inner().clone();
    tokio::task::spawn_blocking(move || state.create_annotation(data))
        .await?
}

#[command]
pub async fn update_annotation(
    state: State<'_, InstrucktState>,
    id: String,
    data: UpdateAnnotation,
) -> Result<Annotation> {
    let state = state.inner().clone();
    tokio::task::spawn_blocking(move || state.update_annotation(&id, data))
        .await?
}
