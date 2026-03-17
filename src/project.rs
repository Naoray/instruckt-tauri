use std::path::Path;

/// Scan a project directory and return a sorted list of frontend source files,
/// excluding common build/dependency directories.
///
/// Files are returned as relative paths from `root`. Only files with frontend-relevant
/// extensions (ts, tsx, js, jsx, vue, svelte, html, css, scss, json, md) are included.
/// The walk is limited to 6 levels of depth.
pub fn get_project_structure(root: &Path) -> Vec<String> {
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

    for entry in walkdir::WalkDir::new(root)
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
            .strip_prefix(root)
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
    files
}
