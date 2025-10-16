use std::{
    fs,
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
    sync::Arc,
};

/// Retrieves the content and content type of a static file based on route and folder.
///
/// This function determines the file path relative to a given folder, reads the file's
/// content, and infers its MIME type based on the file extension.
///
/// # Arguments
///
/// * `route` - A string slice that holds the full route to the file.
/// * `folder` - A string slice that holds the base folder path where files are located.
///
/// # Returns
///
/// A tuple containing:
/// * An `Arc<String>` with the file's content.
/// * A `String` representing the inferred MIME type of the file.
///
/// # Examples
///
/// ```
/// let (content, mime_type) = get_static_file_content("/static/css/style.css", "/var/www");
/// assert_eq!(mime_type, "text/css");
/// ```
pub(crate) fn get_static_file_content(route: &str, folder: &String) -> (Arc<String>, String) {
    let mut route_trimmed = route.trim_start_matches('/');

    let parts: Vec<&str> = route.trim_start_matches('/').splitn(2, '/').collect();
    let relative_path = if parts.len() > 1 { parts[1] } else { "" };
    let file_path = Path::new(folder).join(relative_path);

    log::debug!("Resolved static path: {}", file_path.display());

    let content_type = match file_path.extension().and_then(|e| e.to_str()) {
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("html") => "text/html",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("svg") => "image/svg+xml",
        _ => "text/plain",
    };

    match fs::read_to_string(&file_path) {
        Ok(content) => (Arc::new(content), content_type.to_string()),
        Err(e) => {
            log::warn!("Static file not found: {} ({})", file_path.display(), e);
            (Arc::new(String::new()), String::from("text/plain"))
        }
    }
}

/// Reads the entire content of a file into an `Arc<String>`.
///
/// Opens the specified file and reads its contents into a string. This function
/// panics if it fails to open or read the file.
///
/// # Arguments
///
/// * `file_path` - A reference to a `Path` that specifies the location of the file to read.
///
/// # Returns
///
/// An `Arc<String>` containing the full content of the file.
///
/// # Errors
///
/// This function will panic if the file cannot be opened or read.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// let content = get_file_content(&Path::new("example.txt"));
/// ```
pub(crate) fn get_file_content(file_path: &Path) -> Arc<String> {
    if !Path::exists(file_path) {
        return Arc::new(String::new());
    }

    let file =
        File::open(file_path).unwrap_or_else(|_| panic!("Cannot open {}", file_path.display()));
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader
        .read_to_string(&mut contents)
        .expect("File couldn't be read");
    Arc::new(contents)
}
