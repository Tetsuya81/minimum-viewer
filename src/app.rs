use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::command;
use crate::command::types::CommandId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Browse,
    Filter,
    Command,
    Shell,
}

#[derive(Clone)]
pub struct DirEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: Option<u64>,
}

pub struct App {
    pub mode: Mode,
    pub current_dir: PathBuf,
    pub all_entries: Vec<DirEntry>,
    pub entries: Vec<DirEntry>,
    pub selected_index: usize,
    pub filter_input: String,
    pub command_input: String,
    pub command_candidates: Vec<String>,
    pub command_selected: usize,
    pub shell_input: String,
    pub shell_last_output: Option<ShellResult>,
    pub show_shell_popup: bool,
    pub status_message: String,
}

pub struct ShellResult {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub ran_shell: String,
}

impl App {
    pub fn new() -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut app = Self {
            mode: Mode::Browse,
            current_dir,
            all_entries: Vec::new(),
            entries: Vec::new(),
            selected_index: 0,
            filter_input: String::new(),
            command_input: String::new(),
            command_candidates: command::filter_candidates(""),
            command_selected: 0,
            shell_input: String::new(),
            shell_last_output: None,
            show_shell_popup: false,
            status_message: String::new(),
        };
        app.reload_entries();
        app.sync_cwd_env();
        app
    }

    pub fn reload_entries(&mut self) {
        let dir_str = self.current_dir.to_string_lossy().to_string();
        crate::debug_log::log(
            "app.rs:reload_entries",
            "entry",
            BTreeMap::from([("dir", dir_str)]),
            "H1",
        );

        self.all_entries = match read_sorted_entries(&self.current_dir) {
            Ok(entries) => entries,
            Err(e) => {
                crate::debug_log::log(
                    "app.rs:reload_entries",
                    "read_sorted_entries failed",
                    BTreeMap::from([("err", e.to_string())]),
                    "H1",
                );
                self.status_message = format!("Error: {}", e);
                Vec::new()
            }
        };

        self.apply_entry_filter();

        crate::debug_log::log(
            "app.rs:reload_entries",
            "after filter",
            BTreeMap::from([
                ("all_entries_len", self.all_entries.len().to_string()),
                ("entries_len", self.entries.len().to_string()),
                ("selected_index", self.selected_index.to_string()),
            ]),
            "H2",
        );
    }

    pub fn apply_entry_filter(&mut self) {
        let normalized = self.filter_input.trim().to_lowercase();
        let show_all = normalized.is_empty();

        self.entries = self
            .all_entries
            .iter()
            .filter(|entry| {
                if entry.name == ".." {
                    return true;
                }
                if show_all {
                    return true;
                }
                entry.name.to_lowercase().contains(&normalized)
            })
            .cloned()
            .collect();

        self.clamp_selected_index();
    }

    fn clamp_selected_index(&mut self) {
        if self.selected_index >= self.entries.len() && !self.entries.is_empty() {
            self.selected_index = self.entries.len() - 1;
        } else if self.entries.is_empty() {
            self.selected_index = 0;
        }
    }

    pub fn sync_cwd_env(&self) {
        std::env::set_var(
            "MINIMUM_VIEWER_CWD",
            self.current_dir.to_string_lossy().as_ref(),
        );
    }

    pub fn selected_entry(&self) -> Option<&DirEntry> {
        self.entries.get(self.selected_index)
    }

    pub fn move_selection_up(&mut self) {
        if self.mode == Mode::Command {
            if self.command_selected > 0 {
                self.command_selected -= 1;
            }
            return;
        }
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_selection_down(&mut self) {
        if self.mode == Mode::Command {
            if self.command_selected + 1 < self.command_candidates.len() {
                self.command_selected += 1;
            }
            return;
        }
        if self.selected_index + 1 < self.entries.len() {
            self.selected_index += 1;
        }
    }

    pub fn enter_filter_mode(&mut self) {
        self.mode = Mode::Filter;
    }

    pub fn exit_filter_mode(&mut self, clear: bool) {
        if clear {
            self.clear_filter();
        }
        self.mode = Mode::Browse;
    }

    pub fn filter_push_char(&mut self, c: char) {
        self.filter_input.push(c);
        self.apply_entry_filter();
    }

    pub fn filter_pop_char(&mut self) {
        self.filter_input.pop();
        self.apply_entry_filter();
    }

    pub fn clear_filter(&mut self) {
        self.filter_input.clear();
        self.apply_entry_filter();
    }

    pub fn on_directory_changed(&mut self, new_dir: PathBuf) {
        self.current_dir = new_dir;
        self.mode = Mode::Browse;
        self.clear_filter();
        self.reload_entries();
        self.selected_index = 0;
        self.sync_cwd_env();
    }

    pub fn move_to_parent_directory(&mut self) {
        self.mode = Mode::Browse;
        self.clear_filter();
        if let Some(parent) = self.current_dir.parent() {
            self.on_directory_changed(parent.to_path_buf());
            self.status_message = format!("cd: {}", self.current_dir.display());
        } else {
            self.status_message = "cd: parent directory not found".to_string();
        }
    }

    pub fn enter_command_mode(&mut self) {
        self.mode = Mode::Command;
        self.command_input.clear();
        self.filter_command_candidates();
        self.command_selected = 0;
    }

    pub fn exit_command_mode(&mut self) {
        self.mode = Mode::Browse;
    }

    pub fn command_push_char(&mut self, c: char) {
        self.command_input.push(c);
        self.filter_command_candidates();
        self.command_selected = 0;
    }

    pub fn command_pop_char(&mut self) {
        self.command_input.pop();
        self.filter_command_candidates();
        if self.command_selected >= self.command_candidates.len()
            && !self.command_candidates.is_empty()
        {
            self.command_selected = self.command_candidates.len() - 1;
        } else {
            self.command_selected = 0;
        }
    }

    fn filter_command_candidates(&mut self) {
        self.command_candidates = command::filter_candidates(&self.command_input);
    }

    pub fn execute_selected_command(&mut self) -> bool {
        let input_cmd = self.command_input.trim().to_string();
        let cand_len = self.command_candidates.len();
        let sel = self.command_selected;
        let cmd_opt = self
            .command_candidates
            .get(self.command_selected)
            .map(|s| s.as_str())
            .unwrap_or("(none)");
        crate::debug_log::log(
            "app.rs:execute_selected_command",
            "entry",
            BTreeMap::from([
                ("input", input_cmd.clone()),
                ("candidates_len", cand_len.to_string()),
                ("command_selected", sel.to_string()),
                ("cmd", cmd_opt.to_string()),
            ]),
            "H3",
        );

        let cmd =
            command::resolve_command(&input_cmd, self.command_selected, &self.command_candidates);
        self.exit_command_mode();
        match cmd {
            Some(CommandId::Quit) => return command::quit::run(self),
            Some(CommandId::Cd) => return command::cd::run(self),
            Some(CommandId::Open) => return command::open::run(self),
            Some(CommandId::Mkdir) => return command::mkdir::run(self),
            Some(CommandId::Delete) => return command::delete::run(self),
            Some(CommandId::Rename) => return command::rename::run(self),
            Some(CommandId::Help) => return command::help::run(self),
            None => self.status_message = "no matching command".to_string(),
        }
        false
    }

    pub fn enter_shell_mode(&mut self) {
        self.mode = Mode::Shell;
        self.shell_input.clear();
    }

    pub fn exit_shell_mode(&mut self) {
        self.mode = Mode::Browse;
    }

    pub fn shell_push_char(&mut self, c: char) {
        self.shell_input.push(c);
    }

    pub fn shell_pop_char(&mut self) {
        self.shell_input.pop();
    }

    pub fn execute_shell_input(&mut self) {
        let input = self.shell_input.trim().to_string();
        self.exit_shell_mode();
        if input.is_empty() {
            self.status_message = "shell: empty command".to_string();
            return;
        }

        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        match Command::new(&shell).arg("-lc").arg(&input).output() {
            Ok(output) => {
                let exit_code = output.status.code();
                self.status_message = match exit_code {
                    Some(code) => format!("shell exited with code {}", code),
                    None => "shell terminated by signal".to_string(),
                };
                self.shell_last_output = Some(ShellResult {
                    exit_code,
                    stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                    ran_shell: shell,
                });
                self.show_shell_popup = true;
            }
            Err(err) => {
                self.status_message = format!("shell execution failed: {}", err);
                self.shell_last_output = Some(ShellResult {
                    exit_code: None,
                    stdout: String::new(),
                    stderr: err.to_string(),
                    ran_shell: shell,
                });
                self.show_shell_popup = true;
            }
        }
    }

    pub fn close_shell_popup(&mut self) {
        self.show_shell_popup = false;
        self.shell_last_output = None;
        self.shell_input.clear();
    }

    pub fn open_selected(&mut self) {
        if self.mode == Mode::Command {
            if self.execute_selected_command() {
                return;
            }
            return;
        }
        if let Some(ent) = self.selected_entry().cloned() {
            if ent.is_dir {
                crate::debug_log::log(
                    "app.rs:open_selected",
                    "open dir",
                    BTreeMap::from([("path", ent.path.to_string_lossy().to_string())]),
                    "H4",
                );
                self.on_directory_changed(ent.path);
            } else {
                self.status_message = format!("File: {} (open not implemented)", ent.name);
            }
        } else {
            crate::debug_log::log(
                "app.rs:open_selected",
                "no selected_entry",
                BTreeMap::from([
                    ("entries_len", self.entries.len().to_string()),
                    ("selected_index", self.selected_index.to_string()),
                ]),
                "H2",
            );
        }
    }
}

