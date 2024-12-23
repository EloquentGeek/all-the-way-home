mod pause;

use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::screens::Screen;

use super::Game;

pub fn plugin(app: &mut App) {
    app.add_plugins(pause::plugin);
    // app.add_systems(OnEnter(Screen::Playing), spawn_level);

    // app.load_resource::<PlayingMusic>();
    // app.add_systems(OnEnter(Screen::Playing), play_gameplay_music);
    // app.add_systems(OnExit(Screen::Playing), stop_music);

    app.add_systems(
        Update,
        pause.run_if(in_state(Screen::Playing).and(input_just_pressed(KeyCode::Escape))),
    );
}

// fn spawn_level(mut commands: Commands) {
// commands.queue(spawn_level_command);
// }

// #[derive(Resource, Asset, Reflect, Clone)]
// pub struct PlayingMusic {
//     #[dependency]
//     handle: Handle<AudioSource>,
//     entity: Option<Entity>,
// }
//
// impl FromWorld for PlayingMusic {
//     fn from_world(world: &mut World) -> Self {
//         let assets = world.resource::<AssetServer>();
//         Self {
//             handle: assets.load("audio/music/Fluffing A Duck.ogg"),
//             entity: None,
//         }
//     }
// }
//
// fn play_gameplay_music(mut commands: Commands, mut music: ResMut<PlayingMusic>) {
//     music.entity = Some(
//         commands
//             .spawn((
//                 AudioPlayer(music.handle.clone()),
//                 PlaybackSettings::LOOP,
//                 Music,
//             ))
//             .id(),
//     );
// }
//
// fn stop_music(mut commands: Commands, mut music: ResMut<PlayingMusic>) {
//     if let Some(entity) = music.entity.take() {
//         commands.entity(entity).despawn_recursive();
//     }
// }

fn pause(mut game: ResMut<NextState<Game>>) {
    game.set(Game::Paused);
}
