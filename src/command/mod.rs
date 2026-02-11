pub mod cd;
pub mod delete;
pub mod editor;
pub mod help;
pub mod mkdir;
pub mod path;
pub mod quit;
pub mod rename;
pub mod types;

use types::{CommandId, CommandSpec};

pub const COMMAND_SPECS: &[CommandSpec] = &[
    CommandSpec {
        id: CommandId::Quit,
        name: "quit",
        aliases: &["q"],
        description: "Quit minimum-viewer.",
    },
    CommandSpec {
        id: CommandId::Cd,
        name: "cd",
        aliases: &[],
        description: "Change directory: cd [path].",
    },
    CommandSpec {
        id: CommandId::Mkdir,
        name: "mkdir",
        aliases: &[],
        description: "Create directory: mkdir <name>.",
    },
    CommandSpec {
        id: CommandId::Delete,
        name: "delete",
        aliases: &[],
        description: "Delete file/dir: delete [path].",
    },
    CommandSpec {
        id: CommandId::Rename,
        name: "rename",
        aliases: &[],
        description: "Rename selected entry: rename <new_name>.",
    },
    CommandSpec {
        id: CommandId::Help,
        name: "help",
        aliases: &["?"],
        description: "Show command help.",
    },
];

pub fn filter_candidates(input: &str) -> Vec<String> {
    let normalized = input.trim().to_lowercase();
    COMMAND_SPECS
        .iter()
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

pub fn command_help_lines() -> Vec<String> {
    COMMAND_SPECS
        .iter()
        .map(|spec| {
            let aliases = if spec.aliases.is_empty() {
                String::new()
            } else {
                format!(" [{}]", spec.aliases.join(", "))
            };
            format!("{}{} - {}", spec.name, aliases, spec.description)
        })
        .collect()
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
mod tests {
    use super::*;

    #[test]
    fn filter_candidates_returns_all_for_empty_input() {
        let list = filter_candidates("");
        assert_eq!(
            list,
            vec!["quit", "cd", "mkdir", "delete", "rename", "help"]
        );
    }

    #[test]
    fn filter_candidates_matches_prefix_case_insensitive() {
        assert_eq!(filter_candidates("C"), vec!["cd"]);
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
    fn command_help_lines_includes_descriptions_and_aliases() {
        let lines = command_help_lines();
        assert!(lines.iter().any(|line| line.contains("quit [q] -")));
        assert!(lines.iter().any(|line| line.contains("help [?] -")));
        assert!(lines.iter().any(|line| line.contains("mkdir -")));
    }
}
