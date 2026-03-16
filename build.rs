const COMMANDS: &[&str] = &["get_annotations", "create_annotation", "update_annotation"];

fn main() {
    // Re-embed JS when these files change
    println!("cargo:rerun-if-changed=js/instruckt.iife.js");
    println!("cargo:rerun-if-changed=js/instruckt-init.js");

    tauri_plugin::Builder::new(COMMANDS).build();
}
