#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandId {
    Quit,
    Cd,
    Open,
    Mkdir,
    Delete,
    Rename,
    Help,
}

pub struct CommandSpec {
    pub id: CommandId,
    pub name: &'static str,
    pub aliases: &'static [&'static str],
}
