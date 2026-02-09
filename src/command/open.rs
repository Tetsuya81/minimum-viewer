use crate::app::App;

pub fn run(app: &mut App) -> bool {
    app.open_selected();
    false
}
