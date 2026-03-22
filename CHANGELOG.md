# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-03-22

### Added

- `dismiss` MCP tool — dismiss annotations that don't need fixing (cleans up screenshot)
- `delete_annotation` MCP tool — permanently delete an annotation and its screenshot
- `DELETE /instruckt/annotations/{id}` route in IPC shim
- `allow-delete-annotation` included in default permissions
- Workflow instructions embedded in MCP server tool descriptions
- Project structure utility for `get_project_structure` MCP tool

### Changed

- Unified on `tokio::sync::Mutex` throughout (removed mixed std/tokio mutex usage)
- Replaced magic status strings with `AnnotationStatus` enum and typed `ThreadMessage`
- Encapsulated store state behind proper API boundaries
- Improved naming quality across the entire codebase
- Adopted idiomatic Rust patterns (error propagation, iterators, builder patterns)
- MCP server uses `std::sync::Mutex` + `spawn_blocking` to avoid blocking the tokio runtime

### Fixed

- Error propagation — errors are now returned instead of silently swallowed
- MCP `ServerHandler` now includes `tool_handler` so tools are properly listed
- MCP config location corrected to `.mcp.json`
- IPC shim no longer includes adapter detection (belongs in instruckt JS lib)
- Improved durability of JSON file storage with atomic writes

### Removed

- Dead code, duplication, and manual JSON construction cleaned up

## [0.1.0] - 2026-03-16

### Added

- Initial release of `tauri-plugin-instruckt`
- Tauri v2 plugin with IPC commands for annotation CRUD
- Vendored instruckt JS bundle for zero-config auto-initialization
- IPC shim that patches `fetch()` to route instruckt API calls through Tauri IPC
- JSON file storage in OS app data directory
- MCP server binary (`instruckt-mcp`) with stdio transport
- MCP tools: `get_all_pending`, `get_screenshot`, `resolve`, `get_source_location`, `get_component_stack`, `get_project_structure`
- Tauri v2 permission definitions
- Dev-only injection via `cfg!(debug_assertions)`

[0.2.0]: https://github.com/Naoray/instruckt-tauri/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/Naoray/instruckt-tauri/releases/tag/v0.1.0
