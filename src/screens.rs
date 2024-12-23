mod loading;
mod playing;
mod splash;
mod title;

use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.init_state::<Screen>();
    app.init_state::<Game>();
    app.enable_state_scoped_entities::<Screen>();
    app.enable_state_scoped_entities::<Game>();

    app.add_plugins((
        loading::plugin,
        playing::plugin,
        splash::plugin,
        title::plugin,
    ));
}

#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default)]
pub enum Screen {
    Loading,
    // Over,
    Playing,
    #[default]
    Splash,
    Title,
}

#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
#[source(Screen = Screen::Playing)]
pub enum Game {
    #[default]
    Playing,
    Paused,
}
