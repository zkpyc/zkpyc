use include_dir::{include_dir, Dir};
use std::fs;
use std::path::Path;

// Embed the stdlib directory using include_dir!
static STDLIB_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/stdlib");
const VERSION: &str = env!("ZKPYC_STDLIB_VERSION");

pub struct StdLib;

impl StdLib {
    // Copy embedded stdlib to output_dir
    pub fn copy_stdlib(output_dir: &Path) {
        let stdlib_output_dir = output_dir.join("stdlib");
        fs::create_dir_all(&stdlib_output_dir).expect("Failed to create stdlib output directory");
        Self::copy_dir_recursive(&STDLIB_DIR, &stdlib_output_dir);

        // Embed version number in version.txt
        let version_file_path = stdlib_output_dir.join("version.txt");
        fs::write(version_file_path, VERSION)
            .expect("Failed to write stdlib version file");
    }

    // Helper function to copy directories and files recursively
    fn copy_dir_recursive(dir: &Dir, output_dir: &Path) {
        for entry in dir.entries() {
            match entry {
                include_dir::DirEntry::Dir(sub_dir) => {
                    let sub_dir_output_path = output_dir.join(sub_dir.path());
                    fs::create_dir_all(&sub_dir_output_path)
                        .unwrap_or_else(|_err| panic!("Failed to create subdirectory {}", &sub_dir_output_path.display()));
                    Self::copy_dir_recursive(sub_dir, &output_dir);
                    // println!("Copied: {}", &sub_dir_output_path.display());
                }
                include_dir::DirEntry::File(file) => {
                    let file_output_path = output_dir.join(file.path());
                    fs::write(&file_output_path, file.contents())
                        .unwrap_or_else(|_err| panic!("Failed to write file {}", &file_output_path.display()));
                    // println!("Copied: {}", &file_output_path.display());
                }
            }
        }
    }

    // Return the version number
    pub fn version() -> &'static str {
        VERSION
    }
}
