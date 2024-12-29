use avian2d::prelude::*;
use bevy::prelude::*;

use crate::game::Game;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Game::Playing), init);
}

fn init() {}
