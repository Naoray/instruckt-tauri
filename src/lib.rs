mod commands;
pub mod error;
pub mod project;
pub mod screenshot;
pub mod state;
pub mod store;
pub mod types;

pub mod mcp;

use tauri::plugin::{Builder, TauriPlugin};
use tauri::{Manager, Runtime};

use state::InstrucktState;
use store::Store;

/// The vendored instruckt JS library (IIFE bundle).
/// Exposes the `Instruckt` global with `Instruckt.init()`.
#[cfg(debug_assertions)]
const INSTRUCKT_IIFE_JS: &str = include_str!("../js/instruckt.iife.js");

/// The Tauri IPC shim + auto-initialization script.
/// Patches fetch() and calls `Instruckt.init()` on DOMContentLoaded.
#[cfg(debug_assertions)]
const INSTRUCKT_INIT_JS: &str = include_str!("../js/instruckt-init.js");

/// Initialize the instruckt plugin.
///
/// In debug builds, this injects the instruckt annotation UI and IPC shim
/// into every webview automatically. In release builds, nothing is injected.
///
/// # Usage
///
/// ```rust,ignore
/// fn main() {
///     tauri::Builder::default()
///         .plugin(tauri_plugin_instruckt::init())
///         .run(tauri::generate_context!())
///         .expect("error while running tauri application");
/// }
/// ```
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    let mut builder = Builder::new("instruckt")
        .invoke_handler(tauri::generate_handler![
            commands::get_annotations,
            commands::create_annotation,
            commands::update_annotation,
        ])
        .setup(|app, _api| {
            let data_dir = Store::default_data_dir().map_err(|e| e.to_string())?;
            let store = Store::new(data_dir).map_err(|e| e.to_string())?;
            app.manage(InstrucktState::new(store));

            #[cfg(debug_assertions)]
            log::info!("instruckt plugin initialized (dev mode)");

            Ok(())
        });

    // In debug builds, inject the instruckt UI library + IPC shim as a
    // single script. Multiple js_init_script calls have no guaranteed
    // execution order on WebKit, so we concatenate them to ensure the
    // IIFE (defining the Instruckt global) runs before the init shim.
    #[cfg(debug_assertions)]
    {
        let combined = format!("{}\n{}", INSTRUCKT_IIFE_JS, INSTRUCKT_INIT_JS);
        builder = builder.js_init_script(combined);
    }

    builder.build()
}
