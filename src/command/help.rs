use crate::app::App;

pub fn run(app: &mut App) -> bool {
    let body = super::command_help_lines().join("\n");
    app.open_help_popup(body);
    false
}
