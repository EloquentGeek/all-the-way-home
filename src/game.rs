pub mod level;
pub mod movement;
pub mod rendering;
pub mod yup;

use bevy::prelude::*;

use crate::screens::Screen;

#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
#[source(Screen = Screen::InGame)]
pub enum Game {
    /// Player has successfully completed the level.
    Complete,
    /// Player has failed the level, menu shows with offer to retry or quit.
    Failed,
    /// Player has hit the pause key, pause menu shows.
    Paused,
    /// Active gameplay.
    #[default]
    Playing,
}

pub fn plugin(app: &mut App) {
    app.init_state::<Game>();
    app.enable_state_scoped_entities::<Game>();
    app.add_plugins((level::plugin, movement::plugin, yup::plugin));
}
