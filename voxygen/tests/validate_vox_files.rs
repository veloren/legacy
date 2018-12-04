use std::fs;

#[test]
fn validate_vox_files() {
    let paths = fs::read_dir("../assets/voxygen/cosmetic/creature/friendly").unwrap();
    let mut files_checked = 0;

    for path in paths {
        let path = path.unwrap();
        if path.file_name().into_string().unwrap() == ".gitignore" {
            continue;
        }
        let path_string = path.path().into_os_string().into_string().unwrap();
        let vox = dot_vox::load(&path_string);
        assert_eq!(true, vox.is_ok(), "Failed to validate file '{:?}'", path_string);

        files_checked += 1;
    }

    assert_ne!(
        0, files_checked,
        "No files found in assets/voxygen/cosmetic/creature/friendly"
    );
}
