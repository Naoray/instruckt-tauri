mod commands;
pub mod error;
pub mod screenshot;
pub mod state;
pub mod store;
pub mod types;

pub mod mcp;

use tauri::plugin::{Builder, TauriPlugin};
use tauri::{Manager, Runtime};

use state::InstrucktState;
use store::Store;

/// The JS shim that intercepts instruckt's fetch calls and routes them
/// through Tauri IPC. Only injected in debug builds.
#[cfg(debug_assertions)]
const INSTRUCKT_INIT_JS: &str = include_str!("../js/instruckt-init.js");

/// Initialize the instruckt plugin.
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
            let data_dir = Store::default_data_dir()
                .map_err(|e| e.to_string())?;
            let store = Store::new(data_dir)
                .map_err(|e| e.to_string())?;
            app.manage(InstrucktState::new(store));

            #[cfg(debug_assertions)]
            log::info!("instruckt plugin initialized (dev mode)");

            Ok(())
        });

    // Only inject the JS shim in debug builds
    #[cfg(debug_assertions)]
    {
        builder = builder.js_init_script(INSTRUCKT_INIT_JS.to_string());
    }

    builder.build()
}
