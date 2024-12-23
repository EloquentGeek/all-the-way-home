use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::screens::Screen;

use super::Game;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Game::Paused), init);

    // app.load_resource::<PlayingMusic>();
    // app.add_systems(OnEnter(Screen::Playing), play_gameplay_music);
    // app.add_systems(OnExit(Screen::Playing), stop_music);

    app.add_systems(
        Update,
        pause.run_if(in_state(Game::Paused).and(input_just_pressed(KeyCode::Escape))),
    );
}

fn init(mut commands: Commands) {
    commands
        .spawn((StateScoped(Game::Paused), Name::new("Pause Menu"), Node {
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            height: Val::Percent(100.),
            justify_content: JustifyContent::Start,
            justify_self: JustifySelf::Center,
            padding: UiRect::all(Val::Px(10.)),
            width: Val::Percent(100.),
            ..default()
        }))
        .with_children(|p| {
            p.spawn((Text::new("Game Paused"), TextFont {
                font_size: 30.,
                ..default()
            }));

            p.spawn((Name::new("Exit Game"), Button, Node {
                align_items: AlignItems::Center,
                height: Val::Px(65.0),
                justify_content: JustifyContent::Center,
                width: Val::Px(200.0),
                ..default()
            }))
            .with_children(|p| {
                p.spawn((Name::new("Button Text"), Text::new("exit game")));
            })
            .observe(
                |_ev: Trigger<Pointer<Click>>,
                 mut game: ResMut<NextState<Game>>,
                 mut screen: ResMut<NextState<Screen>>| {
                    game.set(Game::Playing);
                    screen.set(Screen::Title);
                },
            );

            p.spawn((Name::new("Return To Game"), Button, Node {
                align_items: AlignItems::Center,
                height: Val::Px(65.0),
                justify_content: JustifyContent::Center,
                width: Val::Px(200.0),
                ..default()
            }))
            .with_children(|p| {
                p.spawn((Name::new("Button Text"), Text::new("return to game")));
            })
            .observe(
                |_ev: Trigger<Pointer<Click>>, mut next_state: ResMut<NextState<Game>>| {
                    next_state.set(Game::Playing);
                },
            );
        });
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
