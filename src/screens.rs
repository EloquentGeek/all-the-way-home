pub mod ingame;
mod loading;
mod splash;
mod title;

use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.init_state::<Screen>();
    app.enable_state_scoped_entities::<Screen>();

    app.add_plugins((
        loading::plugin,
        ingame::plugin,
        splash::plugin,
        title::plugin,
    ));
}

#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default)]
pub enum Screen {
    Loading,
    // Over,
    InGame,
    #[default]
    Splash,
    Title,
}
