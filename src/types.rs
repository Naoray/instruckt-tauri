use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

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
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub framework: Option<serde_json::Value>,
    #[serde(default)]
    pub thread: Vec<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Data sent from the JS frontend when creating an annotation.
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

/// Subset of fields that can be updated after creation.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateAnnotation {
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub resolved_by: Option<String>,
    #[serde(default)]
    pub resolved_at: Option<String>,
    #[serde(default)]
    pub thread: Option<Vec<serde_json::Value>>,
}
