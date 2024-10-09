use std::env;
use std::path::PathBuf;

pub fn get_artifacts_path() -> PathBuf {
    PathBuf::from(env!("OUT_DIR"))
}