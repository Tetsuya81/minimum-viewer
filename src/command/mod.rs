pub mod cd;
pub mod delete;
pub mod editor;
pub mod help;
pub mod mkdir;
pub mod open;
pub mod quit;
pub mod rename;
pub mod types;

use types::{CommandId, CommandSpec};

pub const COMMAND_SPECS: &[CommandSpec] = &[
    CommandSpec {
        id: CommandId::Quit,
        name: "quit",
        aliases: &["q"],
    },
    CommandSpec {
        id: CommandId::Cd,
        name: "cd",
        aliases: &[],
    },
    CommandSpec {
        id: CommandId::Open,
        name: "open",
        aliases: &[],
    },
    CommandSpec {
        id: CommandId::Editor,
        name: "editor",
        aliases: &["e"],
    },
    CommandSpec {
        id: CommandId::Mkdir,
        name: "mkdir",
        aliases: &[],
    },
    CommandSpec {
        id: CommandId::Delete,
        name: "delete",
        aliases: &[],
    },
    CommandSpec {
        id: CommandId::Rename,
        name: "rename",
        aliases: &[],
    },
    CommandSpec {
        id: CommandId::Help,
        name: "help",
        aliases: &["?"],
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

pub fn resolve_command(input: &str, selected: usize, candidates: &[String]) -> Option<CommandId> {
    let normalized = input.trim().to_lowercase();
    if !normalized.is_empty() {
        if let Some(cmd_id) = find_by_name_or_alias(&normalized) {
            return Some(cmd_id);
        }
    }
    let candidate_name = candidates.get(selected)?;
    find_by_name_or_alias(candidate_name)
}

pub fn command_names_csv() -> String {
    COMMAND_SPECS
        .iter()
        .map(|spec| spec.name)
        .collect::<Vec<_>>()
        .join(", ")
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
            vec![
                "quit", "cd", "open", "editor", "mkdir", "delete", "rename", "help"
            ]
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
            resolve_command("help", 0, &["quit".to_string()]),
            Some(CommandId::Help)
        );
        assert_eq!(
            resolve_command("?", 0, &["quit".to_string()]),
            Some(CommandId::Help)
        );
        assert_eq!(
            resolve_command("editor", 0, &["quit".to_string()]),
            Some(CommandId::Editor)
        );
        assert_eq!(
            resolve_command("e", 0, &["quit".to_string()]),
            Some(CommandId::Editor)
        );
    }

    #[test]
    fn resolve_command_falls_back_to_selected_candidate() {
        let candidates = vec!["open".to_string(), "quit".to_string()];
        assert_eq!(resolve_command("", 1, &candidates), Some(CommandId::Quit));
        assert_eq!(
            resolve_command("zzz", 0, &candidates),
            Some(CommandId::Open)
        );
    }

    #[test]
    fn resolve_command_returns_none_when_no_candidate_exists() {
        assert_eq!(resolve_command("zzz", 0, &[]), None);
    }
}
