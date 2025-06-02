use std::path::Path;

pub fn extract_filename_from_path(path: &Path) -> String {
    path.file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
        .trim()
        .to_string()
}
