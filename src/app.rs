use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::command;
use crate::command::types::CommandId;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Browse,
    Command,
    Shell,
}

pub struct DirEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: Option<u64>,
}

pub struct App {
    pub mode: Mode,
    pub current_dir: PathBuf,
    pub entries: Vec<DirEntry>,
    pub selected_index: usize,
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
            entries: Vec::new(),
            selected_index: 0,
            command_input: String::new(),
            command_candidates: command::filter_candidates(""),
            command_selected: 0,
            shell_input: String::new(),
            shell_last_output: None,
            show_shell_popup: false,
            status_message: String::new(),
        };
        app.reload_entries();
        app
    }

    pub fn reload_entries(&mut self) {
        // #region agent log
        let dir_str = self.current_dir.to_string_lossy().to_string();
        crate::debug_log::log(
            "app.rs:reload_entries",
            "entry",
            BTreeMap::from([("dir", dir_str)]),
            "H1",
        );
        // #endregion
        self.entries = match read_sorted_entries(&self.current_dir) {
            Ok(entries) => entries,
            Err(e) => {
                // #region agent log
                crate::debug_log::log(
                    "app.rs:reload_entries",
                    "read_sorted_entries failed",
                    BTreeMap::from([("err", e.to_string())]),
                    "H1",
                );
                // #endregion
                self.status_message = format!("Error: {}", e);
                Vec::new()
            }
        };
        if self.selected_index >= self.entries.len() && !self.entries.is_empty() {
            self.selected_index = self.entries.len() - 1;
        } else if self.entries.is_empty() {
            self.selected_index = 0;
        }
        // #region agent log
        crate::debug_log::log(
            "app.rs:reload_entries",
            "after clamp",
            BTreeMap::from([
                ("entries_len", self.entries.len().to_string()),
                ("selected_index", self.selected_index.to_string()),
            ]),
            "H2",
        );
        // #endregion
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
        // #region agent log
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
        // #endregion
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
        if let Some(ent) = self.selected_entry() {
            if ent.is_dir {
                // #region agent log
                crate::debug_log::log(
                    "app.rs:open_selected",
                    "open dir",
                    BTreeMap::from([("path", ent.path.to_string_lossy().to_string())]),
                    "H4",
                );
                // #endregion
                self.current_dir = ent.path.clone();
                self.reload_entries();
                self.selected_index = 0;
            } else {
                self.status_message = format!("File: {} (open not implemented)", ent.name);
            }
        } else {
            // #region agent log
            crate::debug_log::log(
                "app.rs:open_selected",
                "no selected_entry",
                BTreeMap::from([
                    ("entries_len", self.entries.len().to_string()),
                    ("selected_index", self.selected_index.to_string()),
                ]),
                "H2",
            );
            // #endregion
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
