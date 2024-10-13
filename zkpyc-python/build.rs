use std::env;
use std::path::PathBuf;
use zkpyc_stdlib::StdLib;

fn main() {
    println!("Running build script in zkpyc_stdlib...");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    StdLib::copy_stdlib(&out_dir.as_path());
    let stdlib_path = out_dir.join("stdlib");

    println!("cargo:rustc-env=ZKPYC_STDLIB_PATH={}", stdlib_path.display());
}