use crate::app::App;
use std::io::ErrorKind;

pub fn run(app: &mut App, args: &[String]) -> bool {
    if args.is_empty() {
        app.status_message = "rename: missing new name".to_string();
        return false;
    }
    if args.len() > 1 {
        app.status_message = "rename: too many arguments".to_string();
        return false;
    }

    let new_name = args[0].trim();
    if new_name.is_empty() {
        app.status_message = "rename: missing new name".to_string();
        return false;
    }
    if new_name.chars().any(std::path::is_separator) {
        app.status_message = "rename: new name must not contain path separators".to_string();
        return false;
    }

    let Some(entry) = app.selected_entry().cloned() else {
        app.status_message = "rename: no selection".to_string();
        return false;
    };
    if entry.name == ".." {
        app.status_message = "rename: cannot rename parent entry".to_string();
        return false;
    }
    if entry.name == new_name {
        app.status_message = "rename: new name is the same as current name".to_string();
        return false;
    }

    let Some(parent) = entry.path.parent() else {
        app.status_message = "rename: cannot resolve parent directory".to_string();
        return false;
    };
    let destination = parent.join(new_name);
    match std::fs::symlink_metadata(&destination) {
        Ok(_) => {
            app.status_message = format!("rename: '{}' already exists", new_name);
            return false;
        }
        Err(err) if err.kind() == ErrorKind::NotFound => {}
        Err(err) => {
            app.status_message = format!("rename: {}: {}", destination.display(), err);
            return false;
        }
    }

    match std::fs::rename(&entry.path, &destination) {
        Ok(()) => {
            app.reload_entries();
            if let Some(index) = app
                .entries
                .iter()
                .position(|candidate| candidate.name == new_name)
            {
                app.selected_index = index;
            }
            app.status_message = format!("rename: '{}' -> '{}'", entry.name, new_name);
        }
        Err(err) => {
            app.status_message = format!(
                "rename: {} -> {}: {}",
                entry.path.display(),
                destination.display(),
                err
            );
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::run;
    use crate::app::{App, DirEntry};
    #[cfg(unix)]
    use std::os::unix::fs::symlink;
    use std::path::PathBuf;

    #[test]
    fn rename_requires_argument() {
        let mut app = App::new();
        run(&mut app, &[]);
        assert_eq!(app.status_message, "rename: missing new name");
    }

    #[test]
    fn rename_rejects_separator() {
        let mut app = App::new();
        run(&mut app, &["a/b".to_string()]);
        assert_eq!(
            app.status_message,
            "rename: new name must not contain path separators"
        );
    }

    #[test]
    fn rename_selected_entry() {
        let base =
            std::env::temp_dir().join(format!("minimum-viewer-rename-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).expect("create base dir");
        std::fs::write(base.join("old.txt"), "x").expect("write file");

        let mut app = App::new();
        app.current_dir = base.clone();
        app.reload_entries();
        app.selected_index = app
            .entries
            .iter()
            .position(|entry| entry.name == "old.txt")
            .expect("must have old.txt");

        run(&mut app, &["new.txt".to_string()]);

        assert!(base.join("new.txt").exists());
        assert!(!base.join("old.txt").exists());
        assert!(app.status_message.contains("old.txt"));
        assert!(app.status_message.contains("new.txt"));

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn rename_rejects_existing_destination() {
        let base = std::env::temp_dir().join(format!(
            "minimum-viewer-rename-exists-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).expect("create base dir");
        std::fs::write(base.join("src.txt"), "x").expect("write src");
        std::fs::write(base.join("dst.txt"), "y").expect("write dst");

        let mut app = App::new();
        app.current_dir = base.clone();
        app.reload_entries();
        app.selected_index = app
            .entries
            .iter()
            .position(|entry| entry.name == "src.txt")
            .expect("must have src.txt");

        run(&mut app, &["dst.txt".to_string()]);

        assert_eq!(app.status_message, "rename: 'dst.txt' already exists");
        assert!(base.join("src.txt").exists());

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn rename_rejects_parent_entry() {
        let mut app = App::new();
        app.entries = vec![DirEntry {
            name: "..".to_string(),
            path: PathBuf::from(".."),
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

        run(&mut app, &["next".to_string()]);
        assert_eq!(app.status_message, "rename: cannot rename parent entry");
    }

    #[cfg(unix)]
    #[test]
    fn rename_rejects_existing_dangling_symlink_destination() {
        let base = std::env::temp_dir().join(format!(
            "minimum-viewer-rename-dangling-dst-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).expect("create base dir");
        std::fs::write(base.join("src.txt"), "x").expect("write src");
        let dangling_dst = base.join("dst-link");
        symlink(base.join("missing-target"), &dangling_dst).expect("create dangling symlink");

        let mut app = App::new();
        app.current_dir = base.clone();
        app.reload_entries();
        app.selected_index = app
            .entries
            .iter()
            .position(|entry| entry.name == "src.txt")
            .expect("must have src.txt");

        run(&mut app, &["dst-link".to_string()]);

        assert_eq!(app.status_message, "rename: 'dst-link' already exists");
        assert!(base.join("src.txt").exists());
        assert!(
            std::fs::symlink_metadata(&dangling_dst).is_ok(),
            "existing dangling destination must remain untouched"
        );

        let _ = std::fs::remove_dir_all(base);
    }
}
