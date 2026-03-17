use tauri::{command, State};

use crate::error::{Error, Result};
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
    input: CreateAnnotation,
) -> Result<Annotation> {
    if input.url.trim().is_empty() {
        return Err(Error::Validation("url cannot be empty".into()));
    }
    if input.comment.trim().is_empty() {
        return Err(Error::Validation("comment cannot be empty".into()));
    }

    let state = state.inner().clone();
    tokio::task::spawn_blocking(move || state.create_annotation(input))
        .await?
}

#[command]
pub async fn update_annotation(
    state: State<'_, InstrucktState>,
    id: String,
    input: UpdateAnnotation,
) -> Result<Annotation> {
    if id.trim().is_empty() {
        return Err(Error::Validation("annotation id cannot be empty".into()));
    }

    let state = state.inner().clone();
    tokio::task::spawn_blocking(move || state.update_annotation(&id, input))
        .await?
}
