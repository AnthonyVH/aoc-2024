pub fn read_resource(file_name: &str) -> Result<String, std::io::Error> {
    // Unfortunately there's no environment variable with the toplevel dir (i.e. the workspace dir).
    // So need to go one directory up, assuming the directory structure is <root>/util/src.
    let input_path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "..", "resources", file_name]
        .iter()
        .collect();
    std::fs::read_to_string(input_path)
}