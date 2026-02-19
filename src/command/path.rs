use std::path::{Path, PathBuf};

pub fn resolve_path(current_dir: &Path, input: &str) -> Result<PathBuf, String> {
    let raw = input.trim();
    if raw.is_empty() {
        return Err("empty path".to_string());
    }

    if raw.starts_with('~') {
        return resolve_tilde_path(raw);
    }

    let path = PathBuf::from(raw);
    if path.is_absolute() {
        Ok(path)
    } else {
        Ok(current_dir.join(path))
    }
}

fn resolve_tilde_path(raw: &str) -> Result<PathBuf, String> {
    if raw == "~" || raw.starts_with("~/") {
        let home = std::env::var("HOME").map_err(|_| "HOME is not set".to_string())?;
        let home = PathBuf::from(home);
        if raw == "~" {
            return Ok(home);
        }
        return Ok(home.join(&raw[2..]));
    }

    Err("~user expansion is not supported".to_string())
}

#[cfg(test)]
mod tests {
    use super::resolve_path;
    use std::path::{Path, PathBuf};

    #[test]
    fn resolve_path_handles_absolute_and_relative() {
        let current = Path::new("/tmp/base");
        assert_eq!(
            resolve_path(current, "/tmp/abs").expect("absolute path"),
            PathBuf::from("/tmp/abs")
        );
        assert_eq!(
            resolve_path(current, "child").expect("relative path"),
            PathBuf::from("/tmp/base/child")
        );
    }

    #[test]
    fn resolve_path_expands_tilde() {
        let _guard = crate::command::env_lock().lock().unwrap();
        let old_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", "/tmp/home");
        let current = Path::new("/tmp/base");
        assert_eq!(
            resolve_path(current, "~").expect("home path"),
            PathBuf::from("/tmp/home")
        );
        assert_eq!(
            resolve_path(current, "~/project").expect("home child path"),
            PathBuf::from("/tmp/home/project")
        );

        if let Some(home) = old_home {
            std::env::set_var("HOME", home);
        } else {
            std::env::remove_var("HOME");
        }
    }

    #[test]
    fn resolve_path_rejects_tilde_user() {
        let current = Path::new("/tmp/base");
        let err = resolve_path(current, "~someone").expect_err("must reject ~user");
        assert_eq!(err, "~user expansion is not supported");
    }
}
