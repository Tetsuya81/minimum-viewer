use std::collections::{BTreeMap, HashMap};
#[cfg(unix)]
use std::ffi::CStr;
use std::fs;
#[cfg(unix)]
use std::os::raw::{c_char, c_uint};
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

use crate::command;
use crate::command::types::CommandId;
use crate::config;

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
    pub modified: Option<SystemTime>,
    pub permissions: Option<String>,
    pub uid: Option<u32>,
    pub gid: Option<u32>,
    pub owner: Option<String>,
    pub group: Option<String>,
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
    pub command_selected: Option<usize>,
    pub shell_input: String,
    pub shell_last_output: Option<ShellResult>,
    pub show_shell_popup: bool,
    pub help_popup_body: Option<String>,
    pub show_help_popup: bool,
    pub needs_full_redraw: bool,
    pub status_bar_expanded: bool,
    pub status_message: String,
    pub cd_on_quit_enabled: bool,
    pub user_name_cache: HashMap<u32, String>,
    pub group_name_cache: HashMap<u32, String>,
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
        let (cd_on_quit_enabled, config_error_message) = match config::load_or_create() {
            Ok(cfg) => (cfg.cd_on_quit, None),
            Err(err) => (false, Some(format!("config: {}", err))),
        };
        let mut app = Self {
            mode: Mode::Browse,
            current_dir,
            all_entries: Vec::new(),
            entries: Vec::new(),
            selected_index: 0,
            filter_input: String::new(),
            command_input: String::new(),
            command_candidates: command::filter_candidates(""),
            command_selected: None,
            shell_input: String::new(),
            shell_last_output: None,
            show_shell_popup: false,
            help_popup_body: None,
            show_help_popup: false,
            needs_full_redraw: false,
            status_bar_expanded: false,
            status_message: config_error_message.unwrap_or_default(),
            cd_on_quit_enabled,
            user_name_cache: HashMap::new(),
            group_name_cache: HashMap::new(),
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

    pub fn ensure_selected_owner_group_resolved(&mut self) {
        let idx = self.selected_index;
        let Some(selected) = self.entries.get(idx) else {
            return;
        };
        if selected.name == ".." {
            return;
        }

        #[cfg(unix)]
        {
            let (uid, gid, owner_missing, group_missing) = {
                let entry = &self.entries[idx];
                (entry.uid, entry.gid, entry.owner.is_none(), entry.group.is_none())
            };

            let owner = if owner_missing {
                uid.map(|u| self.resolve_user_name_cached(u))
            } else {
                None
            };
            let group = if group_missing {
                gid.map(|g| self.resolve_group_name_cached(g))
            } else {
                None
            };

            if owner.is_none() && group.is_none() {
                return;
            }

            if let Some(entry) = self.entries.get_mut(idx) {
                if let Some(owner) = owner {
                    entry.owner = Some(owner);
                }
                if let Some(group) = group {
                    entry.group = Some(group);
                }
            }
        }
    }

    pub fn move_selection_up(&mut self) {
        if self.mode == Mode::Command {
            if let Some(selected) = self.command_selected {
                if selected > 0 {
                    self.command_selected = Some(selected - 1);
                }
            }
            return;
        }
        if self.entries.is_empty() {
            return;
        }
        if self.selected_index == 0 {
            self.selected_index = self.entries.len() - 1;
        } else {
            self.selected_index -= 1;
        }
    }

    pub fn move_selection_down(&mut self) {
        if self.mode == Mode::Command {
            if let Some(selected) = self.command_selected {
                if selected + 1 < self.command_candidates.len() {
                    self.command_selected = Some(selected + 1);
                }
            }
            return;
        }
        if self.entries.is_empty() {
            return;
        }
        if self.selected_index + 1 < self.entries.len() {
            self.selected_index += 1;
        } else {
            self.selected_index = 0;
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
        self.command_selected = None;
    }

    pub fn exit_command_mode(&mut self) {
        self.mode = Mode::Browse;
    }

    pub fn command_push_char(&mut self, c: char) {
        self.command_input.push(c);
        self.filter_command_candidates();
        self.command_selected = None;
    }

    pub fn command_pop_char(&mut self) {
        self.command_input.pop();
        self.filter_command_candidates();
        self.command_selected = None;
    }

    pub fn command_select_next(&mut self) {
        if self.command_candidates.is_empty() {
            return;
        }
        self.command_selected = Some(match self.command_selected {
            None => 0,
            Some(selected) => (selected + 1) % self.command_candidates.len(),
        });
        self.sync_command_input_to_selected();
    }

    pub fn command_select_prev(&mut self) {
        if self.command_candidates.is_empty() {
            return;
        }
        self.command_selected = Some(match self.command_selected {
            None => self.command_candidates.len() - 1,
            Some(0) => self.command_candidates.len() - 1,
            Some(selected) => selected - 1,
        });
        self.sync_command_input_to_selected();
    }

    fn sync_command_input_to_selected(&mut self) {
        if let Some(selected_idx) = self.command_selected {
            if let Some(selected) = self.command_candidates.get(selected_idx) {
                self.command_input = selected.clone();
            }
        }
    }

    fn parse_command_input(&self) -> (String, Vec<String>) {
        let mut parts = self
            .command_input
            .split_whitespace()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        if parts.is_empty() {
            return (String::new(), Vec::new());
        }
        let command_name = parts.remove(0);
        (command_name, parts)
    }

    fn filter_command_candidates(&mut self) {
        let command_token = self.command_input.split_whitespace().next().unwrap_or("");
        self.command_candidates = command::filter_candidates(command_token);
    }

    pub fn execute_selected_command(&mut self) -> bool {
        let input_cmd = self.command_input.trim().to_string();
        let (command_name, args) = self.parse_command_input();
        let cand_len = self.command_candidates.len();
        let sel = self.command_selected
            .map(|idx| idx.to_string())
            .unwrap_or_else(|| "none".to_string());
        let cmd_opt = self.command_selected
            .and_then(|idx| self.command_candidates.get(idx))
            .map(|s| s.as_str())
            .unwrap_or("(none)");
        crate::debug_log::log(
            "app.rs:execute_selected_command",
            "entry",
            BTreeMap::from([
                ("input", input_cmd.clone()),
                ("candidates_len", cand_len.to_string()),
                ("command_selected", sel),
                ("cmd", cmd_opt.to_string()),
            ]),
            "H3",
        );

        if command_name.is_empty() && self.command_selected.is_none() {
            self.exit_command_mode();
            return false;
        }

        let cmd = command::resolve_command(
            &command_name,
            self.command_selected,
            &self.command_candidates,
        );
        self.exit_command_mode();
        match cmd {
            Some(CommandId::Quit) => {
                if !args.is_empty() {
                    self.status_message = "quit: unexpected arguments".to_string();
                    return false;
                }
                return command::quit::run(self);
            }
            Some(CommandId::Cd) => {
                if !args.is_empty() {
                    self.status_message = "cd: unexpected arguments".to_string();
                    return false;
                }
                return command::cd::run(self);
            }
            Some(CommandId::Mkdir) => return command::mkdir::run(self, &args),
            Some(CommandId::Delete) => {
                if !args.is_empty() {
                    self.status_message = "delete: unexpected arguments".to_string();
                    return false;
                }
                return command::delete::run(self);
            }
            Some(CommandId::Rename) => {
                if !args.is_empty() {
                    self.status_message = "rename: unexpected arguments".to_string();
                    return false;
                }
                return command::rename::run(self);
            }
            Some(CommandId::Help) => {
                if !args.is_empty() {
                    self.status_message = "help: unexpected arguments".to_string();
                    return false;
                }
                return command::help::run(self);
            }
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

    pub fn open_help_popup(&mut self, body: String) {
        self.help_popup_body = Some(body);
        self.show_help_popup = true;
    }

    pub fn close_help_popup(&mut self) {
        self.show_help_popup = false;
        self.help_popup_body = None;
    }

    pub fn close_active_popup(&mut self) {
        if self.show_shell_popup {
            self.close_shell_popup();
            return;
        }
        if self.show_help_popup {
            self.close_help_popup();
        }
    }

    pub fn request_full_redraw(&mut self) {
        self.needs_full_redraw = true;
    }

    pub fn consume_full_redraw_request(&mut self) -> bool {
        let requested = self.needs_full_redraw;
        self.needs_full_redraw = false;
        requested
    }

    pub fn toggle_status_bar_expanded(&mut self) {
        self.status_bar_expanded = !self.status_bar_expanded;
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
            let size = meta
                .as_ref()
                .and_then(|m| if m.is_file() { Some(m.len()) } else { None });
            let modified = meta.as_ref().and_then(|m| m.modified().ok());
            #[cfg(unix)]
            let permissions = meta.as_ref().map(format_permissions);
            #[cfg(not(unix))]
            let permissions = None;
            #[cfg(unix)]
            let (uid, gid) = meta
                .as_ref()
                .map(|m| (Some(m.uid()), Some(m.gid())))
                .unwrap_or((None, None));
            #[cfg(not(unix))]
            let (uid, gid) = (None, None);
            DirEntry {
                name,
                path,
                is_dir,
                size,
                modified,
                permissions,
                uid,
                gid,
                owner: None,
                group: None,
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
                modified: None,
                permissions: None,
                uid: None,
                gid: None,
                owner: None,
                group: None,
            },
        );
    }

    Ok(entries)
}

#[cfg(unix)]
fn format_permissions(meta: &fs::Metadata) -> String {
    let mode = meta.mode();
    let masks = [
        (0o400, 'r'),
        (0o200, 'w'),
        (0o100, 'x'),
        (0o040, 'r'),
        (0o020, 'w'),
        (0o010, 'x'),
        (0o004, 'r'),
        (0o002, 'w'),
        (0o001, 'x'),
    ];
    let mut out = String::with_capacity(9);
    for (mask, ch) in masks {
        if mode & mask != 0 {
            out.push(ch);
        } else {
            out.push('-');
        }
    }
    out
}

#[cfg(unix)]
fn lookup_user_name(uid: c_uint) -> Option<String> {
    unsafe {
        let ptr = getpwuid(uid);
        if ptr.is_null() || (*ptr).pw_name.is_null() {
            return None;
        }
        Some(CStr::from_ptr((*ptr).pw_name).to_string_lossy().into_owned())
    }
}

#[cfg(unix)]
fn lookup_group_name(gid: c_uint) -> Option<String> {
    unsafe {
        let ptr = getgrgid(gid);
        if ptr.is_null() || (*ptr).gr_name.is_null() {
            return None;
        }
        Some(CStr::from_ptr((*ptr).gr_name).to_string_lossy().into_owned())
    }
}

#[cfg(unix)]
impl App {
    fn resolve_user_name_cached(&mut self, uid: u32) -> String {
        if let Some(name) = self.user_name_cache.get(&uid) {
            return name.clone();
        }
        let resolved = lookup_user_name(uid as c_uint).unwrap_or_else(|| uid.to_string());
        self.user_name_cache.insert(uid, resolved.clone());
        resolved
    }

    fn resolve_group_name_cached(&mut self, gid: u32) -> String {
        if let Some(name) = self.group_name_cache.get(&gid) {
            return name.clone();
        }
        let resolved = lookup_group_name(gid as c_uint).unwrap_or_else(|| gid.to_string());
        self.group_name_cache.insert(gid, resolved.clone());
        resolved
    }
}

#[cfg(unix)]
#[repr(C)]
struct Passwd {
    pw_name: *mut c_char,
    pw_passwd: *mut c_char,
    pw_uid: c_uint,
    pw_gid: c_uint,
    pw_gecos: *mut c_char,
    pw_dir: *mut c_char,
    pw_shell: *mut c_char,
}

#[cfg(unix)]
#[repr(C)]
struct Group {
    gr_name: *mut c_char,
    gr_passwd: *mut c_char,
    gr_gid: c_uint,
    gr_mem: *mut *mut c_char,
}

#[cfg(unix)]
unsafe extern "C" {
    fn getpwuid(uid: c_uint) -> *mut Passwd;
    fn getgrgid(gid: c_uint) -> *mut Group;
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
            modified: None,
            permissions: None,
            uid: None,
            gid: None,
            owner: None,
            group: None,
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
            command_selected: None,
            shell_input: String::new(),
            shell_last_output: None,
            show_shell_popup: false,
            help_popup_body: None,
            show_help_popup: false,
            needs_full_redraw: false,
            status_bar_expanded: false,
            status_message: String::new(),
            cd_on_quit_enabled: false,
            user_name_cache: HashMap::new(),
            group_name_cache: HashMap::new(),
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
    fn browse_wraps_to_bottom_when_moving_up_from_top() {
        let mut app = test_app();
        app.entries = vec![mk_entry("..", true), mk_entry("src", true), mk_entry("README.md", false)];
        app.selected_index = 0;

        app.move_selection_up();

        assert_eq!(app.selected_index, 2);
    }

    #[test]
    fn browse_wraps_to_top_when_moving_down_from_bottom() {
        let mut app = test_app();
        app.entries = vec![mk_entry("..", true), mk_entry("src", true), mk_entry("README.md", false)];
        app.selected_index = 2;

        app.move_selection_down();

        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn selection_move_noop_when_entries_empty() {
        let mut app = test_app();
        app.selected_index = 0;

        app.move_selection_up();
        app.move_selection_down();

        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn selection_stays_on_single_entry_when_wrapping() {
        let mut app = test_app();
        app.entries = vec![mk_entry("..", true)];
        app.selected_index = 0;

        app.move_selection_up();
        assert_eq!(app.selected_index, 0);

        app.move_selection_down();
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn command_mode_selection_still_clamps() {
        let mut app = test_app();
        app.mode = Mode::Command;
        app.command_candidates = vec!["cd".to_string(), "quit".to_string(), "help".to_string()];
        app.command_selected = Some(0);

        app.move_selection_up();
        assert_eq!(app.command_selected, Some(0));

        app.command_selected = Some(2);
        app.move_selection_down();
        assert_eq!(app.command_selected, Some(2));
    }

    #[test]
    fn command_select_next_cycles_and_syncs_input() {
        let mut app = test_app();
        app.mode = Mode::Command;
        app.command_candidates = vec!["cd".to_string(), "mkdir".to_string()];
        app.command_selected = Some(0);

        app.command_select_next();
        assert_eq!(app.command_selected, Some(1));
        assert_eq!(app.command_input, "mkdir");

        app.command_select_next();
        assert_eq!(app.command_selected, Some(0));
        assert_eq!(app.command_input, "cd");
    }

    #[test]
    fn command_select_prev_cycles_and_syncs_input() {
        let mut app = test_app();
        app.mode = Mode::Command;
        app.command_candidates = vec!["cd".to_string(), "mkdir".to_string()];
        app.command_selected = Some(0);

        app.command_select_prev();
        assert_eq!(app.command_selected, Some(1));
        assert_eq!(app.command_input, "mkdir");
    }

    #[test]
    fn command_mode_starts_without_selected_candidate() {
        let mut app = test_app();
        app.command_input = "abc".to_string();
        app.command_selected = Some(1);

        app.enter_command_mode();

        assert_eq!(app.mode, Mode::Command);
        assert!(app.command_input.is_empty());
        assert_eq!(app.command_selected, None);
    }

    #[test]
    fn command_select_next_starts_from_first_when_unselected() {
        let mut app = test_app();
        app.mode = Mode::Command;
        app.command_candidates = vec!["cd".to_string(), "mkdir".to_string()];
        app.command_selected = None;

        app.command_select_next();

        assert_eq!(app.command_selected, Some(0));
        assert_eq!(app.command_input, "cd");
    }

    #[test]
    fn command_select_prev_starts_from_last_when_unselected() {
        let mut app = test_app();
        app.mode = Mode::Command;
        app.command_candidates = vec!["cd".to_string(), "mkdir".to_string()];
        app.command_selected = None;

        app.command_select_prev();

        assert_eq!(app.command_selected, Some(1));
        assert_eq!(app.command_input, "mkdir");
    }

    #[test]
    fn execute_selected_command_with_empty_input_and_no_selection_exits_command_mode() {
        let mut app = test_app();
        app.mode = Mode::Command;
        app.command_input.clear();
        app.command_selected = None;
        app.status_message = "old".to_string();

        let should_quit = app.execute_selected_command();

        assert!(!should_quit);
        assert_eq!(app.mode, Mode::Browse);
        assert_eq!(app.status_message, "old");
    }

    #[test]
    fn execute_selected_command_rejects_unexpected_args_for_quit() {
        let mut app = test_app();
        app.mode = Mode::Command;
        app.command_input = "quit now".to_string();
        app.command_candidates = vec!["quit".to_string()];
        app.command_selected = Some(0);

        let should_quit = app.execute_selected_command();

        assert!(!should_quit);
        assert_eq!(app.mode, Mode::Browse);
        assert_eq!(app.status_message, "quit: unexpected arguments");
    }

    #[test]
    fn execute_selected_command_rejects_unexpected_args_for_help() {
        let mut app = test_app();
        app.mode = Mode::Command;
        app.command_input = "help extra".to_string();
        app.command_candidates = vec!["help".to_string()];
        app.command_selected = Some(0);

        let should_quit = app.execute_selected_command();

        assert!(!should_quit);
        assert_eq!(app.mode, Mode::Browse);
        assert_eq!(app.status_message, "help: unexpected arguments");
        assert!(!app.show_help_popup);
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

    #[test]
    fn toggle_status_bar_expanded_switches_state() {
        let mut app = test_app();

        assert!(!app.status_bar_expanded);
        app.toggle_status_bar_expanded();
        assert!(app.status_bar_expanded);
        app.toggle_status_bar_expanded();
        assert!(!app.status_bar_expanded);
    }

    #[test]
    fn read_sorted_entries_adds_parent_with_empty_metadata() {
        let base =
            std::env::temp_dir().join(format!("minimum-viewer-parent-meta-{}", std::process::id()));
        let sub = base.join("sub");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&sub).expect("create temp dirs");

        let entries = read_sorted_entries(&sub).expect("read entries");

        assert!(!entries.is_empty());
        let parent = &entries[0];
        assert_eq!(parent.name, "..");
        assert!(parent.modified.is_none());
        assert!(parent.permissions.is_none());
        assert!(parent.uid.is_none());
        assert!(parent.gid.is_none());
        assert!(parent.owner.is_none());
        assert!(parent.group.is_none());

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn read_sorted_entries_does_not_resolve_owner_group_eagerly() {
        let base =
            std::env::temp_dir().join(format!("minimum-viewer-owner-group-{}", std::process::id()));
        let file = base.join("sample.txt");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).expect("create temp dir");
        std::fs::write(&file, "x").expect("write sample file");

        let entries = read_sorted_entries(&base).expect("read entries");
        let sample = entries
            .iter()
            .find(|entry| entry.name == "sample.txt")
            .expect("sample entry must exist");
        assert!(sample.owner.is_none());
        assert!(sample.group.is_none());

        #[cfg(unix)]
        {
            assert!(sample.uid.is_some());
            assert!(sample.gid.is_some());
        }
        #[cfg(not(unix))]
        {
            assert!(sample.uid.is_none());
            assert!(sample.gid.is_none());
        }

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn ensure_selected_owner_group_resolved_uses_cached_names() {
        let mut app = test_app();
        app.entries = vec![DirEntry {
            name: "sample.txt".to_string(),
            path: PathBuf::from("sample.txt"),
            is_dir: false,
            size: Some(1),
            modified: None,
            permissions: Some("rw-r--r--".to_string()),
            uid: Some(42),
            gid: Some(84),
            owner: None,
            group: None,
        }];
        app.user_name_cache.insert(42, "cached-user".to_string());
        app.group_name_cache.insert(84, "cached-group".to_string());

        app.ensure_selected_owner_group_resolved();

        let entry = app.selected_entry().expect("selection must exist");
        assert_eq!(entry.owner.as_deref(), Some("cached-user"));
        assert_eq!(entry.group.as_deref(), Some("cached-group"));
    }

    #[test]
    fn ensure_selected_owner_group_resolved_is_noop_for_parent_entry() {
        let mut app = test_app();
        app.entries = vec![DirEntry {
            name: "..".to_string(),
            path: PathBuf::from(".."),
            is_dir: true,
            size: None,
            modified: None,
            permissions: None,
            uid: Some(1),
            gid: Some(1),
            owner: None,
            group: None,
        }];
        app.user_name_cache.insert(1, "root".to_string());
        app.group_name_cache.insert(1, "wheel".to_string());

        app.ensure_selected_owner_group_resolved();

        let entry = app.selected_entry().expect("selection must exist");
        assert!(entry.owner.is_none());
        assert!(entry.group.is_none());
    }

    #[test]
    fn ensure_selected_owner_group_resolved_is_noop_when_ids_missing() {
        let mut app = test_app();
        app.entries = vec![mk_entry("sample.txt", false)];

        app.ensure_selected_owner_group_resolved();

        let entry = app.selected_entry().expect("selection must exist");
        assert!(entry.owner.is_none());
        assert!(entry.group.is_none());
    }

    #[test]
    fn ensure_selected_owner_group_resolved_uses_cache_for_multiple_entries() {
        let mut app = test_app();
        app.entries = vec![
            DirEntry {
                name: "a.txt".to_string(),
                path: PathBuf::from("a.txt"),
                is_dir: false,
                size: Some(1),
                modified: None,
                permissions: Some("rw-r--r--".to_string()),
                uid: Some(100),
                gid: Some(200),
                owner: None,
                group: None,
            },
            DirEntry {
                name: "b.txt".to_string(),
                path: PathBuf::from("b.txt"),
                is_dir: false,
                size: Some(1),
                modified: None,
                permissions: Some("rw-r--r--".to_string()),
                uid: Some(100),
                gid: Some(200),
                owner: None,
                group: None,
            },
        ];
        app.user_name_cache.insert(100, "same-user".to_string());
        app.group_name_cache.insert(200, "same-group".to_string());

        app.selected_index = 0;
        app.ensure_selected_owner_group_resolved();
        app.selected_index = 1;
        app.ensure_selected_owner_group_resolved();

        assert_eq!(app.entries[0].owner.as_deref(), Some("same-user"));
        assert_eq!(app.entries[0].group.as_deref(), Some("same-group"));
        assert_eq!(app.entries[1].owner.as_deref(), Some("same-user"));
        assert_eq!(app.entries[1].group.as_deref(), Some("same-group"));
    }
}
