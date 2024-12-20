use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::screens::Screen;

pub fn plugin(app: &mut App) {
    app.add_loading_state(LoadingState::new(Screen::Loading).continue_to_state(Screen::Title));
}

#[derive(AssetCollection, Resource)]
pub struct Textures {}
