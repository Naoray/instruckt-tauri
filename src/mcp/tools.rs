use rmcp::schemars;
use schemars::JsonSchema;
use serde::Deserialize;

/// Shared parameters for tools that operate on a single annotation by ID.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AnnotationIdParam {
    /// The annotation ID to operate on.
    pub annotation_id: String,
}

/// Parameters for `get_project_structure` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetProjectStructureParams {
    /// Root directory to scan. Defaults to current working directory.
    pub root_dir: Option<String>,
}
