mod assets;
#[cfg(feature = "dev")]
mod dev_tools;
pub mod game;
pub mod physics;
pub mod screens;
mod ui;

use bevy::{
    asset::AssetMetaCheck,
    audio::{AudioPlugin, Volume},
    prelude::*,
    render::view::RenderLayers,
    window::WindowResolution,
};
use game::{Game, rendering::GameRenderLayers};
use screens::Screen;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(OnEnter(Screen::InGame), GameSet::Init);
        app.configure_sets(
            Update,
            (GameSet::TickTimers, GameSet::RecordInput, GameSet::Update)
                .chain()
                .run_if(in_state(Game::Playing)),
        );
        app.configure_sets(
            Update,
            (NonGameSet::TickTimers, NonGameSet::Update)
                .chain()
                .run_if(not(in_state(Screen::InGame))),
        );

        app.add_systems(Startup, spawn_camera);

        app.add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics on web build on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "all the way home".to_string(),
                        canvas: Some("#bevy".to_string()),
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: true,
                        resizable: false,
                        // TODO: do these need to be hardcoded, or can we set them via `Config`?
                        resolution: WindowResolution::new(1280., 720.)
                            .with_scale_factor_override(1.0),
                        ..default()
                    }
                    .into(),
                    ..default()
                })
                .set(AudioPlugin {
                    global_volume: GlobalVolume {
                        volume: Volume::new(0.3),
                    },
                    ..default()
                }),
        );

        app.add_plugins((
            assets::plugin,
            screens::plugin,
            game::plugin,
            physics::PhysicsPlugin,
        ));

        #[cfg(feature = "dev")]
        app.add_plugins(dev_tools::plugin);
    }
}

// Game systems, always running if playing, but not while paused etc.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum GameSet {
    Init,
    RecordInput,
    TickTimers,
    Update,
}

// Non-game systems, run outside of Screen::InGame.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum NonGameSet {
    TickTimers,
    Update,
}

#[derive(Component)]
pub struct MainCamera;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("Main Camera"),
        MainCamera,
        Camera2d,
        IsDefaultUiCamera,
        // This camera needs to be able to see all our render layers in order to composite the
        // level background and the sprites together into one view.
        RenderLayers::from_layers(&[
            GameRenderLayers::Main.into(),
            GameRenderLayers::Terrain.into(),
        ]),
    ));
}
