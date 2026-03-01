use std::io::{self, stdout};
use std::path::Path;
use std::process::Command;

use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;

use crate::app::App;

const MARKDOWN_SHELL_SNIPPET: &str = "${MARKDOWN_VIEWER} \"${TARGET_PATH}\"";

pub fn run(app: &mut App) -> bool {
    let Some(entry) = app.selected_entry() else {
        app.status_message = "markdown: no selection".to_string();
        return false;
    };
    if entry.is_dir {
        app.status_message = "Please select the file you want to display".to_string();
        return false;
    }
    let target_path = entry.path.clone();

    let viewer = &app.config.markdown_viewer;
    if viewer.trim().is_empty() {
        app.status_message = "markdown: viewer is not configured".to_string();
        return false;
    }

    match open_in_viewer(viewer, &target_path) {
        Ok(Some(code)) => {
            app.status_message = format!("markdown viewer exited with code {}", code);
        }
        Ok(None) => {
            app.status_message = "markdown viewer terminated by signal".to_string();
        }
        Err(err) => {
            app.status_message = format!("markdown viewer failed: {}", err);
        }
    }
    app.request_full_redraw();

    false
}

fn open_in_viewer(viewer: &str, target_path: &Path) -> io::Result<Option<i32>> {
    suspend_tui()?;
    let status_result = build_viewer_command(viewer, target_path).status();
    let restore_result = resume_tui();

    match (status_result, restore_result) {
        (Ok(status), Ok(())) => Ok(status.code()),
        (Err(exec_err), Ok(())) => Err(exec_err),
        (Ok(_), Err(restore_err)) => Err(restore_err),
        (Err(exec_err), Err(restore_err)) => Err(io::Error::new(
            restore_err.kind(),
            format!(
                "{}; also failed to run viewer: {}",
                restore_err, exec_err
            ),
        )),
    }
}

fn build_viewer_command(viewer: &str, target_path: &Path) -> Command {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    let mut command = Command::new(shell);
    command
        .arg("-lc")
        .arg(MARKDOWN_SHELL_SNIPPET)
        .env("MARKDOWN_VIEWER", viewer)
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
    use std::ffi::OsString;
    use std::path::PathBuf;

    #[test]
    fn build_viewer_command_uses_shell_snippet_and_envs() {
        let prev_shell = std::env::var_os("SHELL");
        std::env::set_var("SHELL", "/bin/test-shell");

        let target = PathBuf::from("/tmp/README.md");
        let command = build_viewer_command("treemd", &target);

        assert_eq!(command.get_program(), "/bin/test-shell");
        assert_eq!(
            command
                .get_args()
                .map(|arg| arg.to_os_string())
                .collect::<Vec<OsString>>(),
            vec![
                OsString::from("-lc"),
                OsString::from(MARKDOWN_SHELL_SNIPPET)
            ]
        );

        let viewer_env = command
            .get_envs()
            .find(|(key, _)| *key == "MARKDOWN_VIEWER")
            .and_then(|(_, value)| value.map(|v| v.to_os_string()));
        assert_eq!(viewer_env, Some(OsString::from("treemd")));

        let path_env = command
            .get_envs()
            .find(|(key, _)| *key == "TARGET_PATH")
            .and_then(|(_, value)| value.map(|v| v.to_os_string()));
        assert_eq!(path_env, Some(OsString::from("/tmp/README.md")));

        if let Some(shell) = prev_shell {
            std::env::set_var("SHELL", shell);
        } else {
            std::env::remove_var("SHELL");
        }
    }

    #[test]
    fn run_sets_error_when_no_selection() {
        let mut app = crate::app::App {
            mode: crate::app::Mode::Browse,
            current_dir: PathBuf::from("."),
            all_entries: vec![],
            entries: vec![],
            selected_index: 0,
            filter_input: String::new(),
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
            config: crate::config::Config {
                cd_on_quit: false,
                markdown_viewer: "treemd".to_string(),
            },
            user_name_cache: std::collections::HashMap::new(),
            group_name_cache: std::collections::HashMap::new(),
        };

        let should_quit = run(&mut app);
        assert!(!should_quit);
        assert_eq!(app.status_message, "markdown: no selection");
    }

    #[test]
    fn run_sets_error_when_viewer_is_empty() {
        let mut app = crate::app::App {
            mode: crate::app::Mode::Browse,
            current_dir: PathBuf::from("."),
            all_entries: vec![],
            entries: vec![
                crate::app::DirEntry {
                    name: "README.md".to_string(),
                    path: PathBuf::from("README.md"),
                    is_dir: false,
                    size: Some(1),
                    modified: None,
                    permissions: None,
                    uid: None,
                    gid: None,
                    owner: None,
                    group: None,
                    link_target: None,
                    is_dangling: false,
                },
            ],
            selected_index: 0,
            filter_input: String::new(),
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
            config: crate::config::Config {
                cd_on_quit: false,
                markdown_viewer: String::new(),
            },
            user_name_cache: std::collections::HashMap::new(),
            group_name_cache: std::collections::HashMap::new(),
        };

        let should_quit = run(&mut app);
        assert!(!should_quit);
        assert_eq!(app.status_message, "markdown: viewer is not configured");
    }

    #[test]
    fn run_sets_error_when_directory_selected() {
        let mut app = crate::app::App {
            mode: crate::app::Mode::Browse,
            current_dir: PathBuf::from("."),
            all_entries: vec![],
            entries: vec![crate::app::DirEntry {
                name: "src".to_string(),
                path: PathBuf::from("src"),
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
            filter_input: String::new(),
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
            config: crate::config::Config {
                cd_on_quit: false,
                markdown_viewer: "treemd".to_string(),
            },
            user_name_cache: std::collections::HashMap::new(),
            group_name_cache: std::collections::HashMap::new(),
        };

        let should_quit = run(&mut app);
        assert!(!should_quit);
        assert_eq!(app.status_message, "Please select the file you want to display");
    }
}
