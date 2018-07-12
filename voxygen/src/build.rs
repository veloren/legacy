use std::process::Command;

fn main() {
    // @todo Error checking

    // Git hash
    let output_hash = Command::new("git").args(&["rev-parse", "HEAD"]).output().unwrap();
    let git_hash = String::from_utf8(output_hash.stdout).unwrap();

    // Git time
    let output_time = Command::new("git").args(&["log", "-1", "--pretty=format:%ct"]).output().unwrap();
    let git_time = String::from_utf8(output_time.stdout).unwrap();

    println!("cargo:rustc-env=GIT_HASH={}", git_hash); // Store as GIT_HASH env variable
    println!("cargo:rustc-env=GIT_TIME={}", git_time); // Store as GIT_TIME env variable
}