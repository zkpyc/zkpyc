use std::path::{Path, PathBuf};
use std::fs;


pub fn create_folder(workspace: &Path, folder_name: &str) -> PathBuf {
    let folder_path = workspace.join(folder_name);

    // Create the folder if it doesn't exist
    if !folder_path.exists() {
        fs::create_dir(&folder_path).expect(&format!("Failed to create folder {}", folder_name));
    }

    folder_path.to_path_buf()
}

pub fn rename_zkif_file(
    file_name: &str,
    new_name: &str,
    workspace: &Path,
) -> Result<(), std::io::Error> {
    let original_path = workspace.join(format!("{}.zkif", file_name));
    let new_file_name = format!("{}.zkif", new_name);
    let new_path = workspace.join(new_file_name);

    fs::rename(original_path, new_path)
}