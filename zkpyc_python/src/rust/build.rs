use zkpyc_stdlib::get_artifacts_path;
use std::env;
use std::path::PathBuf;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;

fn main() {
    let stdlib_path = get_artifacts_path().join("zkpyc").join("stdlib");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let mut dest_path = out_dir.clone();
    for _ in 0..6 {
        dest_path = dest_path.parent().expect("Failed to go 6 directories up").to_path_buf();
    }

    dest_path.push("zkpyc");

    std::fs::create_dir_all(&dest_path).expect("Failed to create stdlib directory");
    let mut options = CopyOptions::new();
    options.overwrite = true;
    copy_items(&[stdlib_path], &dest_path, &options).unwrap();
}
