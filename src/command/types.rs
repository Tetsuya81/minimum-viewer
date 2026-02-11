#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandId {
    Quit,
    Cd,
    Mkdir,
    Delete,
    Rename,
    Help,
}

pub struct CommandSpec {
    pub id: CommandId,
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub description: &'static str,
}
