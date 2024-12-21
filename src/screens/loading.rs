use bevy::prelude::*;

use crate::{screens::Screen, ui::Containers};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Loading), spawn_loading_screen);
}

fn spawn_loading_screen(mut commands: Commands) {
    commands
        .ui_root()
        .insert(StateScoped(Screen::Loading))
        .with_children(|p| {
            p.spawn(Text::new("Loading..."));
        });
}
