use serde::{Deserialize, Serialize};
use std::fmt;

/// The lifecycle status of an annotation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnnotationStatus {
    Pending,
    Resolved,
    Dismissed,
}

impl AnnotationStatus {
    /// Whether the annotation has been closed (resolved or dismissed).
    pub fn is_closed(&self) -> bool {
        matches!(self, Self::Resolved | Self::Dismissed)
    }
}

impl fmt::Display for AnnotationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Resolved => write!(f, "resolved"),
            Self::Dismissed => write!(f, "dismissed"),
        }
    }
}

/// A single message in the annotation's conversation thread.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadMessage {
    pub role: String,
    pub text: String,
}

/// Default value for `resolved_by` when an AI agent resolves an annotation.
pub const RESOLVED_BY_AGENT: &str = "agent";

/// Represents the bounding rectangle of an annotated UI element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// A UI annotation capturing user feedback on a specific element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub id: String,
    pub url: String,
    pub x: f64,
    pub y: f64,
    pub comment: String,
    pub element: String,
    pub element_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub css_classes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nearby_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bounding_box: Option<BoundingBox>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screenshot: Option<String>,
    pub intent: String,
    pub severity: String,
    pub status: AnnotationStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub framework: Option<serde_json::Value>,
    #[serde(default)]
    pub thread: Vec<ThreadMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Data required to create a new annotation.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateAnnotation {
    pub url: String,
    pub x: f64,
    pub y: f64,
    pub comment: String,
    pub element: String,
    pub element_path: String,
    #[serde(default)]
    pub css_classes: Option<String>,
    #[serde(default)]
    pub nearby_text: Option<String>,
    #[serde(default)]
    pub selected_text: Option<String>,
    #[serde(default)]
    pub bounding_box: Option<BoundingBox>,
    #[serde(default)]
    pub screenshot: Option<String>,
    pub intent: String,
    pub severity: String,
    #[serde(default)]
    pub framework: Option<serde_json::Value>,
}

/// Optional fields for updating an existing annotation.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateAnnotation {
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    pub status: Option<AnnotationStatus>,
    #[serde(default)]
    pub resolved_by: Option<String>,
    #[serde(default)]
    pub resolved_at: Option<String>,
    #[serde(default)]
    pub thread: Option<Vec<ThreadMessage>>,
}
