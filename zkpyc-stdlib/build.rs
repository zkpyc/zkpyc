use std::env;

fn main() {
    println!("Running build script in zkpyc_stdlib...");
    let version = env!("CARGO_PKG_VERSION");
    println!("cargo:rustc-env=ZKPYC_STDLIB_VERSION={}", version);
    println!("cargo:rerun-if-changed=stdlib");
}
