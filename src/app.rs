use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Browse,
    Command,
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
    pub status_message: String,
}

const COMMAND_LIST: &[&str] = &[
    "quit", "cd", "open", "mkdir", "delete", "rename", "help",
];

impl App {
    pub fn new() -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut app = Self {
            mode: Mode::Browse,
            current_dir,
            entries: Vec::new(),
            selected_index: 0,
            command_input: String::new(),
            command_candidates: COMMAND_LIST.iter().map(|s| s.to_string()).collect(),
            command_selected: 0,
            status_message: String::new(),
        };
        app.reload_entries();
        app
    }

    pub fn reload_entries(&mut self) {
        // #region agent log
        let dir_str = self.current_dir.to_string_lossy().to_string();
        crate::debug_log::log("app.rs:reload_entries", "entry", BTreeMap::from([("dir", dir_str)]), "H1");
        // #endregion
        self.entries = match read_sorted_entries(&self.current_dir) {
            Ok(entries) => entries,
            Err(e) => {
                // #region agent log
                crate::debug_log::log("app.rs:reload_entries", "read_sorted_entries failed", BTreeMap::from([("err", e.to_string())]), "H1");
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
        crate::debug_log::log("app.rs:reload_entries", "after clamp", BTreeMap::from([
            ("entries_len", self.entries.len().to_string()),
            ("selected_index", self.selected_index.to_string()),
        ]), "H2");
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
        if self.command_selected >= self.command_candidates.len() && !self.command_candidates.is_empty() {
            self.command_selected = self.command_candidates.len() - 1;
        } else {
            self.command_selected = 0;
        }
    }

    fn filter_command_candidates(&mut self) {
        let input = self.command_input.trim().to_lowercase();
        self.command_candidates = if input.is_empty() {
            COMMAND_LIST.iter().map(|s| s.to_string()).collect()
        } else {
            COMMAND_LIST
                .iter()
                .filter(|s| s.to_lowercase().starts_with(&input))
                .map(|s| s.to_string())
                .collect()
        };
    }

    pub fn execute_selected_command(&mut self) -> bool {
        let input_cmd = self.command_input.trim().to_lowercase();
        // #region agent log
        let cand_len = self.command_candidates.len();
        let sel = self.command_selected;
        let cmd_opt = self.command_candidates.get(self.command_selected).map(|s| s.as_str()).unwrap_or("(none)");
        crate::debug_log::log("app.rs:execute_selected_command", "entry", BTreeMap::from([
            ("input", input_cmd.clone()),
            ("candidates_len", cand_len.to_string()),
            ("command_selected", sel.to_string()),
            ("cmd", cmd_opt.to_string()),
        ]), "H3");
        // #endregion
        let cmd = if !input_cmd.is_empty() && COMMAND_LIST.contains(&input_cmd.as_str()) {
            Some(input_cmd)
        } else {
            self.command_candidates.get(self.command_selected).cloned()
        };
        self.exit_command_mode();
        match cmd.as_deref() {
            Some("quit") => return true,
            Some("open") => self.open_selected(),
            Some("cd") => {
                if let Some((path, name, is_dir)) = self
                    .selected_entry()
                    .map(|e| (e.path.clone(), e.name.clone(), e.is_dir))
                {
                    if is_dir {
                        // #region agent log
                        crate::debug_log::log("app.rs:execute_selected_command", "cd target", BTreeMap::from([("path", path.to_string_lossy().to_string())]), "H4");
                        // #endregion
                        self.current_dir = path;
                        self.reload_entries();
                        self.selected_index = 0;
                        self.status_message = format!("cd: {}", self.current_dir.display());
                    } else {
                        self.status_message = format!("cd: '{}' is not a directory", name);
                    }
                } else {
                    self.status_message = "cd: no selection".to_string();
                }
            }
            Some("mkdir") => self.status_message = "mkdir: not implemented".to_string(),
            Some("delete") => self.status_message = "delete: not implemented".to_string(),
            Some("rename") => self.status_message = "rename: not implemented".to_string(),
            Some("help") => {
                self.status_message =
                    "commands: quit, cd, open, mkdir, delete, rename, help".to_string();
            }
            Some(other) => self.status_message = format!("unknown command: {}", other),
            None => self.status_message = "no matching command".to_string(),
        }
        false
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
                crate::debug_log::log("app.rs:open_selected", "open dir", BTreeMap::from([("path", ent.path.to_string_lossy().to_string())]), "H4");
                // #endregion
                self.current_dir = ent.path.clone();
                self.reload_entries();
                self.selected_index = 0;
            } else {
                self.status_message = format!("File: {} (open not implemented)", ent.name);
            }
        } else {
            // #region agent log
            crate::debug_log::log("app.rs:open_selected", "no selected_entry", BTreeMap::from([
                ("entries_len", self.entries.len().to_string()),
                ("selected_index", self.selected_index.to_string()),
            ]), "H2");
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

    entries.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
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
