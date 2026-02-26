use crate::app::App;
use crate::command::path::resolve_path;
use std::path::{Path, PathBuf};

pub fn run(app: &mut App, args: &[String]) -> bool {
    // parse_command_input passes the entire argument string as a single element.
    // Split it here to support `cp <src> <dest>`.
    let parsed_args: Vec<String> = args
        .iter()
        .flat_map(|a| a.split_whitespace().map(String::from))
        .collect();

    if parsed_args.len() > 2 {
        app.status_message = "cp: too many arguments".to_string();
        return false;
    }

    let (src, dest) = match parsed_args.len() {
        0 => {
            let Some(entry) = app.selected_entry().cloned() else {
                app.status_message = "cp: no selection".to_string();
                return false;
            };
            if entry.name == ".." {
                app.status_message = "cp: cannot copy parent entry".to_string();
                return false;
            }
            let dest = generate_copy_name(&entry.path);
            (entry.path, dest)
        }
        1 => {
            let src = match resolve_path(&app.current_dir, &parsed_args[0]) {
                Ok(path) => path,
                Err(err) => {
                    app.status_message = format!("cp: {}", err);
                    return false;
                }
            };
            let file_name = match src.file_name() {
                Some(name) => name,
                None => {
                    app.status_message = "cp: cannot determine file name".to_string();
                    return false;
                }
            };
            let dest_candidate = app.current_dir.join(file_name);
            let dest = if dest_candidate == src {
                generate_copy_name(&src)
            } else if std::fs::symlink_metadata(&dest_candidate).is_ok() {
                generate_copy_name(&dest_candidate)
            } else {
                dest_candidate
            };
            (src, dest)
        }
        2 => {
            let src = match resolve_path(&app.current_dir, &parsed_args[0]) {
                Ok(path) => path,
                Err(err) => {
                    app.status_message = format!("cp: {}", err);
                    return false;
                }
            };
            let dest = match resolve_path(&app.current_dir, &parsed_args[1]) {
                Ok(path) => path,
                Err(err) => {
                    app.status_message = format!("cp: {}", err);
                    return false;
                }
            };
            if std::fs::symlink_metadata(&dest).is_ok() {
                app.status_message = format!("cp: '{}' already exists", dest.display());
                return false;
            }
            (src, dest)
        }
        _ => unreachable!(),
    };

    if std::fs::symlink_metadata(&src).is_err() {
        app.status_message = format!("cp: '{}': No such file or directory", src.display());
        return false;
    }

    match copy_entry(&src, &dest) {
        Ok(()) => {
            app.reload_entries();
            if let Some(dest_name) = dest.file_name().and_then(|n| n.to_str()) {
                if let Some(index) = app.entries.iter().position(|e| e.name == dest_name) {
                    app.selected_index = index;
                }
            }
            app.status_message = format!(
                "cp: '{}' -> '{}'",
                src.display(),
                dest.display()
            );
        }
        Err(err) => {
            app.status_message = format!("cp: {}", err);
        }
    }

    false
}

fn generate_copy_name(path: &Path) -> PathBuf {
    let parent = path.parent().unwrap_or(Path::new("."));
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let ext = path.extension().and_then(|e| e.to_str());

    for n in 1..=999 {
        let name = match ext {
            Some(e) => format!("{}({}).{}", stem, n, e),
            None => format!("{}({})", stem, n),
        };
        let candidate = parent.join(&name);
        if std::fs::symlink_metadata(&candidate).is_err() {
            return candidate;
        }
    }

    // Fallback (extremely unlikely)
    let name = match ext {
        Some(e) => format!("{}(copy).{}", stem, e),
        None => format!("{}(copy)", stem),
    };
    parent.join(&name)
}

