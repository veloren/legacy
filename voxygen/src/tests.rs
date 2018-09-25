#[cfg(test)]
mod tests {
    use std::{
        ffi::OsStr,
        fs::{self, DirEntry},
        io,
        path::Path,
        process::Command,
        str, thread,
    };

    use chrono::Datelike;
    use tempfile;

    use get_build_time;
    use get_git_hash;
    use get_git_time;
    use get_profile;
    use shader::Shader;

    fn visit_dirs(dir: &Path, cb: &Fn(&DirEntry)) -> io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    visit_dirs(&path, cb)?;
                } else {
                    cb(&entry);
                }
            }
        }
        Ok(())
    }

    #[test]
    fn check_git_hash() {
        assert_ne!(get_git_hash(), "");
    }

    #[test]
    fn check_git_time() {
        let git_time = get_git_time();
        assert!(git_time.year() > 2017 && git_time.year() <= 3000);
    }

    #[test]
    fn check_profile() {
        assert!(get_profile() == "debug" || get_profile() == "release");
    }

    #[test]
    fn check_build_time() {
        let build_time = get_build_time();
        assert!(build_time.year() > 2017 && build_time.year() <= 3000);
    }

    fn validate_shader(filename: &str, shader_type: &str) -> bool {
        let expanded_shader = Shader::expand(filename).unwrap();
        let tmp_file = tempfile::Builder::new()
            .suffix(&format!(".{}", shader_type))
            .tempfile()
            .unwrap();

        let tmp_filename = tmp_file.path().file_name().and_then(OsStr::to_str).unwrap();

        fs::write(&tmp_filename, &expanded_shader).unwrap();

        let output = Command::new("glslangValidator")
            .arg(&tmp_filename)
            .output()
            .expect("failed to run glslangValidator");

        if !output.status.success() {
            println!(
                "glslangValidator failed for {}: {}",
                filename,
                str::from_utf8(&output.stdout).unwrap()
            );
        }

        let _ = fs::remove_file(&tmp_filename);
        output.status.success()
    }

    #[test]
    fn test_shaders_validity() {
        // Skip the test if glslangValidator is not in PATH.
        if Command::new("glslangValidator").output().is_err() {
            thread::spawn(|| {
                panic!("glslangValidator not found, skipping test");
            });
            return;
        }

        visit_dirs(Path::new("shaders"), &|entry: &DirEntry| {
            let p = entry.path();
            let path = p.to_str().unwrap();
            let ext = p.extension().unwrap();

            if ext == "frag" {
                assert!(validate_shader(path, "frag"));
            } else if ext == "vert" {
                assert!(validate_shader(path, "vert"));
            }
        }).unwrap();
    }
}
