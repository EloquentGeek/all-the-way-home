use bevy::prelude::*;

use crate::screens::Screen;

#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
#[source(Screen = Screen::InGame)]
pub enum Game {
    /// Player has successfully completed the level.
    Complete,
    /// Player has failed the level, menu shows with offer to retry or quit.
    Failed,
    /// Takes care of loading resources for the level to come, and providing hints to the player.
    #[default]
    Intro,
    /// Player has hit the pause key, pause menu shows.
    Paused,
    /// Active gameplay.
    Playing,
}

pub fn plugin(app: &mut App) {
    app.init_state::<Game>();
    app.enable_state_scoped_entities::<Game>();

    app.add_systems(OnEnter(Game::Intro), init);
}

fn init(mut next_state: ResMut<NextState<Game>>) {
    // TODO: later, we can use this state for intro screens and/or resource loading
    next_state.set(Game::Playing);
}
