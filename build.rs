const COMMANDS: &[&str] = &["get_annotations", "create_annotation", "update_annotation"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
