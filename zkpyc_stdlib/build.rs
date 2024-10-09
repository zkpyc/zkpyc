use std::env;
use std::path::PathBuf;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;

fn main() {
    println!("Running build script in zkpyc_stdlib...");
    
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap()).join("zkpyc");
    std::fs::create_dir_all(&out_dir).expect("Failed to create zkpyc directory");
    let mut options = CopyOptions::new();
    options.overwrite = true;
    copy_items(&["stdlib"], &out_dir, &options).unwrap();
        
    println!("stdlib directory copied to {:?}", &out_dir);
    println!("cargo:rerun-if-changed=stdlib");
}