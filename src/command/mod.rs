pub mod cd;
pub mod cp;
pub mod delete;
pub mod editor;
pub mod help;
pub mod mkdir;
pub mod path;
pub mod quit;
pub mod rename;
pub mod types;
pub mod yank;

use types::{CommandId, CommandSpec, HelpItem, KeyBinding};

pub const COMMAND_SPECS: &[CommandSpec] = &[
    CommandSpec {
        id: CommandId::Quit,
        name: "quit",
        aliases: &["q"],
        description: "Quit minimum-viewer.",
        requires_args: false,
    },
    CommandSpec {
        id: CommandId::Cd,
        name: "cd",
        aliases: &[],
        description: "Change directory: cd [path].",
        requires_args: true,
    },
    CommandSpec {
        id: CommandId::Mkdir,
        name: "mkdir",
        aliases: &[],
        description: "Create directory: mkdir <name>.",
        requires_args: true,
    },
    CommandSpec {
        id: CommandId::Delete,
        name: "delete",
        aliases: &[],
        description: "Delete file/dir: delete [path].",
        requires_args: false,
    },
    CommandSpec {
        id: CommandId::Rename,
        name: "rename",
        aliases: &[],
        description: "Rename selected entry: rename <new_name>.",
        requires_args: true,
    },
    CommandSpec {
        id: CommandId::Help,
        name: "help",
        aliases: &["?"],
        description: "Show command help.",
        requires_args: false,
    },
    CommandSpec {
        id: CommandId::Create,
        name: "create",
        aliases: &[],
        description: "Create new file/directory.",
        requires_args: false,
    },
    CommandSpec {
        id: CommandId::Command,
        name: "command",
        aliases: &[],
        description: "Enter command mode.",
        requires_args: false,
    },
    CommandSpec {
        id: CommandId::Shell,
        name: "shell",
        aliases: &[],
        description: "Enter shell mode.",
        requires_args: false,
    },
    CommandSpec {
        id: CommandId::Filter,
        name: "filter",
        aliases: &[],
        description: "Enter filter mode.",
        requires_args: false,
    },
    CommandSpec {
        id: CommandId::Editor,
        name: "editor",
        aliases: &[],
        description: "Open file in $EDITOR.",
        requires_args: false,
    },
    CommandSpec {
        id: CommandId::Status,
        name: "status",
        aliases: &[],
        description: "Toggle status bar.",
        requires_args: false,
    },
    CommandSpec {
        id: CommandId::Parent,
        name: "parent",
        aliases: &[],
        description: "Go to parent directory.",
        requires_args: false,
    },
    CommandSpec {
        id: CommandId::SelectUp,
        name: "up",
        aliases: &[],
        description: "Move cursor up.",
        requires_args: false,
    },
    CommandSpec {
        id: CommandId::SelectDown,
        name: "down",
        aliases: &[],
        description: "Move cursor down.",
        requires_args: false,
    },
    CommandSpec {
        id: CommandId::Yank,
        name: "yank",
        aliases: &["y"],
        description: "Copy selected entry path to clipboard: yank [path].",
        requires_args: false,
    },
    CommandSpec {
        id: CommandId::Cp,
        name: "cp",
        aliases: &[],
        description: "Copy file/dir: cp [src] [dest].",
        requires_args: false,
    },
];

/// Keybindings for Browse mode. Order defines Help screen display order.
pub const BROWSE_KEYBINDINGS: &[KeyBinding] = &[
    KeyBinding {
        command_id: CommandId::SelectUp,
        keys: "Up, k",
    },
    KeyBinding {
        command_id: CommandId::SelectDown,
        keys: "Down, j",
    },
    KeyBinding {
        command_id: CommandId::Parent,
        keys: "Backspace, Delete",
    },
    KeyBinding {
        command_id: CommandId::Filter,
        keys: "/",
    },
    KeyBinding {
        command_id: CommandId::Editor,
        keys: "e",
    },
    KeyBinding {
        command_id: CommandId::Status,
        keys: "m",
    },
    KeyBinding {
        command_id: CommandId::Create,
        keys: "n",
    },
    KeyBinding {
        command_id: CommandId::Shell,
        keys: "!",
    },
    KeyBinding {
        command_id: CommandId::Command,
        keys: ":",
    },
    KeyBinding {
        command_id: CommandId::Help,
        keys: "?",
    },
    KeyBinding {
        command_id: CommandId::Quit,
        keys: "q",
    },
    KeyBinding {
        command_id: CommandId::Delete,
        keys: "Ctrl + d",
    },
    KeyBinding {
        command_id: CommandId::Rename,
        keys: "Ctrl + r",
    },
    KeyBinding {
        command_id: CommandId::Yank,
        keys: "y",
    },
    KeyBinding {
        command_id: CommandId::Cp,
        keys: "Ctrl + c",
    },
];

const NAV_ONLY_COMMANDS: &[CommandId] = &[
    CommandId::SelectUp,
    CommandId::SelectDown,
    CommandId::Parent,
    CommandId::Create,
    CommandId::Command,
    CommandId::Shell,
    CommandId::Filter,
    CommandId::Editor,
    CommandId::Status,
];

