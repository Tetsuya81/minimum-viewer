use std::io::{self, stdout};
use std::path::{Path, PathBuf};
use std::process::Command;

use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;

use crate::app::App;

const EDITOR_SHELL_SNIPPET: &str = "${EDITOR} \"$TARGET_PATH\"";

pub fn run(app: &mut App) -> bool {
    let Some(target_path) = selected_path(app) else {
        app.status_message = "editor: no selection".to_string();
        return false;
    };

    let editor = std::env::var("EDITOR").unwrap_or_default();
    if editor.trim().is_empty() {
        app.status_message = "editor: $EDITOR is not set".to_string();
        return false;
    }

    match open_in_editor(&target_path) {
        Ok(Some(code)) => {
            app.status_message = format!("editor exited with code {}", code);
        }
        Ok(None) => {
            app.status_message = "editor terminated by signal".to_string();
        }
        Err(err) => {
            app.status_message = format!("editor failed: {}", err);
        }
    }
    app.request_full_redraw();

    false
}

fn selected_path(app: &App) -> Option<PathBuf> {
    app.selected_entry().map(|entry| entry.path.clone())
}

fn open_in_editor(target_path: &Path) -> io::Result<Option<i32>> {
    suspend_tui()?;
    let status_result = build_editor_command(target_path).status();
    let restore_result = resume_tui();

    match (status_result, restore_result) {
        (Ok(status), Ok(())) => Ok(status.code()),
        (Err(exec_err), Ok(())) => Err(exec_err),
        (Ok(_), Err(restore_err)) => Err(restore_err),
        (Err(exec_err), Err(restore_err)) => Err(io::Error::new(
            restore_err.kind(),
            format!("{}; also failed to run editor: {}", restore_err, exec_err),
        )),
    }
}

fn build_editor_command(target_path: &Path) -> Command {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    let mut command = Command::new(shell);
    command
        .arg("-lc")
        .arg(EDITOR_SHELL_SNIPPET)
        .env("TARGET_PATH", target_path.as_os_str());
    command
}

fn suspend_tui() -> io::Result<()> {
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()
}

fn resume_tui() -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{App, DirEntry, Mode};
    use std::ffi::OsString;

    fn empty_app() -> App {
        App {
            mode: Mode::Browse,
            current_dir: PathBuf::from("."),
            all_entries: vec![],
            entries: vec![],
            selected_index: 0,
            filter_input: String::new(),
            command_input: String::new(),
            command_candidates: crate::command::filter_candidates(""),
            command_selected: None,
            shell_input: String::new(),
            shell_last_output: None,
            show_shell_popup: false,
            help_popup_body: None,
            show_help_popup: false,
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

    #[test]
    fn run_sets_error_when_no_selection() {
        std::env::remove_var("EDITOR");

        let mut app = empty_app();
        let should_quit = run(&mut app);

        assert!(!should_quit);
        assert_eq!(app.status_message, "editor: no selection");
    }

    #[test]
    fn run_sets_error_when_editor_is_not_set() {
        std::env::remove_var("EDITOR");

        let mut app = empty_app();
        app.entries = vec![DirEntry {
            name: "file.txt".to_string(),
            path: PathBuf::from("file.txt"),
            is_dir: false,
            size: Some(1),
            modified: None,
            permissions: None,
            uid: None,
            gid: None,
            owner: None,
            group: None,
        }];
        let should_quit = run(&mut app);

        assert!(!should_quit);
        assert_eq!(app.status_message, "editor: $EDITOR is not set");
    }

    #[test]
    fn build_editor_command_uses_shell_snippet_and_target_path_env() {
        let prev_shell = std::env::var_os("SHELL");
        std::env::set_var("SHELL", "/bin/test-shell");

        let target = PathBuf::from("/tmp/sample.txt");
        let command = build_editor_command(&target);

        assert_eq!(command.get_program(), "/bin/test-shell");
        assert_eq!(
            command
                .get_args()
                .map(|arg| arg.to_os_string())
                .collect::<Vec<OsString>>(),
            vec![OsString::from("-lc"), OsString::from(EDITOR_SHELL_SNIPPET)]
        );

        let target_env = command
            .get_envs()
            .find(|(key, _)| *key == "TARGET_PATH")
            .and_then(|(_, value)| value.map(|v| v.to_os_string()));
        assert_eq!(target_env, Some(OsString::from("/tmp/sample.txt")));

        if let Some(shell) = prev_shell {
            std::env::set_var("SHELL", shell);
        } else {
            std::env::remove_var("SHELL");
        }
    }
}
