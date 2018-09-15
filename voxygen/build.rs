use std::{
    env,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn main() {
    // Git hash
    let output_hash = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .expect("failed to `call git rev-parse HEAD`");
    let line = String::from_utf8(output_hash.stdout).unwrap();
    let git_hash = line.trim();

    if git_hash.len() != 40 {
        panic!("invalid git commit id: {}", git_hash);
    }

    // Git time
    let output_time = Command::new("git")
        .args(&["log", "-1", "--pretty=format:%ct"])
        .output()
        .expect("failed to call `git log`");
    let git_time = String::from_utf8(output_time.stdout).unwrap();

    // Profile
    let profile = env::var("PROFILE").expect("failed to read PROFILE env variable");

    // Build time
    let build_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("invalid system time")
        .as_secs();

    println!("cargo:rustc-env=GIT_HASH={}", git_hash); // Store as GIT_HASH env variable
    println!("cargo:rustc-env=GIT_TIME={}", git_time); // Store as GIT_TIME env variable
    println!("cargo:rustc-env=PROFILE={}", profile); // Store as PROFILE env variable
    println!("cargo:rustc-env=BUILD_TIME={:?}", build_time); // Store as BUILD_TIME env variable
}
