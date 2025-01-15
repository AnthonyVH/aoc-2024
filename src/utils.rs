pub fn read_resource(file_name: &str) -> Result<String, std::io::Error> {
    let input_path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", file_name]
        .iter()
        .collect();
    std::fs::read_to_string(input_path)
}
