use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::screens::Screen;

pub fn plugin(app: &mut App) {
    app.add_loading_state(
        LoadingState::new(Screen::Loading)
            .continue_to_state(Screen::Title)
            .load_collection::<Characters>()
            .load_collection::<Levels>(),
    );
}

#[derive(AssetCollection, Resource)]
pub struct Levels {
    #[asset(path = "textures/level.png")]
    pub level: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct Characters {
    #[asset(path = "textures/yup.png")]
    pub yup: Handle<Image>,
}
