use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::app::App;
use crate::command::path::resolve_path;

pub fn run(app: &mut App, args: &[String]) -> bool {
    if args.len() > 1 {
        app.status_message = "cd: too many arguments".to_string();
        return false;
    }

    let target = if args.is_empty() {
        match std::env::var("HOME") {
            Ok(home) => PathBuf::from(home),
            Err(_) => {
                app.status_message = "cd: HOME not set".to_string();
                return false;
            }
        }
    } else {
        match resolve_path(&app.current_dir, &args[0]) {
            Ok(path) => path,
            Err(err) => {
                app.status_message = format!("cd: {}", err);
                return false;
            }
        }
    };

    let meta = match std::fs::metadata(&target) {
        Ok(meta) => meta,
        Err(err) => {
            app.status_message = format!("cd: {}: {}", target.display(), err);
            return false;
        }
    };
    if !meta.is_dir() {
        app.status_message = format!("cd: '{}' is not a directory", target.display());
        return false;
    }

    open_directory(app, target);
    false
}

fn open_directory(app: &mut App, path: PathBuf) {
    crate::debug_log::log(
        "command/cd.rs:run",
        "cd target",
        BTreeMap::from([("path", path.to_string_lossy().to_string())]),
        "H4",
    );
    app.on_directory_changed(path);
    app.status_message = format!("cd: {}", app.current_dir.display());
}

#[cfg(test)]
fn test_app(base: &std::path::Path) -> App {
    use crate::app::{DirEntry, Mode};
    App {
        mode: Mode::Browse,
        current_dir: base.to_path_buf(),
        all_entries: vec![],
        entries: vec![DirEntry {
            name: "sub".to_string(),
            path: base.join("sub"),
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
        }],
        selected_index: 0,
        filter_input: "tmp".to_string(),
        command_input: String::new(),
        command_candidates: crate::command::filter_candidates(""),
        command_selected: None,
        shell_input: String::new(),
        create_input: String::new(),
        shell_last_output: None,
        show_shell_popup: false,
        help_list_state: ratatui::widgets::ListState::default(),
        show_delete_confirm: false,
        pending_delete: None,
        needs_full_redraw: false,
        status_bar_expanded: false,
        status_message: String::new(),
        cd_on_quit_enabled: false,
        user_name_cache: std::collections::HashMap::new(),
        group_name_cache: std::collections::HashMap::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::run;

    #[test]
    fn cd_run_no_args_goes_to_home() {
        let base =
            std::env::temp_dir().join(format!("minimum-viewer-cd-home2-{}", std::process::id()));
        let home = base.join("fakehome");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&home).expect("create temp dirs");
        let old_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", &home);

        let mut app = super::test_app(&base);

        let should_quit = run(&mut app, &[]);

        assert!(!should_quit);
        assert_eq!(app.current_dir, home);
        assert!(app.filter_input.is_empty());
        assert!(app.status_message.starts_with("cd: "));

        if let Some(value) = old_home {
            std::env::set_var("HOME", value);
        } else {
            std::env::remove_var("HOME");
        }
        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn cd_run_supports_relative_path_argument() {
        let base =
            std::env::temp_dir().join(format!("minimum-viewer-cd-rel-{}", std::process::id()));
        let sub = base.join("sub");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&sub).expect("create temp dirs");
        let mut app = super::test_app(&base);

        run(&mut app, &["sub".to_string()]);
        assert_eq!(app.current_dir, sub);
        assert!(app.status_message.starts_with("cd: "));

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn cd_run_supports_tilde_path_argument() {
        let root =
            std::env::temp_dir().join(format!("minimum-viewer-cd-home-{}", std::process::id()));
        let home = root.join("home");
        let target = home.join("work");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&target).expect("create temp dirs");
        let old_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", &home);

        let mut app = super::test_app(&root);
        run(&mut app, &["~/work".to_string()]);
        assert_eq!(app.current_dir, target);

        if let Some(value) = old_home {
            std::env::set_var("HOME", value);
        } else {
            std::env::remove_var("HOME");
        }
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn cd_run_rejects_multiple_arguments() {
        let base =
            std::env::temp_dir().join(format!("minimum-viewer-cd-many-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&base);
        let mut app = super::test_app(&base);

        run(&mut app, &["a".to_string(), "b".to_string()]);
        assert_eq!(app.status_message, "cd: too many arguments");

        let _ = std::fs::remove_dir_all(base);
    }
}
