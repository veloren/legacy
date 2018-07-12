use chrono::{Datelike, Utc};

use get_git_hash;
use get_git_time;
use get_profile;
use get_build_time;

#[test]
fn check_git_hash() {
    assert_ne!(get_git_hash(), "");
}

#[test]
fn check_git_time() {
    let git_time = get_git_time();
    assert!(
        git_time.year() > 2017 && git_time.year() <= 3000
    );
}

#[test]
fn check_profile() {
    assert!(
        get_profile() == "debug" || get_profile() == "release"
    );
}

#[test]
fn check_build_time () {
    let build_time = get_build_time();
    assert!(
        build_time.year() > 2017 && build_time.year() <= 3000
    );
}