fn read_sorted_entries(dir: &Path) -> std::io::Result<Vec<DirEntry>> {
    let mut entries: Vec<DirEntry> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| {
            let path = e.path();
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?")
                .to_string();
            let meta = e.metadata().ok();
            let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);
            let size = meta.and_then(|m| if m.is_file() { Some(m.len()) } else { None });
            DirEntry {
                name,
                path,
                is_dir,
                size,
            }
        })
        .collect();

    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    if dir.parent().is_some() {
        let mut parent_path = dir.to_path_buf();
        parent_path.pop();
        entries.insert(
            0,
            DirEntry {
                name: "..".to_string(),
                path: parent_path,
                is_dir: true,
                size: None,
            },
        );
    }

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_entry(name: &str, is_dir: bool) -> DirEntry {
        DirEntry {
            name: name.to_string(),
            path: PathBuf::from(name),
            is_dir,
            size: if is_dir { None } else { Some(1) },
        }
    }

    fn test_app() -> App {
        App {
            mode: Mode::Browse,
            current_dir: PathBuf::from("."),
            all_entries: vec![],
            entries: vec![],
            selected_index: 0,
            filter_input: String::new(),
            command_input: String::new(),
            command_candidates: vec![],
            command_selected: 0,
            shell_input: String::new(),
            shell_last_output: None,
            show_shell_popup: false,
            status_message: String::new(),
        }
    }

    #[test]
    fn empty_filter_shows_all_entries() {
        let mut app = test_app();
        app.all_entries = vec![
            mk_entry("..", true),
            mk_entry("src", true),
            mk_entry("README.md", false),
        ];

        app.apply_entry_filter();

        assert_eq!(app.entries.len(), 3);
    }

    #[test]
    fn filter_matches_contains_case_insensitive() {
        let mut app = test_app();
        app.all_entries = vec![
            mk_entry("..", true),
            mk_entry("Cargo.toml", false),
            mk_entry("README.md", false),
            mk_entry("src", true),
        ];
        app.filter_input = "reAd".to_string();

        app.apply_entry_filter();

        let names: Vec<&str> = app.entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, vec!["..", "README.md"]);
    }

    #[test]
    fn filter_keeps_parent_directory_entry() {
        let mut app = test_app();
        app.all_entries = vec![mk_entry("..", true), mk_entry("src", true)];
        app.filter_input = "zzz".to_string();

        app.apply_entry_filter();

        assert_eq!(app.entries.len(), 1);
        assert_eq!(app.entries[0].name, "..");
    }

    #[test]
    fn filter_clamps_selection_index() {
        let mut app = test_app();
        app.all_entries = vec![
            mk_entry("..", true),
            mk_entry("src", true),
            mk_entry("README.md", false),
        ];
        app.selected_index = 2;
        app.filter_input = "src".to_string();

        app.apply_entry_filter();

        assert_eq!(app.entries.len(), 2);
        assert_eq!(app.selected_index, 1);
    }

    #[test]
    fn on_directory_changed_resets_filter_and_mode() {
        let base = std::env::temp_dir().join(format!("minimum-viewer-test-{}", std::process::id()));
        let sub = base.join("sub");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&sub).expect("create temp dirs");

        let mut app = test_app();
        app.mode = Mode::Filter;
        app.filter_input = "src".to_string();
        app.selected_index = 3;

        app.on_directory_changed(sub.clone());

        assert_eq!(app.mode, Mode::Browse);
        assert!(app.filter_input.is_empty());
        assert_eq!(app.selected_index, 0);
        assert_eq!(app.current_dir, sub);

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn move_to_parent_directory_moves_when_parent_exists() {
        let base =
            std::env::temp_dir().join(format!("minimum-viewer-parent-{}", std::process::id()));
        let sub = base.join("sub");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&sub).expect("create temp dirs");

        let mut app = test_app();
        app.current_dir = sub.clone();
        app.mode = Mode::Filter;
        app.filter_input = "tmp".to_string();

        app.move_to_parent_directory();

        assert_eq!(app.current_dir, base);
        assert_eq!(app.mode, Mode::Browse);
        assert!(app.filter_input.is_empty());
        assert!(app.status_message.starts_with("cd: "));

        let _ = std::fs::remove_dir_all(sub);
        let _ = std::fs::remove_dir_all(app.current_dir.clone());
    }

    #[test]
    fn move_to_parent_directory_sets_message_when_parent_missing() {
        let mut app = test_app();
        app.current_dir = PathBuf::from("/");
        app.mode = Mode::Filter;
        app.filter_input = "tmp".to_string();

        app.move_to_parent_directory();

        assert_eq!(app.current_dir, PathBuf::from("/"));
        assert_eq!(app.mode, Mode::Browse);
        assert!(app.filter_input.is_empty());
        assert_eq!(app.status_message, "cd: parent directory not found");
    }
}
