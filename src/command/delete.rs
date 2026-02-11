use crate::app::App;
use crate::command::path::resolve_path;

pub fn run(app: &mut App, args: &[String]) -> bool {
    if args.len() > 1 {
        app.status_message = "delete: too many arguments".to_string();
        return false;
    }

    let target = if args.is_empty() {
        let Some(entry) = app.selected_entry().cloned() else {
            app.status_message = "delete: no selection".to_string();
            return false;
        };
        if entry.name == ".." {
            app.status_message = "delete: cannot delete parent entry".to_string();
            return false;
        }
        entry.path
    } else {
        match resolve_path(&app.current_dir, &args[0]) {
            Ok(path) => path,
            Err(err) => {
                app.status_message = format!("delete: {}", err);
                return false;
            }
        }
    };

    let meta = match std::fs::metadata(&target) {
        Ok(meta) => meta,
        Err(err) => {
            app.status_message = format!("delete: {}: {}", target.display(), err);
            return false;
        }
    };

    if meta.is_dir() {
        app.open_delete_confirm(target);
        return false;
    }

    delete_file(app, &target)
}

fn delete_file(app: &mut App, target: &std::path::Path) -> bool {
    match std::fs::remove_file(target) {
        Ok(()) => {
            app.reload_entries();
            app.status_message = format!("delete: removed '{}'", target.display());
        }
        Err(err) => {
            app.status_message = format!("delete: {}: {}", target.display(), err);
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::run;
    use crate::app::{App, DirEntry};

    #[test]
    fn delete_rejects_parent_entry_when_no_args() {
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
        }];
        app.selected_index = 0;

        run(&mut app, &[]);
        assert_eq!(app.status_message, "delete: cannot delete parent entry");
    }

    #[test]
    fn delete_file_with_explicit_path() {
        let base =
            std::env::temp_dir().join(format!("minimum-viewer-delete-file-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).expect("create base dir");
        let file = base.join("a.txt");
        std::fs::write(&file, "x").expect("write file");

        let mut app = App::new();
        app.current_dir = base.clone();
        app.reload_entries();

        run(&mut app, &["a.txt".to_string()]);

        assert!(!file.exists());
        assert!(app.status_message.starts_with("delete: removed"));

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn delete_file_without_args_uses_selected_entry() {
        let base = std::env::temp_dir().join(format!(
            "minimum-viewer-delete-selected-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).expect("create base dir");
        let file = base.join("selected.txt");
        std::fs::write(&file, "x").expect("write file");

        let mut app = App::new();
        app.current_dir = base.clone();
        app.reload_entries();
        app.selected_index = app
            .entries
            .iter()
            .position(|entry| entry.name == "selected.txt")
            .expect("selected file must exist");

        run(&mut app, &[]);

        assert!(!file.exists());
        assert!(app.status_message.starts_with("delete: removed"));

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn delete_directory_enters_confirmation_state() {
        let base =
            std::env::temp_dir().join(format!("minimum-viewer-delete-dir-{}", std::process::id()));
        let dir = base.join("dir");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&dir).expect("create dir");

        let mut app = App::new();
        app.current_dir = base.clone();
        app.reload_entries();

        run(&mut app, &["dir".to_string()]);

        assert!(app.show_delete_confirm);
        assert!(app.pending_delete.is_some());
        assert!(dir.exists());

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn delete_directory_is_removed_after_confirmation_yes() {
        let base =
            std::env::temp_dir().join(format!("minimum-viewer-delete-yes-{}", std::process::id()));
        let dir = base.join("dir");
        let file = dir.join("nested.txt");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&dir).expect("create dir");
        std::fs::write(&file, "x").expect("write nested file");

        let mut app = App::new();
        app.current_dir = base.clone();
        app.reload_entries();

        run(&mut app, &["dir".to_string()]);
        app.confirm_delete_yes();

        assert!(!dir.exists());
        assert!(!app.show_delete_confirm);
        assert!(app.pending_delete.is_none());

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn delete_directory_is_not_removed_after_confirmation_no() {
        let base =
            std::env::temp_dir().join(format!("minimum-viewer-delete-no-{}", std::process::id()));
        let dir = base.join("dir");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&dir).expect("create dir");

        let mut app = App::new();
        app.current_dir = base.clone();
        app.reload_entries();

        run(&mut app, &["dir".to_string()]);
        app.confirm_delete_no();

        assert!(dir.exists());
        assert!(!app.show_delete_confirm);
        assert!(app.pending_delete.is_none());

        let _ = std::fs::remove_dir_all(base);
    }
}
