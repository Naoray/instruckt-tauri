use std::sync::Arc;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::{tool, tool_router, ErrorData as McpError, ServerHandler};
use serde_json::json;
use tokio::sync::Mutex;

use crate::store::Store;

use super::tools::*;

/// MCP server that exposes instruckt annotation tools over stdio.
#[derive(Clone)]
#[allow(dead_code)]
pub struct InstrucktMcpServer {
    store: Arc<Mutex<Store>>,
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

impl ServerHandler for InstrucktMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions("instruckt annotation server — manages UI feedback annotations for AI coding agents")
    }
}

#[tool_router]
impl InstrucktMcpServer {
    /// Get all pending annotations. Returns count and annotation details
    /// with `has_screenshot` boolean instead of file paths.
    #[tool(description = "Get all pending UI annotations. Returns annotation details including element, comment, intent, severity, and whether a screenshot is available.")]
    async fn get_all_pending(&self) -> Result<CallToolResult, McpError> {
        let store = self.store.lock().await;
        let pending = store.get_pending().map_err(|e| {
            McpError::internal_error(e.to_string(), None)
        })?;

        let annotations: Vec<serde_json::Value> = pending
            .iter()
            .map(|a| {
                json!({
                    "id": a.id,
                    "url": a.url,
                    "x": a.x,
                    "y": a.y,
                    "comment": a.comment,
                    "element": a.element,
                    "element_path": a.element_path,
                    "css_classes": a.css_classes,
                    "nearby_text": a.nearby_text,
                    "selected_text": a.selected_text,
                    "bounding_box": a.bounding_box,
                    "has_screenshot": a.screenshot.is_some(),
                    "intent": a.intent,
                    "severity": a.severity,
                    "status": a.status,
                    "framework": a.framework,
                    "thread": a.thread,
                    "created_at": a.created_at,
                    "updated_at": a.updated_at,
                })
            })
            .collect();

        let result = json!({
            "count": annotations.len(),
            "annotations": annotations,
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]))
    }

    /// Get the screenshot for an annotation as an MCP image response.
    #[tool(description = "Get the screenshot image for a specific annotation. Returns the base64-encoded PNG or SVG image.")]
    async fn get_screenshot(
        &self,
        Parameters(params): Parameters<GetScreenshotParams>,
    ) -> Result<CallToolResult, McpError> {
        let store = self.store.lock().await;
        let screenshot = store
            .read_screenshot(&params.annotation_id)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::image(
            screenshot.base64,
            screenshot.mime_type,
        )]))
    }

    /// Resolve an annotation — sets status to "resolved", resolved_by to "agent",
    /// and deletes the screenshot file.
    #[tool(description = "Mark an annotation as resolved. Sets status to 'resolved', resolved_by to 'agent', and cleans up the screenshot file.")]
    async fn resolve(
        &self,
        Parameters(params): Parameters<ResolveParams>,
    ) -> Result<CallToolResult, McpError> {
        let store = self.store.lock().await;
        let now = chrono::Utc::now().to_rfc3339();
        let data = crate::types::UpdateAnnotation {
            comment: None,
            status: Some("resolved".to_string()),
            resolved_by: Some("agent".to_string()),
            resolved_at: Some(now),
            thread: None,
        };

        let annotation = store
            .update_annotation(&params.annotation_id, data)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let result = json!({
            "id": annotation.id,
            "status": annotation.status,
            "resolved_by": annotation.resolved_by,
            "resolved_at": annotation.resolved_at,
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]))
    }

    /// Extract source file location from the annotation's framework context.
    #[tool(description = "Get the source file path and line number for an annotation. Extracts from the annotation's framework context (source_file, source_line).")]
    async fn get_source_location(
        &self,
        Parameters(params): Parameters<GetSourceLocationParams>,
    ) -> Result<CallToolResult, McpError> {
        let store = self.store.lock().await;
        let annotation = store
            .get_annotation(&params.annotation_id)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?
            .ok_or_else(|| {
                McpError::invalid_params(
                    format!("Annotation not found: {}", params.annotation_id),
                    None,
                )
            })?;

        let mut source_file = None;
        let mut source_line = None;

        if let Some(ref fw) = annotation.framework {
            source_file = fw.get("source_file").and_then(|v| v.as_str()).map(String::from);
            source_line = fw.get("source_line").and_then(|v| v.as_u64());
        }

        let result = json!({
            "annotation_id": annotation.id,
            "element": annotation.element,
            "element_path": annotation.element_path,
            "source_file": source_file,
            "source_line": source_line,
            "url": annotation.url,
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]))
    }

    /// Extract the component hierarchy from the annotation's framework context.
    #[tool(description = "Get the component hierarchy for an annotation. Returns the component stack from the framework context and element_path.")]
    async fn get_component_stack(
        &self,
        Parameters(params): Parameters<GetComponentStackParams>,
    ) -> Result<CallToolResult, McpError> {
        let store = self.store.lock().await;
        let annotation = store
            .get_annotation(&params.annotation_id)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?
            .ok_or_else(|| {
                McpError::invalid_params(
                    format!("Annotation not found: {}", params.annotation_id),
                    None,
                )
            })?;

        let component_stack = annotation
            .framework
            .as_ref()
            .and_then(|fw| fw.get("component_stack"));

        let result = json!({
            "annotation_id": annotation.id,
            "element_path": annotation.element_path,
            "component_stack": component_stack,
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]))
    }

    /// Scan the project directory and return a filtered tree of frontend source files.
    #[tool(description = "Get the project's frontend source structure. Returns a filtered directory tree excluding node_modules, dist, .git, etc. Defaults to current working directory.")]
    async fn get_project_structure(
        &self,
        Parameters(params): Parameters<GetProjectStructureParams>,
    ) -> Result<CallToolResult, McpError> {
        let root = match params.root_dir {
            Some(dir) => dir,
            None => std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .map_err(|e| McpError::internal_error(
                    format!("Cannot determine current directory: {e}"),
                    None,
                ))?,
        };

        let root_path = std::path::Path::new(&root);
        if !root_path.exists() {
            return Err(McpError::invalid_params(
                format!("Directory not found: {root}"),
                None,
            ));
        }

        let excluded_dirs = [
            "node_modules",
            "dist",
            "build",
            ".git",
            ".next",
            ".nuxt",
            ".svelte-kit",
            "target",
            ".turbo",
            "coverage",
            "__pycache__",
        ];

        let mut files: Vec<String> = Vec::new();

        for entry in walkdir::WalkDir::new(&root)
            .max_depth(6)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !excluded_dirs.contains(&name.as_ref())
            })
        {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();

            // Only include files, not directories
            if !entry.file_type().is_file() {
                continue;
            }

            let relative = path
                .strip_prefix(&root)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            if relative.is_empty() {
                continue;
            }

            // Only include frontend-relevant file types
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            let is_source = matches!(
                ext,
                "ts" | "tsx" | "js" | "jsx" | "vue" | "svelte" | "html" | "css"
                    | "scss" | "json" | "md"
            );
            if !is_source {
                continue;
            }

            files.push(relative);
        }

        files.sort();

        let result = json!({
            "root": root,
            "file_count": files.len(),
            "files": files,
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]))
    }
}
