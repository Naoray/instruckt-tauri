use rmcp::schemars;
use schemars::JsonSchema;
use serde::Deserialize;

/// Parameters for `get_screenshot` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetScreenshotParams {
    /// The annotation ID to get the screenshot for.
    pub annotation_id: String,
}

/// Parameters for `resolve` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ResolveParams {
    /// The annotation ID to mark as resolved.
    pub annotation_id: String,
}

/// Parameters for `get_source_location` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetSourceLocationParams {
    /// The annotation ID to get source location for.
    pub annotation_id: String,
}

/// Parameters for `get_component_stack` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetComponentStackParams {
    /// The annotation ID to get the component stack for.
    pub annotation_id: String,
}

/// Parameters for `get_project_structure` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetProjectStructureParams {
    /// Root directory to scan. Defaults to current working directory.
    pub root_dir: Option<String>,
}