fn copy_entry(src: &Path, dest: &Path) -> std::io::Result<()> {
    let meta = std::fs::symlink_metadata(src)?;

    if meta.is_dir() {
        let src_canonical = std::fs::canonicalize(src)?;
        let dest_abs = if dest.is_absolute() {
            dest.to_path_buf()
        } else {
            std::env::current_dir()?.join(dest)
        };
        let dest_check = if let Some(parent) = dest_abs.parent() {
            std::fs::canonicalize(parent)?.join(dest_abs.file_name().unwrap_or_default())
        } else {
            dest_abs
        };
        if dest_check.starts_with(&src_canonical) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("cannot copy '{}' into itself", src.display()),
            ));
        }
    }

    if meta.is_symlink() {
        let target = std::fs::read_link(src)?;
        #[cfg(unix)]
        std::os::unix::fs::symlink(&target, dest)?;
        #[cfg(not(unix))]
        {
            // On non-Unix, fall back to copying the resolved target
            let resolved_meta = std::fs::metadata(src)?;
            if resolved_meta.is_dir() {
                copy_dir_recursive(src, dest)?;
            } else {
                std::fs::copy(src, dest)?;
            }
        }
    } else if meta.is_dir() {
        copy_dir_recursive(src, dest)?;
    } else {
        std::fs::copy(src, dest)?;
    }

    Ok(())
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> std::io::Result<()> {
    std::fs::create_dir(dest)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());
        copy_entry(&src_path, &dest_path)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{App, DirEntry};

    fn temp_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("minimum-viewer-cp-{}-{}", name, std::process::id()))
    }

    #[test]
    fn cp_rejects_parent_entry() {
        let mut app = App::new();
        app.entries = vec![DirEntry {
            name: "..".to_string(),
            path: app.current_dir.clone(),
            is_dir: true,
            size: None,
            modified: None,
            permissions: None,
            uid: None,
            gid: None,
            owner: None,
            group: None,
            link_target: None,
            is_dangling: false,
        }];
        app.selected_index = 0;

        run(&mut app, &[]);
        assert_eq!(app.status_message, "cp: cannot copy parent entry");
    }

    #[test]
    fn cp_rejects_too_many_args() {
        let mut app = App::new();
        run(&mut app, &["a".into(), "b".into(), "c".into()]);
        assert_eq!(app.status_message, "cp: too many arguments");
    }

    #[test]
    fn cp_file_without_args() {
        let base = temp_dir("no-args");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).expect("create base dir");
        std::fs::write(base.join("file.txt"), "hello").expect("write file");

        let mut app = App::new();
        app.current_dir = base.clone();
        app.reload_entries();
        app.selected_index = app
            .entries
            .iter()
            .position(|e| e.name == "file.txt")
            .expect("must have file.txt");

        run(&mut app, &[]);

        assert!(base.join("file(1).txt").exists());
        assert_eq!(
            std::fs::read_to_string(base.join("file(1).txt")).unwrap(),
            "hello"
        );
        assert!(app.status_message.contains("file(1).txt"));

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn cp_file_increments_number() {
        let base = temp_dir("increment");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).expect("create base dir");
        std::fs::write(base.join("file.txt"), "hello").expect("write file");
        std::fs::write(base.join("file(1).txt"), "existing").expect("write existing copy");

        let mut app = App::new();
        app.current_dir = base.clone();
        app.reload_entries();
        app.selected_index = app
            .entries
            .iter()
            .position(|e| e.name == "file.txt")
            .expect("must have file.txt");

        run(&mut app, &[]);

        assert!(base.join("file(2).txt").exists());
        assert_eq!(
            std::fs::read_to_string(base.join("file(2).txt")).unwrap(),
            "hello"
        );

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn cp_file_with_explicit_dest() {
        let base = temp_dir("explicit-dest");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).expect("create base dir");
        std::fs::write(base.join("src.txt"), "data").expect("write src");

        let mut app = App::new();
        app.current_dir = base.clone();
        app.reload_entries();

        run(&mut app, &["src.txt".into(), "dst.txt".into()]);

        assert!(base.join("dst.txt").exists());
        assert_eq!(
            std::fs::read_to_string(base.join("dst.txt")).unwrap(),
            "data"
        );

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn cp_rejects_existing_explicit_dest() {
        let base = temp_dir("existing-dest");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).expect("create base dir");
        std::fs::write(base.join("src.txt"), "data").expect("write src");
        std::fs::write(base.join("dst.txt"), "existing").expect("write dst");

        let mut app = App::new();
        app.current_dir = base.clone();
        app.reload_entries();

        run(&mut app, &["src.txt".into(), "dst.txt".into()]);

        assert!(app.status_message.contains("already exists"));
        assert_eq!(
            std::fs::read_to_string(base.join("dst.txt")).unwrap(),
            "existing"
        );

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn cp_directory_recursive() {
        let base = temp_dir("dir-recursive");
        let _ = std::fs::remove_dir_all(&base);
        let src_dir = base.join("mydir");
        let nested = src_dir.join("sub");
        std::fs::create_dir_all(&nested).expect("create nested");
        std::fs::write(src_dir.join("a.txt"), "aaa").expect("write a");
        std::fs::write(nested.join("b.txt"), "bbb").expect("write b");

        let mut app = App::new();
        app.current_dir = base.clone();
        app.reload_entries();
        app.selected_index = app
            .entries
            .iter()
            .position(|e| e.name == "mydir")
            .expect("must have mydir");

        run(&mut app, &[]);

        let copy_dir = base.join("mydir(1)");
        assert!(copy_dir.is_dir());
        assert_eq!(
            std::fs::read_to_string(copy_dir.join("a.txt")).unwrap(),
            "aaa"
        );
        assert_eq!(
            std::fs::read_to_string(copy_dir.join("sub").join("b.txt")).unwrap(),
            "bbb"
        );

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn cp_nonexistent_source() {
        let base = temp_dir("nonexistent");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).expect("create base dir");

        let mut app = App::new();
        app.current_dir = base.clone();
        app.reload_entries();

        run(&mut app, &["nosuchfile.txt".into(), "dst.txt".into()]);

        assert!(app.status_message.contains("No such file or directory"));

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn cp_file_no_extension() {
        let base = temp_dir("no-ext");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).expect("create base dir");
        std::fs::write(base.join("Makefile"), "all:").expect("write file");

        let mut app = App::new();
        app.current_dir = base.clone();
        app.reload_entries();
        app.selected_index = app
            .entries
            .iter()
            .position(|e| e.name == "Makefile")
            .expect("must have Makefile");

        run(&mut app, &[]);

        assert!(base.join("Makefile(1)").exists());

        let _ = std::fs::remove_dir_all(base);
    }

    #[cfg(unix)]
    #[test]
    fn cp_symlink_copies_as_link() {
        use std::os::unix::fs::symlink;

        let base = temp_dir("symlink");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).expect("create base dir");
        std::fs::write(base.join("target.txt"), "data").expect("write target");
        symlink("target.txt", base.join("link.txt")).expect("create symlink");

        let mut app = App::new();
        app.current_dir = base.clone();
        app.reload_entries();
        app.selected_index = app
            .entries
            .iter()
            .position(|e| e.name == "link.txt")
            .expect("must have link.txt");

        run(&mut app, &[]);

        let copy_link = base.join("link(1).txt");
        assert!(copy_link.exists() || std::fs::symlink_metadata(&copy_link).is_ok());
        let link_target = std::fs::read_link(&copy_link).expect("should be a symlink");
        assert_eq!(link_target, PathBuf::from("target.txt"));

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn cp_with_one_arg_from_current_dir() {
        let base = temp_dir("one-arg-current");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).expect("create base dir");
        std::fs::write(base.join("file.txt"), "content").expect("write file");

        let mut app = App::new();
        app.current_dir = base.clone();
        app.reload_entries();

        run(&mut app, &["file.txt".into()]);

        assert!(base.join("file(1).txt").exists());
        assert_eq!(
            std::fs::read_to_string(base.join("file(1).txt")).unwrap(),
            "content"
        );

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn cp_rejects_copy_into_self() {
        let base = temp_dir("into-self");
        let _ = std::fs::remove_dir_all(&base);
        let src_dir = base.join("mydir");
        std::fs::create_dir_all(&src_dir).expect("create src dir");
        std::fs::write(src_dir.join("a.txt"), "aaa").expect("write a");

        let mut app = App::new();
        app.current_dir = base.clone();
        app.reload_entries();

        // Try to copy mydir into mydir/copy (dest is inside src)
        run(
            &mut app,
            &[
                src_dir.display().to_string(),
                src_dir.join("copy").display().to_string(),
            ],
        );

        assert!(
            app.status_message.contains("cannot copy"),
            "expected 'cannot copy' error, got: {}",
            app.status_message
        );

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn generate_copy_name_basic() {
        let base = temp_dir("gen-name");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).expect("create base dir");
        std::fs::write(base.join("test.txt"), "x").expect("write file");

        let result = generate_copy_name(&base.join("test.txt"));
        assert_eq!(result, base.join("test(1).txt"));

        let _ = std::fs::remove_dir_all(base);
    }
}
