use std::sync::Arc;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::{tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler};
use serde_json::json;
use tokio::sync::Mutex;

use crate::store::Store;

use super::tools::{AnnotationIdParam, GetProjectStructureParams};

/// MCP server that exposes instruckt annotation tools over stdio.
#[derive(Clone)]
pub struct InstrucktMcpServer {
    store: Arc<Mutex<Store>>,
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
}

impl InstrucktMcpServer {
    pub fn new(store: Store) -> Self {
        Self {
            store: Arc::new(Mutex::new(store)),
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_handler]
impl ServerHandler for InstrucktMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions(concat!(
                "instruckt — UI annotation server for AI coding agents.\n\n",
                "When the user pastes UI feedback annotations, follow this workflow:\n",
                "1. Call `get_all_pending` to get all annotations with full metadata\n",
                "2. For each annotation, call `get_source_location` to get the exact source file and line number\n",
                "3. If an annotation has a screenshot, call `get_screenshot` to view it\n",
                "4. Use `get_component_stack` to understand the component hierarchy when needed\n",
                "5. After fixing each issue, call `resolve` to mark it done\n",
                "6. Use `dismiss` to mark annotations that don't need fixing\n",
                "7. Use `delete_annotation` to permanently remove annotations\n\n",
                "The source location data includes React/Vue/Svelte component names, file paths, and line numbers.",
            ))
    }
}

/// Map a store error to an MCP internal error.
fn store_err(e: crate::error::Error) -> McpError {
    McpError::internal_error(e.to_string(), None)
}

