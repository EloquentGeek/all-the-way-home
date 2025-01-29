use bevy::{
    prelude::*,
    render::render_resource::{TextureFormat, TextureUsages},
};
use tiny_bail::prelude::*;

use crate::{
    assets::Levels,
    game::{Game, level::LevelRenderTargets},
    screens::Screen,
    ui::Containers,
};

const INTRO_TIMER_DURATION_SECS: f32 = 0.5;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(Screen::Intro),
        (spawn_intro_screen, insert_intro_timer, prepare_level_images).chain(),
    );
    app.add_systems(OnExit(Screen::Intro), remove_intro_timer);
    app.add_systems(
        Update,
        (tick_intro_timer, check_intro_timer)
            .chain()
            .run_if(in_state(Screen::Intro)),
    );
}

#[derive(Resource, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
struct IntroTimer(Timer);

impl Default for IntroTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(
            INTRO_TIMER_DURATION_SECS,
            TimerMode::Once,
        ))
    }
}

fn spawn_intro_screen(mut commands: Commands) {
    commands
        .ui_root()
        .insert(StateScoped(Screen::Intro))
        .with_children(|p| {
            p.spawn(Text::new("Intro..."));
        });
}

pub fn prepare_level_images(
    mut images: ResMut<Assets<Image>>,
    mut level_targets: ResMut<LevelRenderTargets>,
    levels: Res<Levels>,
) {
    // NOTE: images loaded via bevy_asset_loader have the default `usage` settings. These need to
    // be modified in order to use the image as a render target. Here, we create two copies of the
    // level image: one to use as "source", the other "destination". These will be swapped after
    // rendering each frame.
    let level_image = r!(images.get_mut(&levels.level.clone()));
    let mut source_image = level_image.clone();
    source_image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT;
    let destination_image = source_image.clone();

    // Update the original image with new usage settings, and add the second copy of it to image assets.
    images.insert(&levels.level.clone(), source_image);

    // Store both image handles on our resource to facilitate swapping them each frame, and so the
    // material spawning system can grab them easily in the next screen.
    level_targets.source = levels.level.clone();
    level_targets.destination = images.add(destination_image);
}

fn insert_intro_timer(mut commands: Commands) {
    commands.init_resource::<IntroTimer>();
}

fn remove_intro_timer(mut commands: Commands) {
    commands.remove_resource::<IntroTimer>();
}

fn tick_intro_timer(time: Res<Time>, mut timer: ResMut<IntroTimer>) {
    timer.0.tick(time.delta());
}

fn check_intro_timer(
    timer: ResMut<IntroTimer>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut next_game_state: ResMut<NextState<Game>>,
) {
    if timer.0.just_finished() {
        next_screen.set(Screen::InGame);
        next_game_state.set(Game::Playing);
    }
}
