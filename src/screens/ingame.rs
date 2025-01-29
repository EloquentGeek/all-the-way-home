pub mod pause;

use crate::game::Game;
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_plugins(pause::plugin);

    // app.load_resource::<PlayingMusic>();
    // app.add_systems(OnEnter(Screen::Playing), play_gameplay_music);
    // app.add_systems(OnExit(Screen::Playing), stop_music);
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
