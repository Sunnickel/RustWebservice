use std::{
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
    sync::Arc,
};

pub fn get_static_file_content<'a>(route: &str, folder: &String) -> (Arc<String>, String) {
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

pub fn get_file_content(file_path: &Path) -> Arc<String> {
    let file =
        File::open(file_path).unwrap_or_else(|_| panic!("Could not open file: {:?}", file_path));

    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();

    buf_reader
        .read_to_string(&mut contents)
        .expect("File couldn't be read");

    Arc::new(contents)
}