fn find_spec(id: CommandId) -> Option<&'static CommandSpec> {
    COMMAND_SPECS.iter().find(|s| s.id == id)
}

pub fn help_items() -> Vec<HelpItem> {
    let mut items = Vec::new();
    let mut seen = Vec::new();

    for kb in BROWSE_KEYBINDINGS {
        if let Some(spec) = find_spec(kb.command_id) {
            items.push(HelpItem {
                command_id: kb.command_id,
                keys_display: Some(kb.keys),
                command_name: spec.name,
                description: spec.description,
                requires_args: spec.requires_args,
            });
            seen.push(kb.command_id);
        }
    }

    for spec in COMMAND_SPECS {
        if !seen.contains(&spec.id) {
            items.push(HelpItem {
                command_id: spec.id,
                keys_display: None,
                command_name: spec.name,
                description: spec.description,
                requires_args: spec.requires_args,
            });
        }
    }

    items
}

pub fn filter_candidates(input: &str) -> Vec<String> {
    let normalized = input.trim().to_lowercase();
    COMMAND_SPECS
        .iter()
        .filter(|spec| !NAV_ONLY_COMMANDS.contains(&spec.id))
        .filter(|spec| normalized.is_empty() || spec.name.starts_with(&normalized))
        .map(|spec| spec.name.to_string())
        .collect()
}

pub fn resolve_command(
    input: &str,
    selected: Option<usize>,
    candidates: &[String],
) -> Option<CommandId> {
    let normalized = input.trim().to_lowercase();
    if !normalized.is_empty() {
        if let Some(cmd_id) = find_by_name_or_alias(&normalized) {
            return Some(cmd_id);
        }
    }
    let candidate_name = candidates.get(selected?)?;
    find_by_name_or_alias(candidate_name)
}

fn find_by_name_or_alias(name: &str) -> Option<CommandId> {
    let normalized = name.trim().to_lowercase();
    COMMAND_SPECS.iter().find_map(|spec| {
        if spec.name == normalized || spec.aliases.iter().any(|alias| *alias == normalized) {
            Some(spec.id)
        } else {
            None
        }
    })
}

#[cfg(test)]
pub(crate) fn env_lock() -> &'static std::sync::Mutex<()> {
    use std::sync::{Mutex, OnceLock};
    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    ENV_LOCK.get_or_init(|| Mutex::new(()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_candidates_returns_command_mode_commands_for_empty_input() {
        let list = filter_candidates("");
        assert!(list.contains(&"quit".to_string()));
        assert!(list.contains(&"cd".to_string()));
        assert!(list.contains(&"mkdir".to_string()));
        assert!(list.contains(&"delete".to_string()));
        assert!(list.contains(&"rename".to_string()));
        assert!(list.contains(&"help".to_string()));
        // Navigation-only commands should be excluded
        assert!(!list.contains(&"up".to_string()));
        assert!(!list.contains(&"down".to_string()));
        assert!(!list.contains(&"parent".to_string()));
    }

    #[test]
    fn filter_candidates_matches_prefix_case_insensitive() {
        assert_eq!(filter_candidates("C"), vec!["cd", "cp"]);
        assert_eq!(filter_candidates("HE"), vec!["help"]);
    }

    #[test]
    fn resolve_command_prefers_exact_input() {
        assert_eq!(
            resolve_command("help", None, &["quit".to_string()]),
            Some(CommandId::Help)
        );
        assert_eq!(
            resolve_command("?", None, &["quit".to_string()]),
            Some(CommandId::Help)
        );
    }

    #[test]
    fn resolve_command_falls_back_to_selected_candidate() {
        let candidates = vec!["mkdir".to_string(), "quit".to_string()];
        assert_eq!(
            resolve_command("", Some(1), &candidates),
            Some(CommandId::Quit)
        );
        assert_eq!(
            resolve_command("zzz", Some(0), &candidates),
            Some(CommandId::Mkdir)
        );
    }

    #[test]
    fn resolve_command_returns_none_when_no_candidate_exists() {
        assert_eq!(resolve_command("zzz", Some(0), &[]), None);
    }

    #[test]
    fn resolve_command_returns_none_when_selected_is_none_and_input_not_exact() {
        let candidates = vec!["mkdir".to_string(), "quit".to_string()];
        assert_eq!(resolve_command("", None, &candidates), None);
        assert_eq!(resolve_command("zzz", None, &candidates), None);
    }

    #[test]
    fn help_items_includes_all_keybindings_first_then_command_only() {
        let items = help_items();
        // First items should be from BROWSE_KEYBINDINGS (have keys_display)
        assert!(items[0].keys_display.is_some());
        // cd and mkdir should appear at end without keys
        let cd_item = items.iter().find(|i| i.command_name == "cd").unwrap();
        assert!(cd_item.keys_display.is_none());
        let mkdir_item = items.iter().find(|i| i.command_name == "mkdir").unwrap();
        assert!(mkdir_item.keys_display.is_none());
    }

    #[test]
    fn help_items_has_no_duplicates() {
        let items = help_items();
        let mut seen = Vec::new();
        for item in &items {
            assert!(
                !seen.contains(&item.command_id),
                "duplicate: {:?}",
                item.command_id
            );
            seen.push(item.command_id);
        }
    }
}
