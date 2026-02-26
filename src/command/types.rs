#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandId {
    Quit,
    Cd,
    Mkdir,
    Delete,
    Rename,
    Help,
    Create,
    Command,
    Shell,
    Filter,
    Editor,
    Status,
    Parent,
    SelectUp,
    SelectDown,
    Yank,
    Cp,
}

pub struct CommandSpec {
    pub id: CommandId,
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub description: &'static str,
    pub requires_args: bool,
}

pub struct KeyBinding {
    pub command_id: CommandId,
    pub keys: &'static str,
}

pub struct HelpItem {
    pub command_id: CommandId,
    pub keys_display: Option<&'static str>,
    pub command_name: &'static str,
    pub description: &'static str,
    pub requires_args: bool,
}
