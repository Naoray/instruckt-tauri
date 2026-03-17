# tauri-plugin-instruckt

Tauri v2 plugin for [instruckt](https://github.com/joshcirre/instruckt) — visual feedback for AI coding agents. Provides the Rust backend, IPC commands, JSON file storage, and a standalone MCP server.

Users annotate elements in the webview, annotations are copied as structured markdown, and your AI agent can also read them via MCP.

**Dev-only** — the annotation UI and IPC shim are only injected in debug builds (`cfg!(debug_assertions)`). Zero bytes in production.

## Requirements

- Rust 1.70+
- Tauri v2
- A Vite-based frontend (React, Vue, Svelte)

## Install

```toml
# src-tauri/Cargo.toml
[dependencies]
tauri-plugin-instruckt = { git = "https://github.com/Naoray/instruckt-tauri.git", tag = "v0.1.0" }
```

## Setup

### 1. Register the plugin

```rust
// src-tauri/src/main.rs (or lib.rs)
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_instruckt::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 2. Add permissions

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

### 3. Add the Vite plugin

The Vite plugin provides the client-side UI. Set `server: false` since Tauri owns the backend:

```js
// vite.config.ts
import instruckt from 'instruckt/vite'

export default defineConfig({
  plugins: [
    instruckt({
      server: false,
      mcp: true,
    }),
  ],
})
```

That's it. Run `cargo tauri dev` and the annotation toolbar appears in the bottom-right corner.

## MCP Server

The plugin includes a standalone MCP server binary that AI agents connect to via stdio.

### Install

```sh
cargo install --git https://github.com/Naoray/instruckt-tauri.git --tag v0.1.0 --bin instruckt-mcp
```

### Configure your agent

Add to `.mcp.json` in your project root:

```json
{
  "mcpServers": {
    "instruckt": {
      "command": "instruckt-mcp"
    }
  }
}
```

> If `instruckt-mcp` isn't on your PATH, use the full path from `which instruckt-mcp`.

### Tools

| Tool | Description |
|------|-------------|
| `get_all_pending` | List all unresolved annotations with metadata |
| `get_screenshot` | Get the screenshot image for an annotation |
| `resolve` | Mark an annotation as resolved (cleans up screenshot) |
| `dismiss` | Dismiss an annotation that doesn't need fixing (cleans up screenshot) |
| `delete_annotation` | Permanently delete an annotation and its screenshot |
| `get_source_location` | Get source file path and line number |
| `get_component_stack` | Get the React/Vue/Svelte component hierarchy |
| `get_project_structure` | Get a filtered directory tree of frontend source files |

## How it works

The plugin injects two scripts into every webview (debug builds only):

1. **instruckt IIFE** — the annotation UI (toolbar, highlight overlay, popup forms, markers)
2. **IPC shim** — patches `fetch()` to route instruckt API calls through Tauri IPC instead of HTTP

The JS library thinks it's talking to an HTTP API (`/instruckt/annotations`), but the shim intercepts those calls and routes them to Rust via `__TAURI_INTERNALS__.invoke`. The shim handles `GET`, `POST`, `PATCH`, and `DELETE` routes. The same instruckt JS works in both web and desktop without changes.

Annotations and screenshots are stored as JSON files in the OS app data directory (`~/.local/share` on Linux, `~/Library/Application Support` on macOS, `AppData` on Windows). The MCP server reads from the same directory, so AI agents see the same annotations.

## Workflow

1. Press **A** to enter annotation mode
2. Click any element — instruckt detects its framework component and source location
3. Type your feedback and save
4. The annotation auto-copies as structured markdown to your clipboard
5. Paste into your AI agent (Claude Code, Cursor, Copilot, etc.)
6. The agent uses MCP tools to view screenshots, read source context, and resolve/dismiss/delete annotations

## Keyboard shortcuts

| Key | Action |
|-----|--------|
| `A` | Toggle annotation mode |
| `F` | Freeze page (pause animations, block navigation) |
| `C` | Screenshot region capture |
| `X` | Clear annotations on current page |
| `Esc` | Cancel current action |

## Supported frameworks

Source file/line detection works with any framework that Vite serves source maps for (which is the default).

Component detection: **React**, **Vue**, **Svelte**.

## Credits

Built on [instruckt](https://github.com/joshcirre/instruckt) by [Josh Cirre](https://github.com/joshcirre). This plugin brings the same annotation workflow to Tauri desktop apps, sharing the JS frontend and MCP tool interface with [instruckt-laravel](https://github.com/joshcirre/instruckt-laravel).

## License

MIT
