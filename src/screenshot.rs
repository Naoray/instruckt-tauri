use base64::Engine;
use std::path::Path;

use crate::error::{Error, Result};

/// Parse a data URL (e.g. `data:image/png;base64,...`) and save it to disk.
/// Returns the relative path from the screenshots directory.
pub fn save_screenshot(screenshots_dir: &Path, id: &str, data_url: &str) -> Result<String> {
    let (mime, data) = parse_data_url(data_url)?;
    let ext = mime_to_extension(&mime);
    let filename = format!("{id}.{ext}");
    let path = screenshots_dir.join(&filename);

    std::fs::create_dir_all(screenshots_dir)?;

    let bytes = base64::engine::general_purpose::STANDARD.decode(data)?;
    std::fs::write(&path, bytes)?;

    Ok(format!("screenshots/{filename}"))
}

/// Delete a screenshot file given its relative path (e.g. "screenshots/abc.png").
pub fn delete_screenshot(data_dir: &Path, relative_path: Option<&str>) {
    if let Some(rel) = relative_path {
        let path = data_dir.join(rel);
        if let Err(e) = std::fs::remove_file(&path) {
            log::warn!("Failed to delete screenshot {}: {e}", path.display());
        }
    }
}

/// Screenshot data with separate base64 payload and MIME type.
#[derive(Debug)]
pub struct ScreenshotData {
    pub base64: String,
    pub mime_type: String,
}

/// Read a screenshot file and return the base64 data + MIME type separately.
pub fn read_screenshot(data_dir: &Path, relative_path: &str) -> Result<ScreenshotData> {
    let path = data_dir.join(relative_path);
    if !path.exists() {
        return Err(Error::NotFound(format!(
            "Screenshot not found: {relative_path}"
        )));
    }
    let bytes = std::fs::read(&path)?;
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png");
    let mime_type = extension_to_mime(ext).to_string();
    let base64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(ScreenshotData { base64, mime_type })
}

fn parse_data_url(data_url: &str) -> Result<(String, &str)> {
    // Format: data:<mime>;base64,<data>
    let rest = data_url
        .strip_prefix("data:")
        .ok_or_else(|| Error::Other("Invalid data URL: missing 'data:' prefix".into()))?;

    let (header, data) = rest
        .split_once(',')
        .ok_or_else(|| Error::Other("Invalid data URL: missing comma separator".into()))?;

    let mime = header
        .strip_suffix(";base64")
        .ok_or_else(|| Error::Other("Invalid data URL: missing ';base64' marker".into()))?;

    Ok((mime.to_string(), data))
}

fn mime_to_extension(mime: &str) -> &str {
    match mime {
        "image/png" => "png",
        "image/svg+xml" => "svg",
        "image/jpeg" | "image/jpg" => "jpg",
        "image/webp" => "webp",
        _ => "png",
    }
}

fn extension_to_mime(ext: &str) -> &str {
    match ext {
        "png" => "image/png",
        "svg" => "image/svg+xml",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        _ => "image/png",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;
    use tempfile::TempDir;

    fn make_png_data_url() -> String {
        // Minimal 1x1 red PNG
        let png_bytes: &[u8] = &[
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
            0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1
            0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, 0xDE,
            0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, // IDAT chunk
            0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01, 0xE2, 0x21,
            0xBC, 0x33,
            0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, // IEND chunk
            0xAE, 0x42, 0x60, 0x82,
        ];
        let b64 = base64::engine::general_purpose::STANDARD.encode(png_bytes);
        format!("data:image/png;base64,{b64}")
    }

    #[test]
    fn test_parse_data_url_png() {
        let data_url = "data:image/png;base64,iVBORw0KGgo=";
        let (mime, data) = parse_data_url(data_url).unwrap();
        assert_eq!(mime, "image/png");
        assert_eq!(data, "iVBORw0KGgo=");
    }

    #[test]
    fn test_parse_data_url_svg() {
        let data_url = "data:image/svg+xml;base64,PHN2Zz48L3N2Zz4=";
        let (mime, data) = parse_data_url(data_url).unwrap();
        assert_eq!(mime, "image/svg+xml");
        assert_eq!(data, "PHN2Zz48L3N2Zz4=");
    }

    #[test]
    fn test_parse_data_url_invalid_prefix() {
        let result = parse_data_url("http://example.com/image.png");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_data_url_missing_comma() {
        let result = parse_data_url("data:image/png;base64");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_data_url_missing_base64_marker() {
        let result = parse_data_url("data:image/png,iVBORw0KGgo=");
        assert!(result.is_err());
    }

    #[test]
    fn test_mime_to_extension() {
        assert_eq!(mime_to_extension("image/png"), "png");
        assert_eq!(mime_to_extension("image/svg+xml"), "svg");
        assert_eq!(mime_to_extension("image/jpeg"), "jpg");
        assert_eq!(mime_to_extension("image/webp"), "webp");
        assert_eq!(mime_to_extension("image/unknown"), "png"); // fallback
    }

    #[test]
    fn test_extension_to_mime() {
        assert_eq!(extension_to_mime("png"), "image/png");
        assert_eq!(extension_to_mime("svg"), "image/svg+xml");
        assert_eq!(extension_to_mime("jpg"), "image/jpeg");
        assert_eq!(extension_to_mime("jpeg"), "image/jpeg");
        assert_eq!(extension_to_mime("webp"), "image/webp");
        assert_eq!(extension_to_mime("bmp"), "image/png"); // fallback
    }

    #[test]
    fn test_save_and_read_screenshot() {
        let dir = TempDir::new().unwrap();
        let screenshots_dir = dir.path().join("screenshots");
        let data_url = make_png_data_url();

        let relative = save_screenshot(&screenshots_dir, "test123", &data_url).unwrap();
        assert_eq!(relative, "screenshots/test123.png");

        // Verify file exists
        let file_path = screenshots_dir.join("test123.png");
        assert!(file_path.exists());

        // Read it back
        let result = read_screenshot(dir.path(), &relative).unwrap();
        assert_eq!(result.mime_type, "image/png");
        assert!(!result.base64.is_empty());
    }

    #[test]
    fn test_delete_screenshot() {
        let dir = TempDir::new().unwrap();
        let screenshots_dir = dir.path().join("screenshots");
        let data_url = make_png_data_url();

        let relative = save_screenshot(&screenshots_dir, "del1", &data_url).unwrap();
        let file_path = screenshots_dir.join("del1.png");
        assert!(file_path.exists());

        delete_screenshot(dir.path(), Some(&relative));
        assert!(!file_path.exists());
    }

    #[test]
    fn test_delete_screenshot_none() {
        let dir = TempDir::new().unwrap();
        // Should not panic
        delete_screenshot(dir.path(), None);
    }

    #[test]
    fn test_read_nonexistent_screenshot() {
        let dir = TempDir::new().unwrap();
        let result = read_screenshot(dir.path(), "screenshots/nonexistent.png");
        assert!(result.is_err());
    }
}
