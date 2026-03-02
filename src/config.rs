use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

const CONFIG_RELATIVE_PATH: &str = "mmv/config.toml";
const STATE_LASTDIR_RELATIVE_PATH: &str = "mmv/lastdir";
const DEFAULT_CONFIG_CONTENT: &str = "cd_on_quit = false\nmarkdown_viewer = \"treemd\"\n";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub cd_on_quit: bool,
    pub markdown_viewer: String,
}

pub fn load_or_create() -> Result<Config, String> {
    load_or_create_from(|name| std::env::var_os(name))
}

fn load_or_create_from<F>(get_env: F) -> Result<Config, String>
where
    F: Fn(&str) -> Option<OsString>,
{
    let path = resolve_config_path_from(&get_env)?;
    ensure_exists_with_default(&path).map_err(|err| err.to_string())?;
    let content = fs::read_to_string(&path).map_err(|err| err.to_string())?;
    parse_config(&content)
}

pub fn resolve_lastdir_path() -> Result<PathBuf, String> {
    resolve_lastdir_path_from(|name| std::env::var_os(name))
}

fn resolve_config_path_from<F>(get_env: F) -> Result<PathBuf, String>
where
    F: Fn(&str) -> Option<OsString>,
{
    if let Some(explicit) = get_env("MINIMUM_VIEWER_CONFIG")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
    {
        return Ok(explicit);
    }

    if let Some(xdg_config_home) = get_env("XDG_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
    {
        return Ok(xdg_config_home.join(CONFIG_RELATIVE_PATH));
    }

    if let Some(home) = get_env("HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
    {
        return Ok(home.join(".config").join(CONFIG_RELATIVE_PATH));
    }

    Err("HOME is not set and no config override was provided".to_string())
}

fn resolve_lastdir_path_from<F>(get_env: F) -> Result<PathBuf, String>
where
    F: Fn(&str) -> Option<OsString>,
{
    if let Some(explicit) = get_env("MINIMUM_VIEWER_LAST_DIR")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
    {
        return Ok(explicit);
    }

    if let Some(xdg_state_home) = get_env("XDG_STATE_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
    {
        return Ok(xdg_state_home.join(STATE_LASTDIR_RELATIVE_PATH));
    }

    if let Some(home) = get_env("HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
    {
        return Ok(home.join(".local/state").join(STATE_LASTDIR_RELATIVE_PATH));
    }

    Err("HOME is not set and no lastdir override was provided".to_string())
}

fn ensure_exists_with_default(path: &Path) -> std::io::Result<()> {
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, DEFAULT_CONFIG_CONTENT)
}

fn parse_config(content: &str) -> Result<Config, String> {
    let mut cd_on_quit = false;
    let mut markdown_viewer = "treemd".to_string();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let Some((key, raw_value)) = trimmed.split_once('=') else {
            return Err(format!("invalid config line: {}", trimmed));
        };

        let value = raw_value.split('#').next().unwrap_or_default().trim();
        match key.trim() {
            "cd_on_quit" => match value {
                "true" => cd_on_quit = true,
                "false" => cd_on_quit = false,
                _ => return Err(format!("cd_on_quit must be true or false, got '{}'", value)),
            },
            "markdown_viewer" => {
                let unquoted = value.trim_matches('"');
                if !unquoted.is_empty() {
                    markdown_viewer = unquoted.to_string();
                }
            }
            _ => {}
        }
    }

    Ok(Config {
        cd_on_quit,
        markdown_viewer,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_path_prefers_minimum_viewer_config() {
        let resolved = resolve_config_path_from(|name| match name {
            "MINIMUM_VIEWER_CONFIG" => Some(OsString::from("/tmp/custom.toml")),
            "XDG_CONFIG_HOME" => Some(OsString::from("/tmp/xdg")),
            "HOME" => Some(OsString::from("/tmp/home")),
            _ => None,
        })
        .expect("path must resolve");

        assert_eq!(resolved, PathBuf::from("/tmp/custom.toml"));
    }

    #[test]
    fn resolve_path_uses_xdg_config_home_when_override_missing() {
        let resolved = resolve_config_path_from(|name| match name {
            "XDG_CONFIG_HOME" => Some(OsString::from("/tmp/xdg")),
            "HOME" => Some(OsString::from("/tmp/home")),
            _ => None,
        })
        .expect("path must resolve");

        assert_eq!(resolved, PathBuf::from("/tmp/xdg/mmv/config.toml"));
    }

    #[test]
    fn resolve_path_falls_back_to_home_config() {
        let resolved = resolve_config_path_from(|name| match name {
            "HOME" => Some(OsString::from("/tmp/home")),
            _ => None,
        })
        .expect("path must resolve");

        assert_eq!(resolved, PathBuf::from("/tmp/home/.config/mmv/config.toml"));
    }

    #[test]
    fn load_or_create_writes_default_config_when_missing() {
        let root = std::env::temp_dir().join(format!(
            "minimum-viewer-config-create-{}",
            std::process::id()
        ));
        let config_path = root.join("config.toml");
        let _ = fs::remove_dir_all(&root);

        let path = config_path.clone();
        let config = load_or_create_from(move |name| match name {
            "MINIMUM_VIEWER_CONFIG" => Some(path.as_os_str().to_owned()),
            _ => None,
        })
        .expect("load must succeed");

        assert!(!config.cd_on_quit);
        assert_eq!(
            fs::read_to_string(&config_path).expect("config content must exist"),
            DEFAULT_CONFIG_CONTENT
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn load_or_create_reads_true_value() {
        let root =
            std::env::temp_dir().join(format!("minimum-viewer-config-read-{}", std::process::id()));
        let config_path = root.join("config.toml");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create temp dir");
        fs::write(&config_path, "cd_on_quit = true\n").expect("write config");

        let path = config_path.clone();
        let config = load_or_create_from(move |name| match name {
            "MINIMUM_VIEWER_CONFIG" => Some(path.as_os_str().to_owned()),
            _ => None,
        })
        .expect("load must succeed");

        assert!(config.cd_on_quit);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn parse_config_rejects_invalid_boolean() {
        let err = parse_config("cd_on_quit = maybe\n").expect_err("must fail");
        assert!(err.contains("cd_on_quit must be true or false"));
    }

    #[test]
    fn resolve_lastdir_path_prefers_override() {
        let resolved = resolve_lastdir_path_from(|name| match name {
            "MINIMUM_VIEWER_LAST_DIR" => Some(OsString::from("/tmp/mmv-lastdir")),
            "XDG_STATE_HOME" => Some(OsString::from("/tmp/state")),
            "HOME" => Some(OsString::from("/tmp/home")),
            _ => None,
        })
        .expect("path must resolve");

        assert_eq!(resolved, PathBuf::from("/tmp/mmv-lastdir"));
    }

    #[test]
    fn resolve_lastdir_path_uses_xdg_state_home() {
        let resolved = resolve_lastdir_path_from(|name| match name {
            "XDG_STATE_HOME" => Some(OsString::from("/tmp/state")),
            "HOME" => Some(OsString::from("/tmp/home")),
            _ => None,
        })
        .expect("path must resolve");

        assert_eq!(resolved, PathBuf::from("/tmp/state/mmv/lastdir"));
    }

    #[test]
    fn resolve_lastdir_path_falls_back_to_home_state_dir() {
        let resolved = resolve_lastdir_path_from(|name| match name {
            "HOME" => Some(OsString::from("/tmp/home")),
            _ => None,
        })
        .expect("path must resolve");

        assert_eq!(
            resolved,
            PathBuf::from("/tmp/home/.local/state/mmv/lastdir")
        );
    }
}
