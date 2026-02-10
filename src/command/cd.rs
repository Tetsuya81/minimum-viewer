use std::collections::BTreeMap;

use crate::app::App;

pub fn run(app: &mut App) -> bool {
    if let Some((path, name, is_dir)) = app
        .selected_entry()
        .map(|e| (e.path.clone(), e.name.clone(), e.is_dir))
    {
        if is_dir {
            crate::debug_log::log(
                "command/cd.rs:run",
                "cd target",
                BTreeMap::from([("path", path.to_string_lossy().to_string())]),
                "H4",
            );
            app.on_directory_changed(path);
            app.status_message = format!("cd: {}", app.current_dir.display());
        } else {
            app.status_message = format!("cd: '{}' is not a directory", name);
        }
    } else {
        app.status_message = "cd: no selection".to_string();
    }
    false
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::app::{App, DirEntry, Mode};

    use super::run;

    #[test]
    fn cd_run_clears_filter_after_directory_change() {
        let base =
            std::env::temp_dir().join(format!("minimum-viewer-cd-test-{}", std::process::id()));
        let sub = base.join("sub");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&sub).expect("create temp dirs");

        let mut app = App {
            mode: Mode::Browse,
            current_dir: base.clone(),
            all_entries: vec![],
            entries: vec![DirEntry {
                name: "sub".to_string(),
                path: sub.clone(),
                is_dir: true,
                size: None,
                modified: None,
                permissions: None,
                owner: None,
                group: None,
            }],
            selected_index: 0,
            filter_input: "tmp".to_string(),
            command_input: String::new(),
            command_candidates: crate::command::filter_candidates(""),
            command_selected: 0,
            shell_input: String::new(),
            shell_last_output: None,
            show_shell_popup: false,
            needs_full_redraw: false,
            status_bar_expanded: false,
            status_message: String::new(),
        };

        let should_quit = run(&mut app);

        assert!(!should_quit);
        assert_eq!(app.mode, Mode::Browse);
        assert!(app.filter_input.is_empty());
        assert_eq!(app.current_dir, sub);
        assert!(app.status_message.starts_with("cd: "));

        let _ = std::fs::remove_dir_all(PathBuf::from(base));
    }
}
