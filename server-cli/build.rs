use std::{
    env,
    process::Command,
};

fn main() {
    // Git hash
    if let Ok(output_hash) = Command::new("git").args(&["rev-parse", "HEAD"]).output() {
        let line = String::from_utf8(output_hash.stdout).unwrap();
        let git_hash = line.trim();
        println!("cargo:rustc-env=GIT_HASH={}", git_hash); // Store as GIT_HASH env variable
    }
}