#[tool_router]
impl InstrucktMcpServer {
    /// Get all pending annotations. Returns count and annotation details
    /// with `has_screenshot` boolean instead of file paths.
    #[tool(
        description = "Get all pending UI annotations. Returns annotation details including element, comment, intent, severity, and whether a screenshot is available."
    )]
    async fn get_all_pending(&self) -> Result<CallToolResult, McpError> {
        let store = self.store.lock().await;
        let pending = store.get_pending().map_err(store_err)?;

        let annotations: Vec<serde_json::Value> = pending
            .iter()
            .map(|annotation| {
                let mut val = serde_json::to_value(annotation).unwrap_or_default();
                if let Some(obj) = val.as_object_mut() {
                    let has_screenshot = obj
                        .remove("screenshot")
                        .map(|v| !v.is_null())
                        .unwrap_or(false);
                    obj.insert("has_screenshot".to_string(), json!(has_screenshot));
                }
                val
            })
            .collect();

        let response = json!({
            "count": annotations.len(),
            "annotations": annotations,
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).map_err(|e| store_err(e.into()))?,
        )]))
    }

    /// Get the screenshot for an annotation as an MCP image response.
    #[tool(
        description = "Get the screenshot image for a specific annotation. Returns the base64-encoded PNG or SVG image."
    )]
    async fn get_screenshot(
        &self,
        Parameters(params): Parameters<AnnotationIdParam>,
    ) -> Result<CallToolResult, McpError> {
        let store = self.store.lock().await;
        let screenshot = store
            .read_screenshot(&params.annotation_id)
            .map_err(store_err)?;

        Ok(CallToolResult::success(vec![Content::image(
            screenshot.base64,
            screenshot.mime_type,
        )]))
    }

    /// Resolve an annotation -- sets status to "resolved", resolved_by to "agent",
    /// and deletes the screenshot file.
    #[tool(
        description = "Mark an annotation as resolved. Sets status to 'resolved', resolved_by to 'agent', and cleans up the screenshot file."
    )]
    async fn resolve(
        &self,
        Parameters(params): Parameters<AnnotationIdParam>,
    ) -> Result<CallToolResult, McpError> {
        let store = self.store.lock().await;
        let now = chrono::Utc::now().to_rfc3339();
        let update = crate::types::UpdateAnnotation {
            status: Some(crate::types::AnnotationStatus::Resolved),
            resolved_by: Some(crate::types::RESOLVED_BY_AGENT.to_string()),
            resolved_at: Some(now),
            ..Default::default()
        };

        let annotation = store
            .update_annotation(&params.annotation_id, update)
            .map_err(store_err)?;

        let response = json!({
            "id": annotation.id,
            "status": annotation.status,
            "resolved_by": annotation.resolved_by,
            "resolved_at": annotation.resolved_at,
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).map_err(|e| store_err(e.into()))?,
        )]))
    }

    /// Dismiss an annotation — sets status to "dismissed", resolved_by to "agent",
    /// and deletes the screenshot file.
    #[tool(
        description = "Dismiss an annotation that doesn't need fixing. Sets status to 'dismissed', resolved_by to 'agent', and cleans up the screenshot file."
    )]
    async fn dismiss(
        &self,
        Parameters(params): Parameters<AnnotationIdParam>,
    ) -> Result<CallToolResult, McpError> {
        let store = self.store.lock().await;
        let now = chrono::Utc::now().to_rfc3339();
        let update = crate::types::UpdateAnnotation {
            status: Some(crate::types::AnnotationStatus::Dismissed),
            resolved_by: Some(crate::types::RESOLVED_BY_AGENT.to_string()),
            resolved_at: Some(now),
            ..Default::default()
        };

        let annotation = store
            .update_annotation(&params.annotation_id, update)
            .map_err(store_err)?;

        let response = json!({
            "id": annotation.id,
            "status": annotation.status,
            "resolved_by": annotation.resolved_by,
            "resolved_at": annotation.resolved_at,
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).map_err(|e| store_err(e.into()))?,
        )]))
    }

    /// Delete an annotation permanently, including its screenshot.
    #[tool(
        description = "Permanently delete an annotation and its screenshot. Use this to remove annotations that are no longer needed."
    )]
    async fn delete_annotation(
        &self,
        Parameters(params): Parameters<AnnotationIdParam>,
    ) -> Result<CallToolResult, McpError> {
        let store = self.store.lock().await;
        store
            .delete_annotation(&params.annotation_id)
            .map_err(store_err)?;

        let response = json!({
            "id": params.annotation_id,
            "deleted": true,
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).map_err(|e| store_err(e.into()))?,
        )]))
    }

    /// Extract source file location from the annotation's framework context.
    #[tool(
        description = "Get the source file path and line number for an annotation. Extracts from the annotation's framework context (source_file, source_line)."
    )]
    async fn get_source_location(
        &self,
        Parameters(params): Parameters<AnnotationIdParam>,
    ) -> Result<CallToolResult, McpError> {
        let store = self.store.lock().await;
        let annotation = store
            .get_annotation(&params.annotation_id)
            .map_err(store_err)?
            .ok_or_else(|| {
                McpError::invalid_params(
                    format!("Annotation not found: {}", params.annotation_id),
                    None,
                )
            })?;

        let mut source_file = None;
        let mut source_line = None;

        if let Some(ref framework) = annotation.framework {
            source_file = framework
                .get("source_file")
                .and_then(|v| v.as_str())
                .map(String::from);
            source_line = framework.get("source_line").and_then(|v| v.as_u64());
        }

        let response = json!({
            "annotation_id": annotation.id,
            "element": annotation.element,
            "element_path": annotation.element_path,
            "source_file": source_file,
            "source_line": source_line,
            "url": annotation.url,
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).map_err(|e| store_err(e.into()))?,
        )]))
    }

    /// Extract the component hierarchy from the annotation's framework context.
    #[tool(
        description = "Get the component hierarchy for an annotation. Returns the component stack from the framework context and element_path."
    )]
    async fn get_component_stack(
        &self,
        Parameters(params): Parameters<AnnotationIdParam>,
    ) -> Result<CallToolResult, McpError> {
        let store = self.store.lock().await;
        let annotation = store
            .get_annotation(&params.annotation_id)
            .map_err(store_err)?
            .ok_or_else(|| {
                McpError::invalid_params(
                    format!("Annotation not found: {}", params.annotation_id),
                    None,
                )
            })?;

        let component_stack = annotation
            .framework
            .as_ref()
            .and_then(|framework| framework.get("component_stack"));

        let response = json!({
            "annotation_id": annotation.id,
            "element_path": annotation.element_path,
            "component_stack": component_stack,
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).map_err(|e| store_err(e.into()))?,
        )]))
    }

    /// Scan the project directory and return a filtered tree of frontend source files.
    #[tool(
        description = "Get the project's frontend source structure. Returns a filtered directory tree excluding node_modules, dist, .git, etc. Defaults to current working directory."
    )]
    async fn get_project_structure(
        &self,
        Parameters(params): Parameters<GetProjectStructureParams>,
    ) -> Result<CallToolResult, McpError> {
        // No store access needed — this tool only does filesystem traversal.
        let root = match params.root_dir {
            Some(dir) => dir,
            None => std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .map_err(|e| {
                    McpError::internal_error(
                        format!("Cannot determine current directory: {e}"),
                        None,
                    )
                })?,
        };

        let root_path = std::path::Path::new(&root);
        if !root_path.exists() {
            return Err(McpError::invalid_params(
                format!("Directory not found: {root}"),
                None,
            ));
        }

        let files = crate::project::get_project_structure(root_path);

        let response = json!({
            "root": root,
            "file_count": files.len(),
            "files": files,
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).map_err(|e| store_err(e.into()))?,
        )]))
    }
}
