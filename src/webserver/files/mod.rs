use std::{
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
pub(crate) fn get_static_file_content<'a>(route: &str, folder: &String) -> (Arc<String>, String) {
    let file_relative_path = route
        .strip_prefix(folder)
        .unwrap_or(route)
        .trim_start_matches('/')
        .split('/')
        .collect::<Vec<_>>()[1];
    let folder_path = Path::new(&folder).components().collect::<PathBuf>();
    let file_path = folder_path.join(file_relative_path);
    let content_type = match file_path.extension().and_then(|e| e.to_str()) {
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("html") => "text/html",
        Some("json") => "application/json",
        _ => "text/plain",
    };
    let content = get_file_content(&file_path);
    (content, content_type.to_string())
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
    let file =
        File::open(file_path).unwrap_or_else(|_| panic!("Could not open file: {:?}", file_path));
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader
        .read_to_string(&mut contents)
        .expect("File couldn't be read");
    Arc::new(contents)
}
