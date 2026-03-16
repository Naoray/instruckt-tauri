# tauri-plugin-instruckt

Visual annotation plugin for Tauri v2. Lets users annotate UI elements directly in the app during development. AI coding agents consume annotations via the included MCP server.

**Dev-only** — zero overhead in release builds (`cfg!(debug_assertions)`).

## Setup

### 1. Add the plugin

```toml
# src-tauri/Cargo.toml
[dependencies]
tauri-plugin-instruckt = { git = "ssh://git@github.com/Naoray/instruckt-tauri.git", branch = "main" }
```

### 2. Register in your app

```rust
// src-tauri/src/main.rs (or lib.rs)
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_instruckt::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 3. Add permissions

```json
// src-tauri/capabilities/default.json
{
  "identifier": "default",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "instruckt:default"
  ]
}
```

### 4. Install the MCP server

```sh
cargo install --git ssh://git@github.com/Naoray/instruckt-tauri.git --bin instruckt-mcp
```

This installs `instruckt-mcp` to `~/.cargo/bin/`.

### 5. Configure Claude Code

Add to your project's `.mcp.json` (at the project root):

```json
{
  "mcpServers": {
    "instruckt": {
      "command": "/path/to/.cargo/bin/instruckt-mcp"
    }
  }
}
```

Replace `/path/to/.cargo/bin/instruckt-mcp` with the output of `which instruckt-mcp`.

## Usage

Run `pnpm tauri dev` (or `cargo tauri dev`). The annotation toolbar appears in the bottom-right corner.

### Keyboard shortcuts

| Key | Action |
|-----|--------|
| `A` | Toggle annotation mode (crosshair cursor, click to annotate) |
| `F` | Freeze page (pause animations/transitions) |
| `C` | Screenshot region selection |
| `X` | Clear annotations on current page |
| `Esc` | Cancel current action |

### Workflow

1. Press `A` to enter annotation mode
2. Click any element to annotate it
3. Add a comment describing the issue
4. Annotations auto-copy as markdown to clipboard
5. Paste into your AI agent's context
6. Agent uses MCP tools to view screenshots and resolve annotations

### MCP tools

| Tool | Description |
|------|-------------|
| `get_all_pending` | Get all unresolved annotations with metadata |
| `get_screenshot` | Get the screenshot image for an annotation |
| `resolve` | Mark an annotation as resolved |
| `get_source_location` | Get source file path and line number from framework context |
| `get_component_stack` | Get the React/Vue/Svelte component hierarchy |
| `get_project_structure` | Get filtered directory tree of frontend source files |

## How it works

The plugin injects two things into every webview (debug builds only):

1. **instruckt IIFE** — the annotation UI (toolbar, highlight overlay, popup forms, markers)
2. **IPC shim** — patches `fetch()` to route instruckt API calls through Tauri IPC instead of HTTP

The JS library thinks it's talking to an HTTP API (`/instruckt/annotations`), but the shim intercepts those calls and routes them to Rust via `__TAURI_INTERNALS__.invoke`. This means the same instruckt JS works in both web (Laravel) and desktop (Tauri) without changes.

Annotations are stored as JSON files in the OS app data directory. The MCP server reads from the same directory, giving AI agents access to the annotations.

## Framework adapters

The plugin auto-detects React components by default. Source file/line info requires your dev server to serve source maps (Vite does this automatically).

Supported frameworks: `react`, `vue`, `svelte`.

## Credits

Inspired by [instruckt-laravel](https://github.com/joshcirre/instruckt-laravel) by Josh Cirre. This plugin brings the same annotation workflow to Tauri desktop apps, sharing the same JS frontend library and MCP tool interface.
