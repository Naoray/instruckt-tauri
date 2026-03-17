use std::io::Write;
use std::path::PathBuf;

use fs2::FileExt;

use crate::error::{Error, Result};
use crate::screenshot;
use crate::types::{Annotation, AnnotationStatus, CreateAnnotation, UpdateAnnotation};

/// JSON file-based annotation store.
///
/// Storage layout:
/// ```text
/// {data_dir}/
/// ├── annotations.json
/// ├── annotations.lock
/// └── screenshots/
///     ├── {id}.png
///     └── {id}.svg
/// ```
pub struct Store {
    data_dir: PathBuf,
}

impl Store {
    pub fn new(data_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&data_dir)?;
        Ok(Self { data_dir })
    }

    /// Default data directory: `~/Library/Application Support/instruckt/` (macOS)
    /// or equivalent on other platforms via `dirs::data_dir()`.
    pub fn default_data_dir() -> Result<PathBuf> {
        dirs::data_dir()
            .map(|dir| dir.join("instruckt"))
            .ok_or_else(|| Error::Other("Could not determine OS data directory".into()))
    }

    fn annotations_path(&self) -> PathBuf {
        self.data_dir.join("annotations.json")
    }

    fn lock_path(&self) -> PathBuf {
        self.data_dir.join("annotations.lock")
    }

    fn screenshots_dir(&self) -> PathBuf {
        self.data_dir.join("screenshots")
    }

    /// Acquire an exclusive lock on the lockfile for cross-process safety.
    /// Returns the locked file handle — lock is released when the handle is dropped.
    fn lock_exclusive(&self) -> Result<std::fs::File> {
        let lock_file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(self.lock_path())?;
        lock_file.lock_exclusive()?;
        Ok(lock_file)
    }

    /// Acquire a shared lock for reading.
    fn lock_shared(&self) -> Result<std::fs::File> {
        let lock_file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(self.lock_path())?;
        lock_file.lock_shared()?;
        Ok(lock_file)
    }

    /// Parse the annotations JSON file, returning an empty vec if the file
    /// does not exist or is empty. Caller is responsible for holding the
    /// appropriate lock.
    fn parse_annotations_file(&self) -> Result<Vec<Annotation>> {
        let path = self.annotations_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let contents = std::fs::read_to_string(&path)?;
        if contents.trim().is_empty() {
            return Ok(Vec::new());
        }
        let annotations: Vec<Annotation> = serde_json::from_str(&contents)?;
        Ok(annotations)
    }

    /// Read all annotations from the JSON file (with shared lock).
    pub fn read_all(&self) -> Result<Vec<Annotation>> {
        let _lock = self.lock_shared()?;
        self.parse_annotations_file()
    }

    /// Read all annotations with an exclusive lock already held.
    /// Used internally by write operations that need read-then-write atomicity.
    fn read_all_locked(&self) -> Result<Vec<Annotation>> {
        self.parse_annotations_file()
    }

    /// Atomically write all annotations to disk.
    /// Caller must hold the exclusive lock.
    fn write_all_locked(&self, annotations: &[Annotation]) -> Result<()> {
        let path = self.annotations_path();
        std::fs::create_dir_all(&self.data_dir)?;

        // Write to temp file via the same fd, then rename for atomicity
        let tmp_path = self.data_dir.join("annotations.json.tmp");
        let json = serde_json::to_string_pretty(annotations)?;

        let mut file = std::fs::File::create(&tmp_path)?;
        file.write_all(json.as_bytes())?;
        file.flush()?;
        file.sync_all()?;

        std::fs::rename(&tmp_path, &path)?;
        Ok(())
    }

    /// Create a new annotation with a generated ULID and timestamp.
    pub fn create_annotation(&self, input: CreateAnnotation) -> Result<Annotation> {
        let _lock = self.lock_exclusive()?;
        let mut annotations = self.read_all_locked()?;
        let now = chrono::Utc::now().to_rfc3339();
        let id = ulid::Ulid::new().to_string().to_lowercase();

        // Save screenshot if provided as data URL
        let screenshot_path = match &input.screenshot {
            Some(data_url) if data_url.starts_with("data:") => {
                Some(screenshot::save_screenshot(&self.screenshots_dir(), &id, data_url)?)
            }
            other => other.clone(),
        };

        let annotation = Annotation {
            id,
            url: input.url,
            x: input.x,
            y: input.y,
            comment: input.comment,
            element: input.element,
            element_path: input.element_path,
            css_classes: input.css_classes,
            nearby_text: input.nearby_text,
            selected_text: input.selected_text,
            bounding_box: input.bounding_box,
            screenshot: screenshot_path,
            intent: input.intent,
            severity: input.severity,
            status: AnnotationStatus::Pending,
            framework: input.framework,
            thread: Vec::new(),
            resolved_by: None,
            resolved_at: None,
            created_at: now.clone(),
            updated_at: now,
        };

        annotations.push(annotation.clone());
        self.write_all_locked(&annotations)?;

        Ok(annotation)
    }

    /// Get a single annotation by ID.
    pub fn get_annotation(&self, id: &str) -> Result<Option<Annotation>> {
        let annotations = self.read_all()?;
        Ok(annotations.into_iter().find(|a| a.id == id))
    }

    /// Update an annotation's mutable fields (status, comment, resolved_by, resolved_at, thread).
    pub fn update_annotation(&self, id: &str, input: UpdateAnnotation) -> Result<Annotation> {
        let _lock = self.lock_exclusive()?;
        let mut annotations = self.read_all_locked()?;
        let now = chrono::Utc::now().to_rfc3339();

        let annotation = annotations
            .iter_mut()
            .find(|a| a.id == id)
            .ok_or_else(|| Error::NotFound(id.to_string()))?;

        if let Some(comment) = input.comment {
            annotation.comment = comment;
        }
        if let Some(status) = input.status {
            annotation.status = status;

            // Clean up screenshot when resolving or dismissing
            if annotation.status.is_closed() {
                screenshot::delete_screenshot(&self.data_dir, annotation.screenshot.as_deref());
                annotation.screenshot = None;
            }
        }
        if let Some(resolved_by) = input.resolved_by {
            annotation.resolved_by = Some(resolved_by);
        }
        if let Some(resolved_at) = input.resolved_at {
            annotation.resolved_at = Some(resolved_at);
        }
        if let Some(thread) = input.thread {
            annotation.thread = thread;
        }

        annotation.updated_at = now;
        let updated = annotation.clone();

        self.write_all_locked(&annotations)?;
        Ok(updated)
    }

    /// Delete an annotation by ID. Also removes its screenshot if present.
    pub fn delete_annotation(&self, id: &str) -> Result<()> {
        let _lock = self.lock_exclusive()?;
        let mut annotations = self.read_all_locked()?;

        let idx = annotations
            .iter()
            .position(|a| a.id == id)
            .ok_or_else(|| Error::NotFound(id.to_string()))?;

        let annotation = &annotations[idx];
        screenshot::delete_screenshot(&self.data_dir, annotation.screenshot.as_deref());

        annotations.remove(idx);
        self.write_all_locked(&annotations)?;

        Ok(())
    }

    /// Get all annotations with status `Pending`.
    pub fn get_pending(&self) -> Result<Vec<Annotation>> {
        let annotations = self.read_all()?;
        Ok(annotations
            .into_iter()
            .filter(|a| a.status == AnnotationStatus::Pending)
            .collect())
    }

    /// Read a screenshot as base64 data with MIME type.
    pub fn read_screenshot(&self, annotation_id: &str) -> Result<screenshot::ScreenshotData> {
        let annotation = self
            .get_annotation(annotation_id)?
            .ok_or_else(|| Error::NotFound(annotation_id.to_string()))?;

        let screenshot_path = annotation
            .screenshot
            .as_deref()
            .ok_or_else(|| Error::NotFound(format!("No screenshot for {annotation_id}")))?;

        screenshot::read_screenshot(&self.data_dir, screenshot_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AnnotationStatus, CreateAnnotation, ThreadMessage, UpdateAnnotation};
    use base64::Engine;
    use tempfile::TempDir;

    fn make_store() -> (TempDir, Store) {
        let dir = TempDir::new().unwrap();
        let store = Store::new(dir.path().to_path_buf()).unwrap();
        (dir, store)
    }

    fn sample_create_data() -> CreateAnnotation {
        CreateAnnotation {
            url: "http://localhost:1420".into(),
            x: 100.0,
            y: 200.0,
            comment: "Fix this button".into(),
            element: "button.submit".into(),
            element_path: "html > body > main > form > button".into(),
            css_classes: Some("btn btn-primary".into()),
            nearby_text: Some("Submit form".into()),
            selected_text: None,
            bounding_box: None,
            screenshot: None,
            intent: "fix".into(),
            severity: "important".into(),
            framework: None,
        }
    }

    #[test]
    fn test_empty_store() {
        let (_dir, store) = make_store();
        let all = store.read_all().unwrap();
        assert!(all.is_empty());
    }

    #[test]
    fn test_create_annotation() {
        let (_dir, store) = make_store();
        let data = sample_create_data();

        let annotation = store.create_annotation(data).unwrap();
        assert!(!annotation.id.is_empty());
        assert_eq!(annotation.status, AnnotationStatus::Pending);
        assert_eq!(annotation.comment, "Fix this button");
        assert_eq!(annotation.intent, "fix");
        assert_eq!(annotation.severity, "important");
        assert!(annotation.thread.is_empty());
        assert!(annotation.resolved_by.is_none());
        assert!(annotation.resolved_at.is_none());
    }

    #[test]
    fn test_create_persists_to_disk() {
        let (_dir, store) = make_store();
        let data = sample_create_data();

        let created = store.create_annotation(data).unwrap();
        let all = store.read_all().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, created.id);
    }

    #[test]
    fn test_create_multiple_annotations() {
        let (_dir, store) = make_store();

        store.create_annotation(sample_create_data()).unwrap();
        store.create_annotation(sample_create_data()).unwrap();
        store.create_annotation(sample_create_data()).unwrap();

        let all = store.read_all().unwrap();
        assert_eq!(all.len(), 3);

        // Each should have a unique ULID
        let ids: std::collections::HashSet<_> = all.iter().map(|a| &a.id).collect();
        assert_eq!(ids.len(), 3);
    }

    #[test]
    fn test_get_annotation_by_id() {
        let (_dir, store) = make_store();
        let created = store.create_annotation(sample_create_data()).unwrap();

        let found = store.get_annotation(&created.id).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, created.id);
    }

    #[test]
    fn test_get_annotation_not_found() {
        let (_dir, store) = make_store();
        let found = store.get_annotation("nonexistent").unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn test_update_annotation_comment() {
        let (_dir, store) = make_store();
        let created = store.create_annotation(sample_create_data()).unwrap();

        let updated = store
            .update_annotation(
                &created.id,
                UpdateAnnotation {
                    comment: Some("Updated comment".into()),
                    status: None,
                    resolved_by: None,
                    resolved_at: None,
                    thread: None,
                },
            )
            .unwrap();

        assert_eq!(updated.comment, "Updated comment");
        assert_eq!(updated.status, AnnotationStatus::Pending); // unchanged
        assert_ne!(updated.updated_at, created.updated_at);
    }

    #[test]
    fn test_update_annotation_resolve() {
        let (_dir, store) = make_store();
        let created = store.create_annotation(sample_create_data()).unwrap();

        let updated = store
            .update_annotation(
                &created.id,
                UpdateAnnotation {
                    comment: None,
                    status: Some(AnnotationStatus::Resolved),
                    resolved_by: Some("agent".into()),
                    resolved_at: Some("2026-03-16T00:00:00Z".into()),
                    thread: None,
                },
            )
            .unwrap();

        assert_eq!(updated.status, AnnotationStatus::Resolved);
        assert_eq!(updated.resolved_by.as_deref(), Some("agent"));
        assert_eq!(updated.resolved_at.as_deref(), Some("2026-03-16T00:00:00Z"));
    }

    #[test]
    fn test_update_annotation_not_found() {
        let (_dir, store) = make_store();
        let result = store.update_annotation(
            "nonexistent",
            UpdateAnnotation {
                comment: Some("test".into()),
                status: None,
                resolved_by: None,
                resolved_at: None,
                thread: None,
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_get_pending() {
        let (_dir, store) = make_store();

        let a1 = store.create_annotation(sample_create_data()).unwrap();
        let _a2 = store.create_annotation(sample_create_data()).unwrap();

        // Resolve one
        store
            .update_annotation(
                &a1.id,
                UpdateAnnotation {
                    comment: None,
                    status: Some(AnnotationStatus::Resolved),
                    resolved_by: Some("human".into()),
                    resolved_at: None,
                    thread: None,
                },
            )
            .unwrap();

        let pending = store.get_pending().unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].status, AnnotationStatus::Pending);
    }

    #[test]
    fn test_update_thread() {
        let (_dir, store) = make_store();
        let created = store.create_annotation(sample_create_data()).unwrap();

        let thread = vec![
            ThreadMessage { role: "user".into(), text: "This needs fixing".into() },
            ThreadMessage { role: "agent".into(), text: "I'll take a look".into() },
        ];

        let updated = store
            .update_annotation(
                &created.id,
                UpdateAnnotation {
                    comment: None,
                    status: None,
                    resolved_by: None,
                    resolved_at: None,
                    thread: Some(thread.clone()),
                },
            )
            .unwrap();

        assert_eq!(updated.thread.len(), 2);
    }

    #[test]
    fn test_delete_annotation() {
        let (_dir, store) = make_store();
        let created = store.create_annotation(sample_create_data()).unwrap();

        store.delete_annotation(&created.id).unwrap();

        let all = store.read_all().unwrap();
        assert!(all.is_empty());
    }

    #[test]
    fn test_delete_annotation_with_screenshot() {
        let (dir, store) = make_store();
        let b64 = base64::engine::general_purpose::STANDARD.encode(b"fake png data");
        let data_url = format!("data:image/png;base64,{b64}");

        let mut data = sample_create_data();
        data.screenshot = Some(data_url);

        let annotation = store.create_annotation(data).unwrap();
        let screenshot_rel = annotation.screenshot.as_ref().unwrap();
        let screenshot_abs = dir.path().join(screenshot_rel);
        assert!(screenshot_abs.exists());

        store.delete_annotation(&annotation.id).unwrap();

        assert!(!screenshot_abs.exists());
        let all = store.read_all().unwrap();
        assert!(all.is_empty());
    }

    #[test]
    fn test_delete_annotation_not_found() {
        let (_dir, store) = make_store();
        let result = store.delete_annotation("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_preserves_other_annotations() {
        let (_dir, store) = make_store();
        let a1 = store.create_annotation(sample_create_data()).unwrap();
        let a2 = store.create_annotation(sample_create_data()).unwrap();
        let a3 = store.create_annotation(sample_create_data()).unwrap();

        store.delete_annotation(&a2.id).unwrap();

        let all = store.read_all().unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].id, a1.id);
        assert_eq!(all[1].id, a3.id);
    }

    #[test]
    fn test_annotation_json_roundtrip() {
        let (_dir, store) = make_store();
        let created = store.create_annotation(sample_create_data()).unwrap();

        // Serialize to JSON and back
        let json = serde_json::to_string(&created).unwrap();
        let deserialized: Annotation = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, created.id);
        assert_eq!(deserialized.comment, created.comment);
        assert_eq!(deserialized.status, created.status);
    }

    #[test]
    fn test_create_with_screenshot_data_url() {
        let (_dir, store) = make_store();
        let b64 = base64::engine::general_purpose::STANDARD.encode(b"fake png data");
        let data_url = format!("data:image/png;base64,{b64}");

        let mut data = sample_create_data();
        data.screenshot = Some(data_url);

        let annotation = store.create_annotation(data).unwrap();
        assert!(annotation.screenshot.is_some());
        let screenshot_path = annotation.screenshot.as_ref().unwrap();
        assert!(screenshot_path.starts_with("screenshots/"));
        assert!(screenshot_path.ends_with(".png"));
    }

    #[test]
    fn test_resolve_deletes_screenshot() {
        let (dir, store) = make_store();
        let b64 = base64::engine::general_purpose::STANDARD.encode(b"fake png data");
        let data_url = format!("data:image/png;base64,{b64}");

        let mut data = sample_create_data();
        data.screenshot = Some(data_url);

        let annotation = store.create_annotation(data).unwrap();
        let screenshot_rel = annotation.screenshot.as_ref().unwrap();
        let screenshot_abs = dir.path().join(screenshot_rel);
        assert!(screenshot_abs.exists());

        // Resolve — should delete screenshot
        store
            .update_annotation(
                &annotation.id,
                UpdateAnnotation {
                    comment: None,
                    status: Some(AnnotationStatus::Resolved),
                    resolved_by: Some("agent".into()),
                    resolved_at: None,
                    thread: None,
                },
            )
            .unwrap();

        assert!(!screenshot_abs.exists());

        // Annotation should have screenshot = None
        let resolved = store.get_annotation(&annotation.id).unwrap().unwrap();
        assert!(resolved.screenshot.is_none());
    }
}
